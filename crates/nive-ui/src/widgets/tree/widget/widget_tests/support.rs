pub(super) use crate::interaction::{
    ActivationBehavior, ActivationTrigger, ClickModifiers, ContextInvocation, ContextPosition,
    ContextRequest, ContextSelectionBehavior, ContextTarget, RenameBehavior, Selection,
    SelectionMode, SelectionSnapshot, Transfer, TransferOperation,
};
pub(super) use crate::theme::ControlSize;
pub(super) use iced::{Length, Point};

pub(super) use super::super::super::event::{TreeEvent, TreeEventKind};
pub(super) use super::super::super::row_height;
pub(super) use super::super::super::state::{TreeState, TreeStateChange};
pub(super) use super::super::super::transfer::{
    TreeDrag, TreeDrop, TreeDropTarget, TreePasteTarget,
};
pub(super) use super::super::super::TreeNode;
pub(super) use super::super::selection::selection_for_click;
pub(super) use super::super::{Tree, TreeExpandBehavior};

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Message {
    Tree(TreeEvent<&'static str>),
}

pub(super) fn expect_selection<'e>(
    event: &'e TreeEvent<&'static str>,
) -> &'e Selection<&'static str> {
    match &event.state_change {
        Some(TreeStateChange::SetSelection(selection)) => selection,
        other => panic!("expected SetSelection change, got {other:?}"),
    }
}
