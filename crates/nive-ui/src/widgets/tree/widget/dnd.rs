use crate::interaction::{ClickModifiers, Transfer, TransferOperation};
use crate::theme::{ControlSize, ToneRole};
use crate::widgets::{LoadingIndicator, TreeItem};
use crate::Element;

use super::super::event::{TreeEvent, TreeEventKind};
use super::super::state::{TreeState, TreeStateChange};
use super::super::transfer::{TreeDrop, TreeDropTarget};
use super::super::visible::{visible_entries, VisibleTreeEntry};
use super::selection::batch_or_single;
use super::Tree;

pub(super) fn loading_row<'a, Message>(depth: usize, size: ControlSize) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    TreeItem::new("Loading...")
        .depth(depth)
        .disabled(true)
        .tone(ToneRole::Neutral)
        .trailing(LoadingIndicator::new().neutral().size(size))
        .size(size)
        .into()
}

impl<'a, Id, Message> Tree<'a, Id, Message>
where
    Id: Clone + Ord + 'a,
    Message: Clone + 'a,
{
    fn drag_operation(
        &self,
        modifiers: ClickModifiers,
        requested: Option<TransferOperation>,
    ) -> Option<TransferOperation> {
        if !self.drag.is_enabled() {
            return None;
        }

        if let Some(requested) = requested {
            if self.drag.allows_operation(requested) {
                return Some(requested);
            }
        }

        if modifiers.primary && self.drag.allows_operation(TransferOperation::Copy) {
            return Some(TransferOperation::Copy);
        }

        Some(self.drag.default_operation_value())
            .filter(|operation| self.drag.allows_operation(*operation))
    }

    pub(crate) fn drag_start_tree_event(
        &self,
        state: &TreeState<Id>,
        id: &Id,
        modifiers: ClickModifiers,
    ) -> Option<TreeEvent<Id>> {
        if !self.drag.is_enabled() || self.is_disabled_visible_node(state, id) {
            return None;
        }

        let operation = self.drag_operation(modifiers, None)?;
        let mut changes = Vec::new();
        let payload = if state.is_selected(id) {
            self.transfer_payload(state, Some(id))?
        } else {
            let mut selected_state = state.clone();
            selected_state.select_only(id.clone());
            changes.push(TreeStateChange::SetSelection(
                selected_state.selection.clone(),
            ));
            self.transfer_payload(&selected_state, Some(id))?
        };

        changes.push(TreeStateChange::SetTransfer(Transfer::Dragging {
            payload,
            operation,
            target: None,
        }));

        Some(TreeEvent {
            state_change: Some(batch_or_single(changes)),
            kind: TreeEventKind::StateChanged,
        })
    }

    pub(crate) fn drop_feedback_tree_event(
        &self,
        state: &TreeState<Id>,
        target: TreeDropTarget<Id>,
        modifiers: ClickModifiers,
        requested: Option<TransferOperation>,
    ) -> Option<TreeEvent<Id>> {
        let Transfer::Dragging { payload, .. } = &state.transfer else {
            return None;
        };
        let operation = self.drag_operation(modifiers, requested)?;
        let drop = TreeDrop {
            payload: payload.clone(),
            target: target.clone(),
            operation,
        };
        let target = self.drag.accepts_drop(&drop).then_some(target);

        Some(TreeEvent {
            state_change: Some(TreeStateChange::SetTransfer(Transfer::Dragging {
                payload: payload.clone(),
                operation,
                target,
            })),
            kind: TreeEventKind::StateChanged,
        })
    }

    pub(crate) fn drop_request_tree_event(
        &self,
        state: &TreeState<Id>,
        target: TreeDropTarget<Id>,
        modifiers: ClickModifiers,
        requested: Option<TransferOperation>,
    ) -> Option<TreeEvent<Id>> {
        let Transfer::Dragging { payload, .. } = &state.transfer else {
            return None;
        };
        let operation = self.drag_operation(modifiers, requested)?;
        let drop = TreeDrop {
            payload: payload.clone(),
            target,
            operation,
        };

        if !self.drag.accepts_drop(&drop) {
            return None;
        }

        Some(TreeEvent {
            state_change: Some(TreeStateChange::SetTransfer(Transfer::None)),
            kind: TreeEventKind::DropRequested(drop),
        })
    }

    pub(crate) fn drop_feedback_at_tree_event(
        &self,
        state: &TreeState<Id>,
        y: f32,
        modifiers: ClickModifiers,
        requested: Option<TransferOperation>,
    ) -> Option<TreeEvent<Id>> {
        let Some(target) = self.drop_target_at(state, y) else {
            return self.drop_feedback_without_target_tree_event(state, modifiers, requested);
        };

        self.drop_feedback_tree_event(state, target, modifiers, requested)
    }

    pub(crate) fn drop_release_at_tree_event(
        &self,
        state: &TreeState<Id>,
        y: f32,
        modifiers: ClickModifiers,
        requested: Option<TransferOperation>,
    ) -> Option<TreeEvent<Id>> {
        if let Some(target) = self.drop_target_at(state, y) {
            if let Some(event) = self.drop_request_tree_event(state, target, modifiers, requested) {
                return Some(event);
            }
        }

        if matches!(state.transfer, Transfer::Dragging { .. }) {
            return Some(TreeEvent {
                state_change: Some(TreeStateChange::SetTransfer(Transfer::None)),
                kind: TreeEventKind::StateChanged,
            });
        }

        None
    }

    fn drop_feedback_without_target_tree_event(
        &self,
        state: &TreeState<Id>,
        modifiers: ClickModifiers,
        requested: Option<TransferOperation>,
    ) -> Option<TreeEvent<Id>> {
        let Transfer::Dragging { payload, .. } = &state.transfer else {
            return None;
        };
        let operation = self.drag_operation(modifiers, requested)?;

        Some(TreeEvent {
            state_change: Some(TreeStateChange::SetTransfer(Transfer::Dragging {
                payload: payload.clone(),
                operation,
                target: None,
            })),
            kind: TreeEventKind::StateChanged,
        })
    }

    pub(crate) fn drop_target_at(
        &self,
        state: &TreeState<Id>,
        y: f32,
    ) -> Option<TreeDropTarget<Id>> {
        let entries = visible_entries(&self.nodes, state);
        if entries.is_empty() {
            return Some(TreeDropTarget::Root);
        }

        let row_height = super::super::row_height(self.size);
        if y >= entries.len() as f32 * row_height {
            return Some(TreeDropTarget::Root);
        }

        let index = (y.max(0.0) / row_height).floor() as usize;
        let position_in_row = y.max(0.0) - index as f32 * row_height;
        let before_band = row_height / 3.0;
        let after_band = row_height * 2.0 / 3.0;

        match entries.get(index)? {
            VisibleTreeEntry::Row(row) if row.disabled => None,
            VisibleTreeEntry::Row(row) if position_in_row < before_band => {
                Some(TreeDropTarget::Before(row.id.clone()))
            }
            VisibleTreeEntry::Row(row) if position_in_row >= after_band => {
                Some(TreeDropTarget::After(row.id.clone()))
            }
            VisibleTreeEntry::Row(row) if row.expanded.is_some() => {
                Some(TreeDropTarget::Into(row.id.clone()))
            }
            VisibleTreeEntry::Row(row) => Some(TreeDropTarget::After(row.id.clone())),
            VisibleTreeEntry::Loading(row) => Some(TreeDropTarget::Into(row.parent.clone())),
        }
    }

    pub(crate) fn row_id_at(&self, state: &TreeState<Id>, y: f32) -> Option<Id> {
        let entries = visible_entries(&self.nodes, state);
        let row_height = super::super::row_height(self.size);
        let index = (y.max(0.0) / row_height).floor() as usize;

        match entries.get(index)? {
            VisibleTreeEntry::Row(row) => Some(row.id.clone()),
            VisibleTreeEntry::Loading(_) => None,
        }
    }
}
