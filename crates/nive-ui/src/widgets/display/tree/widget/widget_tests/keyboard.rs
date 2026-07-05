use super::support::*;

#[test]
fn key_down_moves_focus_and_selection_follows_single_mode() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Single)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_down_tree(&state, ClickModifiers::NONE)
        .expect("down event");

    let selection = expect_selection(&event);
    assert_eq!(selection.focused, Some("b"));
    assert_eq!(selection.selected, ["b"].into_iter().collect());
}

#[test]
fn key_down_at_last_row_is_noop() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.select_only("b");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    assert!(tree
        .handle_key_down_tree(&state, ClickModifiers::NONE)
        .is_none());
}

#[test]
fn key_up_moves_focus_to_previous_row() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("c");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_up_tree(&state, ClickModifiers::NONE)
        .expect("up event");

    assert_eq!(expect_selection(&event).focused, Some("b"));
}

#[test]
fn key_up_at_first_row_is_noop() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.select_only("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    assert!(tree
        .handle_key_up_tree(&state, ClickModifiers::NONE)
        .is_none());
}

#[test]
fn primary_modifier_arrow_moves_focus_only() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.select_only("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Single)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_down_tree(&state, ClickModifiers::new(true, false))
        .expect("focus-only down event");

    let selection = expect_selection(&event);
    assert_eq!(selection.focused, Some("b"));
    assert_eq!(selection.selected, ["a"].into_iter().collect());
}

#[test]
fn none_mode_arrow_moves_focus_only() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.selection.focused = Some("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::None)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_down_tree(&state, ClickModifiers::NONE)
        .expect("down event");

    let selection = expect_selection(&event);
    assert_eq!(selection.focused, Some("b"));
    assert!(selection.selected.is_empty());
}

#[test]
fn shift_down_extends_range_selection_in_multiple_mode() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Multiple)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_down_tree(&state, ClickModifiers::new(false, true))
        .expect("shift down event");

    let selection = expect_selection(&event);
    assert_eq!(selection.selected, ["a", "b"].into_iter().collect());
    assert_eq!(selection.focused, Some("b"));
    assert_eq!(selection.anchor, Some("a"));
}

#[test]
fn shift_up_extends_range_selection_in_multiple_mode() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("c");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Multiple)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_up_tree(&state, ClickModifiers::new(false, true))
        .expect("shift up event");

    let selection = expect_selection(&event);
    assert_eq!(selection.selected, ["b", "c"].into_iter().collect());
    assert_eq!(selection.focused, Some("b"));
}

#[test]
fn disabled_row_is_focusable_but_excluded_from_selection() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B").disabled(true),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Single)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_down_tree(&state, ClickModifiers::NONE)
        .expect("move onto disabled row");

    let selection = expect_selection(&event);
    assert_eq!(selection.focused, Some("b"));
    assert_eq!(selection.selected, ["a"].into_iter().collect());
}

#[test]
fn right_expands_collapsed_loaded_branch() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [TreeNode::leaf("child", "Child")],
    )];
    let mut state = TreeState::default();
    state.selection.focused = Some("root");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree.handle_key_right_tree(&state).expect("expand event");

    assert_eq!(event.kind, TreeEventKind::StateChanged);
    assert_eq!(
        event.state_change,
        Some(TreeStateChange::SetExpanded {
            id: "root",
            expanded: true,
        })
    );
}

#[test]
fn right_on_deferred_branch_expands_and_requests_load() {
    let nodes = vec![TreeNode::branch_deferred("remote", "Remote")];
    let mut state = TreeState::default();
    state.selection.focused = Some("remote");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree.handle_key_right_tree(&state).expect("expand event");

    assert_eq!(event.kind, TreeEventKind::ExpandRequested { id: "remote" });
    assert_eq!(
        event.state_change,
        Some(TreeStateChange::SetExpanded {
            id: "remote",
            expanded: true,
        })
    );
}

#[test]
fn right_on_expanded_branch_moves_focus_to_first_child() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [
            TreeNode::leaf("child1", "Child1"),
            TreeNode::leaf("child2", "Child2"),
        ],
    )];
    let mut state = TreeState::default();
    state.expand("root");
    state.selection.focused = Some("root");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_right_tree(&state)
        .expect("move into first child");

    assert_eq!(expect_selection(&event).focused, Some("child1"));
}

#[test]
fn right_on_expanded_empty_branch_is_noop() {
    let nodes = vec![TreeNode::branch(
        "empty",
        "Empty",
        Vec::<TreeNode<'_, &'static str>>::new(),
    )];
    let mut state = TreeState::default();
    state.expand("empty");
    state.selection.focused = Some("empty");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    assert!(tree.handle_key_right_tree(&state).is_none());
}

#[test]
fn right_on_leaf_is_noop() {
    let nodes = vec![TreeNode::leaf("leaf", "Leaf")];
    let mut state = TreeState::default();
    state.selection.focused = Some("leaf");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    assert!(tree.handle_key_right_tree(&state).is_none());
}

#[test]
fn left_collapses_expanded_branch_and_keeps_focus() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [TreeNode::leaf("child", "Child")],
    )];
    let mut state = TreeState::default();
    state.expand("root");
    state.selection.focused = Some("root");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree.handle_key_left_tree(&state).expect("collapse event");

    assert_eq!(
        event.state_change,
        Some(TreeStateChange::SetExpanded {
            id: "root",
            expanded: false,
        })
    );
}

#[test]
fn left_on_child_moves_focus_to_parent() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [TreeNode::leaf("child", "Child")],
    )];
    let mut state = TreeState::default();
    state.expand("root");
    state.selection.focused = Some("child");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_left_tree(&state)
        .expect("move to parent row");

    assert_eq!(expect_selection(&event).focused, Some("root"));
}

#[test]
fn left_on_top_level_leaf_is_noop() {
    let nodes = vec![TreeNode::leaf("leaf", "Leaf")];
    let mut state = TreeState::default();
    state.selection.focused = Some("leaf");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    assert!(tree.handle_key_left_tree(&state).is_none());
}

#[test]
fn home_moves_focus_to_first_row() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("c");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_home_tree(&state, ClickModifiers::NONE)
        .expect("home event");

    assert_eq!(expect_selection(&event).focused, Some("a"));
}

#[test]
fn end_moves_focus_to_last_row() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_end_tree(&state, ClickModifiers::NONE)
        .expect("end event");

    assert_eq!(expect_selection(&event).focused, Some("c"));
}

#[test]
fn shift_home_extends_range_in_multiple_mode() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("c");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Multiple)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_home_tree(&state, ClickModifiers::new(false, true))
        .expect("shift home event");

    let selection = expect_selection(&event);
    assert_eq!(selection.selected, ["a", "b", "c"].into_iter().collect());
    assert_eq!(selection.focused, Some("a"));
}

#[test]
fn shift_end_extends_range_in_multiple_mode() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Multiple)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_end_tree(&state, ClickModifiers::new(false, true))
        .expect("shift end event");

    let selection = expect_selection(&event);
    assert_eq!(selection.selected, ["a", "b", "c"].into_iter().collect());
    assert_eq!(selection.focused, Some("c"));
}

#[test]
fn page_down_moves_focus_by_page_rows() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
        TreeNode::leaf("d", "D"),
        TreeNode::leaf("e", "E"),
    ];
    let mut state = TreeState::default();
    state.select_only("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .page_rows(2)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_page_down_tree(&state, ClickModifiers::NONE)
        .expect("page down event");

    assert_eq!(expect_selection(&event).focused, Some("c"));
}

#[test]
fn page_down_clamps_at_last_row() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .page_rows(10)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_page_down_tree(&state, ClickModifiers::NONE)
        .expect("clamped page down event");

    assert_eq!(expect_selection(&event).focused, Some("c"));
}

#[test]
fn page_up_moves_focus_by_page_rows() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("c");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .page_rows(1)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_page_up_tree(&state, ClickModifiers::NONE)
        .expect("page up event");

    assert_eq!(expect_selection(&event).focused, Some("b"));
}

#[test]
fn page_up_clamps_at_first_row() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("c");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .page_rows(10)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_page_up_tree(&state, ClickModifiers::NONE)
        .expect("clamped page up event");

    assert_eq!(expect_selection(&event).focused, Some("a"));
}

#[test]
fn focus_recovery_resolves_to_first_selected_row_then_moves() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.selection.focused = Some("missing");
    state.selection.selected.insert("b");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_down_tree(&state, ClickModifiers::NONE)
        .expect("recovers then moves down");

    assert_eq!(expect_selection(&event).focused, Some("c"));
}

#[test]
fn focus_recovery_resolves_to_first_row_when_nothing_selected() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.selection.focused = Some("missing");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_right_tree(&state)
        .expect("recovery-only event");

    assert_eq!(expect_selection(&event).focused, Some("a"));
}

#[test]
fn focus_recovery_at_boundary_still_emits_recovered_focus() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.selection.focused = Some("missing");
    state.selection.selected.insert("b");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_key_down_tree(&state, ClickModifiers::NONE)
        .expect("recovery emitted even though already at last row");

    assert_eq!(expect_selection(&event).focused, Some("b"));
}
