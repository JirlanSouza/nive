use super::support::*;

#[test]
fn type_ahead_matches_next_row_by_prefix() {
    let nodes = vec![
        TreeNode::leaf("a", "Apple"),
        TreeNode::leaf("b", "Banana"),
        TreeNode::leaf("c", "Cherry"),
    ];
    let mut state = TreeState::default();
    state.selection.focused = Some("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_type_ahead_tree(&state, "b")
        .expect("type-ahead match");

    assert_eq!(expect_selection(&event).focused, Some("b"));
}

#[test]
fn type_ahead_is_case_insensitive() {
    let nodes = vec![TreeNode::leaf("a", "Apple"), TreeNode::leaf("b", "Banana")];
    let mut state = TreeState::default();
    state.selection.focused = Some("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_type_ahead_tree(&state, "BAN")
        .expect("case-insensitive match");

    assert_eq!(expect_selection(&event).focused, Some("b"));
}

#[test]
fn type_ahead_wraps_past_the_end() {
    let nodes = vec![
        TreeNode::leaf("a", "Apple"),
        TreeNode::leaf("b", "Banana"),
        TreeNode::leaf("c", "Cherry"),
    ];
    let mut state = TreeState::default();
    state.selection.focused = Some("c");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_type_ahead_tree(&state, "a")
        .expect("wraps around to apple");

    assert_eq!(expect_selection(&event).focused, Some("a"));
}

#[test]
fn type_ahead_skips_disabled_rows() {
    let nodes = vec![
        TreeNode::leaf("a", "Apple"),
        TreeNode::leaf("b", "Banana").disabled(true),
        TreeNode::leaf("c", "Banjo"),
    ];
    let mut state = TreeState::default();
    state.selection.focused = Some("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_type_ahead_tree(&state, "ban")
        .expect("skips disabled banana row");

    assert_eq!(expect_selection(&event).focused, Some("c"));
}

#[test]
fn type_ahead_disabled_by_builder_returns_none() {
    let nodes = vec![TreeNode::leaf("a", "Apple"), TreeNode::leaf("b", "Banana")];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .type_ahead(false)
        .on_event(Message::Tree);

    assert!(tree.handle_type_ahead_tree(&state, "b").is_none());
}

#[test]
fn type_ahead_without_match_returns_none() {
    let nodes = vec![TreeNode::leaf("a", "Apple"), TreeNode::leaf("b", "Banana")];
    let mut state = TreeState::default();
    state.selection.focused = Some("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    assert!(tree.handle_type_ahead_tree(&state, "z").is_none());
}
