use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use iced::keyboard;
use iced::{window, ContentFit, Point, Size, Subscription, Task};

use super::update::RuntimeCommand;
use super::{
    Application, ApplicationConfig, CloseDecision, CommandRejected, CommandRejectionReason,
    Context, Effect, Error, ExitDecision, MessageContext, MessageSource, Result, RuntimeEvent,
    WindowCommand, WindowContext, WindowQuery, WindowRegistration,
};
#[cfg(feature = "devtools")]
use crate::devtools::probe::{NoProbe, ProbeCatalogEntry};
#[cfg(feature = "devtools")]
use crate::devtools::{DevtoolsConfig, DevtoolsHostState, DevtoolsWindowSpec};
#[cfg(feature = "devtools")]
use crate::devtools::{DevtoolsPanelEffect, DevtoolsPanelMessage};
use crate::input::{action_message_for_event, shortcut_message_for_event};
use crate::lifecycle::bootstrap::{
    minimum_duration_task, BootstrapController, BootstrapTransition,
};
use crate::lifecycle::WindowLifecycle;
use crate::{
    ActionMap, BootstrapSpec, DialogRequest, KeyboardNavigation, RuntimeSession, ScreenView,
    SettingsConfig, SettingsError, SettingsErrorKind, ShortcutMap, ThemeController, ThemeEvent,
    ToastId, ToastInsets, ToastItem, ToastPosition, ToastState, UserFacingResult,
    WindowCardinality, WindowChrome, WindowHandle, WindowMode, WindowRegistry, WindowRole,
    WindowSpec,
};

#[cfg(not(feature = "devtools"))]
mod no_devtools_probe {
    pub trait ProbeCatalogEntry: Clone + Send + 'static {}

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NoProbe {}

    impl ProbeCatalogEntry for NoProbe {}
}

#[cfg(not(feature = "devtools"))]
use no_devtools_probe::{NoProbe, ProbeCatalogEntry};

const TOAST_TICK_INTERVAL: Duration = Duration::from_millis(500);

/// Auto-registers a single [`WindowSpec::app()`] when `A::Window = ()` and
/// `ApplicationConfig` has no explicit `.window(...)` calls. Apps with custom
/// `type Window` enums must declare windows in `Application::config()`.
///
/// Detection uses `TypeId` so that the explicit `()` defaulting flow stays
/// generic-friendly without unstable specialization.
fn auto_register_default_window<A>(config: &mut ApplicationConfig<A::Window, A::Bootstrap>)
where
    A: Application,
{
    use std::any::TypeId;

    if !config.windows.is_empty() || !config.initial_windows.is_empty() {
        return;
    }

    if TypeId::of::<A::Window>() != TypeId::of::<()>() {
        return;
    }

    // SAFETY: We only enter this branch when `A::Window = ()`, verified above by
    // `TypeId` equality. `()` is a zero-sized type, so `transmute_copy` reads no
    // bytes and the resulting value is the canonical `()`.
    let default_kind = unsafe { std::mem::transmute_copy::<(), A::Window>(&()) };
    config.windows.push(WindowRegistration {
        kind: default_kind,
        spec: WindowSpec::app(),
    });
    config.initial_windows.push(default_kind);
}

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
    let devtools = DevtoolsRuntime::<A>::new(DevtoolsConfig::from_env());
    run_inner::<A, NoProbe>(Some(devtools))
}

#[cfg(feature = "devtools")]
fn run_inner<A, P>(devtools: Option<DevtoolsRuntime<A>>) -> Result
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

    for font in nive_ui::fonts::bundled() {
        daemon = daemon.font(*font);
    }

    for font in fonts {
        daemon = daemon.font(font);
    }

    daemon.run().map_err(Error::from)
}

struct Program<A: Application, P: ProbeCatalogEntry = NoProbe> {
    core: NiveCore<A::Window, A::Message>,
    app: Option<A>,
    bootstrap: Option<BootstrapRuntime<A::Bootstrap>>,
    #[cfg(feature = "devtools")]
    devtools: Option<DevtoolsRuntime<A>>,
    _probe: PhantomData<P>,
}

#[cfg(feature = "devtools")]
struct DevtoolsRuntime<A: Application> {
    host: DevtoolsHostState,
    cached_snapshot: crate::devtools::DevtoolStateSnapshot,
    window_id: Option<window::Id>,
    start_open: bool,
    config: DevtoolsConfig,
    collect: fn(&mut A) -> crate::devtools::DevtoolStateSnapshot,
    apply: fn(&mut A, &str, &crate::devtools::SimulateAction) -> crate::devtools::SimulateResult,
}

#[cfg(feature = "devtools")]
impl<A: crate::devtools::DevtoolsApp> DevtoolsRuntime<A> {
    fn new(config: DevtoolsConfig) -> Self {
        Self {
            host: DevtoolsHostState::disabled(),
            cached_snapshot: Default::default(),
            window_id: None,
            start_open: config.enabled(),
            config,
            collect: crate::devtools::collect_snapshot,
            apply: crate::devtools::apply_simulate,
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
        source: MessageSource,
        message: M,
    },
    #[cfg(feature = "devtools")]
    Devtools(DevtoolsPanelMessage),
    Probe(std::marker::PhantomData<P>),
}

#[derive(Debug, Clone)]
enum CoreMessage<K> {
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    WindowFocused(window::Id),
    WindowUnfocused(window::Id),
    WindowMoved(window::Id, Point),
    WindowResized(window::Id, Size),
    WindowCloseRequested(window::Id),
    Theme(ThemeEvent),
    ConfirmClose(window::Id),
    ConfirmExit,
    Rejected(CommandRejected<K>),
    ToastDismiss(ToastId),
    ToastHoverEntered,
    ToastHoverLeft,
    ToastFocusWithinEntered,
    ToastFocusWithinLeft,
    ToastTick(Instant),
    /// Published by the window's `FocusRoot` when modal activity changes,
    /// aggregating every open session of the shared modal kernel (`Dialog`,
    /// `CommandPalette`, and any future consumer).
    ModalActive(bool),
    KeyboardNavigation(KeyboardNavigation),
    KeyboardEvent(keyboard::Event),
    SettingsSaved(std::result::Result<(), SettingsError>),
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
            Self::App {
                window_id,
                source,
                message,
            } => Self::App {
                window_id: *window_id,
                source: *source,
                message: message.clone(),
            },
            #[cfg(feature = "devtools")]
            Self::Devtools(message) => Self::Devtools(message.clone()),
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

struct NiveCore<K, Message> {
    app_id: String,
    app_name: String,
    windows: Vec<(K, WindowSpec)>,
    initial_windows: Vec<K>,
    registry: WindowRegistry<K>,
    theme: ThemeController,
    exiting: bool,
    window_icon: Option<window::Icon>,
    toast_position: ToastPosition,
    toast_insets: ToastInsets,
    toasts: ToastState<Message>,
    pending_app_closes: HashSet<window::Id>,
    /// Pending `WindowCommand::Replace` handoffs, keyed by the opening
    /// replacement target's window id and mapping to the `current` window id
    /// that should close once the target finishes opening (or is rejected).
    pending_replacements: HashMap<window::Id, window::Id>,
    settings: Option<SettingsRuntime>,
}

#[derive(Debug, Clone)]
struct SettingsRuntime {
    config: SettingsConfig,
    session: RuntimeSession,
}

impl<A, P> Program<A, P>
where
    A: Application,
    P: ProbeCatalogEntry,
{
    fn new(
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

    fn update(&mut self, message: RuntimeMessage<A, P>) -> Task<RuntimeMessage<A, P>> {
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

    fn view(&self, window_id: window::Id) -> nive_ui::Element<'_, RuntimeMessage<A, P>> {
        nive_ui::accessibility::FocusRoot::new(self.window_content(window_id))
            .on_modal_change(|active| NiveMessage::Core(CoreMessage::ModalActive(active)))
            .into()
    }

    fn window_content(&self, window_id: window::Id) -> nive_ui::Element<'_, RuntimeMessage<A, P>> {
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
                self.app.as_ref().map(|app| {
                    app.window_title(self.core.context(), window)
                        .into()
                        .into_owned()
                })
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

    #[cfg(feature = "devtools")]
    fn initialize_devtools(&mut self) -> Task<RuntimeMessage<A, P>> {
        if self.devtools.is_none() {
            return Task::none();
        }

        let config = self.devtools.as_ref().unwrap().config;
        let panel = crate::devtools::DevtoolsPanelState::new().with_config(config);

        let (devtools_opt, app_opt) = (&mut self.devtools, &mut self.app);
        let devtools = devtools_opt.as_mut().unwrap();
        devtools.host = DevtoolsHostState::new(Some(panel));
        if let Some(app) = app_opt.as_mut() {
            let collect = devtools.collect;
            devtools.cached_snapshot = collect(app);
        }

        let start_open = {
            let devtools = self.devtools.as_mut().unwrap();
            let v = devtools.start_open;
            devtools.start_open = false;
            v
        };

        if start_open {
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

        devtools_window(panel, &devtools.cached_snapshot, NiveMessage::Devtools)
    }

    #[cfg(feature = "devtools")]
    fn update_devtools(&mut self, message: DevtoolsPanelMessage) -> Task<RuntimeMessage<A, P>> {
        let effect = {
            let Some(devtools) = self.devtools.as_mut() else {
                return Task::none();
            };
            devtools.host.update(message)
        };
        let Some(DevtoolsPanelEffect::Simulate { path, action }) = effect else {
            return Task::none();
        };

        let is_resource = self
            .devtools
            .as_ref()
            .and_then(|d| d.cached_snapshot.entries.iter().find(|e| e.path == path))
            .map(|e| e.is_resource())
            .unwrap_or(true);

        let (devtools_opt, app_opt) = (&mut self.devtools, &mut self.app);
        let (Some(devtools), Some(app)) = (devtools_opt.as_mut(), app_opt.as_mut()) else {
            return Task::none();
        };
        let collect = devtools.collect;
        let apply = devtools.apply;
        let result = apply(app, &path, &action);
        devtools.cached_snapshot = collect(app);
        let new_entries = devtools.cached_snapshot.entries.clone();

        if let Some(panel) = devtools.host.panel_mut() {
            panel.entries = new_entries;
            panel.record_simulate_result(&path, is_resource, result);
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

                let opened = self.emit_runtime_event(RuntimeEvent::WindowOpened(handle.into()));
                let Some(current_id) = self.core.pending_replacements.remove(&window_id) else {
                    return opened;
                };

                opened.chain(self.request_close(current_id))
            }
            CoreMessage::WindowClosed(window_id) => {
                self.core.pending_app_closes.remove(&window_id);
                // Clears a pending `Replace` handoff if its target closes
                // before ever finishing opening.
                self.core.pending_replacements.remove(&window_id);
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

                let closed = self.emit_runtime_event(RuntimeEvent::WindowClosed(handle.into()));
                if handle.role == WindowRole::App && self.core.registry.app_window_count() == 0 {
                    let last_closed = self.emit_runtime_event(RuntimeEvent::LastAppWindowClosed);
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
                self.core.toasts.set_window_active(true, Instant::now());

                self.emit_runtime_event(RuntimeEvent::WindowFocused(handle.into()))
            }
            CoreMessage::WindowUnfocused(window_id) => {
                if self.is_bootstrap_window(window_id) || self.is_devtools_window(window_id) {
                    return Task::none();
                }
                self.core.toasts.set_window_active(false, Instant::now());
                Task::none()
            }
            CoreMessage::WindowMoved(window_id, position) => {
                self.save_window_position(window_id, position)
            }
            CoreMessage::WindowResized(window_id, size) => self.save_window_size(window_id, size),
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
                let system_changed = self.core.theme.handle(event);
                if self.app.is_some() {
                    let resolved = self.resolve_theme_preference(None);
                    let effective_changed = self.core.theme.set_preference(resolved);
                    let event_task = if effective_changed {
                        self.emit_runtime_event(RuntimeEvent::ThemeChanged(
                            self.core.theme.effective(),
                        ))
                    } else {
                        Task::none()
                    };
                    return event_task;
                }
                if system_changed {
                    self.emit_runtime_event(RuntimeEvent::ThemeChanged(self.core.theme.effective()))
                } else {
                    Task::none()
                }
            }
            CoreMessage::ConfirmClose(window_id) => window::close(window_id),
            CoreMessage::ConfirmExit => self.accept_exit(),
            CoreMessage::Rejected(rejection) => {
                self.emit_runtime_event(RuntimeEvent::CommandRejected(rejection))
            }
            CoreMessage::ToastDismiss(id) => {
                self.core.toasts.dismiss(id, Instant::now());
                Task::none()
            }
            CoreMessage::ToastHoverEntered => {
                self.core.toasts.set_hover(true, Instant::now());
                Task::none()
            }
            CoreMessage::ToastHoverLeft => {
                self.core.toasts.set_hover(false, Instant::now());
                Task::none()
            }
            CoreMessage::ToastFocusWithinEntered => {
                self.core.toasts.set_focus_within(true, Instant::now());
                Task::none()
            }
            CoreMessage::ToastFocusWithinLeft => {
                self.core.toasts.set_focus_within(false, Instant::now());
                Task::none()
            }
            CoreMessage::ToastTick(now) => {
                self.core.toasts.tick(now);
                Task::none()
            }
            CoreMessage::ModalActive(active) => {
                self.core.toasts.set_modal_active(active, Instant::now());
                Task::none()
            }
            CoreMessage::KeyboardNavigation(navigation) => navigation.task(),
            CoreMessage::KeyboardEvent(event) => self.handle_keyboard_event(event),
            CoreMessage::SettingsSaved(result) => {
                if let Err(error) = result {
                    log::warn!(
                        target: "nive_runtime::settings",
                        "settings.save_failed path={} error={}",
                        error.path().display(),
                        error.detail()
                    );
                }
                Task::none()
            }
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
        update: impl Into<Effect<A::Message, A::Window>>,
    ) -> (RuntimeTask<A, P>, RuntimeTask<A, P>) {
        let (task, commands) = update.into().into_parts();
        let app_task = task.map(|message| NiveMessage::App {
            window_id: None,
            source: MessageSource::Task,
            message,
        });
        let runtime_task = commands.into_iter().fold(Task::none(), |task, command| {
            task.chain(self.handle_runtime_command(command, None))
        });

        (app_task, runtime_task)
    }

    fn apply_update(
        &mut self,
        update: impl Into<Effect<A::Message, A::Window>>,
    ) -> Task<RuntimeMessage<A, P>> {
        self.apply_update_from(update, None)
    }

    fn apply_update_from(
        &mut self,
        update: impl Into<Effect<A::Message, A::Window>>,
        origin_window: Option<window::Id>,
    ) -> Task<RuntimeMessage<A, P>> {
        let (task, commands) = update.into().into_parts();
        let app_task = task.map(|message| NiveMessage::App {
            window_id: None,
            source: MessageSource::Task,
            message,
        });
        let runtime_task = commands.into_iter().fold(Task::none(), |task, command| {
            task.chain(self.handle_runtime_command(command, origin_window))
        });

        Task::batch([app_task, runtime_task])
    }

    fn handle_runtime_command(
        &mut self,
        command: RuntimeCommand<A::Message, A::Window>,
        origin_window: Option<window::Id>,
    ) -> Task<RuntimeMessage<A, P>> {
        match command {
            RuntimeCommand::Toast(toast) => {
                self.core.toasts.push(toast, Instant::now(), origin_window);
                Task::none()
            }
            RuntimeCommand::Window(command) => self.handle_window_command(command),
            RuntimeCommand::Theme(preference) => {
                let preference_changed = self.core.theme.preference() != preference;
                // Persist the emitted preference so that future restarts and
                // `Application::theme` calls returning `System` re-resolve to it.
                let save_task = if preference_changed {
                    self.save_theme_preference(preference)
                } else {
                    Task::none()
                };
                // Consult `Application::theme` to apply the tie-break rule.
                let resolved = self.resolve_theme_preference(Some(preference));
                let effective_changed = self.core.theme.set_preference(resolved);
                let event_task = if effective_changed {
                    self.emit_runtime_event(RuntimeEvent::ThemeChanged(self.core.theme.effective()))
                } else {
                    Task::none()
                };

                Task::batch([event_task, save_task])
            }
            RuntimeCommand::Exit => self.request_exit(),
        }
    }

    /// Resolves the theme preference after consulting `Application::theme`
    /// using the documented tie-break rule:
    /// - `Light`/`Dark` from `Application::theme` wins.
    /// - `System` (or no app yet) defers to `emitted` if `Some`, else the
    ///   runtime's current preference (already set by the OS handler or
    ///   settings).
    fn resolve_theme_preference(
        &self,
        emitted: Option<crate::ThemePreference>,
    ) -> crate::ThemePreference {
        let context = self.core.context();
        let window = self.first_app_window_context();
        let app_pref = self.app.as_ref().map(|app| app.theme(context, window));
        match app_pref {
            Some(crate::ThemePreference::Light) | Some(crate::ThemePreference::Dark) => {
                app_pref.unwrap()
            }
            _ => emitted.unwrap_or_else(|| self.core.theme.preference()),
        }
    }

    fn first_app_window_context(&self) -> Option<WindowContext<A::Window>> {
        self.core
            .registry
            .most_recent_app_window()
            .map(|handle| WindowContext {
                id: handle.id,
                kind: handle.kind,
                role: handle.role,
            })
    }

    fn save_theme_preference(
        &mut self,
        preference: crate::ThemePreference,
    ) -> Task<RuntimeMessage<A, P>> {
        let Some(settings) = self.core.settings.as_mut() else {
            return Task::none();
        };

        settings.session.set_theme_preference(Some(preference));
        self.save_runtime_session()
    }

    fn save_window_size(
        &mut self,
        window_id: window::Id,
        size: Size,
    ) -> Task<RuntimeMessage<A, P>> {
        let Some(key) = self.core.window_session_key(window_id) else {
            return Task::none();
        };
        let Some(settings) = self.core.settings.as_mut() else {
            return Task::none();
        };

        if settings.session.set_window_size(key, size) {
            self.save_runtime_session()
        } else {
            Task::none()
        }
    }

    fn save_window_position(
        &mut self,
        window_id: window::Id,
        position: Point,
    ) -> Task<RuntimeMessage<A, P>> {
        let Some(key) = self.core.window_session_key(window_id) else {
            return Task::none();
        };
        let Some(settings) = self.core.settings.as_mut() else {
            return Task::none();
        };

        if settings.session.set_window_position(key, position) {
            self.save_runtime_session()
        } else {
            Task::none()
        }
    }

    fn save_runtime_session(&self) -> Task<RuntimeMessage<A, P>> {
        let Some(settings) = self.core.settings.as_ref() else {
            return Task::none();
        };

        let config = settings.config.clone();
        let session = settings.session.clone();

        Task::perform(
            async move { crate::settings::save_session(&config, &session) },
            |result| NiveMessage::Core(CoreMessage::SettingsSaved(result)),
        )
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
            WindowCommand::CloseAllKind(kind) => {
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
            WindowCommand::Replace { current, next } => self.replace_window(current, next),
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
            if let Some(existing) = self.core.registry.latest(kind) {
                self.core.registry.set_focused(existing.id);
                return window::gain_focus(existing.id);
            }
            if self.core.registry.contains(kind) {
                return Task::none();
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

    /// Opens `next` and closes `current` only once `next` has actually
    /// become available (opened, or an existing single-cardinality instance
    /// focused). `current` stays open and registered for the entire duration
    /// of a pending open, so a rejected or still-opening `next` never leaves
    /// the app without a window.
    fn replace_window(
        &mut self,
        current: window::Id,
        next: A::Window,
    ) -> Task<RuntimeMessage<A, P>> {
        let command = WindowCommand::Replace { current, next };

        if self.core.exiting {
            return self.reject(command, CommandRejectionReason::Exiting);
        }
        let Some(current_handle) = self.core.registry.get(current) else {
            return self.reject(command, CommandRejectionReason::MissingWindow);
        };
        if current_handle.role != WindowRole::App
            || self.core.registry.lifecycle(current) != Some(WindowLifecycle::Open)
        {
            return self.reject(command, CommandRejectionReason::InvalidState);
        }
        let Some(spec) = self.core.window_spec(next) else {
            return self.reject(command, CommandRejectionReason::MissingWindowSpec);
        };
        if spec.role != WindowRole::App {
            return self.reject(command, CommandRejectionReason::InvalidState);
        }

        if spec.cardinality == WindowCardinality::Single {
            if let Some(existing) = self.core.registry.latest(next) {
                if existing.id == current {
                    return self.reject(command, CommandRejectionReason::InvalidState);
                }
                self.core.registry.set_focused(existing.id);
                return window::gain_focus(existing.id).chain(self.request_close(current));
            }
            if let Some(opening) = self.core.registry.opening(next) {
                self.core.pending_replacements.insert(opening.id, current);
                return Task::none();
            }
        }

        let (next_id, task) = window::open(spec.settings(self.core.window_icon.clone()));
        self.core.registry.set_opening(WindowHandle {
            kind: next,
            id: next_id,
            role: spec.role,
        });
        self.core.pending_replacements.insert(next_id, current);

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
                    source: MessageSource::Task,
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
            ExitDecision::Exit => self.accept_exit(),
            ExitDecision::Cancel => Task::none(),
            ExitDecision::Defer(task) => {
                self.core.exiting = true;
                task.map(|message| NiveMessage::App {
                    window_id: None,
                    source: MessageSource::Task,
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

    fn emit_runtime_event(&mut self, event: RuntimeEvent<A::Window>) -> Task<RuntimeMessage<A, P>> {
        let update: Effect<A::Message, A::Window> = {
            let Some(app) = self.app.as_mut() else {
                return Task::none();
            };
            let context = self.core.context();
            app.on_runtime_event(context, event).into()
        };
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
        let Some(app) = self.app.as_ref() else {
            return Task::none();
        };
        let context = self.core.context();
        let actions = app.actions(context);
        let shortcuts = app.shortcuts(context);

        shortcut_message_from_event::<A, P>(&actions, &shortcuts, event)
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
    actions: &ActionMap<A::Message>,
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

    action_message_for_event(actions, &event)
        .or_else(|| shortcut_message_for_event(shortcuts, &event))
        .map(|message| NiveMessage::App {
            window_id: None,
            source: MessageSource::Action,
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

fn clamp_window_size(size: Size, min_size: Option<Size>, max_size: Option<Size>) -> Size {
    let min_width = min_size.map(|size| size.width).unwrap_or(1.0);
    let min_height = min_size.map(|size| size.height).unwrap_or(1.0);
    let max_width = max_size
        .map(|size| size.width)
        .unwrap_or(f32::MAX)
        .max(min_width);
    let max_height = max_size
        .map(|size| size.height)
        .unwrap_or(f32::MAX)
        .max(min_height);

    Size::new(
        size.width.clamp(min_width, max_width),
        size.height.clamp(min_height, max_height),
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
        session_key: None,
    }
    .settings(icon)
}

impl<K, Message> NiveCore<K, Message>
where
    K: Copy + Eq,
{
    fn new<B>(config: &ApplicationConfig<K, B>, settings: Option<SettingsRuntime>) -> Self {
        let core = Self {
            app_id: config.app_id.clone(),
            app_name: config.app_name.clone(),
            windows: config
                .windows
                .iter()
                .map(|registration| (registration.kind, registration.spec))
                .collect(),
            initial_windows: config.initial_windows.clone(),
            registry: WindowRegistry::new(),
            theme: ThemeController::with_catalog(config.theme_preference, config.theme_catalog),
            exiting: false,
            window_icon: config.window_icon.clone(),
            toast_position: config.toast_position,
            toast_insets: config.toast_insets,
            toasts: ToastState::default(),
            pending_app_closes: HashSet::new(),
            pending_replacements: HashMap::new(),
            settings,
        };
        core.report_duplicate_window_session_keys();
        core
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
            .map(|(_, spec)| self.apply_window_session(*spec))
    }

    fn apply_window_session(&self, mut spec: WindowSpec) -> WindowSpec {
        let Some(key) = spec.configured_session_key() else {
            return spec;
        };
        let Some(session) = self
            .settings
            .as_ref()
            .and_then(|settings| settings.session.window(key))
        else {
            return spec;
        };

        if let Some(size) = session.size() {
            spec.size = clamp_window_size(size, spec.min_size, spec.max_size);
        }
        if let Some(position) = session.position() {
            spec.position = window::Position::Specific(position);
        }

        spec
    }

    fn window_session_key(&self, window_id: window::Id) -> Option<&'static str> {
        let kind = self.registry.kind(window_id)?;
        self.windows
            .iter()
            .find(|(registered_kind, _)| *registered_kind == kind)
            .and_then(|(_, spec)| spec.configured_session_key())
    }

    fn toast_position(&self) -> ToastPosition {
        self.toast_position
    }

    fn toast_insets(&self) -> ToastInsets {
        self.toast_insets
    }

    fn effective_app_window_count(&self) -> usize {
        self.registry
            .handles()
            .filter(|handle| {
                handle.role == WindowRole::App && !self.pending_app_closes.contains(&handle.id)
            })
            .count()
    }

    fn report_duplicate_window_session_keys(&self) {
        let mut seen = Vec::new();
        for (_, spec) in &self.windows {
            let Some(key) = spec.configured_session_key() else {
                continue;
            };
            if seen.contains(&key) {
                log::warn!(
                    target: "nive_runtime::settings",
                    "settings.duplicate_window_key key={key}"
                );
            } else {
                seen.push(key);
            }
        }
    }
}

impl SettingsRuntime {
    fn load(config: Option<&SettingsConfig>) -> Option<Self> {
        let config = config?.clone();
        let session = match crate::settings::load_session(&config) {
            Ok(Some(session)) => session,
            Ok(None) => {
                log::debug!(
                    target: "nive_runtime::settings",
                    "settings.load_missing path={}",
                    config.path().display()
                );
                RuntimeSession::default()
            }
            Err(error) if error.kind() == SettingsErrorKind::UnsupportedVersion => {
                log::warn!(
                    target: "nive_runtime::settings",
                    "settings.unsupported_version path={} version={}",
                    error.path().display(),
                    error.detail()
                );
                RuntimeSession::default()
            }
            Err(error) => {
                log::warn!(
                    target: "nive_runtime::settings",
                    "settings.load_failed path={} error={}",
                    error.path().display(),
                    error.detail()
                );
                RuntimeSession::default()
            }
        };

        Some(Self { config, session })
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
mod tests;
