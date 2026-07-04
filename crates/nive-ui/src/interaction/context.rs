use iced::Point;

use super::SelectionSnapshot;

/// Context-menu request emitted by collection widgets.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextRequest<Id> {
    /// Requested target.
    pub target: ContextTarget<Id>,
    /// Selection snapshot in visible order.
    pub selection: SelectionSnapshot<Id>,
    /// Pointer or focused-item position.
    pub position: ContextPosition,
    /// Invocation source.
    pub invocation: ContextInvocation,
}

/// Context request target.
///
/// This enum is non-exhaustive; app matches should include a wildcard arm.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContextTarget<Id> {
    /// A concrete item.
    Item(Id),
    /// Empty collection space.
    Empty,
}

/// Context request position.
///
/// This enum is non-exhaustive; app matches should include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum ContextPosition {
    /// Pointer position in widget coordinates.
    Pointer(Point),
    /// Focused item position. The widget owns exact geometry.
    FocusedItem,
}

/// Context invocation source.
///
/// This enum is non-exhaustive; app matches should include a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContextInvocation {
    /// Secondary pointer click.
    SecondaryClick,
    /// Keyboard context key or shortcut.
    Keyboard,
}

/// How a context request updates selection first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContextSelectionBehavior {
    /// Select and focus an unselected target before requesting context.
    #[default]
    SelectTargetIfUnselected,
    /// Preserve the current selection.
    PreserveSelection,
    /// Move focus only.
    FocusOnly,
}
