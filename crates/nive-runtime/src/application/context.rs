use iced::window;
use nive_ui::theme::{Theme, ThemePreference};

use crate::{WindowRegistry, WindowRole};

/// Read-only runtime context handed to [`Application`](super::Application) hooks.
///
/// Exposes the application identity, active theme, window registry query, and
/// whether the runtime is shutting down. Cheap to copy and pass around.
#[derive(Clone, Copy)]
pub struct Context<'a, K> {
    pub(super) app_id: &'a str,
    pub(super) app_name: &'a str,
    pub(super) theme: Theme,
    pub(super) theme_preference: ThemePreference,
    pub(super) windows: WindowQuery<'a, K>,
    pub(super) exiting: bool,
}

impl<'a, K> Context<'a, K> {
    /// The stable application identifier (used for settings paths, etc.).
    pub fn app_id(self) -> &'a str {
        self.app_id
    }

    /// The human-facing application name.
    pub fn app_name(self) -> &'a str {
        self.app_name
    }

    /// The currently active [`Theme`].
    pub fn theme(self) -> Theme {
        self.theme
    }

    /// The user's theme preference (light/dark/system).
    pub fn theme_preference(self) -> ThemePreference {
        self.theme_preference
    }

    /// A query over the open window registry.
    pub fn windows(self) -> WindowQuery<'a, K> {
        self.windows
    }

    /// Whether the runtime is currently shutting down.
    pub fn is_exiting(self) -> bool {
        self.exiting
    }
}

/// Identifies a single open window within an [`Application`](super::Application).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowContext<K> {
    /// The Iced window id.
    pub id: window::Id,
    /// The application's logical window kind.
    pub kind: K,
    /// The window's role (app, auxiliary, etc.).
    pub role: WindowRole,
}

/// Identifies why a message reached
/// [`Application::update`](super::Application::update).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageSource {
    /// A widget or dialog view produced the message.
    View,
    /// An [`Effect`](super::Effect) task resolved to the message.
    Task,
    /// An application subscription emitted the message.
    Subscription,
    /// An [`ActionMap`](crate::ActionMap) or [`ShortcutMap`](crate::ShortcutMap)
    /// dispatched the message.
    Action,
}

/// The context passed to [`Application::update`](super::Application::update):
/// the source window, if any, and why the message was dispatched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageContext<K> {
    /// The window this message is associated with, if any.
    pub window: Option<WindowContext<K>>,
    /// Why the message reached `update`.
    pub source: MessageSource,
}

/// A read-only query over the runtime's open-window registry.
#[derive(Clone, Copy)]
pub struct WindowQuery<'a, K> {
    pub(super) registry: &'a WindowRegistry<K>,
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

    /// Returns the most recently active/opened window matching `kind`.
    pub fn latest(self, kind: K) -> Option<WindowContext<K>> {
        self.registry.latest(kind).map(|handle| WindowContext {
            id: handle.id,
            kind: handle.kind,
            role: handle.role,
        })
    }

    /// Returns the id of the most recently active/opened window matching
    /// `kind`.
    pub fn latest_id(self, kind: K) -> Option<window::Id> {
        self.latest(kind).map(|context| context.id)
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
