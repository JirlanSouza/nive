use std::{fmt, sync::Arc};

use crate::interaction::{CollectionTransferPayload, TransferOperation, TransferOperations};

type CanDrop<Id> = Arc<dyn Fn(&TreeDrop<Id>) -> bool + Send + Sync + 'static>;

/// Drop request emitted by `Tree` after a candidate target is accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TreeDrop<Id> {
    /// Payload being dropped.
    pub payload: CollectionTransferPayload<Id>,
    /// Accepted tree drop target.
    pub target: TreeDropTarget<Id>,
    /// Effective transfer operation for the drop.
    pub operation: TransferOperation,
}

/// Tree-specific drop placement target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeDropTarget<Id> {
    /// Drop before the target node.
    Before(Id),
    /// Drop after the target node.
    After(Id),
    /// Drop into the target branch node.
    Into(Id),
    /// Drop at the tree root.
    Root,
}

/// Tree-specific paste target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreePasteTarget<Id> {
    /// Paste into the target branch node.
    Into(Id),
    /// Paste at the tree root.
    Root,
}

/// Drag/drop configuration for the high-level tree widget.
///
/// `Default` is disabled. Use [`TreeDrag::enabled`] to allow app-internal tree
/// drag/drop and optionally narrow accepted drops with [`TreeDrag::can_drop`].
#[derive(Clone)]
pub struct TreeDrag<Id> {
    enabled: bool,
    operations: TransferOperations,
    default_operation: TransferOperation,
    can_drop: Option<CanDrop<Id>>,
}

impl<Id> TreeDrag<Id> {
    /// Builds a disabled drag configuration.
    pub fn disabled() -> Self {
        Self::default()
    }

    /// Builds an enabled drag configuration using `Move` as the only operation.
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            operations: TransferOperations::MOVE,
            default_operation: TransferOperation::Move,
            can_drop: None,
        }
    }

    /// Replaces the allowed operations from an iterator.
    ///
    /// When the current default operation is not allowed by the new set, the
    /// default moves to the first allowed operation in stable preference order.
    pub fn operations(mut self, operations: impl IntoIterator<Item = TransferOperation>) -> Self {
        self.operations = operations
            .into_iter()
            .fold(TransferOperations::NONE, |operations, operation| {
                operations.with(operation)
            });

        if !self.operations.is_empty() && !self.operations.contains(self.default_operation) {
            self.default_operation = self.operations.first().unwrap_or(self.default_operation);
        }

        self
    }

    /// Sets the default operation and ensures it is allowed.
    pub fn default_operation(mut self, operation: TransferOperation) -> Self {
        self.default_operation = operation;
        self.operations = self.operations.with(operation);
        self
    }

    /// Sets the drop validation callback.
    pub fn can_drop(
        mut self,
        can_drop: impl Fn(&TreeDrop<Id>) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.can_drop = Some(Arc::new(can_drop));
        self
    }

    /// Returns whether drag/drop is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the allowed transfer operations.
    pub fn allowed_operations(&self) -> TransferOperations {
        self.operations
    }

    /// Returns the default operation used when no modifier selects another one.
    pub fn default_operation_value(&self) -> TransferOperation {
        self.default_operation
    }

    /// Returns whether the operation is enabled by this configuration.
    pub fn allows_operation(&self, operation: TransferOperation) -> bool {
        self.operations.contains(operation)
    }

    /// Returns whether a candidate drop is enabled, allowed, and accepted.
    pub fn accepts_drop(&self, tree_drop: &TreeDrop<Id>) -> bool {
        self.enabled
            && self.operations.contains(tree_drop.operation)
            && self
                .can_drop
                .as_ref()
                .is_none_or(|can_drop| can_drop(tree_drop))
    }
}

impl<Id> Default for TreeDrag<Id> {
    fn default() -> Self {
        Self {
            enabled: false,
            operations: TransferOperations::NONE,
            default_operation: TransferOperation::Move,
            can_drop: None,
        }
    }
}

impl<Id> fmt::Debug for TreeDrag<Id> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TreeDrag")
            .field("enabled", &self.enabled)
            .field("operations", &self.operations)
            .field("default_operation", &self.default_operation)
            .field("can_drop", &self.can_drop.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tree_transfer_tests {
    use super::*;

    fn tree_drop(target: TreeDropTarget<i32>, operation: TransferOperation) -> TreeDrop<i32> {
        TreeDrop {
            payload: CollectionTransferPayload::new(vec![1, 2], vec![1]),
            target,
            operation,
        }
    }

    #[test]
    fn drop_and_paste_targets_keep_payload_shape() {
        let payload = CollectionTransferPayload::new(vec![1, 2], vec![1]);
        let tree_drop = TreeDrop {
            payload: payload.clone(),
            target: TreeDropTarget::Into(3),
            operation: TransferOperation::Move,
        };

        assert_eq!(tree_drop.payload, payload);
        assert_eq!(tree_drop.target, TreeDropTarget::Into(3));
        assert_eq!(tree_drop.operation, TransferOperation::Move);
        assert_eq!(TreeDropTarget::Before(3), TreeDropTarget::Before(3));
        assert_eq!(TreeDropTarget::After(3), TreeDropTarget::After(3));
        assert_eq!(TreeDropTarget::<i32>::Root, TreeDropTarget::Root);
        assert_eq!(TreePasteTarget::Into(3), TreePasteTarget::Into(3));
        assert_eq!(TreePasteTarget::<i32>::Root, TreePasteTarget::Root);
    }

    #[test]
    fn drag_default_is_disabled() {
        let drag = TreeDrag::<i32>::default();

        assert!(!drag.is_enabled());
        assert_eq!(drag.allowed_operations(), TransferOperations::NONE);
        assert_eq!(drag.default_operation_value(), TransferOperation::Move);
        assert!(!drag.accepts_drop(&tree_drop(TreeDropTarget::Root, TransferOperation::Move)));
    }

    #[test]
    fn enabled_drag_defaults_to_move() {
        let drag = TreeDrag::<i32>::enabled();

        assert!(drag.is_enabled());
        assert_eq!(drag.allowed_operations(), TransferOperations::MOVE);
        assert!(drag.allows_operation(TransferOperation::Move));
        assert_eq!(drag.default_operation_value(), TransferOperation::Move);
        assert!(drag.accepts_drop(&tree_drop(TreeDropTarget::Root, TransferOperation::Move)));
    }

    #[test]
    fn drag_operations_and_default_operation_are_configurable() {
        let drag = TreeDrag::<i32>::enabled()
            .operations([TransferOperation::Copy, TransferOperation::Link])
            .default_operation(TransferOperation::Link);

        assert!(drag.allows_operation(TransferOperation::Copy));
        assert!(!drag.allows_operation(TransferOperation::Move));
        assert!(drag.allows_operation(TransferOperation::Link));
        assert_eq!(drag.default_operation_value(), TransferOperation::Link);
    }

    #[test]
    fn operations_replace_unsupported_default_with_first_allowed() {
        let drag = TreeDrag::<i32>::enabled()
            .default_operation(TransferOperation::Link)
            .operations([TransferOperation::Copy]);

        assert_eq!(drag.default_operation_value(), TransferOperation::Copy);
    }

    #[test]
    fn can_drop_callback_validates_candidate_drops_and_clones() {
        let drag = TreeDrag::<i32>::enabled()
            .operations([TransferOperation::Move, TransferOperation::Copy])
            .can_drop(|tree_drop| {
                matches!(tree_drop.target, TreeDropTarget::Into(3))
                    && tree_drop.operation == TransferOperation::Move
            });
        let cloned = drag.clone();

        assert!(cloned.accepts_drop(&tree_drop(TreeDropTarget::Into(3), TransferOperation::Move)));
        assert!(!cloned.accepts_drop(&tree_drop(
            TreeDropTarget::Before(3),
            TransferOperation::Move
        )));
        assert!(!cloned.accepts_drop(&tree_drop(TreeDropTarget::Into(3), TransferOperation::Copy)));
    }
}
