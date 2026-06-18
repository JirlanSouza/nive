use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

use iced::{window, Font, Subscription, Task};
use nive_ui::theme::{Theme, ThemePreference};

use crate::{
    AppUpdate, BootstrapSpec, ScreenView, ToastPosition, UserFacingError, WindowRegistry,
    WindowRole, WindowSpec,
};

mod program;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    BootstrapUnavailable,
    Iced(iced::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BootstrapUnavailable => {
                formatter.write_str("bootstrap runtime is not implemented yet")
            }
            Self::Iced(error) => std::fmt::Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Iced(error) => Some(error),
            Self::BootstrapUnavailable => None,
        }
    }
}

impl From<iced::Error> for Error {
    fn from(error: iced::Error) -> Self {
        Self::Iced(error)
    }
}

pub trait Application: Sized + 'static {
    type Message: Clone + Debug + Send + 'static;
    type Window: Copy + Eq + Hash + Debug + Send + 'static;
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
    program::run::<A>()
}

pub struct ApplicationConfig<K, B> {
    app_id: String,
    app_name: String,
    windows: Vec<WindowRegistration<K>>,
    initial_windows: Vec<K>,
    theme_preference: ThemePreference,
    toast_position: ToastPosition,
    bootstrap: Option<BootstrapSpec<B>>,
    immediate_bootstrap: Option<B>,
    fonts: Vec<Cow<'static, [u8]>>,
    default_font: Font,
    window_icon: Option<window::Icon>,
}

#[derive(Debug, Clone, Copy)]
pub struct WindowRegistration<K> {
    pub kind: K,
    pub spec: WindowSpec,
}

impl<K> ApplicationConfig<K, ()> {
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
            immediate_bootstrap: Some(()),
            fonts: Vec::new(),
            default_font: Font::DEFAULT,
            window_icon: None,
        }
    }
}

impl<K, B> ApplicationConfig<K, B> {
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

    pub fn bootstrap<T>(self, bootstrap: BootstrapSpec<T>) -> ApplicationConfig<K, T> {
        ApplicationConfig {
            app_id: self.app_id,
            app_name: self.app_name,
            windows: self.windows,
            initial_windows: self.initial_windows,
            theme_preference: self.theme_preference,
            toast_position: self.toast_position,
            bootstrap: Some(bootstrap),
            immediate_bootstrap: None,
            fonts: self.fonts,
            default_font: self.default_font,
            window_icon: self.window_icon,
        }
    }

    pub fn font(mut self, font: impl Into<Cow<'static, [u8]>>) -> Self {
        self.fonts.push(font.into());
        self
    }

    pub fn default_font(mut self, font: Font) -> Self {
        self.default_font = font;
        self
    }

    pub fn window_icon(mut self, icon: window::Icon) -> Self {
        self.window_icon = Some(icon);
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

    pub fn fonts(&self) -> &[Cow<'static, [u8]>] {
        self.fonts.as_slice()
    }

    pub fn configured_default_font(&self) -> Font {
        self.default_font
    }

    pub fn configured_window_icon(&self) -> Option<&window::Icon> {
        self.window_icon.as_ref()
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
    registry: &'a WindowRegistry<K>,
}

impl<'a, K> WindowQuery<'a, K>
where
    K: Copy + Eq,
{
    pub fn get(self, id: window::Id) -> Option<WindowContext<K>> {
        self.registry.get(id).map(|handle| WindowContext {
            id: handle.id,
            kind: handle.kind,
            role: handle.role,
        })
    }

    pub fn contains(self, kind: K) -> bool {
        self.registry.contains(kind)
    }

    pub fn first(self, kind: K) -> Option<WindowContext<K>> {
        self.registry.first(kind).map(|handle| WindowContext {
            id: handle.id,
            kind: handle.kind,
            role: handle.role,
        })
    }

    pub fn all(self, kind: K) -> impl Iterator<Item = WindowContext<K>> + 'a {
        self.registry.all(kind).map(|handle| WindowContext {
            id: handle.id,
            kind: handle.kind,
            role: handle.role,
        })
    }

    pub fn app_window_count(self) -> usize {
        self.registry.app_window_count()
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
