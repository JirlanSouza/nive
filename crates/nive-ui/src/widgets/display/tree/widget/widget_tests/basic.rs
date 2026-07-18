use super::support::*;

#[test]
fn row_focus_border_is_projected_only_while_tree_focus_is_visible() {
    assert!(super::super::project_row_focus(true, true));
    assert!(!super::super::project_row_focus(false, true));
    assert!(!super::super::project_row_focus(true, false));
}

#[test]
fn defaults_match_tree_contract() {
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new());

    assert_eq!(tree.selection_mode, SelectionMode::Single);
    assert_eq!(tree.activation_behavior, ActivationBehavior::Platform);
    assert_eq!(tree.rename_behavior, RenameBehavior::Platform);
    assert_eq!(tree.expand_behavior, TreeExpandBehavior::SingleClick);
    assert_eq!(
        tree.context_selection_behavior,
        ContextSelectionBehavior::SelectTargetIfUnselected
    );
    assert!(tree.type_ahead);
    assert_eq!(tree.page_rows, 10);
    assert_eq!(tree.width, Length::Fill);
    assert_eq!(tree.height, Length::Fill);
    assert!(tree.has_internal_scroll());
}

#[test]
fn no_scroll_opts_out_of_internal_viewport() {
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new()).no_scroll();

    assert!(!tree.has_internal_scroll());
}

#[test]
fn page_rows_clamps_to_one() {
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new()).page_rows(0);

    assert_eq!(tree.page_rows, 1);
}

#[test]
fn public_api_compile_contract() {
    let nodes = vec![
        TreeNode::branch(
            "src",
            "src",
            [
                TreeNode::leaf("main", "main.rs"),
                TreeNode::leaf("lib", "lib.rs"),
            ],
        ),
        TreeNode::branch_deferred("deps", "Dependencies"),
        TreeNode::leaf("cargo", "Cargo.toml"),
    ];

    let mut state = TreeState::default();
    state.expand("src");

    {
        let mode = SelectionMode::Multiple;
        let _tree: Tree<'_, &'static str, Message> = Tree::new(nodes)
            .state(&state)
            .selection_mode(mode)
            .on_event(Message::Tree);
    }

    let event = TreeEvent {
        state_change: Some(TreeStateChange::SetExpanded {
            id: "deps",
            expanded: true,
        }),
        kind: TreeEventKind::ExpandRequested { id: "deps" },
    };

    state.apply(&event);

    if let TreeEventKind::ExpandRequested { id } = event.kind {
        assert_eq!(id, "deps");
    }

    assert!(state.is_expanded(&"deps"));

    let state_changed = TreeEvent {
        state_change: Some(TreeStateChange::SetSelection(
            crate::interaction::Selection {
                selected: ["main"].into_iter().collect(),
                focused: Some("main"),
                anchor: Some("main"),
            },
        )),
        kind: TreeEventKind::StateChanged,
    };

    state.apply(&state_changed);

    if let TreeEventKind::ExpandRequested { .. } = state_changed.kind {
        unreachable!()
    }

    assert!(state.is_selected(&"main"));

    {
        let _none_tree: Tree<'_, &'static str, Message> =
            Tree::new(Vec::<TreeNode<'_, &'static str>>::new())
                .state(&state)
                .selection_mode(SelectionMode::None)
                .on_event(Message::Tree);
    }

    {
        let _single_tree: Tree<'_, &'static str, Message> =
            Tree::new(Vec::<TreeNode<'_, &'static str>>::new())
                .state(&state)
                .selection_mode(SelectionMode::Single)
                .on_event(Message::Tree);
    }
}
