use std::{sync::Arc, thread};

use super::{
    lock_coordinator, ActivationVisibility, FocusCoordinator, FocusToken, FocusTransition,
    InputOrigin,
};

#[derive(Debug, Clone, Copy)]
enum Step {
    PointerActivate(FocusToken, ActivationVisibility),
    Activate(FocusToken, ActivationVisibility),
    Deactivate(FocusToken),
    WindowDeactivate,
    WindowActivate,
}

fn apply(coordinator: &mut FocusCoordinator, step: Step) -> FocusTransition {
    match step {
        Step::PointerActivate(token, visibility) => {
            coordinator.activate_from_pointer(token, visibility)
        }
        Step::Activate(token, visibility) => coordinator.activate(token, visibility),
        Step::Deactivate(token) => coordinator.deactivate(token),
        Step::WindowDeactivate => coordinator.deactivate_window(),
        Step::WindowActivate => coordinator.activate_window(),
    }
}

#[test]
fn transitions_preserve_uniqueness_anchor_and_visibility_invariants() {
    let first = FocusToken::new();
    let second = FocusToken::new();
    let cases = [
        vec![Step::Activate(first, ActivationVisibility::Auto)],
        vec![
            Step::Activate(first, ActivationVisibility::Auto),
            Step::PointerActivate(second, ActivationVisibility::Auto),
        ],
        vec![
            Step::PointerActivate(first, ActivationVisibility::Always),
            Step::Deactivate(first),
        ],
        vec![
            Step::Activate(first, ActivationVisibility::Auto),
            Step::WindowDeactivate,
            Step::WindowActivate,
        ],
    ];

    for steps in cases {
        let mut coordinator = FocusCoordinator::default();
        for step in steps {
            apply(&mut coordinator, step);
            let snapshot = coordinator.snapshot();
            assert!(snapshot.active.is_none() || snapshot.active == snapshot.anchor);
            assert!(!snapshot.visible || snapshot.active.is_some());
        }
    }
}

#[test]
fn auto_defaults_to_keyboard_visible_and_pointer_activation_replaces_the_anchor() {
    let first = FocusToken::new();
    let second = FocusToken::new();
    let mut coordinator = FocusCoordinator::default();

    coordinator.activate(first, ActivationVisibility::Auto);
    assert!(coordinator.is_active(first));
    assert!(coordinator.is_visible(first));

    coordinator.activate_from_pointer(second, ActivationVisibility::Auto);
    assert!(!coordinator.is_current(first));
    assert!(coordinator.is_active(second));
    assert!(!coordinator.is_visible(second));
}

#[test]
fn pointer_deactivation_retains_the_anchor_and_always_visibility_is_explicit() {
    let token = FocusToken::new();
    let mut coordinator = FocusCoordinator::default();

    coordinator.activate_from_pointer(token, ActivationVisibility::Always);
    assert!(coordinator.is_visible(token));

    coordinator.deactivate(token);
    assert!(!coordinator.is_active(token));
    assert!(!coordinator.is_visible(token));
    assert!(coordinator.is_current(token));
}

#[test]
fn conditional_unfocus_is_safe_in_both_iced_operation_orders() {
    let first = FocusToken::new();
    let second = FocusToken::new();

    let mut unfocus_then_focus = FocusCoordinator::default();
    unfocus_then_focus.activate(first, ActivationVisibility::Auto);
    unfocus_then_focus.unfocus_if_owned(first);
    unfocus_then_focus.activate(second, ActivationVisibility::Auto);
    assert!(unfocus_then_focus.is_active(second));
    assert!(!unfocus_then_focus.is_current(first));

    let mut focus_then_unfocus = FocusCoordinator::default();
    focus_then_unfocus.activate(first, ActivationVisibility::Auto);
    focus_then_unfocus.activate(second, ActivationVisibility::Auto);
    focus_then_unfocus.unfocus_if_owned(first);
    assert!(focus_then_unfocus.is_active(second));
    assert!(!focus_then_unfocus.is_current(first));
}

#[test]
fn clear_and_liveness_invalidation_remove_only_stale_owners() {
    let first = FocusToken::new();
    let second = FocusToken::new();
    let mut coordinator = FocusCoordinator::default();

    coordinator.activate(first, ActivationVisibility::Auto);
    coordinator.clear(second);
    assert!(coordinator.is_current(first));

    let generation = coordinator.begin_liveness();
    coordinator.observe_live(second, generation);
    coordinator.finish_liveness(generation);
    assert!(!coordinator.is_current(first));
    assert!(!coordinator.is_current(second));
}

#[test]
fn liveness_keeps_a_replacement_activated_after_it_was_observed() {
    let first = FocusToken::new();
    let second = FocusToken::new();
    let mut coordinator = FocusCoordinator::default();
    coordinator.activate(first, ActivationVisibility::Auto);

    let generation = coordinator.begin_liveness();
    coordinator.observe_live(first, generation);
    coordinator.observe_live(second, generation);
    coordinator.activate(second, ActivationVisibility::Auto);
    coordinator.finish_liveness(generation);

    assert!(!coordinator.is_current(first));
    assert!(coordinator.is_active(second));
}

#[test]
fn window_deactivation_clears_transient_state_but_keeps_position() {
    let token = FocusToken::new();
    let mut coordinator = FocusCoordinator::default();
    coordinator.activate(token, ActivationVisibility::Auto);

    coordinator.deactivate_window();
    let inactive = coordinator.snapshot();
    assert!(!inactive.window_active);
    assert_eq!(inactive.anchor, Some(token));
    assert_eq!(inactive.active, None);
    assert!(!inactive.visible);

    coordinator.activate_window();
    let reactivated = coordinator.snapshot();
    assert!(reactivated.window_active);
    assert_eq!(reactivated.anchor, Some(token));
    assert_eq!(reactivated.active, None);
    assert!(!reactivated.visible);
}

#[test]
fn origin_recording_and_empty_deactivation_are_independent_transitions() {
    let token = FocusToken::new();
    let mut coordinator = FocusCoordinator::default();
    coordinator.activate(token, ActivationVisibility::Auto);

    coordinator.record_origin(InputOrigin::Pointer);
    coordinator.deactivate_current();

    let snapshot = coordinator.snapshot();
    assert_eq!(snapshot.origin, InputOrigin::Pointer);
    assert_eq!(snapshot.anchor, Some(token));
    assert_eq!(snapshot.active, None);
}

#[test]
fn poison_recovery_retains_coordinator_state() {
    let token = FocusToken::new();
    let coordinator = FocusCoordinator::shared();
    let poisoned = Arc::clone(&coordinator);

    let result = thread::spawn(move || {
        let mut coordinator = poisoned.lock().expect("initial lock");
        coordinator.activate(token, ActivationVisibility::Auto);
        panic!("poison coordinator for recovery test");
    })
    .join();
    assert!(result.is_err());

    assert!(lock_coordinator(&coordinator).is_active(token));
}
