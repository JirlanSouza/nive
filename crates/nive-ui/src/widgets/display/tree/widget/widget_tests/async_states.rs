//! Chrome-row exclusion: the canonical loading/failed/empty rows rendered for
//! `Deferred`/`Failed`/empty-`Loaded` branches never participate in keyboard
//! navigation, type-ahead, selection, or drag/drop, matching how `Loading`
//! already behaved before `Failed`/empty rows were added.

use super::support::*;

struct TestError;

impl nive_core::ErrorPresentation for TestError {
    fn summary(&self) -> &str {
        "Load failed"
    }

    fn detail(&self) -> &str {
        "Load failed: connection reset"
    }
}

fn nodes_with_async_branches() -> Vec<TreeNode<'static, &'static str>> {
    vec![
        TreeNode::branch_deferred("loading-branch", "Loading branch"),
        TreeNode::branch_failed("failed-branch", "Failed branch", &TestError),
        TreeNode::branch("empty-branch", "Empty branch", Vec::new()),
        TreeNode::leaf("after", "After"),
    ]
}

fn expanded_state() -> TreeState<&'static str> {
    let mut state = TreeState::default();
    state.expand("loading-branch");
    state.expand("failed-branch");
    state.expand("empty-branch");
    state
}

#[test]
fn key_down_skips_loading_failed_and_empty_chrome_rows() {
    let nodes = nodes_with_async_branches();
    let mut state = expanded_state();
    state.select_only("loading-branch");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_down_tree(&state, ClickModifiers::NONE)
        .expect("down event");

    // From `loading-branch`, Down must land on `failed-branch` (the next real
    // row), skipping the loading chrome row in between.
    assert_eq!(expect_selection(&event).focused, Some("failed-branch"));
}

#[test]
fn key_up_skips_loading_failed_and_empty_chrome_rows() {
    let nodes = nodes_with_async_branches();
    let mut state = expanded_state();
    state.select_only("after");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_up_tree(&state, ClickModifiers::NONE)
        .expect("up event");

    // From `after`, Up must land on `empty-branch` (the previous real row),
    // skipping the empty chrome row in between.
    assert_eq!(expect_selection(&event).focused, Some("empty-branch"));
}

#[test]
fn type_ahead_never_matches_chrome_row_labels() {
    let nodes = nodes_with_async_branches();
    let mut state = expanded_state();
    state.select_only("after");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    // The loading placeholder renders "Loading...", but type-ahead for "l"
    // must resolve to the real `loading-branch` row, not the chrome text.
    let event = tree
        .handle_type_ahead_tree(&state, "l")
        .expect("type-ahead event");

    assert_eq!(expect_selection(&event).focused, Some("loading-branch"));
}

#[test]
fn row_id_at_chrome_row_offset_returns_none() {
    let nodes = nodes_with_async_branches();
    let state = expanded_state();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    // Row order: loading-branch(0), loading-chrome(1), failed-branch(2),
    // failed-chrome(3), empty-branch(4), empty-chrome(5), after(6).
    let row_height = row_height(ControlSize::Sm);
    assert!(tree.row_id_at(&state, 1.5 * row_height).is_none());
    assert!(tree.row_id_at(&state, 3.5 * row_height).is_none());
    assert!(tree.row_id_at(&state, 5.5 * row_height).is_none());
    assert_eq!(tree.row_id_at(&state, 6.5 * row_height), Some("after"));
}

#[test]
fn drop_target_at_chrome_row_targets_the_owning_branch() {
    let nodes = nodes_with_async_branches();
    let state = expanded_state();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let row_height = row_height(ControlSize::Sm);
    assert_eq!(
        tree.drop_target_at(&state, 1.5 * row_height),
        Some(TreeDropTarget::Into("loading-branch"))
    );
    assert_eq!(
        tree.drop_target_at(&state, 3.5 * row_height),
        Some(TreeDropTarget::Into("failed-branch"))
    );
    assert_eq!(
        tree.drop_target_at(&state, 5.5 * row_height),
        Some(TreeDropTarget::Into("empty-branch"))
    );
}

#[test]
fn failed_branch_retry_re_emits_expand_requested() {
    let nodes = vec![TreeNode::branch_failed("remote", "Remote", &TestError)];
    let tree = Tree::<_, Message>::new(nodes).on_event(Message::Tree);

    // Retry reuses the same expand-intent path a fresh `Deferred` expansion
    // uses; both are keyed off `ExpandRequested { id }`.
    let event = tree
        .toggle_tree_event("remote", Some(false))
        .expect("toggle event");

    assert_eq!(event.kind, TreeEventKind::ExpandRequested { id: "remote" });
}
