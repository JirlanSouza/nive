mod coordinator;
mod target;

#[cfg(test)]
pub(crate) use coordinator::FocusTransition;
pub(crate) use coordinator::{
    lock_coordinator, ActivationVisibility, FocusCoordinator, FocusGeneration, FocusToken,
    InputOrigin, SharedFocusCoordinator,
};
#[cfg(test)]
pub(crate) use target::{capture_focus_target, restore_focus_target};
pub(crate) use target::{contains_focus_target, FocusTarget, FocusTargetContext};

#[cfg(test)]
mod coordinator_tests;

#[cfg(test)]
mod target_tests;

#[cfg(test)]
mod migration_gate_tests;

#[cfg(test)]
mod operation_feasibility_tests;

#[cfg(test)]
mod overlay_feasibility_tests;
