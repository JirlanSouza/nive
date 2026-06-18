use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use iced::{window, Subscription, Task};
use nive_ui::theme::{Theme, ThemePreference};

use crate::{
    AppUpdate, BootstrapSpec, ScreenView, ToastPosition, UserFacingError, WindowHandle, WindowRole,
    WindowSpec,
};

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    RunnerUnavailable,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RunnerUnavailable => formatter.write_str("Nive runner is not implemented yet"),
        }
    }
}

impl std::error::Error for Error {}

pub trait Application: Sized + 'static {
    type Message: Clone + Debug + 'static;
    type Window: Copy + Eq + Hash + Debug + 'static;
    type Bootstrap: Send + 'static;

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap>;

    fn init(
        context: Context<'_, Self::Window>,
        bootstrap: Self::Bootstrap,
    ) -> (Self, AppUpdate<Self::Message, Self::Window>);

    fn update(
        &mut self,
        context: Context<'_, Self::Window>,
        window: Option<WindowContext<Self::Window>>,
        message: Self::Message,
    ) -> AppUpdate<Self::Message, Self::Window>;

    fn view(
        &self,
        context: Context<'_, Self::Window>,
        window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message>;

    fn subscription(&self, _context: Context<'_, Self::Window>) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn shortcuts(&self, _context: Context<'_, Self::Window>) -> ShortcutMap<Self::Message> {
        ShortcutMap::new()
    }

    fn window_title<'a>(
        &'a self,
        context: Context<'a, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> Cow<'a, str> {
        Cow::Borrowed(context.app_name())
    }

    fn on_core_event(
        &mut self,
        _context: Context<'_, Self::Window>,
        _event: CoreEvent<Self::Window>,
    ) -> AppUpdate<Self::Message, Self::Window> {
        AppUpdate::none()
    }

    fn on_window_close_requested(
        &mut self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> CloseDecision<Self::Message> {
        CloseDecision::Close
    }

    fn on_exit_requested(
        &mut self,
        _context: Context<'_, Self::Window>,
    ) -> ExitDecision<Self::Message> {
        ExitDecision::Accept
    }
}

pub fn run<A: Application>() -> Result {
    let _ = PhantomData::<A>;
    Err(Error::RunnerUnavailable)
}

pub struct ApplicationConfig<K, B> {
    app_id: String,
    app_name: String,
    windows: Vec<WindowRegistration<K>>,
    initial_windows: Vec<K>,
    theme_preference: ThemePreference,
    toast_position: ToastPosition,
    bootstrap: Option<BootstrapSpec<B>>,
}

#[derive(Debug, Clone, Copy)]
pub struct WindowRegistration<K> {
    pub kind: K,
    pub spec: WindowSpec,
}

impl<K, B> ApplicationConfig<K, B> {
    pub fn new(app_id: impl Into<String>) -> Self {
        let app_id = app_id.into();
        Self {
            app_name: app_id.clone(),
            app_id,
            windows: Vec::new(),
            initial_windows: Vec::new(),
            theme_preference: ThemePreference::System,
            toast_position: ToastPosition::BottomRight,
            bootstrap: None,
        }
    }

    pub fn app_id(&self) -> &str {
        self.app_id.as_str()
    }

    pub fn app_name(&self) -> &str {
        self.app_name.as_str()
    }

    pub fn name(mut self, app_name: impl Into<String>) -> Self {
        self.app_name = app_name.into();
        self
    }

    pub fn window(mut self, kind: K, spec: WindowSpec) -> Self {
        self.windows.push(WindowRegistration { kind, spec });
        self
    }

    pub fn initial_window(mut self, kind: K) -> Self {
        self.initial_windows.push(kind);
        self
    }

    pub fn theme_preference(mut self, preference: ThemePreference) -> Self {
        self.theme_preference = preference;
        self
    }

    pub fn toast_position(mut self, position: ToastPosition) -> Self {
        self.toast_position = position;
        self
    }

    pub fn bootstrap(mut self, bootstrap: BootstrapSpec<B>) -> Self {
        self.bootstrap = Some(bootstrap);
        self
    }

    pub fn windows(&self) -> &[WindowRegistration<K>] {
        self.windows.as_slice()
    }

    pub fn initial_windows(&self) -> &[K] {
        self.initial_windows.as_slice()
    }

    pub fn initial_theme_preference(&self) -> ThemePreference {
        self.theme_preference
    }

    pub fn initial_toast_position(&self) -> ToastPosition {
        self.toast_position
    }

    pub fn bootstrap_spec(&self) -> Option<&BootstrapSpec<B>> {
        self.bootstrap.as_ref()
    }
}

#[derive(Clone, Copy)]
pub struct Context<'a, K> {
    app_id: &'a str,
    app_name: &'a str,
    theme: Theme,
    theme_preference: ThemePreference,
    windows: WindowQuery<'a, K>,
    exiting: bool,
}

impl<'a, K> Context<'a, K> {
    pub fn app_id(self) -> &'a str {
        self.app_id
    }

    pub fn app_name(self) -> &'a str {
        self.app_name
    }

    pub fn theme(self) -> Theme {
        self.theme
    }

    pub fn theme_preference(self) -> ThemePreference {
        self.theme_preference
    }

    pub fn windows(self) -> WindowQuery<'a, K> {
        self.windows
    }

    pub fn is_exiting(self) -> bool {
        self.exiting
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowContext<K> {
    pub id: window::Id,
    pub kind: K,
    pub role: WindowRole,
}

#[derive(Clone, Copy)]
pub struct WindowQuery<'a, K> {
    handles: &'a [WindowHandle<K>],
}

impl<'a, K> WindowQuery<'a, K>
where
    K: Copy + Eq,
{
    pub fn get(self, id: window::Id) -> Option<WindowContext<K>> {
        self.handles
            .iter()
            .find(|handle| handle.id == id)
            .map(|handle| WindowContext {
                id: handle.id,
                kind: handle.kind,
                role: handle.role,
            })
    }

    pub fn contains(self, kind: K) -> bool {
        self.handles.iter().any(|handle| handle.kind == kind)
    }

    pub fn first(self, kind: K) -> Option<WindowContext<K>> {
        self.handles
            .iter()
            .find(|handle| handle.kind == kind)
            .map(|handle| WindowContext {
                id: handle.id,
                kind: handle.kind,
                role: handle.role,
            })
    }

    pub fn all(self, kind: K) -> impl Iterator<Item = WindowContext<K>> + 'a {
        self.handles
            .iter()
            .filter(move |handle| handle.kind == kind)
            .map(|handle| WindowContext {
                id: handle.id,
                kind: handle.kind,
                role: handle.role,
            })
    }

    pub fn app_window_count(self) -> usize {
        self.handles
            .iter()
            .filter(|handle| handle.role == WindowRole::App)
            .count()
    }
}

#[derive(Debug)]
pub enum CloseDecision<M> {
    Close,
    Defer(Task<M>),
    Cancel,
}

#[derive(Debug)]
pub enum ExitDecision<M> {
    Accept,
    Defer(Task<M>),
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowCommand<K> {
    Open(K),
    Close(window::Id),
    CloseKind(K),
    Focus(window::Id),
    FocusKind(K),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreEvent<K> {
    WindowOpened(WindowContext<K>),
    WindowClosed(WindowContext<K>),
    WindowFocused(WindowContext<K>),
    LastAppWindowClosed,
    ThemeChanged(Theme),
    CommandRejected(CommandRejected<K>),
    PlatformError(PlatformError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRejected<K> {
    pub command: WindowCommand<K>,
    pub reason: CommandRejectionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandRejectionReason {
    MissingWindowSpec,
    MissingWindow,
    InvalidState,
    Exiting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformError {
    pub operation: &'static str,
    pub error: UserFacingError,
}

#[derive(Debug, Clone)]
pub struct ShortcutMap<M> {
    messages: Vec<M>,
}

impl<M> ShortcutMap<M> {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl<M> Default for ShortcutMap<M> {
    fn default() -> Self {
        Self::new()
    }
}
