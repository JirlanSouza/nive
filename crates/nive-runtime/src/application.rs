use std::{borrow::Cow, fmt::Debug, hash::Hash};

use iced::Subscription;

use crate::actions::ActionMap;
use crate::input::ShortcutMap;
use crate::ScreenView;
use crate::ThemePreference;

mod config;
mod context;
mod event;
mod program;
mod task;
mod theme;
mod update;

pub use crate::lifecycle::{
    CloseDecision, CommandRejected, CommandRejectionReason, ExitDecision, WindowCommand,
};
pub use config::{ApplicationConfig, WindowRegistration};
pub use context::{Context, WindowContext, WindowQuery};
pub use event::CoreEvent;
pub use task::client_task;
pub use theme::{ThemeController, ThemeEvent};
pub use update::{AppUpdate, Never, RuntimeCommand, Update};

/// Convenience [`Result`] for runtime entry points.
pub type Result<T = ()> = std::result::Result<T, Error>;

/// Errors returned by the runtime entry points (`run` / `run_with_devtools`).
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The application did not provide bootstrap configuration.
    BootstrapUnavailable,
    /// An error surfaced from the underlying Iced runtime.
    Iced(iced::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BootstrapUnavailable => {
                formatter.write_str("application bootstrap configuration is unavailable")
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

/// The stable product contract implemented by every Nive desktop application.
///
/// An `Application` owns its state, message type, window enum, and bootstrap
/// result. The runtime drives the application through [`Application::init`],
/// [`Application::update`], and [`Application::view`], while optional hooks
/// provide subscriptions, keyboard shortcuts, window titles, and lifecycle
/// decisions. The private Iced program runner sits behind [`run`].
///
/// Applications stay domain-agnostic at the framework boundary: product
/// clients and services are constructed during bootstrap and transferred into
/// [`Application::init`]; the runtime itself never depends on app-domain types.
pub trait Application: Sized + 'static {
    /// The application's top-level message type, produced by views and
    /// subscriptions and consumed by [`Application::update`].
    type Message: Clone + Debug + Send + 'static;
    /// The enum identifying the application's logical windows.
    ///
    /// Single-window apps set `type Window = ();` (the unit marker) and let
    /// the runtime auto-register one [`WindowSpec::app()`] via
    /// [`ApplicationConfig::new`]. Multi-window apps declare an `enum Window`
    /// and call [`ApplicationConfig::window`] for each variant.
    ///
    /// [`WindowSpec::app()`]: crate::WindowSpec::app
    /// [`ApplicationConfig::new`]: ApplicationConfig::new
    /// [`ApplicationConfig::window`]: ApplicationConfig::window
    type Window: Copy + Eq + Hash + Debug + Send + 'static;
    /// The value produced by a successful bootstrap, handed to
    /// [`Application::init`]. Carries product clients and services out of the
    /// bootstrap task and into application state.
    ///
    /// Apps without a splash set `type Bootstrap = ();` and let
    /// [`ApplicationConfig::new`] supply an instant `()` bootstrap.
    type Bootstrap: Send + 'static;

    /// Returns the static application configuration: registered windows,
    /// bootstrap spec, and brand assets.
    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap>;

    /// Constructs the initial application state from a successful bootstrap.
    ///
    /// Receives the runtime [`Context`] and the bootstrap result, and returns
    /// the initial state together with the first [`AppUpdate`]. Returning `()`
    /// is equivalent to [`AppUpdate::none()`] when no side effects are needed.
    fn init(
        context: Context<'_, Self::Window>,
        bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<AppUpdate<Self::Message, Self::Window>>);

    /// Applies a message, mutating application state and returning the
    /// [`AppUpdate`] describing follow-up tasks and runtime effects.
    ///
    /// Returning `()` is equivalent to [`AppUpdate::none()`] when no side
    /// effects are needed.
    fn update(
        &mut self,
        context: Context<'_, Self::Window>,
        window: Option<WindowContext<Self::Window>>,
        message: Self::Message,
    ) -> impl Into<AppUpdate<Self::Message, Self::Window>>;

    /// Renders the content of a window as a [`ScreenView`].
    fn view(
        &self,
        context: Context<'_, Self::Window>,
        window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message>;

    /// Returns the event subscription for the application. Defaults to none.
    fn subscription(&self, _context: Context<'_, Self::Window>) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Returns the app's command/action catalog. Defaults to none.
    fn actions(&self, _context: Context<'_, Self::Window>) -> ActionMap<Self::Message> {
        ActionMap::new()
    }

    /// Returns the keyboard shortcut map for the application. Defaults to none.
    fn shortcuts(&self, _context: Context<'_, Self::Window>) -> ShortcutMap<Self::Message> {
        ShortcutMap::new()
    }

    /// Returns the title for a window. Defaults to the application name.
    ///
    /// Accepts any `impl Into<Cow<'a, str>>`, so implementations can return
    /// `&'static str` (common case) or `String` (dynamic case) without
    /// importing `std::borrow::Cow`.
    fn window_title<'a>(
        &'a self,
        context: Context<'a, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> impl Into<Cow<'a, str>> + 'a {
        Cow::Borrowed(context.app_name())
    }

    /// Returns the application's preferred theme. Defaults to
    /// [`ThemePreference::System`], which preserves the pre-change single
    /// global-theme flow: the OS theme and the persisted preference (loaded
    /// from settings) resolve the active [`crate::Theme`].
    ///
    /// The runtime consults this method at app initialization, on
    /// [`AppUpdate::theme`] emissions, and on OS theme changes. The runtime
    /// applies the following tie-break rule when both this method returns and
    /// `AppUpdate::theme(pref)` is emitted:
    ///
    /// - `Light` or `Dark` returned from this method **wins**; the emitted
    ///   preference is recorded to settings only (for restarts) but does not
    ///   change the active theme.
    /// - `System` returned from this method **defers** to the OS theme and the
    ///   emitted/per persisted preference (the emitted value becomes the
    ///   persisted preference and the OS resolves Light or Dark).
    ///
    /// The `window` argument is `None` when the runtime has no open windows
    /// (e.g., during initial bootstrap before the first window is opened).
    /// Otherwise, it passes the [`WindowContext`] of the most-recently
    /// focused app window — useful for apps that want per-window theme logic
    /// in future revisions.
    fn theme(
        &self,
        _context: Context<'_, Self::Window>,
        _window: Option<WindowContext<Self::Window>>,
    ) -> ThemePreference {
        ThemePreference::System
    }

    /// Reacts to a runtime [`CoreEvent`] (window focus, theme change, etc.).
    /// Defaults to no update.
    fn on_core_event(
        &mut self,
        _context: Context<'_, Self::Window>,
        _event: CoreEvent<Self::Window>,
    ) -> AppUpdate<Self::Message, Self::Window> {
        AppUpdate::none()
    }

    /// Decides whether a window's close request is accepted, deferred, or
    /// should run follow-up tasks. Defaults to closing the window.
    fn on_window_close_requested(
        &mut self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> CloseDecision<Self::Message> {
        CloseDecision::Close
    }

    /// Decides whether an application exit request is accepted or deferred.
    /// Defaults to accepting the exit.
    fn on_exit_requested(
        &mut self,
        _context: Context<'_, Self::Window>,
    ) -> ExitDecision<Self::Message> {
        ExitDecision::Accept
    }
}

/// Runs an [`Application`] with the default runtime.
pub fn run<A: Application>() -> Result {
    program::run::<A>()
}

/// Marker trait for single-window, no-splash Nive applications.
///
/// Apps implementing [`Application`] with `type Window = ();` and
/// `type Bootstrap = ();` automatically satisfy this marker. The runtime
/// detects `A::Window = ()` (via `TypeId`) and auto-registers one
/// [`WindowSpec::app()`] when the [`ApplicationConfig`] returned by
/// [`Application::config`] has zero `.window(...)` calls.
///
/// Apps **never** `impl SimpleApplication for MyApp` directly. The blanket
/// impl below supplies the marker for any single-window application; users
/// just `impl Application` with the two unit associated types set.
///
/// Why this trait exists: associated-type *defaults* (`type Window = ()` on
/// the trait itself) would let apps omit these lines entirely, but that
/// feature is unstable on stable Rust. This marker is a stable-compatible
/// stand-in: it gives templates a single named incident ("implement
/// `SimpleApplication`-shaped `Application`") while letting multi-window
/// apps continue their explicit enum flow untouched.
///
/// [`WindowSpec::app()`]: crate::WindowSpec::app
pub trait SimpleApplication: Application<Window = (), Bootstrap = ()> {}

impl<A> SimpleApplication for A where A: Application<Window = (), Bootstrap = ()> {}

/// Runs an [`crate::devtools::DevtoolsApp`] with the optional devtools layer
/// enabled (requires the `devtools` feature).
#[cfg(feature = "devtools")]
pub fn run_with_devtools<A>() -> Result
where
    A: crate::devtools::DevtoolsApp,
{
    program::run_with_devtools::<A>()
}
