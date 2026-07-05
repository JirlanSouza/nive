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
fn expander_only_row_click_does_not_toggle_expansion() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [TreeNode::leaf("child", "Child")],
    )];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .expand_behavior(TreeExpandBehavior::ExpanderOnly)
        .on_event(Message::Tree);

    let event = tree
        .row_tree_event(&state, &"root", Some(false), ClickModifiers::NONE)
        .expect("row event");

    assert!(matches!(
        event.state_change,
        Some(TreeStateChange::SetSelection(_))
    ));
}

#[test]
fn expander_only_expander_still_toggles() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [TreeNode::leaf("child", "Child")],
    )];
    let tree = Tree::<_, Message>::new(nodes)
        .expand_behavior(TreeExpandBehavior::ExpanderOnly)
        .on_event(Message::Tree);

    let event = tree
        .toggle_tree_event("root", Some(false))
        .expect("toggle event");

    assert_eq!(
        event.state_change,
        Some(TreeStateChange::SetExpanded {
            id: "root",
            expanded: true,
        })
    );
}

#[test]
fn double_click_single_click_does_not_toggle_expansion() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [TreeNode::leaf("child", "Child")],
    )];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .expand_behavior(TreeExpandBehavior::DoubleClick)
        .on_event(Message::Tree);

    let event = tree
        .row_tree_event(&state, &"root", Some(false), ClickModifiers::NONE)
        .expect("row event");

    assert!(matches!(
        event.state_change,
        Some(TreeStateChange::SetSelection(_))
    ));
}

#[test]
fn double_click_toggles_expansion() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [TreeNode::leaf("child", "Child")],
    )];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .expand_behavior(TreeExpandBehavior::DoubleClick)
        .activation_behavior(ActivationBehavior::Enter)
        .on_event(Message::Tree);

    let event = tree
        .double_click_row_tree_event(&state, &"root", Some(false))
        .expect("double click event");

    assert_eq!(event.kind, TreeEventKind::StateChanged);
    assert!(matches!(
        event.state_change,
        Some(TreeStateChange::Batch(changes)) if changes.len() == 2
    ));
}

#[test]
fn double_click_with_activation_emits_composed_event() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [TreeNode::leaf("child", "Child")],
    )];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .expand_behavior(TreeExpandBehavior::DoubleClick)
        .activation_behavior(ActivationBehavior::DoubleClick)
        .on_event(Message::Tree);

    let event = tree
        .double_click_row_tree_event(&state, &"root", Some(false))
        .expect("double click event");

    assert_eq!(
        event.kind,
        TreeEventKind::Activate {
            id: "root",
            trigger: ActivationTrigger::DoubleClick,
        }
    );
    assert!(matches!(
        event.state_change,
        Some(TreeStateChange::Batch(changes)) if changes.len() == 2
    ));
}

#[test]
fn double_click_on_leaf_with_activation_emits_activate() {
    let nodes = vec![TreeNode::leaf("leaf", "Leaf")];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .expand_behavior(TreeExpandBehavior::DoubleClick)
        .activation_behavior(ActivationBehavior::DoubleClick)
        .on_event(Message::Tree);

    let event = tree
        .double_click_row_tree_event(&state, &"leaf", None)
        .expect("double click event");

    assert_eq!(
        event.kind,
        TreeEventKind::Activate {
            id: "leaf",
            trigger: ActivationTrigger::DoubleClick,
        }
    );
}

#[test]
fn double_click_disabled_row_emits_nothing() {
    let nodes =
        vec![TreeNode::branch("root", "Root", [TreeNode::leaf("child", "Child")]).disabled(true)];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .expand_behavior(TreeExpandBehavior::DoubleClick)
        .on_event(Message::Tree);

    let result = tree.double_click_row_event(&state, &"root", Some(false), true);

    assert!(result.is_none());
}

#[test]
fn single_click_on_leaf_selects_without_expansion() {
    let nodes = vec![TreeNode::leaf("leaf", "Leaf")];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Single)
        .on_event(Message::Tree);

    let event = tree
        .row_tree_event(&state, &"leaf", None, ClickModifiers::NONE)
        .expect("leaf click");

    assert_eq!(event.kind, TreeEventKind::StateChanged);
    assert!(matches!(
        event.state_change,
        Some(TreeStateChange::SetSelection(_))
    ));
}

#[test]
fn single_click_collapses_expanded_branch() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [TreeNode::leaf("child", "Child")],
    )];
    let mut state = TreeState::default();
    state.expand("root");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .expand_behavior(TreeExpandBehavior::SingleClick)
        .on_event(Message::Tree);

    let event = tree
        .row_tree_event(&state, &"root", Some(true), ClickModifiers::NONE)
        .expect("collapse event");

    assert!(matches!(
        event.state_change,
        Some(TreeStateChange::Batch(changes)) if changes.iter().any(|c| matches!(
            c,
            TreeStateChange::SetExpanded { id, expanded: false } if *id == "root"
        ))
    ));
}
