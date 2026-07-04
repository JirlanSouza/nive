use crate::interaction::{CollectionTransferPayload, TransferOperation};

use super::super::event::{TreeEvent, TreeEventKind};
use super::*;

#[test]
fn apply_changes_expansion() {
    let mut state = TreeState::default();
    let event = TreeEvent {
        state_change: Some(TreeStateChange::SetExpanded {
            id: 1,
            expanded: true,
        }),
        kind: TreeEventKind::StateChanged,
    };

    state.apply(&event);

    assert!(state.is_expanded(&1));
}

#[test]
fn apply_replaces_selection_focus_and_anchor() {
    let mut state = TreeState::default();
    let selection = Selection {
        selected: [1, 3].into_iter().collect(),
        focused: Some(3),
        anchor: Some(1),
    };

    state.apply_change(&TreeStateChange::SetSelection(selection.clone()));

    assert_eq!(state.selection, selection);
}

#[test]
fn apply_replaces_transfer() {
    let mut state = TreeState::default();
    let transfer = Transfer::Clipboard {
        payload: CollectionTransferPayload::flat([1, 2]),
        operation: TransferOperation::Move,
    };

    state.apply_change(&TreeStateChange::SetTransfer(transfer.clone()));

    assert_eq!(state.transfer, transfer);
}

#[test]
fn clear_selection_preserves_focus() {
    let mut state = TreeState {
        selection: Selection {
            selected: [1, 2].into_iter().collect(),
            focused: Some(2),
            anchor: Some(1),
        },
        ..TreeState::default()
    };

    state.apply_change(&TreeStateChange::ClearSelection);

    assert!(state.selection.selected.is_empty());
    assert_eq!(state.selection.focused, Some(2));
    assert_eq!(state.selection.anchor, None);
}

#[test]
fn batch_applies_changes_in_order() {
    let mut state = TreeState::default();

    state.apply_change(&TreeStateChange::Batch(vec![
        TreeStateChange::SetExpanded {
            id: 1,
            expanded: true,
        },
        TreeStateChange::SetExpanded {
            id: 1,
            expanded: false,
        },
    ]));

    assert!(!state.is_expanded(&1));
}

#[test]
fn reveal_expands_loaded_ancestors_and_focuses_target() {
    let nodes = vec![TreeNode::branch(
        "root",
        "Root",
        [TreeNode::branch(
            "src",
            "src",
            [TreeNode::leaf("main", "main.rs")],
        )],
    )];
    let mut state = TreeState::default();

    assert!(state.reveal(&nodes, &"main"));

    assert!(state.is_expanded(&"root"));
    assert!(state.is_expanded(&"src"));
    assert_eq!(state.focused(), Some(&"main"));
}

#[test]
fn reveal_returns_false_for_absent_ids() {
    let nodes = vec![TreeNode::leaf("root", "Root")];
    let mut state = TreeState::default();

    assert!(!state.reveal(&nodes, &"missing"));
    assert_eq!(state.focused(), None);
}

#[test]
fn retain_ids_prunes_state_and_transfer() {
    let mut state = TreeState {
        expanded: [1, 2].into_iter().collect(),
        selection: Selection {
            selected: [1, 3].into_iter().collect(),
            focused: Some(3),
            anchor: Some(2),
        },
        transfer: Transfer::Dragging {
            payload: CollectionTransferPayload::new(vec![1, 2, 3], vec![1, 3]),
            operation: TransferOperation::Move,
            target: Some(TreeDropTarget::Into(3)),
        },
    };

    state.retain_ids(|id| *id != 3);

    assert_eq!(state.expanded, [1, 2].into_iter().collect());
    assert_eq!(state.selection.selected, [1].into_iter().collect());
    assert_eq!(state.selection.focused, None);
    assert_eq!(state.selection.anchor, Some(2));
    assert_eq!(
        state.transfer,
        Transfer::Dragging {
            payload: CollectionTransferPayload::new(vec![1, 2], vec![1]),
            operation: TransferOperation::Move,
            target: None,
        }
    );
}

#[test]
fn retain_ids_prunes_clipboard_transfer() {
    let mut state = TreeState {
        expanded: BTreeSet::new(),
        selection: Selection::default(),
        transfer: Transfer::Clipboard {
            payload: CollectionTransferPayload::new(vec![1, 2, 3], vec![1, 3]),
            operation: TransferOperation::Move,
        },
    };

    state.retain_ids(|id| *id != 3);

    assert_eq!(
        state.transfer,
        Transfer::Clipboard {
            payload: CollectionTransferPayload::new(vec![1, 2], vec![1]),
            operation: TransferOperation::Move,
        }
    );
}

#[test]
fn helpers_update_state() {
    let mut state = TreeState::default();

    state.expand(1);
    assert!(state.is_expanded(&1));
    assert!(!state.toggle_expanded(1));
    state.select_only(2);
    assert!(state.is_selected(&2));
    assert_eq!(state.focused(), Some(&2));
    state.select_many([3, 4]);
    assert_eq!(state.selection.selected, [3, 4].into_iter().collect());
    state.set_focused(Some(4));
    assert_eq!(state.focused(), Some(&4));
    state.collapse(&1);
    assert!(!state.is_expanded(&1));
}
