use super::support::*;

#[test]
fn escape_cancels_active_drag() {
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
        .on_event(Message::Tree);

    let event = tree.handle_escape_tree(&state).expect("cancel drag event");

    assert_eq!(
        event.state_change,
        Some(TreeStateChange::SetTransfer(Transfer::None))
    );
}

#[test]
fn escape_clears_cut_clipboard_feedback() {
    let state = TreeState {
        transfer: Transfer::Clipboard {
            payload: crate::interaction::CollectionTransferPayload::flat(["a"]),
            operation: TransferOperation::Move,
        },
        ..TreeState::default()
    };
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .state(&state)
        .on_event(Message::Tree);

    let event = tree
        .handle_escape_tree(&state)
        .expect("clear cut feedback event");

    assert_eq!(
        event.state_change,
        Some(TreeStateChange::SetTransfer(Transfer::None))
    );
}

#[test]
fn escape_does_not_clear_copy_clipboard_feedback() {
    let state = TreeState {
        transfer: Transfer::Clipboard {
            payload: crate::interaction::CollectionTransferPayload::flat(["a"]),
            operation: TransferOperation::Copy,
        },
        ..TreeState::default()
    };
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .state(&state)
        .on_event(Message::Tree);

    assert!(tree.handle_escape_tree(&state).is_none());
}

#[test]
fn escape_is_noop_without_transfer_state() {
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(Vec::<TreeNode<'_, &'static str>>::new())
        .state(&state)
        .on_event(Message::Tree);

    assert!(tree.handle_escape_tree(&state).is_none());
}

#[test]
fn transfer_payload_preserves_visible_order_and_normalizes_roots() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [
            TreeNode::leaf("a", "A"),
            TreeNode::branch("b", "B", [TreeNode::leaf("c", "C")]),
        ],
    )];
    let mut state = TreeState::default();
    state.expand("root");
    state.expand("b");
    state.selection.selected = ["root", "a", "c"].into_iter().collect();
    state.selection.focused = Some("root");

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let payload = tree
        .transfer_payload(&state, Some(&"root"))
        .expect("payload");

    assert_eq!(payload.ids, vec!["root", "a", "c"]);
    assert_eq!(payload.root_ids, vec!["root"]);
}

#[test]
fn transfer_payload_validates_initiating_node() {
    let nodes = vec![TreeNode::leaf("a", "A"), TreeNode::leaf("b", "B")];
    let mut state = TreeState::default();
    state.selection.selected.insert("a");

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    assert!(tree.transfer_payload(&state, Some(&"b")).is_none());
}

#[test]
fn transfer_payload_skips_disabled_selected_rows() {
    let nodes = vec![
        TreeNode::leaf("a", "A"),
        TreeNode::leaf("b", "B").disabled(true),
    ];
    let mut state = TreeState::default();
    state.selection.selected = ["a", "b"].into_iter().collect();
    state.selection.focused = Some("a");

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let payload = tree.transfer_payload(&state, Some(&"a")).expect("payload");

    assert_eq!(payload.ids, vec!["a"]);
    assert_eq!(payload.root_ids, vec!["a"]);
}

#[test]
fn copy_emits_payload_without_transfer_state() {
    let nodes = vec![TreeNode::leaf("a", "A")];
    let mut state = TreeState::default();
    state.select_only("a");

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree.handle_copy_tree(&state).expect("copy event");

    assert!(event.state_change.is_none());
    assert_eq!(
        event.kind,
        TreeEventKind::CopyRequested(crate::interaction::CollectionTransferPayload::flat(["a"]))
    );
}

#[test]
fn cut_emits_payload_and_cut_feedback_state() {
    let nodes = vec![TreeNode::leaf("a", "A")];
    let mut state = TreeState::default();
    state.select_only("a");

    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree.handle_cut_tree(&state).expect("cut event");
    let payload = crate::interaction::CollectionTransferPayload::flat(["a"]);

    assert_eq!(event.kind, TreeEventKind::CutRequested(payload.clone()));
    assert_eq!(
        event.state_change,
        Some(TreeStateChange::SetTransfer(Transfer::Clipboard {
            payload,
            operation: TransferOperation::Move,
        }))
    );
}

#[test]
fn empty_selection_does_not_copy_or_cut() {
    let nodes = vec![TreeNode::leaf("a", "A")];
    let state = TreeState::default();
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    assert!(tree.handle_copy_tree(&state).is_none());
    assert!(tree.handle_cut_tree(&state).is_none());
}

#[test]
fn paste_targets_focused_enabled_branch() {
    let nodes = vec![TreeNode::branch(
        "folder",
        "Folder",
        Vec::<TreeNode<'_, &str>>::new(),
    )];
    let mut state = TreeState::default();
    state.selection.focused = Some("folder");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree.handle_paste_tree(&state).expect("paste event");

    assert_eq!(
        event.kind,
        TreeEventKind::PasteRequested(TreePasteTarget::Into("folder"))
    );
}

#[test]
fn paste_targets_selected_enabled_branch_when_focus_is_not_branch() {
    let nodes = vec![
        TreeNode::leaf("file", "File"),
        TreeNode::branch("folder", "Folder", Vec::<TreeNode<'_, &str>>::new()),
    ];
    let mut state = TreeState::default();
    state.selection.focused = Some("file");
    state.selection.selected.insert("folder");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree.handle_paste_tree(&state).expect("paste event");

    assert_eq!(
        event.kind,
        TreeEventKind::PasteRequested(TreePasteTarget::Into("folder"))
    );
}

#[test]
fn paste_targets_root_without_branch_target() {
    let nodes = vec![TreeNode::leaf("file", "File")];
    let mut state = TreeState::default();
    state.select_only("file");
    let tree = Tree::<_, Message>::new(nodes)
        .state(&state)
        .on_event(Message::Tree);

    let event = tree.handle_paste_tree(&state).expect("paste event");

    assert_eq!(
        event.kind,
        TreeEventKind::PasteRequested(TreePasteTarget::Root)
    );
}
