use super::support::*;

#[test]
fn click_event_selects_and_expands_loaded_branch() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [TreeNode::leaf("child", "Child")],
    )];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .row_tree_event(&state, &"root", Some(false), ClickModifiers::NONE)
        .expect("row event");

    assert_eq!(event.kind, TreeEventKind::StateChanged);
    assert!(matches!(
        event.state_change,
        Some(TreeStateChange::Batch(changes)) if changes.len() == 2
    ));
}

#[test]
fn deferred_expansion_requests_loading() {
    let nodes = vec![TreeNode::branch_deferred("remote", "Remote")];
    let tree = Tree::<_, Message>::new(nodes).on_event(Message::Tree);

    let event = tree
        .toggle_tree_event("remote", Some(false))
        .expect("toggle event");

    assert_eq!(event.kind, TreeEventKind::ExpandRequested { id: "remote" });
}

#[test]
fn toggle_loaded_branch_emits_state_changed() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [TreeNode::leaf("child", "Child")],
    )];
    let tree = Tree::<_, Message>::new(nodes).on_event(Message::Tree);

    let event = tree
        .toggle_tree_event("root", Some(false))
        .expect("toggle event");

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
fn toggle_disabled_branch_emits_nothing() {
    let nodes =
        vec![TreeNode::branch("root", "Root", [TreeNode::leaf("child", "Child")]).disabled(true)];
    let tree = Tree::<_, Message>::new(nodes).on_event(Message::Tree);

    let result = tree.toggle_event("root", Some(false), true);

    assert!(result.is_none());
}

#[test]
fn re_expand_deferred_branch_re_emits_expand_requested() {
    let nodes = vec![TreeNode::branch_deferred("remote", "Remote")];
    let tree = Tree::<_, Message>::new(nodes).on_event(Message::Tree);

    let first = tree
        .toggle_tree_event("remote", Some(false))
        .expect("first toggle");
    let second = tree
        .toggle_tree_event("remote", Some(false))
        .expect("second toggle");

    assert_eq!(first.kind, TreeEventKind::ExpandRequested { id: "remote" });
    assert_eq!(second.kind, TreeEventKind::ExpandRequested { id: "remote" });
}

#[test]
fn collapse_deferred_branch_emits_state_changed() {
    let nodes = vec![TreeNode::branch_deferred("remote", "Remote")];
    let tree = Tree::<_, Message>::new(nodes).on_event(Message::Tree);

    let event = tree
        .toggle_tree_event("remote", Some(true))
        .expect("collapse event");

    assert_eq!(event.kind, TreeEventKind::StateChanged);
    assert_eq!(
        event.state_change,
        Some(TreeStateChange::SetExpanded {
            id: "remote",
            expanded: false,
        })
    );
}

#[test]
fn none_mode_does_not_store_selection() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let state = TreeState::default();

    let result = selection_for_click(
        &state,
        "a",
        SelectionMode::None,
        ClickModifiers::NONE,
        &nodes,
    )
    .expect("selection");

    assert!(result.selected.is_empty());
    assert_eq!(result.focused, Some("a"));
    assert_eq!(result.anchor, None);
}

#[test]
fn single_mode_stores_one_id() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.selection.selected.insert("a");

    let result = selection_for_click(
        &state,
        "b",
        SelectionMode::Single,
        ClickModifiers::NONE,
        &nodes,
    )
    .expect("selection");

    assert_eq!(result.selected, ["b"].into_iter().collect());
    assert_eq!(result.focused, Some("b"));
    assert_eq!(result.anchor, Some("b"));
}

#[test]
fn multiple_plain_click_resets_to_one() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.selection.selected.insert("a");
    state.selection.selected.insert("b");

    let result = selection_for_click(
        &state,
        "c",
        SelectionMode::Multiple,
        ClickModifiers::NONE,
        &nodes,
    )
    .expect("selection");

    assert_eq!(result.selected, ["c"].into_iter().collect());
    assert_eq!(result.focused, Some("c"));
    assert_eq!(result.anchor, Some("c"));
}

#[test]
fn multiple_primary_modifier_toggles() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.selection.selected.insert("a");
    state.selection.selected.insert("b");
    state.selection.anchor = Some("a");

    let add_result = selection_for_click(
        &state,
        "c",
        SelectionMode::Multiple,
        ClickModifiers::new(true, false),
        &nodes,
    )
    .expect("add selection");

    assert_eq!(add_result.selected, ["a", "b", "c"].into_iter().collect());
    assert_eq!(add_result.focused, Some("c"));
    assert_eq!(add_result.anchor, Some("a"));

    let remove_result = selection_for_click(
        &state,
        "b",
        SelectionMode::Multiple,
        ClickModifiers::new(true, false),
        &nodes,
    )
    .expect("remove selection");

    assert_eq!(remove_result.selected, ["a"].into_iter().collect());
    assert_eq!(remove_result.focused, Some("b"));
    assert_eq!(remove_result.anchor, Some("a"));
}

#[test]
fn multiple_shift_range_selection() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
        TreeNode::leaf("d", "D"),
        TreeNode::leaf("e", "E"),
    ];
    let mut state = TreeState::default();
    state.selection.anchor = Some("b");

    let result = selection_for_click(
        &state,
        "d",
        SelectionMode::Multiple,
        ClickModifiers::new(false, true),
        &nodes,
    )
    .expect("range selection");

    assert_eq!(result.selected, ["b", "c", "d"].into_iter().collect());
    assert_eq!(result.focused, Some("d"));
}

#[test]
fn multiple_shift_range_reversed() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
        TreeNode::leaf("d", "D"),
    ];
    let mut state = TreeState::default();
    state.selection.anchor = Some("d");

    let result = selection_for_click(
        &state,
        "b",
        SelectionMode::Multiple,
        ClickModifiers::new(false, true),
        &nodes,
    )
    .expect("reverse range selection");

    assert_eq!(result.selected, ["b", "c", "d"].into_iter().collect());
}

#[test]
fn shift_range_skips_disabled_nodes() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C").disabled(true),
        TreeNode::leaf("d", "D"),
        TreeNode::leaf("e", "E"),
    ];
    let mut state = TreeState::default();
    state.selection.anchor = Some("b");

    let result = selection_for_click(
        &state,
        "e",
        SelectionMode::Multiple,
        ClickModifiers::new(false, true),
        &nodes,
    )
    .expect("range with disabled");

    assert_eq!(result.selected, ["b", "d", "e"].into_iter().collect());
}

#[test]
fn shift_range_without_anchor_uses_clicked_as_anchor() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let state = TreeState::default();

    let result = selection_for_click(
        &state,
        "b",
        SelectionMode::Multiple,
        ClickModifiers::new(false, true),
        &nodes,
    )
    .expect("shift without anchor");

    assert_eq!(result.selected, ["b"].into_iter().collect());
    assert_eq!(result.focused, Some("b"));
}

#[test]
fn empty_space_click_emits_clear_selection_single_mode() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.select_only("a");

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Single)
        .on_event(Message::Tree);

    let event = tree.empty_space_event(&state).expect("clear event");

    assert_eq!(
        event,
        Message::Tree(TreeEvent {
            state_change: Some(TreeStateChange::ClearSelection),
            kind: TreeEventKind::StateChanged,
        })
    );
}

#[test]
fn empty_space_click_emits_clear_selection_multiple_mode() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.selection.selected.insert("a");
    state.selection.selected.insert("b");
    state.selection.anchor = Some("a");

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Multiple)
        .on_event(Message::Tree);

    let event = tree.empty_space_event(&state).expect("clear event");

    assert_eq!(
        event,
        Message::Tree(TreeEvent {
            state_change: Some(TreeStateChange::ClearSelection),
            kind: TreeEventKind::StateChanged,
        })
    );
}

#[test]
fn empty_space_click_does_nothing_in_none_mode() {
    let nodes = vec![TreeNode::leaf("a", "A")];
    let mut state = TreeState::default();
    state.selection.focused = Some("a");

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::None)
        .on_event(Message::Tree);

    assert!(tree.empty_space_event(&state).is_none());
}

#[test]
fn empty_space_click_does_nothing_when_already_empty() {
    let nodes = vec![TreeNode::leaf("a", "A")];
    let state = TreeState::default();

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Single)
        .on_event(Message::Tree);

    assert!(tree.empty_space_event(&state).is_none());
}

#[test]
fn clear_selection_preserves_focused_expansion_and_transfer() {
    let mut state = TreeState::default();
    state.expand("root");
    state.selection = Selection {
        selected: ["child"].into_iter().collect(),
        focused: Some("child"),
        anchor: Some("child"),
    };

    state.apply_change(&TreeStateChange::ClearSelection);

    assert!(state.selection.selected.is_empty());
    assert_eq!(state.selection.focused, Some("child"));
    assert_eq!(state.selection.anchor, None);
    assert!(state.is_expanded(&"root"));
}

#[test]
fn shift_range_with_nested_expanded_nodes() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [
            TreeNode::leaf("child1", "Child 1"),
            TreeNode::leaf("child2", "Child 2"),
            TreeNode::leaf("child3", "Child 3"),
        ],
    )];
    let mut state = TreeState::default();
    state.expand("root");
    state.selection.anchor = Some("child1");

    let result = selection_for_click(
        &state,
        "child3",
        SelectionMode::Multiple,
        ClickModifiers::new(false, true),
        &nodes,
    )
    .expect("nested range");

    assert_eq!(
        result.selected,
        ["child1", "child2", "child3"].into_iter().collect()
    );
}

#[test]
fn disabled_node_click_does_not_select_in_single_mode() {
    let nodes = vec![
        TreeNode::leaf("a", "A").disabled(true),
        TreeNode::leaf("b", "B"),
    ];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Single)
        .on_event(Message::Tree);

    let result = tree.row_event(&state, &"a", None, true);

    assert!(result.is_none());
}

#[test]
fn disabled_node_click_does_not_select_in_multiple_mode() {
    let nodes = vec![
        TreeNode::leaf("a", "A").disabled(true),
        TreeNode::leaf("b", "B"),
    ];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Multiple)
        .on_event(Message::Tree);

    let result = tree.row_event(&state, &"a", None, true);

    assert!(result.is_none());
}

#[test]
fn disabled_node_with_primary_modifier_does_not_toggle_in_multiple_mode() {
    let nodes = vec![
        TreeNode::leaf("a", "A").disabled(true),
        TreeNode::leaf("b", "B"),
    ];
    let mut state = TreeState::default();
    state.selection.selected.insert("b");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Multiple)
        .on_event(Message::Tree);

    let result = tree.row_event(&state, &"a", None, true);

    assert!(result.is_none());
}

#[test]
fn primary_modifier_in_single_mode_behaves_like_plain_click() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.select_only("a");

    let result = selection_for_click(
        &state,
        "b",
        SelectionMode::Single,
        ClickModifiers::new(true, false),
        &nodes,
    )
    .expect("primary click in single mode");

    assert_eq!(result.selected, ["b"].into_iter().collect());
    assert_eq!(result.focused, Some("b"));
    assert_eq!(result.anchor, Some("b"));
}

#[test]
fn shift_in_single_mode_behaves_like_plain_click() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("a");

    let result = selection_for_click(
        &state,
        "c",
        SelectionMode::Single,
        ClickModifiers::new(false, true),
        &nodes,
    )
    .expect("shift click in single mode");

    assert_eq!(result.selected, ["c"].into_iter().collect());
    assert_eq!(result.focused, Some("c"));
    assert_eq!(result.anchor, Some("c"));
}

#[test]
fn none_mode_with_modifiers_stores_no_selection() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let state = TreeState::default();

    let result = selection_for_click(
        &state,
        "a",
        SelectionMode::None,
        ClickModifiers::new(true, true),
        &nodes,
    )
    .expect("modifiers in none mode");

    assert!(result.selected.is_empty());
    assert_eq!(result.focused, Some("a"));
    assert_eq!(result.anchor, None);
}
