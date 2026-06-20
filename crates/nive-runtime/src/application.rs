use std::{borrow::Cow, fmt::Debug, hash::Hash};

use iced::Subscription;

use crate::ScreenView;

mod config;
mod context;
mod event;
mod program;
mod task;
mod theme;
mod update;

pub use crate::input::{ShortcutBinding, ShortcutKey, ShortcutMap};
pub use crate::lifecycle::{
    CloseDecision, CommandRejected, CommandRejectionReason, ExitDecision, WindowCommand,
};
pub use config::{ApplicationConfig, WindowRegistration};
pub use context::{Context, WindowContext, WindowQuery};
pub use event::CoreEvent;
pub use task::client_task;
pub use theme::{ThemeController, ThemeEvent};
pub use update::{AppUpdate, Never, RuntimeCommand, Update};

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

#[cfg(feature = "devtools")]
pub fn run_with_devtools<A>() -> Result
where
    A: crate::devtools::DevtoolsApp,
{
    program::run_with_devtools::<A>()
}
