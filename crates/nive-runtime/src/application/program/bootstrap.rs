use std::sync::{Arc, Mutex};
use std::time::Instant;

use iced::{window, ContentFit, Task};

use crate::application::program::{
    BootstrapMessage, BootstrapRuntime, BootstrapUiMessage, CoreMessage, NiveMessage,
    ProbeCatalogEntry, Program, RuntimeMessage,
};
use crate::application::Application;
use crate::lifecycle::bootstrap::{
    minimum_duration_task, BootstrapController, BootstrapTransition,
};
use crate::{
    BootstrapSpec, DialogRequest, ScreenView, WindowCardinality, WindowChrome, WindowMode,
    WindowRole, WindowSpec,
};

impl<A, P> Program<A, P>
where
    A: Application,
    P: ProbeCatalogEntry,
{
    pub(super) fn start_bootstrap(
        &mut self,
        spec: BootstrapSpec<A::Bootstrap>,
    ) -> Task<RuntimeMessage<A, P>> {
        let started_at = Instant::now();
        let controller = BootstrapController::new(started_at, spec.configured_minimum_duration());
        let attempt = controller.attempt();
        let (window_id, open_task) =
            window::open(bootstrap_window_spec(self.core.window_icon.clone()));
        self.bootstrap = Some(BootstrapRuntime {
            spec,
            controller,
            window_id,
        });

        Task::batch([
            open_task.map(|window_id| NiveMessage::Core(CoreMessage::WindowOpened(window_id))),
            self.bootstrap_attempt_task(attempt, started_at),
        ])
    }

    pub(super) fn bootstrap_attempt_task(
        &self,
        attempt: u64,
        started_at: Instant,
    ) -> Task<RuntimeMessage<A, P>> {
        let Some(bootstrap) = self.bootstrap.as_ref() else {
            return Task::none();
        };

        let result = bootstrap.spec.run().map(move |result| {
            NiveMessage::Bootstrap(BootstrapMessage::Finished {
                attempt,
                result: Arc::new(Mutex::new(Some(result))),
            })
        });
        let minimum = minimum_duration_task(
            started_at,
            bootstrap.spec.configured_minimum_duration(),
            move || NiveMessage::Bootstrap(BootstrapMessage::MinimumElapsed { attempt }),
        );

        Task::batch([result, minimum])
    }

    pub(super) fn update_bootstrap(
        &mut self,
        message: BootstrapMessage<A::Bootstrap>,
    ) -> Task<RuntimeMessage<A, P>> {
        match message {
            BootstrapMessage::Finished { attempt, result } => {
                let result = result
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take();
                let Some(result) = result else {
                    return Task::none();
                };
                let transition = self
                    .bootstrap
                    .as_mut()
                    .map(|bootstrap| bootstrap.controller.finish(attempt, result, Instant::now()))
                    .unwrap_or(BootstrapTransition::Ignored);
                self.handle_bootstrap_transition(transition)
            }
            BootstrapMessage::MinimumElapsed { attempt } => {
                let transition = self
                    .bootstrap
                    .as_mut()
                    .map(|bootstrap| {
                        bootstrap
                            .controller
                            .minimum_elapsed(attempt, Instant::now())
                    })
                    .unwrap_or(BootstrapTransition::Ignored);
                self.handle_bootstrap_transition(transition)
            }
            BootstrapMessage::Retry => {
                let started_at = Instant::now();
                let attempt = self
                    .bootstrap
                    .as_mut()
                    .and_then(|bootstrap| bootstrap.controller.retry(started_at));

                attempt
                    .map(|attempt| self.bootstrap_attempt_task(attempt, started_at))
                    .unwrap_or_else(Task::none)
            }
            BootstrapMessage::ShowDetails => {
                if let Some(bootstrap) = self.bootstrap.as_mut() {
                    bootstrap.controller.show_details();
                }
                Task::none()
            }
            BootstrapMessage::CloseDetails => {
                if let Some(bootstrap) = self.bootstrap.as_mut() {
                    bootstrap.controller.close_details();
                }
                Task::none()
            }
        }
    }

    pub(super) fn handle_bootstrap_transition(
        &mut self,
        transition: BootstrapTransition<A::Bootstrap>,
    ) -> Task<RuntimeMessage<A, P>> {
        match transition {
            BootstrapTransition::Ready(bootstrap) => self.complete_bootstrap(bootstrap),
            BootstrapTransition::Ignored
            | BootstrapTransition::Pending
            | BootstrapTransition::Failed => Task::none(),
        }
    }

    pub(super) fn complete_bootstrap(
        &mut self,
        result: A::Bootstrap,
    ) -> Task<RuntimeMessage<A, P>> {
        let Some(bootstrap) = self.bootstrap.take() else {
            return Task::none();
        };
        let splash_window = bootstrap.window_id;

        self.initialize_app(result)
            .chain(window::close(splash_window))
    }

    pub(super) fn initialize_app(&mut self, bootstrap: A::Bootstrap) -> Task<RuntimeMessage<A, P>> {
        // Compute `init`'s `(app, impl Into<Effect>)` return inside a block
        // and eagerly `.into()` it so the RPIT hidden borrows (against
        // `context`, which borrows `self.core`) are discharged before we
        // mutate `self.app` / `self.core.theme` below.
        let (app, update) = {
            let context = self.core.context();
            let (app, update) = A::init(context, bootstrap);
            (app, update.into())
        };
        self.app = Some(app);

        // Consult `Application::theme` once the app exists so that an
        // app-driven preference (e.g. always `Dark`) takes effect before any
        // `Effect::theme` is emitted.
        let resolved = self.resolve_theme_preference(None);
        self.core.theme.set_preference(resolved);

        let (app_task, runtime_task) = self.apply_initial_update(update);
        let init_task = Task::batch([app_task, runtime_task.chain(self.open_initial_windows())]);
        let devtools_task = self.initialize_devtools();
        Task::batch([init_task, devtools_task])
    }

    pub(super) fn bootstrap_view(&self) -> nive_ui::Element<'_, RuntimeMessage<A, P>> {
        let Some(bootstrap) = self.bootstrap.as_ref() else {
            return iced::widget::text("").into();
        };
        let brand = bootstrap.spec.brand_content();
        let background = bootstrap.spec.splash_background();
        let error = bootstrap
            .controller
            .error()
            .map(|error| nive_ui::BootstrapError {
                summary: error.summary(),
                detail: error.detail(),
                has_diagnostic_detail: error.has_diagnostic_detail(),
                details_visible: bootstrap.controller.details_visible(),
            });
        let view = nive_ui::BootstrapView::new(
            brand
                .map(|brand| brand.title())
                .unwrap_or(self.core.app_name.as_str()),
            bootstrap.spec.loading_text(),
            bootstrap.spec.failure_heading(),
            bootstrap.spec.failure_text(),
            BootstrapUiMessage::Retry,
            BootstrapUiMessage::ShowDetails,
            BootstrapUiMessage::CloseDetails,
        )
        .subtitle(brand.and_then(|brand| brand.subtitle_text()))
        .logo(brand.and_then(|brand| brand.logo_svg()))
        .background(
            background.map(|background| background.svg_bytes()),
            background
                .map(|background| match background.fit_mode() {
                    crate::BackgroundFit::Contain => ContentFit::Contain,
                    crate::BackgroundFit::Cover => ContentFit::Cover,
                    crate::BackgroundFit::Fill => ContentFit::Fill,
                })
                .unwrap_or(ContentFit::Cover),
            background
                .map(|background| background.opacity_value())
                .unwrap_or(1.0),
        )
        .error(error);
        let dialog = view.details_dialog().map(|content| {
            DialogRequest::new(content)
                .dismiss_on_backdrop_or_escape(BootstrapUiMessage::CloseDetails)
        });

        ScreenView::new(view.content())
            .dialog_maybe(dialog)
            .map(map_bootstrap_ui_message)
            .into_element()
    }
}

pub(super) fn map_bootstrap_ui_message<K, M, B, P>(
    message: BootstrapUiMessage,
) -> NiveMessage<K, M, B, P> {
    let message = match message {
        BootstrapUiMessage::Retry => BootstrapMessage::Retry,
        BootstrapUiMessage::ShowDetails => BootstrapMessage::ShowDetails,
        BootstrapUiMessage::CloseDetails => BootstrapMessage::CloseDetails,
    };

    NiveMessage::Bootstrap(message)
}

pub(super) fn bootstrap_window_spec(icon: Option<window::Icon>) -> window::Settings {
    let size = iced::Size::new(560.0, 360.0);

    WindowSpec {
        role: WindowRole::App,
        cardinality: WindowCardinality::Single,
        size,
        position: window::Position::Centered,
        min_size: Some(size),
        max_size: Some(size),
        resizable: false,
        decorations: true,
        transparent: false,
        mode: WindowMode::Windowed,
        chrome: WindowChrome::AppOwned,
        level: window::Level::Normal,
        session_key: None,
    }
    .settings(icon)
}
