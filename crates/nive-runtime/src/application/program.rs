use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use iced::keyboard;
use iced::{window, ContentFit, Subscription, Task};

use super::{
    Application, ApplicationConfig, CloseDecision, CommandRejected, CommandRejectionReason,
    Context, CoreEvent, Error, ExitDecision, Result, ShortcutMap, WindowCommand, WindowContext,
    WindowQuery,
};
use crate::bootstrap::{minimum_duration_task, BootstrapController, BootstrapTransition};
#[cfg(feature = "devtools")]
use crate::devtools::DevtoolsPanelMessage;
#[cfg(feature = "devtools")]
use crate::devtools::{DevtoolsConfig, DevtoolsHostState, DevtoolsWindowSpec};
use crate::keyboard_navigation::KeyboardNavigation;
use crate::{
    AppUpdate, BootstrapSpec, DialogRequest, NoProbe, ProbeCatalogEntry, RuntimeCommand,
    ScreenView, ThemeController, ThemeEvent, ToastId, ToastPosition, ToastState, UserFacingResult,
    WindowCardinality, WindowChrome, WindowHandle, WindowMode, WindowRegistry, WindowRole,
    WindowSpec,
};

const TOAST_TICK_INTERVAL: Duration = Duration::from_millis(500);

pub(super) fn run<A: Application>() -> Result {
    #[cfg(feature = "devtools")]
    {
        run_inner::<A, NoProbe>(None)
    }

    #[cfg(not(feature = "devtools"))]
    {
        run_inner::<A, NoProbe>()
    }
}

#[cfg(feature = "devtools")]
pub(super) fn run_with_devtools<A>() -> Result
where
    A: crate::devtools::DevtoolsApp,
{
    let devtools = DevtoolsRuntime::<A, A::Probe>::new(DevtoolsConfig::from_env());
    run_inner::<A, A::Probe>(Some(devtools))
}

#[cfg(feature = "devtools")]
fn run_inner<A, P>(devtools: Option<DevtoolsRuntime<A, P>>) -> Result
where
    A: Application,
    P: ProbeCatalogEntry,
{
    let config = A::config();
    let fonts = config.fonts.clone();
    let default_font = config.default_font;
    let (program, initial_task) = Program::<A, P>::new(config, devtools)?;
    run_program::<A, P>(program, initial_task, fonts, default_font)
}

#[cfg(not(feature = "devtools"))]
fn run_inner<A, P>() -> Result
where
    A: Application,
    P: ProbeCatalogEntry,
{
    let config = A::config();
    let fonts = config.fonts.clone();
    let default_font = config.default_font;
    let (program, initial_task) = Program::<A, P>::new(config)?;
    run_program::<A, P>(program, initial_task, fonts, default_font)
}

fn run_program<A, P>(
    program: Program<A, P>,
    initial_task: RuntimeTask<A, P>,
    fonts: Vec<std::borrow::Cow<'static, [u8]>>,
    default_font: iced::Font,
) -> Result
where
    A: Application,
    P: ProbeCatalogEntry,
{
    let boot = Mutex::new(Some((program, initial_task)));

    let mut daemon = iced::daemon(
        move || {
            boot.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
                .unwrap_or_else(|| panic!("Nive program booted more than once"))
        },
        Program::<A, P>::update,
        Program::<A, P>::view,
    )
    .title(Program::<A, P>::title)
    .theme(Program::<A, P>::theme)
    .subscription(Program::<A, P>::subscription)
    .default_font(default_font);

    for font in fonts {
        daemon = daemon.font(font);
    }

    daemon.run().map_err(Error::from)
}

struct Program<A: Application, P: ProbeCatalogEntry = NoProbe> {
    core: NiveCore<A::Window>,
    app: Option<A>,
    bootstrap: Option<BootstrapRuntime<A::Bootstrap>>,
    #[cfg(feature = "devtools")]
    devtools: Option<DevtoolsRuntime<A, P>>,
    _probe: PhantomData<P>,
}

#[cfg(feature = "devtools")]
struct DevtoolsRuntime<A: Application, P: ProbeCatalogEntry> {
    host: DevtoolsHostState<P>,
    window_id: Option<window::Id>,
    start_open: bool,
    config: DevtoolsConfig,
    #[cfg(feature = "devtools")]
    snapshot: fn(&A) -> crate::devtools::DevtoolStateSnapshot,
    #[cfg(feature = "devtools")]
    apply_command:
        fn(&mut A, &crate::devtools::DevtoolCommand) -> crate::devtools::DevtoolCommandResult,
    probe_snapshot: fn(&A) -> crate::ProbeInjectionSnapshot<P>,
    #[cfg(feature = "devtools")]
    apply_probe_effect: fn(&mut A, crate::ProbePanelEffect<P>),
}

#[cfg(feature = "devtools")]
impl<A> DevtoolsRuntime<A, A::Probe>
where
    A: crate::devtools::DevtoolsApp,
{
    fn new(config: DevtoolsConfig) -> Self {
        Self {
            host: DevtoolsHostState::disabled(),
            window_id: None,
            start_open: config.enabled(),
            config,
            #[cfg(feature = "devtools")]
            snapshot: A::devtools_snapshot,
            #[cfg(feature = "devtools")]
            apply_command: A::apply_devtools_command,
            probe_snapshot: A::devtools_probe_snapshot,
            #[cfg(feature = "devtools")]
            apply_probe_effect: A::devtools_apply_probe_effect,
        }
    }
}

type RuntimeMessage<A, P = NoProbe> = NiveMessage<
    <A as Application>::Window,
    <A as Application>::Message,
    <A as Application>::Bootstrap,
    P,
>;
type RuntimeTask<A, P = NoProbe> = Task<RuntimeMessage<A, P>>;
type ProgramBoot<A, P> = (Program<A, P>, RuntimeTask<A, P>);

enum NiveMessage<K, M, B, P> {
    Core(CoreMessage<K>),
    Bootstrap(BootstrapMessage<B>),
    App {
        window_id: Option<window::Id>,
        message: M,
    },
    #[cfg(feature = "devtools")]
    Devtools(DevtoolsPanelMessage<P>),
    #[cfg(not(feature = "devtools"))]
    Probe(std::marker::PhantomData<P>),
}

#[derive(Debug, Clone)]
enum CoreMessage<K> {
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    WindowFocused(window::Id),
    WindowCloseRequested(window::Id),
    Theme(ThemeEvent),
    ConfirmClose(window::Id),
    ConfirmExit,
    Rejected(CommandRejected<K>),
    ToastDismiss(ToastId),
    ToastHoverEntered,
    ToastHoverLeft,
    ToastTick(Instant),
    KeyboardNavigation(KeyboardNavigation),
    KeyboardEvent(keyboard::Event),
    #[cfg(feature = "devtools")]
    ToggleDevtools,
}

enum BootstrapMessage<B> {
    Finished {
        attempt: u64,
        result: SharedBootstrapResult<B>,
    },
    MinimumElapsed {
        attempt: u64,
    },
    Retry,
    ShowDetails,
    CloseDetails,
}

#[derive(Debug, Clone, Copy)]
enum BootstrapUiMessage {
    Retry,
    ShowDetails,
    CloseDetails,
}

type SharedBootstrapResult<B> = Arc<Mutex<Option<UserFacingResult<B>>>>;

struct BootstrapRuntime<B> {
    spec: BootstrapSpec<B>,
    controller: BootstrapController<B>,
    window_id: window::Id,
}

impl<K, M, B, P> Clone for NiveMessage<K, M, B, P>
where
    K: Clone,
    M: Clone,
    P: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Core(message) => Self::Core(message.clone()),
            Self::Bootstrap(message) => Self::Bootstrap(message.clone()),
            Self::App { window_id, message } => Self::App {
                window_id: *window_id,
                message: message.clone(),
            },
            #[cfg(feature = "devtools")]
            Self::Devtools(message) => Self::Devtools(message.clone()),
            #[cfg(not(feature = "devtools"))]
            Self::Probe(marker) => Self::Probe(*marker),
        }
    }
}

impl<B> Clone for BootstrapMessage<B> {
    fn clone(&self) -> Self {
        match self {
            Self::Finished { attempt, result } => Self::Finished {
                attempt: *attempt,
                result: Arc::clone(result),
            },
            Self::MinimumElapsed { attempt } => Self::MinimumElapsed { attempt: *attempt },
            Self::Retry => Self::Retry,
            Self::ShowDetails => Self::ShowDetails,
            Self::CloseDetails => Self::CloseDetails,
        }
    }
}

struct NiveCore<K> {
    app_id: String,
    app_name: String,
    windows: Vec<(K, WindowSpec)>,
    initial_windows: Vec<K>,
    registry: WindowRegistry<K>,
    theme: ThemeController,
    exiting: bool,
    window_icon: Option<window::Icon>,
    toast_position: ToastPosition,
    toasts: ToastState,
    toasts_hovered: bool,
    pending_app_closes: HashSet<window::Id>,
}

impl<A, P> Program<A, P>
where
    A: Application,
    P: ProbeCatalogEntry,
{
    fn new(
        mut config: ApplicationConfig<A::Window, A::Bootstrap>,
        #[cfg(feature = "devtools")] devtools: Option<DevtoolsRuntime<A, P>>,
    ) -> Result<ProgramBoot<A, P>> {
        let core = NiveCore::new(&config);
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

    fn update(&mut self, message: RuntimeMessage<A, P>) -> Task<RuntimeMessage<A, P>> {
        match message {
            NiveMessage::Core(message) => self.update_core(message),
            NiveMessage::Bootstrap(message) => self.update_bootstrap(message),
            NiveMessage::App { window_id, message } => {
                let Some(app) = self.app.as_mut() else {
                    return Task::none();
                };
                let window = window_id.and_then(|id| self.core.window_context(id));
                let context = self.core.context();
                let update = app.update(context, window, message);
                self.apply_update(update)
            }
            #[cfg(feature = "devtools")]
            NiveMessage::Devtools(message) => self.update_devtools(message),
            #[cfg(not(feature = "devtools"))]
            NiveMessage::Probe(_) => Task::none(),
        }
    }

    fn view(&self, window_id: window::Id) -> nive_ui::Element<'_, RuntimeMessage<A, P>> {
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

        let content = app
            .view(self.core.context(), window)
            .map(move |message| NiveMessage::App {
                window_id: Some(window_id),
                message,
            })
            .into_element();

        if window.role != WindowRole::App || !self.core.toasts.has_visible() {
            return content;
        }

        nive_ui::ToastHost::new(content)
            .position(self.core.toast_position().into())
            .on_hover(
                NiveMessage::Core(CoreMessage::ToastHoverEntered),
                NiveMessage::Core(CoreMessage::ToastHoverLeft),
            )
            .toasts(self.core.toasts.visible(), |id: ToastId| {
                NiveMessage::Core(CoreMessage::ToastDismiss(id))
            })
            .into()
    }

    fn title(&self, window_id: window::Id) -> String {
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
                self.app
                    .as_ref()
                    .map(|app| app.window_title(self.core.context(), window).into_owned())
            })
            .unwrap_or_else(|| self.core.app_name.clone())
    }

    fn theme(&self, _window_id: window::Id) -> nive_ui::Theme {
        self.core.theme.effective()
    }

    fn subscription(&self) -> Subscription<RuntimeMessage<A, P>> {
        let window_events = window::events().filter_map(|(window_id, event)| match event {
            window::Event::Closed => Some(NiveMessage::Core(CoreMessage::WindowClosed(window_id))),
            window::Event::Focused => {
                Some(NiveMessage::Core(CoreMessage::WindowFocused(window_id)))
            }
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

    fn start_bootstrap(&mut self, spec: BootstrapSpec<A::Bootstrap>) -> Task<RuntimeMessage<A, P>> {
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

    fn bootstrap_attempt_task(
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

    fn update_bootstrap(
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

    fn handle_bootstrap_transition(
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

    fn complete_bootstrap(&mut self, result: A::Bootstrap) -> Task<RuntimeMessage<A, P>> {
        let Some(bootstrap) = self.bootstrap.take() else {
            return Task::none();
        };
        let splash_window = bootstrap.window_id;

        self.initialize_app(result)
            .chain(window::close(splash_window))
    }

    fn initialize_app(&mut self, bootstrap: A::Bootstrap) -> Task<RuntimeMessage<A, P>> {
        let context = self.core.context();
        let (app, update) = A::init(context, bootstrap);
        self.app = Some(app);

        let (app_task, runtime_task) = self.apply_initial_update(update);
        let init_task = Task::batch([app_task, runtime_task.chain(self.open_initial_windows())]);
        let devtools_task = self.initialize_devtools();
        Task::batch([init_task, devtools_task])
    }

    #[cfg(feature = "devtools")]
    fn initialize_devtools(&mut self) -> Task<RuntimeMessage<A, P>> {
        let Some(devtools) = self.devtools.as_mut() else {
            return Task::none();
        };
        let Some(app) = self.app.as_ref() else {
            return Task::none();
        };

        let probe_snapshot = (devtools.probe_snapshot)(app);
        let panel = crate::devtools::DevtoolsPanelState::from_probe_snapshot_with_config(
            &probe_snapshot,
            devtools.config,
        );
        devtools.host = DevtoolsHostState::new(Some(panel));

        if devtools.start_open {
            devtools.start_open = false;
            self.open_devtools_window()
        } else {
            Task::none()
        }
    }

    #[cfg(not(feature = "devtools"))]
    fn initialize_devtools(&mut self) -> Task<RuntimeMessage<A, P>> {
        Task::none()
    }

    #[cfg(feature = "devtools")]
    fn open_devtools_window(&mut self) -> Task<RuntimeMessage<A, P>> {
        let Some(devtools) = self.devtools.as_mut() else {
            return Task::none();
        };
        if let Some(window_id) = devtools.window_id {
            return window::gain_focus(window_id);
        }

        let spec = DevtoolsWindowSpec::default().window_spec();
        let (window_id, open_task) = window::open(spec.settings(self.core.window_icon.clone()));
        devtools.window_id = Some(window_id);
        open_task.map(|id| NiveMessage::Core(CoreMessage::WindowOpened(id)))
    }

    #[cfg(feature = "devtools")]
    fn is_devtools_window(&self, window_id: window::Id) -> bool {
        self.devtools
            .as_ref()
            .is_some_and(|devtools| devtools.window_id == Some(window_id))
    }

    #[cfg(not(feature = "devtools"))]
    fn is_devtools_window(&self, _window_id: window::Id) -> bool {
        false
    }

    #[cfg(feature = "devtools")]
    fn devtools_view(&self) -> nive_ui::Element<'_, RuntimeMessage<A, P>> {
        use crate::devtools::devtools_window;

        let Some(devtools) = self.devtools.as_ref() else {
            return iced::widget::text("").into();
        };
        let Some(panel) = devtools.host.panel() else {
            return iced::widget::text("").into();
        };
        let Some(app) = self.app.as_ref() else {
            return iced::widget::text("").into();
        };
        let snapshot = (devtools.snapshot)(app);

        devtools_window(panel, snapshot, NiveMessage::Devtools)
    }

    #[cfg(feature = "devtools")]
    fn update_devtools(&mut self, message: DevtoolsPanelMessage<P>) -> Task<RuntimeMessage<A, P>> {
        let Some(devtools) = self.devtools.as_mut() else {
            return Task::none();
        };

        let effect = devtools.host.update(message);
        let Some(effect) = effect else {
            return Task::none();
        };

        match effect {
            crate::devtools::DevtoolsPanelEffect::Command(command) => {
                let Some(app) = self.app.as_mut() else {
                    return Task::none();
                };
                let result = (devtools.apply_command)(app, &command);
                devtools.host.record_command_result(&command, result);
            }
            crate::devtools::DevtoolsPanelEffect::Probe(effect) => {
                let Some(app) = self.app.as_mut() else {
                    return Task::none();
                };
                (devtools.apply_probe_effect)(app, effect);
            }
        }
        Task::none()
    }

    fn open_initial_windows(&mut self) -> Task<RuntimeMessage<A, P>> {
        self.core
            .initial_windows
            .clone()
            .into_iter()
            .fold(Task::none(), |task, kind| {
                task.chain(self.handle_window_command(WindowCommand::Open(kind)))
            })
    }

    fn bootstrap_view(&self) -> nive_ui::Element<'_, RuntimeMessage<A, P>> {
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

    fn update_core(&mut self, message: CoreMessage<A::Window>) -> Task<RuntimeMessage<A, P>> {
        match message {
            CoreMessage::WindowOpened(window_id) => {
                if self.is_bootstrap_window(window_id) {
                    return Task::none();
                }
                if self.is_devtools_window(window_id) {
                    return Task::none();
                }
                let Some(handle) = self.core.registry.mark_opened(window_id) else {
                    return Task::none();
                };

                self.emit_core_event(CoreEvent::WindowOpened(handle.into()))
            }
            CoreMessage::WindowClosed(window_id) => {
                self.core.pending_app_closes.remove(&window_id);
                if self.is_bootstrap_window(window_id) {
                    if let Some(mut bootstrap) = self.bootstrap.take() {
                        bootstrap.controller.cancel();
                    }
                    self.core.exiting = true;
                    return iced::exit();
                }
                #[cfg(feature = "devtools")]
                {
                    if self.is_devtools_window(window_id) {
                        if let Some(devtools) = self.devtools.as_mut() {
                            devtools.window_id = None;
                        }
                        return Task::none();
                    }
                }
                let Some(handle) = self.core.registry.get(window_id) else {
                    return Task::none();
                };
                self.core.registry.set_closed(window_id);

                let closed = self.emit_core_event(CoreEvent::WindowClosed(handle.into()));
                if handle.role == WindowRole::App && self.core.registry.app_window_count() == 0 {
                    let last_closed = self.emit_core_event(CoreEvent::LastAppWindowClosed);
                    let exit = if self.core.exiting {
                        Task::none()
                    } else {
                        self.request_exit()
                    };

                    Task::batch([closed, last_closed, exit])
                } else {
                    closed
                }
            }
            CoreMessage::WindowFocused(window_id) => {
                if self.is_bootstrap_window(window_id) {
                    return Task::none();
                }
                if self.is_devtools_window(window_id) {
                    return Task::none();
                }
                let Some(handle) = self.core.registry.set_focused(window_id) else {
                    return Task::none();
                };

                self.emit_core_event(CoreEvent::WindowFocused(handle.into()))
            }
            CoreMessage::WindowCloseRequested(window_id) => {
                if self.is_bootstrap_window(window_id) {
                    self.core.exiting = true;
                    if let Some(bootstrap) = self.bootstrap.as_mut() {
                        bootstrap.controller.cancel();
                    }
                    iced::exit()
                } else {
                    #[cfg(feature = "devtools")]
                    if self.is_devtools_window(window_id) {
                        return window::close(window_id);
                    }
                    self.request_close(window_id)
                }
            }
            CoreMessage::Theme(event) => {
                if self.core.theme.handle(event) {
                    self.emit_core_event(CoreEvent::ThemeChanged(self.core.theme.effective()))
                } else {
                    Task::none()
                }
            }
            CoreMessage::ConfirmClose(window_id) => window::close(window_id),
            CoreMessage::ConfirmExit => self.accept_exit(),
            CoreMessage::Rejected(rejection) => {
                self.emit_core_event(CoreEvent::CommandRejected(rejection))
            }
            CoreMessage::ToastDismiss(id) => {
                self.core.toasts.dismiss(id, Instant::now());
                Task::none()
            }
            CoreMessage::ToastHoverEntered => {
                self.core.toasts_hovered = true;
                Task::none()
            }
            CoreMessage::ToastHoverLeft => {
                self.core.toasts_hovered = false;
                Task::none()
            }
            CoreMessage::ToastTick(now) => {
                self.core.toasts.handle_tick(now, !self.core.toasts_hovered);
                Task::none()
            }
            CoreMessage::KeyboardNavigation(navigation) => navigation.task(),
            CoreMessage::KeyboardEvent(event) => self.handle_keyboard_event(event),
            #[cfg(feature = "devtools")]
            CoreMessage::ToggleDevtools => {
                let Some(devtools) = self.devtools.as_ref() else {
                    return Task::none();
                };
                if let Some(window_id) = devtools.window_id {
                    window::gain_focus(window_id)
                } else {
                    self.open_devtools_window()
                }
            }
        }
    }

    fn apply_initial_update(
        &mut self,
        update: AppUpdate<A::Message, A::Window>,
    ) -> (RuntimeTask<A, P>, RuntimeTask<A, P>) {
        let (task, _, commands) = update.into_parts();
        let app_task = task.map(|message| NiveMessage::App {
            window_id: None,
            message,
        });
        let runtime_task = commands.into_iter().fold(Task::none(), |task, command| {
            task.chain(self.handle_runtime_command(command))
        });

        (app_task, runtime_task)
    }

    fn apply_update(
        &mut self,
        update: AppUpdate<A::Message, A::Window>,
    ) -> Task<RuntimeMessage<A, P>> {
        let (task, _, commands) = update.into_parts();
        let app_task = task.map(|message| NiveMessage::App {
            window_id: None,
            message,
        });
        let runtime_task = commands.into_iter().fold(Task::none(), |task, command| {
            task.chain(self.handle_runtime_command(command))
        });

        Task::batch([app_task, runtime_task])
    }

    fn handle_runtime_command(
        &mut self,
        command: RuntimeCommand<A::Window>,
    ) -> Task<RuntimeMessage<A, P>> {
        match command {
            RuntimeCommand::Toast(toast) => {
                self.core.toasts.push(toast, Instant::now());
                Task::none()
            }
            RuntimeCommand::Window(command) => self.handle_window_command(command),
            RuntimeCommand::Theme(preference) => {
                if self.core.theme.set_preference(preference) {
                    self.emit_core_event(CoreEvent::ThemeChanged(self.core.theme.effective()))
                } else {
                    Task::none()
                }
            }
            RuntimeCommand::Exit => self.request_exit(),
        }
    }

    fn handle_window_command(
        &mut self,
        command: WindowCommand<A::Window>,
    ) -> Task<RuntimeMessage<A, P>> {
        match command {
            WindowCommand::Open(kind) => self.open_window(kind),
            WindowCommand::Close(window_id) => {
                if self.core.registry.get(window_id).is_some() {
                    self.request_close(window_id)
                } else {
                    self.reject(command, CommandRejectionReason::MissingWindow)
                }
            }
            WindowCommand::CloseKind(kind) => {
                let window_ids = self
                    .core
                    .registry
                    .all(kind)
                    .map(|handle| handle.id)
                    .collect::<Vec<_>>();

                if window_ids.is_empty() {
                    self.reject(command, CommandRejectionReason::MissingWindow)
                } else {
                    window_ids
                        .into_iter()
                        .fold(Task::none(), |task, window_id| {
                            task.chain(self.request_close(window_id))
                        })
                }
            }
            WindowCommand::Focus(window_id) => {
                if self.core.registry.set_focused(window_id).is_some() {
                    window::gain_focus(window_id)
                } else {
                    self.reject(command, CommandRejectionReason::MissingWindow)
                }
            }
            WindowCommand::FocusKind(kind) => {
                if let Some(handle) = self.core.registry.first(kind) {
                    self.core.registry.set_focused(handle.id);
                    window::gain_focus(handle.id)
                } else {
                    self.reject(command, CommandRejectionReason::MissingWindow)
                }
            }
        }
    }

    fn open_window(&mut self, kind: A::Window) -> Task<RuntimeMessage<A, P>> {
        let command = WindowCommand::Open(kind);
        if self.core.exiting {
            return self.reject(command, CommandRejectionReason::Exiting);
        }

        let Some(spec) = self.core.window_spec(kind) else {
            return self.reject(command, CommandRejectionReason::MissingWindowSpec);
        };

        if spec.cardinality == WindowCardinality::Single {
            if let Some(existing) = self.core.registry.first(kind) {
                self.core.registry.set_focused(existing.id);
                return window::gain_focus(existing.id);
            }
        }

        let (window_id, task) = window::open(spec.settings(self.core.window_icon.clone()));
        self.core.registry.set_opening(WindowHandle {
            kind,
            id: window_id,
            role: spec.role,
        });

        task.map(|window_id| NiveMessage::Core(CoreMessage::WindowOpened(window_id)))
    }

    fn request_close(&mut self, window_id: window::Id) -> Task<RuntimeMessage<A, P>> {
        let Some(window) = self.core.window_context(window_id) else {
            return Task::none();
        };

        if window.role == WindowRole::Auxiliary {
            return window::close(window_id);
        }

        if self.core.pending_app_closes.contains(&window_id) {
            return Task::none();
        }

        if self.core.effective_app_window_count() <= 1 {
            return self.request_exit();
        }

        let context = self.core.context();
        let Some(app) = self.app.as_mut() else {
            return Task::none();
        };
        match app.on_window_close_requested(context, window) {
            CloseDecision::Close => {
                self.core.pending_app_closes.insert(window_id);
                window::close(window_id)
            }
            CloseDecision::Cancel => Task::none(),
            CloseDecision::Defer(task) => {
                self.core.pending_app_closes.insert(window_id);
                task.map(|message| NiveMessage::App {
                    window_id: None,
                    message,
                })
                .chain(Task::done(NiveMessage::Core(CoreMessage::ConfirmClose(
                    window_id,
                ))))
            }
        }
    }

    fn request_exit(&mut self) -> Task<RuntimeMessage<A, P>> {
        if self.core.exiting {
            return Task::none();
        }

        let Some(app) = self.app.as_mut() else {
            self.core.exiting = true;
            if let Some(bootstrap) = self.bootstrap.as_mut() {
                bootstrap.controller.cancel();
            }
            return iced::exit();
        };
        let context = self.core.context();
        match app.on_exit_requested(context) {
            ExitDecision::Accept => self.accept_exit(),
            ExitDecision::Cancel => Task::none(),
            ExitDecision::Defer(task) => {
                self.core.exiting = true;
                task.map(|message| NiveMessage::App {
                    window_id: None,
                    message,
                })
                .chain(Task::done(NiveMessage::Core(CoreMessage::ConfirmExit)))
            }
        }
    }

    fn accept_exit(&mut self) -> Task<RuntimeMessage<A, P>> {
        self.core.exiting = true;
        let close_auxiliary = self
            .core
            .registry
            .handles()
            .filter(|handle| handle.role == WindowRole::Auxiliary)
            .fold(Task::none(), |task, handle| {
                task.chain(window::close(handle.id))
            });
        #[cfg(feature = "devtools")]
        let close_devtools = self
            .devtools
            .as_ref()
            .and_then(|devtools| devtools.window_id)
            .map(window::close)
            .unwrap_or(Task::none());
        #[cfg(not(feature = "devtools"))]
        let close_devtools = Task::none();

        close_auxiliary.chain(close_devtools).chain(iced::exit())
    }

    fn reject(
        &self,
        command: WindowCommand<A::Window>,
        reason: CommandRejectionReason,
    ) -> Task<RuntimeMessage<A, P>> {
        Task::done(NiveMessage::Core(CoreMessage::Rejected(CommandRejected {
            command,
            reason,
        })))
    }

    fn emit_core_event(&mut self, event: CoreEvent<A::Window>) -> Task<RuntimeMessage<A, P>> {
        let Some(app) = self.app.as_mut() else {
            return Task::none();
        };
        let context = self.core.context();
        let update = app.on_core_event(context, event);
        self.apply_update(update)
    }

    fn is_bootstrap_window(&self, window_id: window::Id) -> bool {
        self.bootstrap
            .as_ref()
            .is_some_and(|bootstrap| bootstrap.window_id == window_id)
    }

    fn shortcut_subscription(&self) -> Subscription<RuntimeMessage<A, P>> {
        if self.app.is_none() {
            return Subscription::none();
        }

        keyboard::listen().map(|event| NiveMessage::Core(CoreMessage::KeyboardEvent(event)))
    }

    fn handle_keyboard_event(&mut self, event: keyboard::Event) -> Task<RuntimeMessage<A, P>> {
        let shortcuts = self
            .app
            .as_ref()
            .map(|app| app.shortcuts(self.core.context()))
            .unwrap_or_default();

        shortcut_message_from_event::<A, P>(&shortcuts, event)
            .map(Task::done)
            .unwrap_or_else(Task::none)
    }
}

fn map_bootstrap_ui_message<K, M, B, P>(message: BootstrapUiMessage) -> NiveMessage<K, M, B, P> {
    let message = match message {
        BootstrapUiMessage::Retry => BootstrapMessage::Retry,
        BootstrapUiMessage::ShowDetails => BootstrapMessage::ShowDetails,
        BootstrapUiMessage::CloseDetails => BootstrapMessage::CloseDetails,
    };

    NiveMessage::Bootstrap(message)
}

fn shortcut_message_from_event<A, P>(
    shortcuts: &ShortcutMap<A::Message>,
    event: keyboard::Event,
) -> Option<RuntimeMessage<A, P>>
where
    A: Application,
    P: ProbeCatalogEntry,
{
    if let Some(navigation) = keyboard_navigation_from_event(&event) {
        return Some(NiveMessage::Core(CoreMessage::KeyboardNavigation(
            navigation,
        )));
    }
    if is_escape_key_event(&event) {
        return None;
    }

    #[cfg(feature = "devtools")]
    if let Some(message) = devtools_toggle_from_event(event.clone()) {
        return Some(message);
    }

    shortcuts
        .message_for_event(&event)
        .map(|message| NiveMessage::App {
            window_id: None,
            message,
        })
}

fn keyboard_navigation_from_event(event: &keyboard::Event) -> Option<KeyboardNavigation> {
    crate::direction_from_keyboard_event(event).map(KeyboardNavigation::from)
}

fn is_escape_key_event(event: &keyboard::Event) -> bool {
    matches!(
        event,
        keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            modifiers,
            repeat: false,
            ..
        } if modifiers.is_empty()
    )
}

#[cfg(feature = "devtools")]
fn devtools_toggle_from_event<K, M, B, P>(
    event: keyboard::Event,
) -> Option<NiveMessage<K, M, B, P>> {
    match event {
        keyboard::Event::KeyPressed { key, modifiers, .. } => {
            let is_devtools_key = matches!(
                key,
                keyboard::Key::Character(c) if c.eq_ignore_ascii_case("i")
            );
            let is_devtools_modifier = if cfg!(target_os = "macos") {
                modifiers.command() && modifiers.alt()
            } else {
                modifiers.control() && modifiers.alt()
            };
            if is_devtools_key && is_devtools_modifier {
                Some(NiveMessage::Core(CoreMessage::ToggleDevtools))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn bootstrap_window_spec(icon: Option<window::Icon>) -> window::Settings {
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
    }
    .settings(icon)
}

impl<K> NiveCore<K>
where
    K: Copy + Eq,
{
    fn new<B>(config: &ApplicationConfig<K, B>) -> Self {
        Self {
            app_id: config.app_id.clone(),
            app_name: config.app_name.clone(),
            windows: config
                .windows
                .iter()
                .map(|registration| (registration.kind, registration.spec))
                .collect(),
            initial_windows: config.initial_windows.clone(),
            registry: WindowRegistry::new(),
            theme: ThemeController::new(config.theme_preference),
            exiting: false,
            window_icon: config.window_icon.clone(),
            toast_position: config.toast_position,
            toasts: ToastState::default(),
            toasts_hovered: false,
            pending_app_closes: HashSet::new(),
        }
    }

    fn context(&self) -> Context<'_, K> {
        Context {
            app_id: self.app_id.as_str(),
            app_name: self.app_name.as_str(),
            theme: self.theme.effective(),
            theme_preference: self.theme.preference(),
            windows: WindowQuery {
                registry: &self.registry,
            },
            exiting: self.exiting,
        }
    }

    fn window_context(&self, window_id: window::Id) -> Option<WindowContext<K>> {
        self.registry.get(window_id).map(Into::into)
    }

    fn window_spec(&self, kind: K) -> Option<WindowSpec> {
        self.windows
            .iter()
            .find(|(registered_kind, _)| *registered_kind == kind)
            .map(|(_, spec)| *spec)
    }

    fn toast_position(&self) -> ToastPosition {
        self.toast_position
    }

    fn effective_app_window_count(&self) -> usize {
        self.registry
            .handles()
            .filter(|handle| {
                handle.role == WindowRole::App && !self.pending_app_closes.contains(&handle.id)
            })
            .count()
    }
}

impl<K> From<WindowHandle<K>> for WindowContext<K> {
    fn from(handle: WindowHandle<K>) -> Self {
        Self {
            id: handle.id,
            kind: handle.kind,
            role: handle.role,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::time::Duration;

    use super::*;
    use crate::{DialogDismiss, ScreenView, ShortcutBinding, ToastRequest};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestWindow {
        Main,
        Secondary,
        Multiple,
        Missing,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestMessage {
        Shortcut,
    }

    #[derive(Debug, Clone)]
    struct TestApp {
        cancel_exit: bool,
        close_requests: usize,
        rejections: usize,
        show_dialog: bool,
    }

    #[derive(Debug)]
    struct BootstrapTestApp {
        bootstrap: String,
    }

    #[derive(Debug)]
    struct PendingInitTaskApp;

    #[cfg(feature = "devtools")]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestProbe {
        One,
    }

    #[cfg(feature = "devtools")]
    impl ProbeCatalogEntry for TestProbe {
        const ALL: &'static [Self] = &[Self::One];

        fn meta(self) -> crate::ProbeMeta {
            crate::ProbeMeta::new(
                "test.one",
                "one",
                "Test probe",
                crate::ProbeErrorScope::Custom("test"),
            )
        }
    }

    impl Application for TestApp {
        type Message = TestMessage;
        type Window = TestWindow;
        type Bootstrap = ();

        fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
            ApplicationConfig::new("test")
                .window(TestWindow::Main, WindowSpec::app())
                .window(TestWindow::Secondary, WindowSpec::app())
                .window(TestWindow::Multiple, WindowSpec::app().multiple())
                .initial_window(TestWindow::Main)
        }

        fn init(
            _context: Context<'_, Self::Window>,
            _bootstrap: Self::Bootstrap,
        ) -> (Self, AppUpdate<Self::Message, Self::Window>) {
            (
                Self {
                    cancel_exit: false,
                    close_requests: 0,
                    rejections: 0,
                    show_dialog: false,
                },
                AppUpdate::none(),
            )
        }

        fn update(
            &mut self,
            _context: Context<'_, Self::Window>,
            _window: Option<WindowContext<Self::Window>>,
            _message: Self::Message,
        ) -> AppUpdate<Self::Message, Self::Window> {
            AppUpdate::none()
        }

        fn view(
            &self,
            _context: Context<'_, Self::Window>,
            _window: WindowContext<Self::Window>,
        ) -> ScreenView<'_, Self::Message> {
            let base = iced::widget::text("");
            if self.show_dialog {
                ScreenView::new(base).dialog(
                    DialogRequest::new(iced::widget::text("dialog"))
                        .dismiss(DialogDismiss::OnEscape(TestMessage::Shortcut)),
                )
            } else {
                ScreenView::new(base)
            }
        }

        fn window_title<'a>(
            &'a self,
            _context: Context<'a, Self::Window>,
            _window: WindowContext<Self::Window>,
        ) -> Cow<'a, str> {
            Cow::Borrowed("Test")
        }

        fn shortcuts(&self, _context: Context<'_, Self::Window>) -> ShortcutMap<Self::Message> {
            ShortcutMap::new()
                .bind(
                    ShortcutBinding::character('k', keyboard::Modifiers::CTRL),
                    TestMessage::Shortcut,
                )
                .bind(
                    ShortcutBinding::named(keyboard::key::Named::Tab, keyboard::Modifiers::NONE),
                    TestMessage::Shortcut,
                )
        }

        fn on_core_event(
            &mut self,
            _context: Context<'_, Self::Window>,
            event: CoreEvent<Self::Window>,
        ) -> AppUpdate<Self::Message, Self::Window> {
            if matches!(event, CoreEvent::CommandRejected(_)) {
                self.rejections += 1;
            }

            AppUpdate::none()
        }

        fn on_window_close_requested(
            &mut self,
            _context: Context<'_, Self::Window>,
            _window: WindowContext<Self::Window>,
        ) -> CloseDecision<Self::Message> {
            self.close_requests += 1;
            CloseDecision::Close
        }

        fn on_exit_requested(
            &mut self,
            _context: Context<'_, Self::Window>,
        ) -> ExitDecision<Self::Message> {
            if self.cancel_exit {
                ExitDecision::Cancel
            } else {
                ExitDecision::Accept
            }
        }
    }

    impl Application for BootstrapTestApp {
        type Message = TestMessage;
        type Window = TestWindow;
        type Bootstrap = String;

        fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
            ApplicationConfig::new("bootstrap-test")
                .window(TestWindow::Main, WindowSpec::app())
                .initial_window(TestWindow::Main)
                .bootstrap(
                    BootstrapSpec::new(|| Task::done(Ok(String::from("services"))))
                        .minimum_duration(Duration::ZERO),
                )
        }

        fn init(
            _context: Context<'_, Self::Window>,
            bootstrap: Self::Bootstrap,
        ) -> (Self, AppUpdate<Self::Message, Self::Window>) {
            (Self { bootstrap }, AppUpdate::none())
        }

        fn update(
            &mut self,
            _context: Context<'_, Self::Window>,
            _window: Option<WindowContext<Self::Window>>,
            _message: Self::Message,
        ) -> AppUpdate<Self::Message, Self::Window> {
            AppUpdate::none()
        }

        fn view(
            &self,
            _context: Context<'_, Self::Window>,
            _window: WindowContext<Self::Window>,
        ) -> ScreenView<'_, Self::Message> {
            ScreenView::new(iced::widget::text(""))
        }
    }

    impl Application for PendingInitTaskApp {
        type Message = TestMessage;
        type Window = TestWindow;
        type Bootstrap = ();

        fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
            ApplicationConfig::new("pending-init-test")
                .window(TestWindow::Main, WindowSpec::app())
                .initial_window(TestWindow::Main)
        }

        fn init(
            _context: Context<'_, Self::Window>,
            _bootstrap: Self::Bootstrap,
        ) -> (Self, AppUpdate<Self::Message, Self::Window>) {
            (
                Self,
                AppUpdate::from_task(Task::perform(std::future::pending::<()>(), |_| {
                    TestMessage::Shortcut
                })),
            )
        }

        fn update(
            &mut self,
            _context: Context<'_, Self::Window>,
            _window: Option<WindowContext<Self::Window>>,
            _message: Self::Message,
        ) -> AppUpdate<Self::Message, Self::Window> {
            AppUpdate::none()
        }

        fn view(
            &self,
            _context: Context<'_, Self::Window>,
            _window: WindowContext<Self::Window>,
        ) -> ScreenView<'_, Self::Message> {
            ScreenView::new(iced::widget::text(""))
        }
    }

    #[cfg(feature = "devtools")]
    impl crate::devtools::Devtools for TestApp {}

    #[cfg(feature = "devtools")]
    impl crate::devtools::DevtoolsApp for TestApp {
        type Probe = TestProbe;

        fn devtools_snapshot(&self) -> crate::devtools::DevtoolStateSnapshot {
            crate::devtools::DevtoolStateSnapshot::default()
        }

        fn apply_devtools_command(
            &mut self,
            _command: &crate::devtools::DevtoolCommand,
        ) -> crate::devtools::DevtoolCommandResult {
            crate::devtools::DevtoolCommandResult::not_handled()
        }

        fn devtools_probe_snapshot(&self) -> crate::ProbeInjectionSnapshot<Self::Probe> {
            crate::ProbeInjectionSnapshot {
                scenarios: Vec::new(),
                unknown: Vec::new(),
            }
        }

        fn devtools_apply_probe_effect(&mut self, _effect: crate::ProbePanelEffect<Self::Probe>) {}
    }

    fn program() -> Program<TestApp> {
        plain_program::<TestApp>()
            .map(|(program, _)| program)
            .unwrap_or_else(|error| panic!("test program failed: {error}"))
    }

    #[cfg(feature = "devtools")]
    fn plain_program<A>() -> Result<ProgramBoot<A, NoProbe>>
    where
        A: Application,
    {
        Program::<A>::new(A::config(), None)
    }

    #[cfg(not(feature = "devtools"))]
    fn plain_program<A>() -> Result<ProgramBoot<A, NoProbe>>
    where
        A: Application,
    {
        Program::<A>::new(A::config())
    }

    #[cfg(feature = "devtools")]
    fn devtools_program(config: DevtoolsConfig) -> Program<TestApp, TestProbe> {
        Program::new(
            TestApp::config(),
            Some(DevtoolsRuntime::<TestApp, TestProbe>::new(config)),
        )
        .map(|(program, _)| program)
        .unwrap_or_else(|error| panic!("test program failed: {error}"))
    }

    #[test]
    fn configured_initial_window_is_registered_as_opening() {
        let program = program();

        assert!(program.core.registry.contains(TestWindow::Main));
        assert_eq!(program.core.registry.app_window_count(), 1);
    }

    #[test]
    fn initial_windows_open_without_waiting_for_init_task() {
        let (program, _task) = plain_program::<PendingInitTaskApp>()
            .unwrap_or_else(|error| panic!("test program failed: {error}"));

        assert!(program.core.registry.contains(TestWindow::Main));
        assert_eq!(program.core.registry.app_window_count(), 1);
    }

    #[cfg(feature = "devtools")]
    #[test]
    fn devtools_runtime_starts_closed_and_ready_for_shortcut() {
        let program = devtools_program(DevtoolsConfig::default());
        let devtools = program
            .devtools
            .as_ref()
            .unwrap_or_else(|| panic!("devtools runtime should be installed"));

        assert!(devtools.host.is_enabled());
        assert_eq!(devtools.window_id, None);
    }

    #[cfg(feature = "devtools")]
    #[test]
    fn devtools_open_config_creates_initial_auxiliary_window() {
        let program = devtools_program(DevtoolsConfig::from_env_value(Some("open")));

        assert!(program
            .devtools
            .as_ref()
            .is_some_and(|devtools| devtools.window_id.is_some()));
    }

    #[cfg(feature = "devtools")]
    #[test]
    fn devtools_config_applies_initial_panel_tab() {
        let program = devtools_program(
            DevtoolsConfig::default().with_initial_tab(crate::devtools::DevtoolsPanelTab::Probes),
        );
        let active_tab = program
            .devtools
            .as_ref()
            .and_then(|devtools| devtools.host.panel())
            .map(|panel| panel.active_tab);

        assert_eq!(active_tab, Some(crate::devtools::DevtoolsPanelTab::Probes));
    }

    #[cfg(feature = "devtools")]
    #[test]
    fn closing_devtools_allows_shortcut_to_open_it_again() {
        let mut program = devtools_program(DevtoolsConfig::from_env_value(Some("open")));
        let first_window = program
            .devtools
            .as_ref()
            .and_then(|devtools| devtools.window_id)
            .unwrap_or_else(window::Id::unique);

        let _task = program.update_core(CoreMessage::WindowClosed(first_window));
        assert!(program
            .devtools
            .as_ref()
            .is_some_and(|devtools| devtools.window_id.is_none()));

        let _task = program.update_core(CoreMessage::ToggleDevtools);

        assert!(program
            .devtools
            .as_ref()
            .is_some_and(|devtools| devtools.window_id.is_some()));
    }

    #[cfg(feature = "devtools")]
    #[test]
    fn platform_shortcut_routes_to_devtools_toggle() {
        use iced::keyboard::key::{Code, Physical};
        use iced::keyboard::{Key, Location, Modifiers};

        let modifiers = if cfg!(target_os = "macos") {
            Modifiers::COMMAND | Modifiers::ALT
        } else {
            Modifiers::CTRL | Modifiers::ALT
        };
        let event = keyboard::Event::KeyPressed {
            key: Key::Character("i".into()),
            modified_key: Key::Character("i".into()),
            physical_key: Physical::Code(Code::KeyI),
            location: Location::Standard,
            modifiers,
            text: Some("i".into()),
            repeat: false,
        };

        assert!(matches!(
            devtools_toggle_from_event::<TestWindow, (), (), TestProbe>(event),
            Some(NiveMessage::Core(CoreMessage::ToggleDevtools))
        ));
    }

    fn key_pressed(
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
        repeat: bool,
    ) -> keyboard::Event {
        use iced::keyboard::key::{Code, Physical};
        use iced::keyboard::Location;

        keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Code(Code::KeyK),
            location: Location::Standard,
            modifiers,
            text: None,
            repeat,
        }
    }

    #[test]
    fn product_shortcut_routes_to_unscoped_app_message() {
        let shortcuts = ShortcutMap::new().bind(
            ShortcutBinding::character('K', keyboard::Modifiers::CTRL),
            TestMessage::Shortcut,
        );
        let event = key_pressed(
            keyboard::Key::Character("k".into()),
            keyboard::Modifiers::CTRL,
            false,
        );

        assert!(matches!(
            shortcut_message_from_event::<TestApp, NoProbe>(&shortcuts, event),
            Some(NiveMessage::App {
                window_id: None,
                message: TestMessage::Shortcut
            })
        ));
    }

    #[test]
    fn repeated_product_shortcut_keypress_is_ignored() {
        let shortcuts = ShortcutMap::new().bind(
            ShortcutBinding::character('k', keyboard::Modifiers::CTRL),
            TestMessage::Shortcut,
        );
        let event = key_pressed(
            keyboard::Key::Character("k".into()),
            keyboard::Modifiers::CTRL,
            true,
        );

        assert!(shortcut_message_from_event::<TestApp, NoProbe>(&shortcuts, event).is_none());
    }

    #[test]
    fn framework_shortcut_wins_product_conflict() {
        let shortcuts = ShortcutMap::new().bind(
            ShortcutBinding::named(keyboard::key::Named::Tab, keyboard::Modifiers::NONE),
            TestMessage::Shortcut,
        );
        let event = key_pressed(
            keyboard::Key::Named(keyboard::key::Named::Tab),
            keyboard::Modifiers::NONE,
            false,
        );

        assert!(matches!(
            shortcut_message_from_event::<TestApp, NoProbe>(&shortcuts, event),
            Some(NiveMessage::Core(CoreMessage::KeyboardNavigation(
                KeyboardNavigation::FocusNext
            )))
        ));
    }

    #[test]
    fn single_window_open_focuses_existing_instance() {
        let mut program = program();

        let _task = program.handle_window_command(WindowCommand::Open(TestWindow::Main));

        assert_eq!(program.core.registry.all(TestWindow::Main).count(), 1);
    }

    #[test]
    fn multiple_window_spec_preserves_each_instance() {
        let mut program = program();

        let _first = program.handle_window_command(WindowCommand::Open(TestWindow::Multiple));
        let _second = program.handle_window_command(WindowCommand::Open(TestWindow::Multiple));

        assert_eq!(program.core.registry.all(TestWindow::Multiple).count(), 2);
    }

    #[test]
    fn non_final_app_window_uses_close_hook() {
        let mut program = program();
        program.core.registry.set_opened(WindowHandle::new(
            TestWindow::Secondary,
            window::Id::unique(),
        ));
        let main_id = program
            .core
            .registry
            .first(TestWindow::Main)
            .map(|handle| handle.id)
            .unwrap_or_else(window::Id::unique);

        let _task = program.request_close(main_id);

        assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(1));
        assert!(!program.core.exiting);
    }

    #[test]
    fn last_app_window_uses_exit_hook() {
        let mut program = program();
        let main_id = program
            .core
            .registry
            .first(TestWindow::Main)
            .map(|handle| handle.id)
            .unwrap_or_else(window::Id::unique);

        let _task = program.request_close(main_id);

        assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(0));
        assert!(program.core.exiting);
    }

    #[test]
    fn simultaneous_closes_treat_second_app_window_as_exit_request() {
        let mut program = program();
        program.core.registry.set_opened(WindowHandle::new(
            TestWindow::Secondary,
            window::Id::unique(),
        ));
        if let Some(app) = program.app.as_mut() {
            app.cancel_exit = true;
        }
        let main_id = main_window_id(&program);
        let secondary_id = program
            .core
            .registry
            .first(TestWindow::Secondary)
            .map(|handle| handle.id)
            .unwrap_or_else(window::Id::unique);

        let _task = program.request_close(main_id);
        let _task = program.request_close(secondary_id);

        assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(1));
        assert!(program.core.pending_app_closes.contains(&main_id));
        assert!(!program.core.pending_app_closes.contains(&secondary_id));
        assert!(!program.core.exiting);
    }

    #[test]
    fn close_kind_all_windows_respects_cancelled_final_exit() {
        let mut program = program();
        program
            .core
            .registry
            .set_opened(WindowHandle::new(TestWindow::Main, window::Id::unique()));
        if let Some(app) = program.app.as_mut() {
            app.cancel_exit = true;
        }

        let _task = program.handle_window_command(WindowCommand::CloseKind(TestWindow::Main));

        assert_eq!(program.app.as_ref().map(|app| app.close_requests), Some(1));
        assert_eq!(program.core.effective_app_window_count(), 1);
        assert!(!program.core.exiting);
    }

    #[test]
    fn cancelled_exit_keeps_runtime_active() {
        let mut program = program();
        if let Some(app) = program.app.as_mut() {
            app.cancel_exit = true;
        }

        let _task = program.request_exit();

        assert!(!program.core.exiting);
    }

    #[test]
    fn command_rejection_is_forwarded_as_core_event() {
        let mut program = program();
        let rejection = CommandRejected {
            command: WindowCommand::Open(TestWindow::Missing),
            reason: CommandRejectionReason::MissingWindowSpec,
        };

        let _task = program.update_core(CoreMessage::Rejected(rejection));

        assert_eq!(program.app.as_ref().map(|app| app.rejections), Some(1));
    }

    #[test]
    fn configured_bootstrap_delays_app_init_and_initial_windows() {
        let (program, _task) = plain_program::<BootstrapTestApp>()
            .unwrap_or_else(|error| panic!("test program failed: {error}"));

        assert!(program.app.is_none());
        assert!(program.bootstrap.is_some());
        assert!(!program.core.registry.contains(TestWindow::Main));
    }

    #[test]
    fn successful_bootstrap_transfers_result_into_app_init() {
        let (mut program, _task) = plain_program::<BootstrapTestApp>()
            .unwrap_or_else(|error| panic!("test program failed: {error}"));
        let result = Arc::new(Mutex::new(Some(Ok(String::from("services")))));

        let _task = program.update_bootstrap(BootstrapMessage::Finished { attempt: 1, result });

        assert_eq!(
            program.app.as_ref().map(|app| app.bootstrap.as_str()),
            Some("services")
        );
        assert!(program.bootstrap.is_none());
        assert!(program.core.registry.contains(TestWindow::Main));
    }

    #[test]
    fn closing_splash_cancels_bootstrap_without_creating_app() {
        let (mut program, _task) = plain_program::<BootstrapTestApp>()
            .unwrap_or_else(|error| panic!("test program failed: {error}"));
        let splash_window = program
            .bootstrap
            .as_ref()
            .map(|bootstrap| bootstrap.window_id)
            .unwrap_or_else(window::Id::unique);

        let _task = program.update_core(CoreMessage::WindowCloseRequested(splash_window));

        assert!(program.core.exiting);
        assert!(program.app.is_none());
    }

    fn main_window_id(program: &Program<TestApp>) -> window::Id {
        program
            .core
            .registry
            .first(TestWindow::Main)
            .map(|handle| handle.id)
            .unwrap_or_else(window::Id::unique)
    }

    #[test]
    fn toast_runtime_command_enqueues_visible_toast() {
        let mut program = program();

        let _task =
            program.handle_runtime_command(RuntimeCommand::Toast(ToastRequest::info("Saved")));

        assert!(program.core.toasts.has_visible());
        assert!(program.core.toasts.should_subscribe());
    }

    #[test]
    fn toast_tick_expires_visible_toast() {
        let now = Instant::now();
        let mut program = program();
        let _task =
            program.handle_runtime_command(RuntimeCommand::Toast(ToastRequest::info("Saved")));

        let _task = program.update_core(CoreMessage::ToastTick(now + Duration::from_secs(5)));

        assert!(!program.core.toasts.has_visible());
    }

    #[test]
    fn toast_dismiss_message_removes_toast() {
        let now = Instant::now();
        let mut program = program();
        let id = program.core.toasts.push(ToastRequest::info("Saved"), now);

        let _task = program.update_core(CoreMessage::ToastDismiss(id));

        assert!(!program.core.toasts.has_visible());
    }

    #[test]
    fn toast_hover_pauses_expiry_and_resume_lets_it_expire() {
        let now = Instant::now();
        let mut program = program();
        let _id = program.core.toasts.push(ToastRequest::info("Saved"), now);

        let _task = program.update_core(CoreMessage::ToastHoverEntered);
        let _task = program.update_core(CoreMessage::ToastTick(now + Duration::from_secs(5)));

        assert!(
            program.core.toasts.has_visible(),
            "toast stays visible while hovered"
        );

        let _task = program.update_core(CoreMessage::ToastHoverLeft);
        let _task = program.update_core(CoreMessage::ToastTick(now + Duration::from_secs(9)));

        assert!(
            !program.core.toasts.has_visible(),
            "toast expires after hover ends"
        );
    }

    #[test]
    fn toast_host_decorates_app_view_when_toast_visible() {
        let now = Instant::now();
        let mut program = program();
        let _id = program.core.toasts.push(ToastRequest::info("Saved"), now);
        let main_id = main_window_id(&program);

        let _element: nive_ui::Element<'_, RuntimeMessage<TestApp>> = program.view(main_id);

        assert!(program.core.toasts.has_visible());
    }

    #[test]
    fn toast_and_dialog_coexist_in_app_view() {
        let now = Instant::now();
        let mut program = program();
        if let Some(app) = program.app.as_mut() {
            app.show_dialog = true;
        }
        let _id = program.core.toasts.push(ToastRequest::info("Saved"), now);
        let main_id = main_window_id(&program);

        let _element: nive_ui::Element<'_, RuntimeMessage<TestApp>> = program.view(main_id);

        assert!(program.core.toasts.has_visible());
    }

    #[test]
    fn auxiliary_window_view_skips_toast_decoration() {
        let now = Instant::now();
        let mut program = program();
        program.core.registry.set_opened(WindowHandle::auxiliary(
            TestWindow::Secondary,
            window::Id::unique(),
        ));
        let _id = program.core.toasts.push(ToastRequest::info("Saved"), now);
        let auxiliary_id = program
            .core
            .registry
            .first(TestWindow::Secondary)
            .map(|handle| handle.id)
            .unwrap_or_else(window::Id::unique);

        let _element: nive_ui::Element<'_, RuntimeMessage<TestApp>> = program.view(auxiliary_id);

        assert!(program.core.toasts.has_visible());
    }
}
