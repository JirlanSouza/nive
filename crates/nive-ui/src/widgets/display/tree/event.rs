use crate::interaction::{ActivationTrigger, CollectionTransferPayload, ContextRequest};

use super::{
    state::TreeStateChange,
    transfer::{TreeDrop, TreePasteTarget},
};

/// Semantic event emitted by the high-level tree widget.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeEvent<Id> {
    /// Optional standard state delta for the app-owned `TreeState`.
    pub state_change: Option<TreeStateChange<Id>>,
    /// Domain-neutral tree intent or state-only notification.
    pub kind: TreeEventKind<Id>,
}

/// Tree event kind.
///
/// This enum is non-exhaustive; app matches should include a wildcard arm.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TreeEventKind<Id> {
    /// The event only carries a standard tree state change.
    StateChanged,
    /// A deferred branch was expanded and should be loaded by the app.
    ExpandRequested {
        /// Branch ID that needs app-side loading.
        id: Id,
    },
    /// A node activation/default-action intent.
    Activate {
        /// Activated node ID.
        id: Id,
        /// Input source that triggered activation.
        trigger: ActivationTrigger,
    },
    /// A node rename intent.
    RenameRequested {
        /// Node ID requested for rename.
        id: Id,
    },
    /// Context-menu intent.
    ContextRequested(ContextRequest<Id>),
    /// Copy intent with normalized collection payload.
    CopyRequested(CollectionTransferPayload<Id>),
    /// Cut intent with normalized collection payload.
    CutRequested(CollectionTransferPayload<Id>),
    /// Paste intent with a tree-specific target.
    PasteRequested(TreePasteTarget<Id>),
    /// Validated drop intent.
    DropRequested(TreeDrop<Id>),
}

#[cfg(test)]
mod tree_event_tests {
    use iced::Point;

    use crate::interaction::{
        ContextInvocation, ContextPosition, ContextTarget, SelectionSnapshot, TransferOperation,
    };

    use super::super::transfer::TreeDropTarget;
    use super::*;

    #[test]
    fn event_carries_optional_state_change_and_kind() {
        let event: TreeEvent<i32> = TreeEvent {
            state_change: None,
            kind: TreeEventKind::StateChanged,
        };

        assert_eq!(event.state_change, None);
        assert_eq!(event.kind, TreeEventKind::StateChanged);
    }

    #[test]
    fn event_kinds_cover_tree_domain_intents() {
        let payload = CollectionTransferPayload::new(vec![1, 2], vec![1]);
        let context_request = ContextRequest {
            target: ContextTarget::Item(1),
            selection: SelectionSnapshot {
                selected: vec![1],
                focused: Some(1),
                anchor: Some(1),
            },
            position: ContextPosition::Pointer(Point::new(4.0, 8.0)),
            invocation: ContextInvocation::SecondaryClick,
        };
        let tree_drop = TreeDrop {
            payload: payload.clone(),
            target: TreeDropTarget::After(3),
            operation: TransferOperation::Move,
        };

        assert_eq!(
            TreeEventKind::ExpandRequested { id: 1 },
            TreeEventKind::ExpandRequested { id: 1 }
        );
        assert_eq!(
            TreeEventKind::Activate {
                id: 1,
                trigger: ActivationTrigger::DoubleClick
            },
            TreeEventKind::Activate {
                id: 1,
                trigger: ActivationTrigger::DoubleClick
            }
        );
        assert_eq!(
            TreeEventKind::RenameRequested { id: 1 },
            TreeEventKind::RenameRequested { id: 1 }
        );
        assert_eq!(
            TreeEventKind::ContextRequested(context_request.clone()),
            TreeEventKind::ContextRequested(context_request)
        );
        assert_eq!(
            TreeEventKind::CopyRequested(payload.clone()),
            TreeEventKind::CopyRequested(payload.clone())
        );
        assert_eq!(
            TreeEventKind::CutRequested(payload.clone()),
            TreeEventKind::CutRequested(payload.clone())
        );
        assert_eq!(
            TreeEventKind::PasteRequested(TreePasteTarget::<i32>::Root),
            TreeEventKind::PasteRequested(TreePasteTarget::Root)
        );
        assert_eq!(
            TreeEventKind::DropRequested(tree_drop.clone()),
            TreeEventKind::DropRequested(tree_drop)
        );
    }
}
