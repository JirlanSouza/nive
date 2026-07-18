use std::sync::Arc;

use iced::{
    advanced::widget::{operation, Id},
    Rectangle,
};

use super::{FocusState, FocusVisibility};
use crate::focus::{lock_coordinator, FocusCoordinator, FocusGeneration};

fn is_focused(state: &FocusState) -> bool {
    operation::Focusable::is_focused(state)
}

fn focus(state: &mut FocusState) {
    operation::Focusable::focus(state);
}

fn unfocus(state: &mut FocusState) {
    operation::Focusable::unfocus(state);
}

fn bind(state: &mut FocusState, coordinator: &crate::focus::SharedFocusCoordinator) {
    let generation = lock_coordinator(coordinator).begin_liveness();
    state.bind(coordinator, generation);
    lock_coordinator(coordinator).finish_liveness(generation);
}

#[test]
fn unrooted_auto_and_always_visibility_follow_local_fallback() {
    let mut automatic = FocusState::new(FocusVisibility::Auto);
    automatic.focus_from_pointer();
    assert!(automatic.is_active());
    assert!(!automatic.is_focus_visible());
    assert!(is_focused(&automatic));
    automatic.deactivate();
    assert!(!automatic.is_active());
    assert!(!is_focused(&automatic));

    let mut always = FocusState::new(FocusVisibility::AlwaysWhileActive);
    always.focus_from_pointer();
    assert!(always.is_active());
    assert!(always.is_focus_visible());
}

#[test]
fn unrooted_operation_focus_is_keyboard_visible_and_unfocus_clears_position() {
    let mut state = FocusState::default();

    focus(&mut state);
    assert!(state.is_active());
    assert!(state.is_focus_visible());
    assert!(is_focused(&state));

    unfocus(&mut state);
    assert!(!state.is_active());
    assert!(!state.is_focus_visible());
    assert!(!is_focused(&state));
}

#[test]
fn rooted_deactivation_retains_only_the_logical_position() {
    let coordinator = FocusCoordinator::shared();
    let mut state = FocusState::default();
    bind(&mut state, &coordinator);

    state.focus_from_pointer();
    state.deactivate();

    assert!(!state.is_active());
    assert!(!state.is_focus_visible());
    assert!(is_focused(&state));
}

#[test]
fn focus_state_is_safe_in_both_iced_unfocus_orders() {
    let coordinator = FocusCoordinator::shared();
    let mut first = FocusState::default();
    let mut second = FocusState::default();
    let generation = lock_coordinator(&coordinator).begin_liveness();
    first.bind(&coordinator, generation);
    second.bind(&coordinator, generation);
    lock_coordinator(&coordinator).finish_liveness(generation);

    focus(&mut first);
    unfocus(&mut first);
    focus(&mut second);
    assert!(is_focused(&second));
    assert!(!is_focused(&first));

    focus(&mut first);
    focus(&mut second);
    unfocus(&mut first);
    assert!(is_focused(&second));
    assert!(!is_focused(&first));
}

#[test]
fn rebinding_clears_a_stale_owner_from_the_previous_root() {
    let first_root = FocusCoordinator::shared();
    let second_root = FocusCoordinator::shared();
    let mut state = FocusState::default();
    bind(&mut state, &first_root);
    focus(&mut state);
    assert!(lock_coordinator(&first_root).is_current(state.token()));

    bind(&mut state, &second_root);

    assert!(!lock_coordinator(&first_root).is_current(state.token()));
    assert!(!lock_coordinator(&second_root).is_current(state.token()));
    focus(&mut state);
    assert!(lock_coordinator(&second_root).is_current(state.token()));
}

#[derive(Debug, Default)]
struct RegistrationProbe {
    calls: Vec<&'static str>,
    id: Option<Id>,
    bounds: Option<Rectangle>,
    coordinator: crate::focus::SharedFocusCoordinator,
    generation: FocusGeneration,
}

impl operation::Operation for RegistrationProbe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
        operate(self);
    }

    fn custom(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn std::any::Any) {
        self.calls.push("custom");
        self.id.clone_from(&id.cloned());
        self.bounds = Some(bounds);
        if let Some(state) = state.downcast_mut::<FocusState>() {
            state.bind(&self.coordinator, self.generation);
        }
    }

    fn focusable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        self.calls.push("focusable");
        state.focus();
    }
}

#[test]
fn registration_exposes_managed_state_before_focusable_with_id_and_bounds() {
    let coordinator = FocusCoordinator::shared();
    let generation = lock_coordinator(&coordinator).begin_liveness();
    let id = Id::unique();
    let bounds = Rectangle::new(iced::Point::new(4.0, 8.0), iced::Size::new(20.0, 12.0));
    let mut probe = RegistrationProbe {
        coordinator: Arc::clone(&coordinator),
        generation,
        ..RegistrationProbe::default()
    };
    let mut state = FocusState::default();

    state.register(&mut probe, Some(&id), bounds);
    lock_coordinator(&coordinator).finish_liveness(generation);

    assert_eq!(probe.calls, vec!["custom", "focusable"]);
    assert_eq!(probe.id, Some(id));
    assert_eq!(probe.bounds, Some(bounds));
    assert!(state.is_active());
    assert!(state.is_focus_visible());
}

#[test]
fn clear_only_removes_ownership_still_held_by_the_state() {
    let coordinator = FocusCoordinator::shared();
    let mut first = FocusState::default();
    let mut second = FocusState::default();
    let generation = lock_coordinator(&coordinator).begin_liveness();
    first.bind(&coordinator, generation);
    second.bind(&coordinator, generation);
    lock_coordinator(&coordinator).finish_liveness(generation);

    focus(&mut first);
    focus(&mut second);
    first.clear();

    assert!(!is_focused(&first));
    assert!(is_focused(&second));
}
