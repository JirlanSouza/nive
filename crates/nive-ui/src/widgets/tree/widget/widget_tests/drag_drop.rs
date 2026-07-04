use super::support::*;

#[test]
fn drag_starts_from_selected_node_set() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.selection.selected = ["a", "b"].into_iter().collect();
    state.selection.focused = Some("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .drag(TreeDrag::enabled())
        .on_event(Message::Tree);

    let event = tree
        .drag_start_tree_event(&state, &"a", ClickModifiers::NONE)
        .expect("drag start");

    assert_eq!(
        event.state_change,
        Some(TreeStateChange::SetTransfer(Transfer::Dragging {
            payload: crate::interaction::CollectionTransferPayload::flat(["a", "b"]),
            operation: TransferOperation::Move,
            target: None,
        }))
    );
}

#[test]
fn drag_from_unselected_node_selects_it_first() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.select_only("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .drag(TreeDrag::enabled())
        .on_event(Message::Tree);

    let event = tree
        .drag_start_tree_event(&state, &"b", ClickModifiers::NONE)
        .expect("drag start");

    assert!(matches!(
        event.state_change,
        Some(TreeStateChange::Batch(changes))
            if matches!(changes.first(), Some(TreeStateChange::SetSelection(selection))
                if selection.selected == ["b"].into_iter().collect())
                && matches!(changes.get(1), Some(TreeStateChange::SetTransfer(
                    Transfer::Dragging { payload, operation: TransferOperation::Move, target: None }
                )) if payload.ids == vec!["b"])
    ));
}

#[test]
fn drag_does_not_start_from_disabled_node() {
    let nodes = vec![TreeNode::leaf("a", "A").disabled(true)];
    let mut state = TreeState::default();
    state.selection.selected.insert("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .drag(TreeDrag::enabled())
        .on_event(Message::Tree);

    assert!(tree
        .drag_start_tree_event(&state, &"a", ClickModifiers::NONE)
        .is_none());
}

#[test]
fn valid_drop_feedback_sets_target() {
    let payload = crate::interaction::CollectionTransferPayload::flat(["a"]);
    let state = TreeState {
        transfer: Transfer::Dragging {
            payload: payload.clone(),
            operation: TransferOperation::Move,
            target: None,
        },
        ..TreeState::default()
    };
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .state(&state)
        .drag(
            TreeDrag::enabled()
                .can_drop(|drop| matches!(drop.target, TreeDropTarget::Into("folder"))),
        )
        .on_event(Message::Tree);

    let event = tree
        .drop_feedback_tree_event(
            &state,
            TreeDropTarget::Into("folder"),
            ClickModifiers::NONE,
            None,
        )
        .expect("drop feedback");

    assert_eq!(
        event.state_change,
        Some(TreeStateChange::SetTransfer(Transfer::Dragging {
            payload,
            operation: TransferOperation::Move,
            target: Some(TreeDropTarget::Into("folder")),
        }))
    );
}

#[test]
fn invalid_drop_feedback_clears_target() {
    let payload = crate::interaction::CollectionTransferPayload::flat(["a"]);
    let state = TreeState {
        transfer: Transfer::Dragging {
            payload: payload.clone(),
            operation: TransferOperation::Move,
            target: Some(TreeDropTarget::Into("folder")),
        },
        ..TreeState::default()
    };
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .state(&state)
        .drag(
            TreeDrag::enabled()
                .can_drop(|drop| matches!(drop.target, TreeDropTarget::Into("allowed"))),
        )
        .on_event(Message::Tree);

    let event = tree
        .drop_feedback_tree_event(
            &state,
            TreeDropTarget::Before("folder"),
            ClickModifiers::NONE,
            None,
        )
        .expect("drop feedback");

    assert_eq!(
        event.state_change,
        Some(TreeStateChange::SetTransfer(Transfer::Dragging {
            payload,
            operation: TransferOperation::Move,
            target: None,
        }))
    );
}

#[test]
fn valid_drop_emits_request_and_clears_transfer() {
    let payload = crate::interaction::CollectionTransferPayload::flat(["a"]);
    let state = TreeState {
        transfer: Transfer::Dragging {
            payload: payload.clone(),
            operation: TransferOperation::Move,
            target: Some(TreeDropTarget::Into("folder")),
        },
        ..TreeState::default()
    };
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .state(&state)
        .drag(
            TreeDrag::enabled()
                .can_drop(|drop| matches!(drop.target, TreeDropTarget::Into("folder"))),
        )
        .on_event(Message::Tree);

    let event = tree
        .drop_request_tree_event(
            &state,
            TreeDropTarget::Into("folder"),
            ClickModifiers::NONE,
            None,
        )
        .expect("drop request");

    assert_eq!(
        event.state_change,
        Some(TreeStateChange::SetTransfer(Transfer::None))
    );
    assert_eq!(
        event.kind,
        TreeEventKind::DropRequested(TreeDrop {
            payload,
            target: TreeDropTarget::Into("folder"),
            operation: TransferOperation::Move,
        })
    );
}

#[test]
fn invalid_drop_emits_no_request() {
    let state = TreeState {
        transfer: Transfer::Dragging {
            payload: crate::interaction::CollectionTransferPayload::flat(["a"]),
            operation: TransferOperation::Move,
            target: None,
        },
        ..TreeState::default()
    };
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .state(&state)
        .drag(TreeDrag::enabled().can_drop(|_| false))
        .on_event(Message::Tree);

    assert!(tree
        .drop_request_tree_event(&state, TreeDropTarget::Root, ClickModifiers::NONE, None)
        .is_none());
}

#[test]
fn primary_modifier_switches_drag_operation_to_copy_when_enabled() {
    let nodes = vec![TreeNode::leaf("a", "A")];
    let mut state = TreeState::default();
    state.select_only("a");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .drag(TreeDrag::enabled().operations([TransferOperation::Move, TransferOperation::Copy]))
        .on_event(Message::Tree);

    let event = tree
        .drag_start_tree_event(&state, &"a", ClickModifiers::new(true, false))
        .expect("copy drag start");

    assert!(matches!(
        event.state_change,
        Some(TreeStateChange::SetTransfer(Transfer::Dragging {
            operation: TransferOperation::Copy,
            ..
        }))
    ));
}

#[test]
fn explicit_link_operation_is_used_when_allowed() {
    let state = TreeState {
        transfer: Transfer::Dragging {
            payload: crate::interaction::CollectionTransferPayload::flat(["a"]),
            operation: TransferOperation::Move,
            target: None,
        },
        ..TreeState::default()
    };
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .state(&state)
        .drag(TreeDrag::enabled().operations([TransferOperation::Move, TransferOperation::Link]))
        .on_event(Message::Tree);

    let event = tree
        .drop_feedback_tree_event(
            &state,
            TreeDropTarget::Root,
            ClickModifiers::NONE,
            Some(TransferOperation::Link),
        )
        .expect("link feedback");

    assert!(matches!(
        event.state_change,
        Some(TreeStateChange::SetTransfer(Transfer::Dragging {
            operation: TransferOperation::Link,
            target: Some(TreeDropTarget::Root),
            ..
        }))
    ));
}

#[test]
fn drop_target_empty_tree_is_root() {
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .state(&state)
        .on_event(Message::Tree);

    assert_eq!(tree.drop_target_at(&state, 0.0), Some(TreeDropTarget::Root));
}

#[test]
fn drop_target_row_bands_map_to_before_into_and_after() {
    let nodes = vec![TreeNode::branch(
        "folder",
        "Folder",
        [TreeNode::leaf("child", "Child")],
    )];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);
    let height = row_height(ControlSize::Sm);

    assert_eq!(
        tree.drop_target_at(&state, 1.0),
        Some(TreeDropTarget::Before("folder"))
    );
    assert_eq!(
        tree.drop_target_at(&state, height / 2.0),
        Some(TreeDropTarget::Into("folder"))
    );
    assert_eq!(
        tree.drop_target_at(&state, height - 1.0),
        Some(TreeDropTarget::After("folder"))
    );
}

#[test]
fn drop_target_leaf_middle_maps_to_after() {
    let nodes = vec![TreeNode::leaf("file", "File")];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);
    let height = row_height(ControlSize::Sm);

    assert_eq!(
        tree.drop_target_at(&state, height / 2.0),
        Some(TreeDropTarget::After("file"))
    );
}

#[test]
fn drop_target_disabled_row_is_none() {
    let nodes = vec![TreeNode::leaf("file", "File").disabled(true)];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    assert_eq!(tree.drop_target_at(&state, 1.0), None);
}

#[test]
fn drop_target_loading_placeholder_maps_into_parent() {
    let nodes = vec![TreeNode::branch_deferred("remote", "Remote")];
    let mut state = TreeState::default();
    state.expand("remote");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);
    let height = row_height(ControlSize::Sm);

    assert_eq!(
        tree.drop_target_at(&state, height + 1.0),
        Some(TreeDropTarget::Into("remote"))
    );
}

#[test]
fn drop_target_below_last_row_is_root() {
    let nodes = vec![TreeNode::leaf("file", "File")];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);
    let height = row_height(ControlSize::Sm);

    assert_eq!(
        tree.drop_target_at(&state, height + 10.0),
        Some(TreeDropTarget::Root)
    );
}
