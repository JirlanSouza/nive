use super::support::*;

#[test]
fn context_request_on_selected_node_preserves_selection() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("b");

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Single)
        .on_event(Message::Tree);

    let event = tree
        .context_request_tree_event_for_node(&state, &"b", Point::new(10.0, 20.0), false)
        .expect("context request");

    assert!(event.state_change.is_none());
    assert_eq!(
        event.kind,
        TreeEventKind::ContextRequested(ContextRequest {
            target: ContextTarget::Item("b"),
            selection: SelectionSnapshot {
                selected: vec!["b"],
                focused: Some("b"),
                anchor: Some("b"),
            },
            position: ContextPosition::Pointer(Point::new(10.0, 20.0)),
            invocation: ContextInvocation::SecondaryClick,
        })
    );
}

#[test]
fn context_request_on_unselected_node_with_select_target() {
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
        .context_selection_behavior(ContextSelectionBehavior::SelectTargetIfUnselected)
        .on_event(Message::Tree);

    let event = tree
        .context_request_tree_event_for_node(&state, &"b", Point::new(10.0, 20.0), false)
        .expect("context request");

    assert!(matches!(
        event.state_change,
        Some(TreeStateChange::SetSelection(ref sel))
            if sel.selected == ["b"].into_iter().collect()
                && sel.focused == Some("b")
                && sel.anchor == Some("b")
    ));

    assert_eq!(
        event.kind,
        TreeEventKind::ContextRequested(ContextRequest {
            target: ContextTarget::Item("b"),
            selection: SelectionSnapshot {
                selected: vec!["b"],
                focused: Some("b"),
                anchor: Some("b"),
            },
            position: ContextPosition::Pointer(Point::new(10.0, 20.0)),
            invocation: ContextInvocation::SecondaryClick,
        })
    );
}

#[test]
fn context_request_on_unselected_node_with_preserve_selection() {
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
        .context_selection_behavior(ContextSelectionBehavior::PreserveSelection)
        .on_event(Message::Tree);

    let event = tree
        .context_request_tree_event_for_node(&state, &"b", Point::new(10.0, 20.0), false)
        .expect("context request");

    assert!(event.state_change.is_none());
    assert_eq!(
        event.kind,
        TreeEventKind::ContextRequested(ContextRequest {
            target: ContextTarget::Item("b"),
            selection: SelectionSnapshot {
                selected: vec!["a"],
                focused: Some("a"),
                anchor: Some("a"),
            },
            position: ContextPosition::Pointer(Point::new(10.0, 20.0)),
            invocation: ContextInvocation::SecondaryClick,
        })
    );
}

#[test]
fn context_request_on_empty_space() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.select_only("a");

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Single)
        .on_event(Message::Tree);

    let event = tree.context_request_tree_event_for_empty_space(&state, Point::new(50.0, 100.0));

    assert!(event.state_change.is_none());
    assert_eq!(
        event.kind,
        TreeEventKind::ContextRequested(ContextRequest {
            target: ContextTarget::Empty,
            selection: SelectionSnapshot {
                selected: vec!["a"],
                focused: Some("a"),
                anchor: Some("a"),
            },
            position: ContextPosition::Pointer(Point::new(50.0, 100.0)),
            invocation: ContextInvocation::SecondaryClick,
        })
    );
}

#[test]
fn context_request_keyboard_on_focused_node() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B"),
        TreeNode::leaf("c", "C"),
    ];
    let mut state = TreeState::default();
    state.select_only("b");

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Single)
        .on_event(Message::Tree);

    let event = tree
        .context_request_tree_event_for_keyboard(&state)
        .expect("keyboard context request");

    assert!(event.state_change.is_none());
    assert_eq!(
        event.kind,
        TreeEventKind::ContextRequested(ContextRequest {
            target: ContextTarget::Item("b"),
            selection: SelectionSnapshot {
                selected: vec!["b"],
                focused: Some("b"),
                anchor: Some("b"),
            },
            position: ContextPosition::FocusedItem,
            invocation: ContextInvocation::Keyboard,
        })
    );
}

#[test]
fn context_request_disabled_node_emits_nothing() {
    let nodes = vec![
        TreeNode::leaf("a", "A").disabled(true),
        TreeNode::leaf("b", "B"),
    ];
    let state = TreeState::default();

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let result =
        tree.context_request_tree_event_for_node(&state, &"a", Point::new(10.0, 20.0), true);

    assert!(result.is_none());
}

#[test]
fn context_request_keyboard_without_focus_returns_none() {
    let nodes = vec![TreeNode::leaf("a", "A")];
    let state = TreeState::default();

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let result = tree.context_request_tree_event_for_keyboard(&state);

    assert!(result.is_none());
}

#[test]
fn context_request_keyboard_selects_unselected_focused_node() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.selection.focused = Some("b");
    state.selection.selected.insert("a");
    state.selection.anchor = Some("a");

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Single)
        .context_selection_behavior(ContextSelectionBehavior::SelectTargetIfUnselected)
        .on_event(Message::Tree);

    let event = tree
        .context_request_tree_event_for_keyboard(&state)
        .expect("keyboard context request");

    assert!(matches!(
        event.state_change,
        Some(TreeStateChange::SetSelection(ref sel))
            if sel.selected == ["b"].into_iter().collect()
                && sel.focused == Some("b")
                && sel.anchor == Some("b")
    ));

    assert_eq!(
        event.kind,
        TreeEventKind::ContextRequested(ContextRequest {
            target: ContextTarget::Item("b"),
            selection: SelectionSnapshot {
                selected: vec!["b"],
                focused: Some("b"),
                anchor: Some("b"),
            },
            position: ContextPosition::FocusedItem,
            invocation: ContextInvocation::Keyboard,
        })
    );
}

#[test]
fn context_request_with_focus_only_behavior() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.select_only("a");

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Single)
        .context_selection_behavior(ContextSelectionBehavior::FocusOnly)
        .on_event(Message::Tree);

    let event = tree
        .context_request_tree_event_for_node(&state, &"b", Point::new(10.0, 20.0), false)
        .expect("context request");

    assert!(matches!(
        event.state_change,
        Some(TreeStateChange::SetSelection(ref sel))
            if sel.selected == ["a"].into_iter().collect()
                && sel.focused == Some("b")
    ));
}
