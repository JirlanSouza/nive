use std::marker::PhantomData;

use iced::{window, Subscription, Task};

use crate::application::program::run::auto_register_default_window;
#[cfg(feature = "devtools")]
use crate::application::program::DevtoolsRuntime;
use crate::application::program::{
    CoreMessage, NiveCore, NiveMessage, ProbeCatalogEntry, Program, ProgramBoot, RuntimeMessage,
    SettingsRuntime, TOAST_TICK_INTERVAL,
};
use crate::application::{
    Application, ApplicationConfig, Effect, Error, MessageContext, MessageSource, Result,
};
#[cfg(feature = "devtools")]
use crate::devtools::DevtoolsWindowSpec;
use crate::{ToastId, ToastItem, WindowRole};

impl<A, P> Program<A, P>
where
    A: Application,
    P: ProbeCatalogEntry,
{
    pub(super) fn new(
        mut config: ApplicationConfig<A::Window, A::Bootstrap>,
        #[cfg(feature = "devtools")] devtools: Option<DevtoolsRuntime<A>>,
    ) -> Result<ProgramBoot<A, P>> {
        let settings = SettingsRuntime::load(config.settings.as_ref());
        if let Some(preference) = settings
            .as_ref()
            .and_then(|settings| settings.session.theme_preference())
        {
            config.theme_preference = preference;
        }
        // Auto-register a single WindowSpec::app() for apps with `type Window =
        // ()` whose `ApplicationConfig` has no explicit windows. Apps with custom
        // `type Window` enums must call `.window(...)` in `Application::config()`.
        auto_register_default_window::<A>(&mut config);
        let core = NiveCore::new(&config, settings);
        let theme_task = core
            .theme
            .initial_task()
            .map(|event| NiveMessage::Core(CoreMessage::Theme(event)));
        let mut program = Self {
            core,
            app: None,
            bootstrap: None,
            #[cfg(feature = "devtools")]
            devtools,
            _probe: PhantomData,
        };

        let startup_task = if let Some(bootstrap) = config.immediate_bootstrap.take() {
            program.initialize_app(bootstrap)
        } else if let Some(spec) = config.bootstrap.take() {
            program.start_bootstrap(spec)
        } else {
            return Err(Error::BootstrapUnavailable);
        };

        Ok((program, Task::batch([startup_task, theme_task])))
    }

    pub(super) fn update(&mut self, message: RuntimeMessage<A, P>) -> Task<RuntimeMessage<A, P>> {
        match message {
            NiveMessage::Core(message) => self.update_core(message),
            NiveMessage::Bootstrap(message) => self.update_bootstrap(message),
            NiveMessage::App {
                window_id,
                source,
                message,
            } => {
                let effect: Effect<A::Message, A::Window> = {
                    let Some(app) = self.app.as_mut() else {
                        return Task::none();
                    };
                    let window = window_id.and_then(|id| self.core.window_context(id));
                    let message_context = MessageContext { window, source };
                    let context = self.core.context();
                    app.update(context, message_context, message).into()
                };
                self.apply_update_from(effect, window_id)
            }
            #[cfg(feature = "devtools")]
            NiveMessage::Devtools(message) => self.update_devtools(message),
            NiveMessage::Probe(_) => Task::none(),
        }
    }

    pub(super) fn view(&self, window_id: window::Id) -> nive_ui::Element<'_, RuntimeMessage<A, P>> {
        nive_ui::accessibility::FocusRoot::new(self.window_content(window_id))
            .on_modal_change(|active| NiveMessage::Core(CoreMessage::ModalActive(active)))
            .into()
    }

    pub(super) fn window_content(
        &self,
        window_id: window::Id,
    ) -> nive_ui::Element<'_, RuntimeMessage<A, P>> {
        if self
            .bootstrap
            .as_ref()
            .is_some_and(|bootstrap| bootstrap.window_id == window_id)
        {
            return self.bootstrap_view();
        }

        #[cfg(feature = "devtools")]
        if self.is_devtools_window(window_id) {
            return self.devtools_view();
        }

        let Some(window) = self.core.window_context(window_id) else {
            return iced::widget::text("").into();
        };
        let Some(app) = self.app.as_ref() else {
            return iced::widget::text("").into();
        };

        let mut screen =
            app.view(self.core.context(), window)
                .map(move |message| NiveMessage::App {
                    window_id: Some(window_id),
                    source: MessageSource::View,
                    message,
                });

        if window.role != WindowRole::App {
            return screen.into_element();
        }

        let active_window = self
            .core
            .registry
            .most_recent_app_window()
            .map(|handle| handle.id);

        // The toast stack is composed *inside* the screen content so the
        // modal kernel wraps it: `ScreenView::into_element` puts this under
        // `DialogHost`, which already owns the scrim, paint order, and
        // inert/event-captured base content. Hosting toasts outside that
        // wrapper would leave them interactive under an open dialog's scrim.
        screen.content = nive_ui::widgets::overlays::ToastHost::new(screen.content)
            .position(self.core.toast_position())
            .safe_insets(self.core.toast_insets())
            .on_hover(
                NiveMessage::Core(CoreMessage::ToastHoverEntered),
                NiveMessage::Core(CoreMessage::ToastHoverLeft),
            )
            .on_focus_within(
                NiveMessage::Core(CoreMessage::ToastFocusWithinEntered),
                NiveMessage::Core(CoreMessage::ToastFocusWithinLeft),
            )
            .toasts(
                self.core.toasts.visible_for(window_id, active_window),
                |id: ToastId| NiveMessage::Core(CoreMessage::ToastDismiss(id)),
                move |item: &ToastItem<A::Message>| {
                    item.request().action().map(|(label, message)| {
                        (
                            label,
                            NiveMessage::App {
                                window_id: Some(window_id),
                                source: MessageSource::View,
                                message: message.clone(),
                            },
                        )
                    })
                },
            )
            .into();

        screen.into_element()
    }

    pub(super) fn title(&self, window_id: window::Id) -> String {
        if self
            .bootstrap
            .as_ref()
            .is_some_and(|bootstrap| bootstrap.window_id == window_id)
        {
            return self.core.app_name.clone();
        }

        #[cfg(feature = "devtools")]
        if self.is_devtools_window(window_id) {
            return DevtoolsWindowSpec::title_for_app(&self.core.app_name);
        }

        self.core
            .window_context(window_id)
            .and_then(|window| {
                self.app.as_ref().map(|app| {
                    app.window_title(self.core.context(), window)
                        .into()
                        .into_owned()
                })
            })
            .unwrap_or_else(|| self.core.app_name.clone())
    }

    pub(super) fn theme(&self, _window_id: window::Id) -> nive_ui::Theme {
        self.core.theme.effective()
    }

    pub(super) fn subscription(&self) -> Subscription<RuntimeMessage<A, P>> {
        let window_events = window::events().filter_map(|(window_id, event)| match event {
            window::Event::Closed => Some(NiveMessage::Core(CoreMessage::WindowClosed(window_id))),
            window::Event::Focused => {
                Some(NiveMessage::Core(CoreMessage::WindowFocused(window_id)))
            }
            window::Event::Unfocused => {
                Some(NiveMessage::Core(CoreMessage::WindowUnfocused(window_id)))
            }
            window::Event::Moved(position) => Some(NiveMessage::Core(CoreMessage::WindowMoved(
                window_id, position,
            ))),
            window::Event::Resized(size) => Some(NiveMessage::Core(CoreMessage::WindowResized(
                window_id, size,
            ))),
            window::Event::CloseRequested => Some(NiveMessage::Core(
                CoreMessage::WindowCloseRequested(window_id),
            )),
            _ => None,
        });
        let theme = self
            .core
            .theme
            .subscription()
            .map(|event| NiveMessage::Core(CoreMessage::Theme(event)));
        let app = self
            .app
            .as_ref()
            .map(|app| {
                app.subscription(self.core.context())
                    .map(|message| NiveMessage::App {
                        window_id: None,
                        source: MessageSource::Subscription,
                        message,
                    })
            })
            .unwrap_or_else(Subscription::none);
        let toasts = if self.core.toasts.should_subscribe() {
            iced::time::every(TOAST_TICK_INTERVAL)
                .map(|now| NiveMessage::Core(CoreMessage::ToastTick(now)))
        } else {
            Subscription::none()
        };
        let shortcuts = self.shortcut_subscription();
        let subscriptions = vec![window_events, theme, app, toasts, shortcuts];

        Subscription::batch(subscriptions)
    }
}
