use iced::{keyboard, Point};

use super::{TransferData, TransferOperation, TransferOperations};

/// Drag source description shared by widgets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drag<T, Origin = ()> {
    /// Payload transferred by the drag.
    pub payload: TransferData<T>,
    /// App or widget source identity.
    pub origin: Origin,
    /// Operations allowed by the source.
    pub operations: TransferOperations,
    /// Preferred source operation when no modifier requests an allowed one.
    pub preferred: TransferOperation,
}

/// Drop target probe context.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropContext<'a, T, Origin = ()> {
    /// Drag payload.
    pub payload: &'a TransferData<T>,
    /// Drag source identity.
    pub origin: &'a Origin,
    /// Operations allowed by the source.
    pub operations: TransferOperations,
    /// Source preferred operation when no modifier requests an allowed one.
    pub preferred: TransferOperation,
    /// Operation requested by current modifiers, if any.
    pub requested: Option<TransferOperation>,
    /// Current pointer position.
    pub position: Point,
    /// Current keyboard modifiers.
    pub modifiers: keyboard::Modifiers,
}

impl<T, Origin> DropContext<'_, T, Origin> {
    /// Returns the source operation preferred for this context.
    pub fn preferred_operation(&self) -> Option<TransferOperation> {
        self.operations.preferred(self.requested, self.preferred)
    }
}

/// Drop target decision carrying a widget-specific target.
///
/// This enum is non-exhaustive; app matches should include a wildcard arm.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DropDecision<Target> {
    /// Reject the current payload or operation.
    Reject,
    /// Accept with a widget-specific target and effective operation.
    Accept {
        /// Widget-specific target.
        target: Target,
        /// Effective operation.
        operation: TransferOperation,
    },
}

impl<Target> DropDecision<Target> {
    /// Accepts a drop target with an effective operation.
    pub fn accept(target: Target, operation: TransferOperation) -> Self {
        Self::Accept { target, operation }
    }

    /// Returns whether the decision accepts the drop.
    pub fn is_accept(&self) -> bool {
        matches!(self, Self::Accept { .. })
    }
}

/// Successful drop commit emitted by widgets.
#[derive(Debug, Clone, PartialEq)]
pub struct DropCommit<T, Origin = (), Target = ()> {
    /// Transferred payload.
    pub payload: TransferData<T>,
    /// Drag source identity.
    pub origin: Origin,
    /// Accepted widget-specific target.
    pub target: Target,
    /// Effective operation.
    pub operation: TransferOperation,
    /// Release position.
    pub position: Point,
}

mod session;

#[allow(unused_imports)]
pub(crate) use session::*;
