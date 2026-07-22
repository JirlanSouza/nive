use super::*;

#[test]
fn push_keeps_newest_three_toasts() {
    let now = Instant::now();
    let mut state = ToastState::default();

    for index in 0..4 {
        push(&mut state, Toast::info(format!("Toast {index}")), now);
    }

    let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
    assert_eq!(titles, vec!["Toast 3", "Toast 2", "Toast 1"]);
}

#[test]
fn dismiss_removes_matching_toast() {
    let now = Instant::now();
    let mut state = ToastState::default();
    let first = push(&mut state, Toast::info("First"), now);
    push(&mut state, Toast::info("Second"), now);

    state.dismiss(first, now);

    let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
    assert_eq!(titles, vec!["Second"]);
}

#[test]
fn overflow_toasts_are_queued_and_promoted_after_dismiss() {
    let now = Instant::now();
    let mut state = ToastState::default();

    for index in 0..4 {
        push(&mut state, Toast::info(format!("Toast {index}")), now);
    }

    let newest = state
        .visible()
        .next()
        .map(|item| item.id())
        .expect("newest toast is visible");
    state.dismiss(newest, now + Duration::from_secs(1));

    let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
    assert_eq!(titles, vec!["Toast 2", "Toast 1", "Toast 0"]);
}

#[test]
fn overflow_toasts_are_queued_and_promoted_after_expiration() {
    let now = Instant::now();
    let mut state = ToastState::default();

    for index in 0..4 {
        push(&mut state, Toast::info(format!("Toast {index}")), now);
    }

    state.expire(now + Duration::from_secs(5));

    let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
    assert_eq!(titles, vec!["Toast 0"]);

    state.expire(now + Duration::from_secs(8));

    let titles: Vec<&str> = state.visible().map(|item| item.request().title()).collect();
    assert_eq!(titles, vec!["Toast 0"]);
}

#[test]
fn actionable_toasts_are_never_coalesced() {
    let now = Instant::now();
    let mut state: ToastState<u8> = ToastState::default();

    let first = state.push(Toast::info("Undo?").with_action("Undo", 1), now, None);
    let second = state.push(Toast::info("Undo?").with_action("Undo", 2), now, None);

    assert_ne!(first, second);
}

#[test]
fn duplicate_toasts_are_coalesced() {
    let now = Instant::now();
    let mut state = ToastState::default();

    let first = push(&mut state, Toast::info("Saved"), now);
    let second = push(&mut state, Toast::info("Saved"), now);

    assert_eq!(first, second);
    assert_eq!(state.visible().count(), 1);
}

#[test]
fn distinct_toasts_with_the_same_title_but_different_bodies_are_not_coalesced() {
    let now = Instant::now();
    let mut state = ToastState::default();

    push(&mut state, Toast::info("Saved").with_body("Item A"), now);
    push(&mut state, Toast::info("Saved").with_body("Item B"), now);

    assert_eq!(state.visible().count(), 2);
}

#[test]
fn queue_is_bounded_and_evicts_the_oldest_stale_entry() {
    let now = Instant::now();
    let mut state = ToastState::default();

    for index in 0..(MAX_VISIBLE_TOASTS + MAX_QUEUED_TOASTS + 5) {
        push(&mut state, Toast::info(format!("Toast {index}")), now);
    }

    assert!(state.queued.len() <= MAX_QUEUED_TOASTS);
}

#[test]
fn queue_eviction_prefers_non_actionable_entries() {
    let now = Instant::now();
    let mut state: ToastState<u8> = ToastState::default();

    for index in 0..MAX_VISIBLE_TOASTS {
        state.push(Toast::info(format!("filler {index}")), now, None);
    }
    // The first three actionable pushes bump the fillers into the
    // queue; the rest bump earlier actionable toasts, filling the
    // queue to exactly its bound with fillers first, actionable after.
    for index in 0..MAX_QUEUED_TOASTS {
        state.push(
            Toast::info(format!("Actionable {index}")).with_action("Undo", index as u8),
            now,
            None,
        );
    }
    assert_eq!(state.queued.len(), MAX_QUEUED_TOASTS);
    assert!(state
        .queued
        .iter()
        .take(MAX_VISIBLE_TOASTS)
        .all(|item| item.request.action().is_none()));

    // One more push exceeds the bound; the oldest non-actionable queued
    // entry is evicted before any actionable one.
    state.push(Toast::info("final"), now, None);

    assert_eq!(state.queued.len(), MAX_QUEUED_TOASTS);
    assert_eq!(
        state
            .queued
            .iter()
            .filter(|item| item.request.action().is_none())
            .count(),
        MAX_VISIBLE_TOASTS - 1
    );
}

#[test]
fn dismiss_promotes_the_next_queued_toast_immediately() {
    let now = Instant::now();
    let mut state = ToastState::default();
    for index in 0..4 {
        push(&mut state, Toast::info(format!("Toast {index}")), now);
    }
    assert_eq!(state.visible().count(), MAX_VISIBLE_TOASTS);

    let newest = state.visible().next().map(|item| item.id()).unwrap();
    state.dismiss(newest, now);

    assert_eq!(state.visible().count(), MAX_VISIBLE_TOASTS);
}
