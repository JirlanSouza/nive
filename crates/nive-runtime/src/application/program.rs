use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use iced::keyboard;
use iced::{window, Point, Size, Task};

use super::{
    Application, ApplicationConfig, CommandRejected, Error, MessageSource, Result,
    WindowRegistration,
};
#[cfg(feature = "devtools")]
use crate::devtools::probe::{NoProbe, ProbeCatalogEntry};
#[cfg(feature = "devtools")]
use crate::devtools::DevtoolsPanelMessage;
#[cfg(feature = "devtools")]
use crate::devtools::{DevtoolsConfig, DevtoolsHostState};
use crate::lifecycle::bootstrap::BootstrapController;
use crate::{
    BootstrapSpec, KeyboardNavigation, RuntimeSession, SettingsConfig, SettingsError,
    ThemeController, ThemeEvent, ToastId, ToastInsets, ToastPosition, ToastState, UserFacingResult,
    WindowRegistry, WindowSpec,
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

mod bootstrap;
mod core_runtime;
mod devtools;
mod dispatch;
mod lifecycle;
mod session;
mod shortcuts;
mod windows;

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

#[cfg(test)]
mod tests;
