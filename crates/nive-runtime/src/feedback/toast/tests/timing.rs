use nive_core::ToastPresentation;

use super::*;
use crate::UserFacingError;

#[test]
fn default_duration_depends_on_tone() {
    assert_eq!(
        Toast::<()>::info("Info").duration.as_duration(),
        Some(Duration::from_secs(4))
    );
    assert_eq!(
        Toast::<()>::success("Success").duration.as_duration(),
        Some(Duration::from_secs(4))
    );
    assert_eq!(
        Toast::<()>::warning("Warning").duration.as_duration(),
        Some(Duration::from_secs(6))
    );
    assert_eq!(
        Toast::<()>::danger("Danger").duration.as_duration(),
        Some(Duration::from_secs(8))
    );
}

#[test]
fn persistent_toast_never_auto_expires() {
    let now = Instant::now();
    assert_eq!(
        Toast::<()>::info("Pinned")
            .persistent()
            .duration
            .as_duration(),
        None
    );

    let mut state = ToastState::default();
    push(&mut state, Toast::info("Pinned").persistent(), now);

    state.expire(now + Duration::from_secs(60 * 60));

    let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
    assert_eq!(titles, vec!["Pinned"]);
}

#[test]
fn action_forces_persistent_duration() {
    let toast = Toast::info("Undo?").with_action("Undo", 42_u8);

    assert_eq!(toast.duration.as_duration(), None);
    assert_eq!(toast.action(), Some(("Undo", &42_u8)));
}

#[test]
fn expire_removes_elapsed_toasts() {
    let now = Instant::now();
    let mut state = ToastState::default();
    push(&mut state, Toast::info("Short"), now);
    push(&mut state, Toast::info("Long").long(), now);

    state.expire(now + Duration::from_secs(5));

    let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
    assert_eq!(titles, vec!["Long"]);
}

#[test]
fn hover_pause_and_resume_preserves_remaining_toast_duration() {
    let now = Instant::now();
    let mut state = ToastState::default();
    push(&mut state, Toast::info("Short"), now);

    state.set_hover(true, now + Duration::from_secs(1));
    state.set_hover(false, now + Duration::from_secs(3));
    state.expire(now + Duration::from_secs(5));

    let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
    assert_eq!(titles, vec!["Short"]);

    state.expire(now + Duration::from_secs(7));

    assert!(state.visible().next().is_none());
}

#[test]
fn focus_within_pauses_the_stack() {
    let now = Instant::now();
    let mut state = ToastState::default();
    push(&mut state, Toast::info("Short"), now);

    state.set_focus_within(true, now + Duration::from_secs(1));
    state.tick(now + Duration::from_secs(6));

    assert!(state.has_visible());
}

#[test]
fn inactive_window_pauses_the_stack() {
    let now = Instant::now();
    let mut state = ToastState::default();
    push(&mut state, Toast::info("Short"), now);

    state.set_window_active(false, now + Duration::from_secs(2));
    state.tick(now + Duration::from_secs(6));

    let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
    assert_eq!(titles, vec!["Short"]);

    state.set_window_active(true, now + Duration::from_secs(9));
    state.tick(now + Duration::from_secs(12));

    assert!(state.visible().next().is_none());
}

#[test]
fn modal_active_pauses_the_stack() {
    let now = Instant::now();
    let mut state = ToastState::default();
    push(&mut state, Toast::info("Short"), now);

    state.set_modal_active(true, now + Duration::from_secs(1));
    state.tick(now + Duration::from_secs(6));

    assert!(state.has_visible());
}

#[test]
fn dismissing_a_toast_clears_stale_focus_within_left_behind_by_its_own_button() {
    let now = Instant::now();
    let mut state = ToastState::default();
    let id = push(&mut state, Toast::info("Short"), now);

    // A click on a toast's own dismiss/action button focuses that button
    // in the same pass that removes it, so the host's focus-within
    // tracker never gets a chance to observe the button disappearing —
    // `dismiss` must clear the stale flag itself.
    state.set_focus_within(true, now);
    state.dismiss(id, now);

    assert!(!state.is_paused());

    push(
        &mut state,
        Toast::info("Later"),
        now + Duration::from_secs(1),
    );
    state.tick(now + Duration::from_secs(10));

    assert!(state.visible().next().is_none());
}

#[test]
fn dismissing_the_last_visible_toast_clears_stale_hover() {
    let now = Instant::now();
    let mut state = ToastState::default();
    let id = push(&mut state, Toast::info("Short"), now);

    // Dismissing the last visible toast unmounts the host's hover-tracking
    // wrapper entirely (an empty stack has no widget left to report the
    // pointer leaving), so a lingering hover input would otherwise pause
    // every future toast forever.
    state.set_hover(true, now);
    state.dismiss(id, now);

    assert!(!state.is_paused());

    push(
        &mut state,
        Toast::info("Later"),
        now + Duration::from_secs(1),
    );
    state.tick(now + Duration::from_secs(10));

    assert!(state.visible().next().is_none());
}

#[test]
fn should_subscribe_when_any_toast_is_visible() {
    let now = Instant::now();
    let mut state = ToastState::default();

    assert!(!state.should_subscribe());

    push(&mut state, Toast::info("Toast"), now);

    assert!(state.should_subscribe());
}

#[test]
fn tick_expires_toasts_when_not_paused() {
    let now = Instant::now();
    let mut state = ToastState::default();
    push(&mut state, Toast::info("Toast"), now);

    state.tick(now + Duration::from_secs(5));

    assert!(state.visible().next().is_none());
}

#[test]
fn error_toast_shows_only_the_safe_summary() {
    let error = UserFacingError::custom("catalog", "Record not found (record_id: r1)");

    let toast: Toast = Toast::error(error);

    assert_eq!(toast.title(), "Record not found");
    assert_eq!(toast.body(), None);
    assert_eq!(toast.tone(), ToastTone::Danger);
}

#[test]
fn toast_item_implements_presentation_contract() {
    let now = Instant::now();
    let mut state = ToastState::default();
    let id = push(
        &mut state,
        Toast::success("Project created").with_body("details"),
        now,
    );

    let item = state.visible().next().expect("toast is visible");

    assert_eq!(ToastPresentation::id(item), id);
    assert_eq!(item.title(), "Project created");
    assert_eq!(item.body(), Some("details"));
    assert_eq!(item.tone(), ToastTone::Success);
}

#[test]
fn window_scoped_toast_renders_only_in_its_origin_window() {
    let now = Instant::now();
    let mut state = ToastState::<()>::default();
    let (a, b) = (window::Id::unique(), window::Id::unique());
    state.push(Toast::info("A only"), now, Some(a));

    assert_eq!(state.visible_for(a, Some(a)).count(), 1);
    assert_eq!(state.visible_for(b, Some(a)).count(), 0);
}

#[test]
fn global_toast_renders_once_in_the_active_window_only() {
    let now = Instant::now();
    let mut state = ToastState::<()>::default();
    let (a, b) = (window::Id::unique(), window::Id::unique());
    // Originates in `a`, but only the *active* window should show it.
    state.push(Toast::info("Everywhere").global(), now, Some(a));

    assert_eq!(state.visible_for(a, Some(b)).count(), 0);
    assert_eq!(state.visible_for(b, Some(b)).count(), 1);
}

#[test]
fn origin_less_toast_falls_back_to_global_rendering() {
    let now = Instant::now();
    let mut state = ToastState::<()>::default();
    let (a, b) = (window::Id::unique(), window::Id::unique());
    state.push(Toast::info("No context"), now, None);

    assert_eq!(state.visible_for(a, Some(b)).count(), 0);
    assert_eq!(state.visible_for(b, Some(b)).count(), 1);
}
