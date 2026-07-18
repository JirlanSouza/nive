use super::{lock_coordinator, ActivationVisibility, FocusCoordinator, FocusTarget, FocusToken};

fn observe(coordinator: &crate::focus::SharedFocusCoordinator, tokens: &[FocusToken]) {
    let mut coordinator = lock_coordinator(coordinator);
    let generation = coordinator.begin_liveness();
    for token in tokens {
        coordinator.observe_live(*token, generation);
    }
    coordinator.finish_liveness(generation);
}

#[test]
fn opaque_target_validity_tracks_liveness() {
    let coordinator = FocusCoordinator::shared();
    let token = FocusToken::new();
    observe(&coordinator, &[token]);
    lock_coordinator(&coordinator).activate(token, ActivationVisibility::Auto);
    let target = FocusTarget::capture(&coordinator).expect("target");
    assert!(target.is_valid());

    observe(&coordinator, &[]);
    assert!(!target.is_valid());
}

#[test]
fn conditional_restore_requires_the_expected_activation_revision() {
    let coordinator = FocusCoordinator::shared();
    let anchor = FocusToken::new();
    let overlay = FocusToken::new();
    observe(&coordinator, &[anchor, overlay]);

    lock_coordinator(&coordinator).activate(anchor, ActivationVisibility::Auto);
    let captured = FocusTarget::capture(&coordinator).expect("captured anchor");
    lock_coordinator(&coordinator).activate(overlay, ActivationVisibility::Auto);
    let expected = FocusTarget::capture(&coordinator).expect("overlay target");

    assert!(lock_coordinator(&coordinator).restore_if_current(
        captured.token,
        Some(expected.identity()),
        ActivationVisibility::Auto,
    ));
    assert_eq!(
        lock_coordinator(&coordinator)
            .target_identity()
            .map(|target| target.0),
        Some(anchor)
    );

    lock_coordinator(&coordinator).activate(overlay, ActivationVisibility::Auto);
    let stale_expected = FocusTarget::capture(&coordinator).expect("expected target");
    lock_coordinator(&coordinator).activate(overlay, ActivationVisibility::Auto);
    assert!(!lock_coordinator(&coordinator).restore_if_current(
        captured.token,
        Some(stale_expected.identity()),
        ActivationVisibility::Auto,
    ));
    assert_eq!(
        lock_coordinator(&coordinator)
            .target_identity()
            .map(|target| target.0),
        Some(overlay)
    );
}

#[test]
fn anchor_only_restore_preserves_position_without_active_or_visible_focus() {
    let coordinator = FocusCoordinator::shared();
    let anchor = FocusToken::new();
    let dialog = FocusToken::new();
    observe(&coordinator, &[anchor, dialog]);

    lock_coordinator(&coordinator).activate(anchor, ActivationVisibility::Auto);
    let captured = FocusTarget::capture(&coordinator).expect("captured anchor");
    lock_coordinator(&coordinator).activate(dialog, ActivationVisibility::Auto);
    let expected = FocusTarget::capture(&coordinator).expect("dialog target");

    assert!(lock_coordinator(&coordinator)
        .restore_anchor_if_current(captured.token, Some(expected.identity())));
    let snapshot = lock_coordinator(&coordinator).snapshot();
    assert_eq!(snapshot.anchor, Some(anchor));
    assert_eq!(snapshot.active, None);
    assert!(!snapshot.visible);

    lock_coordinator(&coordinator).activate(dialog, ActivationVisibility::Auto);
    let stale_expected = FocusTarget::capture(&coordinator).expect("expected target");
    lock_coordinator(&coordinator).activate(dialog, ActivationVisibility::Auto);
    assert!(!lock_coordinator(&coordinator)
        .restore_anchor_if_current(captured.token, Some(stale_expected.identity())));
    assert!(lock_coordinator(&coordinator).is_active(dialog));
}

#[test]
fn targets_cannot_restore_across_independent_roots() {
    let first = FocusCoordinator::shared();
    let second = FocusCoordinator::shared();
    let token = FocusToken::new();
    observe(&first, &[token]);
    lock_coordinator(&first).activate(token, ActivationVisibility::Auto);
    let target = FocusTarget::capture(&first).expect("target");

    assert!(!target.matches_root(&second));
}
