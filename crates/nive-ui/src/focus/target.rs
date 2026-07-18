use std::sync::{Arc, Weak};

use iced::{
    advanced::widget::{operation, Id},
    Rectangle,
};

use super::{lock_coordinator, ActivationVisibility, FocusToken, SharedFocusCoordinator};
#[cfg(test)]
use crate::accessibility::focus_root::FocusRootState;
use crate::advanced::focus::FocusState;

/// Root-bound access to opaque focus capture and conditional restoration.
#[derive(Debug, Default)]
pub(crate) struct FocusTargetContext {
    coordinator: Weak<std::sync::Mutex<super::FocusCoordinator>>,
}

impl FocusTargetContext {
    pub(crate) fn expose(&mut self, operation: &mut dyn operation::Operation, bounds: Rectangle) {
        operation.custom(None, bounds, self);
    }

    pub(crate) fn capture(&self) -> Option<FocusTarget> {
        self.coordinator
            .upgrade()
            .and_then(|coordinator| FocusTarget::capture(&coordinator))
    }

    pub(crate) fn restore(&self, captured: &FocusTarget, expected: Option<&FocusTarget>) -> bool {
        let Some(coordinator) = self.coordinator.upgrade() else {
            return false;
        };
        if !captured.matches_root(&coordinator)
            || expected.is_some_and(|expected| !expected.matches_root(&coordinator))
        {
            return false;
        }

        let restored = lock_coordinator(&coordinator).restore_if_current(
            captured.token,
            expected.map(FocusTarget::identity),
            ActivationVisibility::Auto,
        );
        restored
    }

    pub(crate) fn restore_anchor(
        &self,
        captured: &FocusTarget,
        expected: Option<&FocusTarget>,
    ) -> bool {
        let Some(coordinator) = self.coordinator.upgrade() else {
            return false;
        };
        if !captured.matches_root(&coordinator)
            || expected.is_some_and(|expected| !expected.matches_root(&coordinator))
        {
            return false;
        }

        let restored = lock_coordinator(&coordinator)
            .restore_anchor_if_current(captured.token, expected.map(FocusTarget::identity));
        restored
    }

    pub(crate) fn bind(&mut self, coordinator: &SharedFocusCoordinator) {
        self.coordinator = Arc::downgrade(coordinator);
    }
}

/// Opaque crate-private reference to one managed focus target and activation.
#[derive(Debug, Clone)]
pub(crate) struct FocusTarget {
    pub(super) token: FocusToken,
    revision: u64,
    coordinator: Weak<std::sync::Mutex<super::FocusCoordinator>>,
}

impl FocusTarget {
    pub(crate) fn capture(coordinator: &SharedFocusCoordinator) -> Option<Self> {
        let (token, revision) = lock_coordinator(coordinator).target_identity()?;
        Some(Self {
            token,
            revision,
            coordinator: Arc::downgrade(coordinator),
        })
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.coordinator
            .upgrade()
            .is_some_and(|coordinator| lock_coordinator(&coordinator).is_live(self.token))
    }

    pub(super) fn matches_root(&self, coordinator: &SharedFocusCoordinator) -> bool {
        self.coordinator
            .upgrade()
            .is_some_and(|owner| Arc::ptr_eq(&owner, coordinator))
    }

    pub(super) fn identity(&self) -> (FocusToken, u64) {
        (self.token, self.revision)
    }

    fn matches_state(&self, state: &FocusState) -> bool {
        self.token == state.token()
    }
}

pub(crate) fn contains_focus_target(target: FocusTarget) -> impl operation::Operation<bool> {
    ContainsFocusTarget {
        target,
        contained: false,
    }
}

struct ContainsFocusTarget {
    target: FocusTarget,
    contained: bool,
}

impl operation::Operation<bool> for ContainsFocusTarget {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation<bool>)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
        if state
            .downcast_ref::<FocusState>()
            .is_some_and(|state| self.target.matches_state(state))
        {
            self.contained = true;
        }
    }

    fn finish(&self) -> operation::Outcome<bool> {
        operation::Outcome::Some(self.contained)
    }
}

#[cfg(test)]
pub(crate) fn capture_focus_target() -> impl operation::Operation<Option<FocusTarget>> {
    CaptureFocusTarget { target: None }
}

#[cfg(test)]
struct CaptureFocusTarget {
    target: Option<FocusTarget>,
}

#[cfg(test)]
impl operation::Operation<Option<FocusTarget>> for CaptureFocusTarget {
    fn traverse(
        &mut self,
        operate: &mut dyn FnMut(&mut dyn operation::Operation<Option<FocusTarget>>),
    ) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
        if self.target.is_none() {
            if let Some(root) = state.downcast_ref::<FocusRootState>() {
                self.target = FocusTarget::capture(&root.coordinator);
            }
        }
    }

    fn finish(&self) -> operation::Outcome<Option<FocusTarget>> {
        operation::Outcome::Some(self.target.clone())
    }
}

#[cfg(test)]
pub(crate) fn restore_focus_target(
    captured: FocusTarget,
    expected: Option<FocusTarget>,
) -> impl operation::Operation<bool> {
    RestoreFocusTarget {
        captured,
        expected,
        restored: false,
    }
}

#[cfg(test)]
struct RestoreFocusTarget {
    captured: FocusTarget,
    expected: Option<FocusTarget>,
    restored: bool,
}

#[cfg(test)]
impl operation::Operation<bool> for RestoreFocusTarget {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation<bool>)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
        let Some(root) = state.downcast_ref::<FocusRootState>() else {
            return;
        };
        if !self.captured.matches_root(&root.coordinator)
            || self
                .expected
                .as_ref()
                .is_some_and(|expected| !expected.matches_root(&root.coordinator))
        {
            return;
        }

        self.restored = lock_coordinator(&root.coordinator).restore_if_current(
            self.captured.token,
            self.expected.as_ref().map(FocusTarget::identity),
            ActivationVisibility::Auto,
        );
    }

    fn finish(&self) -> operation::Outcome<bool> {
        operation::Outcome::Some(self.restored)
    }
}
