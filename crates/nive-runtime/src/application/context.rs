use iced::window;
use nive_ui::theme::{Theme, ThemePreference};

use crate::{WindowRegistry, WindowRole};

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
