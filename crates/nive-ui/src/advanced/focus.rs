//! Logical-focus state for custom widget authors.

use std::sync::{Arc, Weak};

use iced::{
    advanced::widget::{operation, Id},
    Rectangle,
};

use crate::focus::{
    lock_coordinator, ActivationVisibility, FocusGeneration, FocusToken, SharedFocusCoordinator,
};

/// Controls when an active managed widget presents its focus indication.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum FocusVisibility {
    /// Shows focus for keyboard-origin activation and hides it for pointer or
    /// touch activation.
    #[default]
    Auto,
    /// Shows focus whenever the widget is active, including pointer or touch
    /// activation.
    AlwaysWhileActive,
}

impl From<FocusVisibility> for ActivationVisibility {
    fn from(visibility: FocusVisibility) -> Self {
        match visibility {
            FocusVisibility::Auto => Self::Auto,
            FocusVisibility::AlwaysWhileActive => Self::Always,
        }
    }
}

#[derive(Debug, Default)]
struct LocalFocus {
    current: bool,
    active: bool,
    visible: bool,
}

/// Persistent logical-focus state for one custom widget focus target.
///
/// Register the state from the widget's `operate` implementation. Under a
/// [`crate::accessibility::FocusRoot`], it participates in the root's unique
/// logical anchor. Without a root, it remains locally focusable and focus
/// visible, but deactivation does not retain a cross-widget anchor.
///
/// A custom focusable normally stores one `FocusState` in its persistent Iced
/// `Tree` state, replaces its former `operation.focusable(...)` registration
/// with [`FocusState::register`], calls [`FocusState::focus_from_pointer`] on a
/// claimed primary pointer or touch press, and projects
/// [`FocusState::is_focus_visible`] into paint only. Durable selection and a
/// composite's highlighted/roving item remain separate widget-owned state.
/// Use [`FocusVisibility::AlwaysWhileActive`] only when active presentation
/// communicates editing state, such as a caret-owning input.
///
/// ```ignore
/// // Inside Widget::operate, using the FocusState stored in Tree state:
/// state.focus.register(operation, self.id.as_ref(), layout.bounds());
///
/// // After this widget claims a primary pointer/touch press:
/// state.focus.focus_from_pointer();
///
/// // Inside draw/style projection; this must not affect geometry:
/// let focus_visible = state.focus.is_focus_visible();
/// ```
///
/// ```
/// use nive_ui::advanced::focus::{FocusState, FocusVisibility};
///
/// let focus = FocusState::new(FocusVisibility::Auto);
/// assert!(!focus.is_active());
/// assert!(!focus.is_focus_visible());
/// ```
///
/// Register this state from a custom widget's `Widget::operate` method. It is
/// intentionally not part of `nive_ui::prelude`:
///
/// ```compile_fail
/// use nive_ui::prelude::FocusState;
/// ```
#[derive(Debug)]
pub struct FocusState {
    token: FocusToken,
    visibility: FocusVisibility,
    coordinator: Option<Weak<std::sync::Mutex<crate::focus::FocusCoordinator>>>,
    local: LocalFocus,
    registered_id: Option<Id>,
    id_initialized: bool,
}

impl FocusState {
    /// Creates locally persistent focus state with the requested visibility
    /// policy.
    pub fn new(visibility: FocusVisibility) -> Self {
        Self {
            token: FocusToken::new(),
            visibility,
            coordinator: None,
            local: LocalFocus::default(),
            registered_id: None,
            id_initialized: false,
        }
    }

    /// Registers this state as one focusable target at `bounds`.
    pub fn register(
        &mut self,
        operation: &mut dyn operation::Operation,
        id: Option<&Id>,
        bounds: Rectangle,
    ) {
        self.reconcile_id(id);
        operation.custom(id, bounds, self);
        operation.focusable(id, bounds, self);
    }

    pub(crate) fn expose(
        &mut self,
        operation: &mut dyn operation::Operation,
        id: Option<&Id>,
        bounds: Rectangle,
    ) {
        self.reconcile_id(id);
        operation.custom(id, bounds, self);
    }

    pub(crate) fn focus_from_keyboard(&mut self) {
        if let Some(coordinator) = self.coordinator() {
            let mut coordinator = lock_coordinator(&coordinator);
            coordinator.record_origin(crate::focus::InputOrigin::Keyboard);
            coordinator.activate(self.token, self.visibility.into());
        } else {
            self.local.current = true;
            self.local.active = true;
            self.local.visible = true;
        }
    }

    /// Activates this target from a primary pointer or touch interaction.
    pub fn focus_from_pointer(&mut self) {
        if let Some(coordinator) = self.coordinator() {
            lock_coordinator(&coordinator)
                .activate_from_pointer(self.token, self.visibility.into());
        } else {
            self.local.current = true;
            self.local.active = true;
            self.local.visible = self.visibility == FocusVisibility::AlwaysWhileActive;
        }
    }

    /// Clears active presentation while retaining a rooted logical anchor.
    pub fn deactivate(&mut self) {
        if let Some(coordinator) = self.coordinator() {
            lock_coordinator(&coordinator).deactivate(self.token);
        } else {
            self.local = LocalFocus::default();
        }
    }

    /// Removes this target's active and logical focus ownership.
    pub fn clear(&mut self) {
        if let Some(coordinator) = self.coordinator() {
            lock_coordinator(&coordinator).clear(self.token);
        }
        self.local = LocalFocus::default();
    }

    /// Returns whether this target currently owns active focus.
    pub fn is_active(&self) -> bool {
        self.coordinator().map_or(self.local.active, |coordinator| {
            lock_coordinator(&coordinator).is_active(self.token)
        })
    }

    /// Returns whether this target should present its focus indication.
    pub fn is_focus_visible(&self) -> bool {
        self.coordinator()
            .map_or(self.local.visible, |coordinator| {
                lock_coordinator(&coordinator).is_visible(self.token)
            })
    }

    pub(crate) fn bind(
        &mut self,
        coordinator: &SharedFocusCoordinator,
        generation: FocusGeneration,
    ) {
        if let Some(previous) = self.coordinator() {
            if !Arc::ptr_eq(&previous, coordinator) {
                lock_coordinator(&previous).clear(self.token);
            }
        }

        self.coordinator = Some(Arc::downgrade(coordinator));
        self.local = LocalFocus::default();
        lock_coordinator(coordinator).observe_live(self.token, generation);
    }

    pub(crate) fn token(&self) -> FocusToken {
        self.token
    }

    #[cfg(test)]
    pub(crate) fn root_identity(&self) -> Option<usize> {
        self.coordinator
            .as_ref()
            .map(|coordinator| Weak::as_ptr(coordinator) as usize)
    }

    fn coordinator(&self) -> Option<SharedFocusCoordinator> {
        self.coordinator.as_ref().and_then(Weak::upgrade)
    }

    fn reconcile_id(&mut self, id: Option<&Id>) {
        if !self.id_initialized {
            self.registered_id = id.cloned();
            self.id_initialized = true;
            return;
        }
        if self.registered_id.as_ref() == id {
            return;
        }

        self.clear();
        self.token = FocusToken::new();
        self.registered_id = id.cloned();
    }
}

impl Default for FocusState {
    fn default() -> Self {
        Self::new(FocusVisibility::Auto)
    }
}

impl operation::Focusable for FocusState {
    fn is_focused(&self) -> bool {
        self.coordinator()
            .map_or(self.local.current, |coordinator| {
                lock_coordinator(&coordinator).is_current(self.token)
            })
    }

    fn focus(&mut self) {
        if let Some(coordinator) = self.coordinator() {
            lock_coordinator(&coordinator).activate(self.token, self.visibility.into());
        } else {
            self.local.current = true;
            self.local.active = true;
            self.local.visible = true;
        }
    }

    fn unfocus(&mut self) {
        if let Some(coordinator) = self.coordinator() {
            lock_coordinator(&coordinator).unfocus_if_owned(self.token);
        } else {
            self.local = LocalFocus::default();
        }
    }
}

#[cfg(test)]
mod focus_state_tests;
