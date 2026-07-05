use std::collections::BTreeSet;

use iced::Point;

use crate::interaction::{
    ActivationTrigger, ClickModifiers, CollectionTransferPayload, ContextInvocation,
    ContextPosition, ContextRequest, ContextSelectionBehavior, ContextTarget, Selection,
    SelectionMode, SelectionSnapshot, Transfer, TransferOperation,
};

use super::super::event::{TreeEvent, TreeEventKind};
use super::super::state::{TreeState, TreeStateChange};
use super::super::transfer::TreePasteTarget;
use super::super::visible::{visible_entries, VisibleTreeEntry};
use super::nav::is_visible_enabled_branch;
use super::selection::{batch_or_single, is_deferred_branch, selection_for_click};
use super::{Tree, TreeExpandBehavior};

impl<'a, Id, Message> Tree<'a, Id, Message>
where
    Id: Clone + Ord + 'a,
    Message: Clone + 'a,
{
    pub(crate) fn empty_space_event(&self, state: &TreeState<Id>) -> Option<Message> {
        if self.selection_mode == SelectionMode::None {
            return None;
        }
        if state.selection.selected.is_empty() && state.selection.anchor.is_none() {
            return None;
        }
        let event = TreeEvent {
            state_change: Some(TreeStateChange::ClearSelection),
            kind: TreeEventKind::StateChanged,
        };
        self.on_event.as_ref().map(|on_event| on_event(event))
    }

    pub(crate) fn row_event(
        &self,
        state: &TreeState<Id>,
        id: &Id,
        expanded: Option<bool>,
        disabled: bool,
    ) -> Option<Message> {
        if disabled {
            return None;
        }

        let event = self.row_tree_event(state, id, expanded, ClickModifiers::NONE)?;
        self.on_event.as_ref().map(|on_event| on_event(event))
    }

    pub(crate) fn toggle_event(
        &self,
        id: Id,
        expanded: Option<bool>,
        disabled: bool,
    ) -> Option<Message> {
        if disabled {
            return None;
        }

        let event = self.toggle_tree_event(id, expanded)?;
        self.on_event.as_ref().map(|on_event| on_event(event))
    }

    pub(crate) fn row_tree_event(
        &self,
        state: &TreeState<Id>,
        id: &Id,
        expanded: Option<bool>,
        modifiers: ClickModifiers,
    ) -> Option<TreeEvent<Id>> {
        let mut changes = Vec::new();
        if let Some(selection) = selection_for_click(
            state,
            id.clone(),
            self.selection_mode,
            modifiers,
            &self.nodes,
        ) {
            changes.push(TreeStateChange::SetSelection(selection));
        }

        match self.expand_behavior {
            TreeExpandBehavior::ExpanderOnly | TreeExpandBehavior::DoubleClick => {}
            TreeExpandBehavior::SingleClick => {
                if expanded.is_some() {
                    changes.push(TreeStateChange::SetExpanded {
                        id: id.clone(),
                        expanded: !expanded.unwrap_or(false),
                    });
                }
            }
        }

        if changes.is_empty() {
            return None;
        }

        Some(TreeEvent {
            state_change: Some(batch_or_single(changes)),
            kind: self.expand_event_kind(id.clone(), expanded),
        })
    }

    pub(crate) fn toggle_tree_event(
        &self,
        id: Id,
        expanded: Option<bool>,
    ) -> Option<TreeEvent<Id>> {
        let expanded = expanded?;
        Some(TreeEvent {
            state_change: Some(TreeStateChange::SetExpanded {
                id: id.clone(),
                expanded: !expanded,
            }),
            kind: self.expand_event_kind(id, Some(expanded)),
        })
    }

    // Double-click row dispatch is not yet wired to a real pointer
    // double-click event from `build_content`; that wiring is pointer-event
    // plumbing outside this task's scope, tracked separately.
    #[allow(dead_code)]
    pub(crate) fn double_click_row_event(
        &self,
        state: &TreeState<Id>,
        id: &Id,
        expanded: Option<bool>,
        disabled: bool,
    ) -> Option<Message> {
        if disabled {
            return None;
        }

        let event = self.double_click_row_tree_event(state, id, expanded)?;
        self.on_event.as_ref().map(|on_event| on_event(event))
    }

    #[allow(dead_code)]
    pub(crate) fn double_click_row_tree_event(
        &self,
        state: &TreeState<Id>,
        id: &Id,
        expanded: Option<bool>,
    ) -> Option<TreeEvent<Id>> {
        let mut changes = Vec::new();

        if let Some(selection) = selection_for_click(
            state,
            id.clone(),
            self.selection_mode,
            ClickModifiers::NONE,
            &self.nodes,
        ) {
            changes.push(TreeStateChange::SetSelection(selection));
        }

        let should_toggle_expansion =
            self.expand_behavior == TreeExpandBehavior::DoubleClick && expanded.is_some();

        if should_toggle_expansion {
            changes.push(TreeStateChange::SetExpanded {
                id: id.clone(),
                expanded: !expanded.unwrap_or(false),
            });
        }

        if changes.is_empty() {
            return None;
        }

        let activation_includes_double_click = self
            .activation_behavior
            .includes(ActivationTrigger::DoubleClick);

        let kind = if activation_includes_double_click {
            TreeEventKind::Activate {
                id: id.clone(),
                trigger: ActivationTrigger::DoubleClick,
            }
        } else {
            self.expand_event_kind(id.clone(), expanded)
        };

        Some(TreeEvent {
            state_change: Some(batch_or_single(changes)),
            kind,
        })
    }

    pub(crate) fn expand_event_kind(&self, id: Id, expanded: Option<bool>) -> TreeEventKind<Id> {
        if expanded == Some(false) && is_deferred_branch(&self.nodes, &id) {
            TreeEventKind::ExpandRequested { id }
        } else {
            TreeEventKind::StateChanged
        }
    }

    pub(crate) fn selection_snapshot_from(
        &self,
        state: &TreeState<Id>,
        selection: &Selection<Id>,
    ) -> SelectionSnapshot<Id> {
        let entries = visible_entries(&self.nodes, state);
        let visible_ids = entries.iter().filter_map(|entry| match entry {
            VisibleTreeEntry::Row(row) if !row.disabled => Some(row.id.clone()),
            _ => None,
        });
        SelectionSnapshot::from_visible_order(visible_ids, selection)
    }

    pub(crate) fn context_request_tree_event_for_node(
        &self,
        state: &TreeState<Id>,
        id: &Id,
        position: Point,
        disabled: bool,
    ) -> Option<TreeEvent<Id>> {
        if disabled {
            return None;
        }

        let is_selected = state.is_selected(id);

        let state_change = if !is_selected {
            match self.context_selection_behavior {
                ContextSelectionBehavior::SelectTargetIfUnselected => {
                    let mut selection = state.selection.clone();
                    selection.selected.clear();
                    selection.selected.insert(id.clone());
                    selection.focused = Some(id.clone());
                    selection.anchor = Some(id.clone());
                    Some(TreeStateChange::SetSelection(selection))
                }
                ContextSelectionBehavior::PreserveSelection => None,
                ContextSelectionBehavior::FocusOnly => {
                    let mut selection = state.selection.clone();
                    selection.focused = Some(id.clone());
                    Some(TreeStateChange::SetSelection(selection))
                }
            }
        } else {
            None
        };

        let effective_selection = match &state_change {
            Some(TreeStateChange::SetSelection(sel)) => sel,
            _ => &state.selection,
        };
        let snapshot = self.selection_snapshot_from(state, effective_selection);

        Some(TreeEvent {
            state_change,
            kind: TreeEventKind::ContextRequested(ContextRequest {
                target: ContextTarget::Item(id.clone()),
                selection: snapshot,
                position: ContextPosition::Pointer(position),
                invocation: ContextInvocation::SecondaryClick,
            }),
        })
    }

    pub(crate) fn context_request_tree_event_for_empty_space(
        &self,
        state: &TreeState<Id>,
        position: Point,
    ) -> TreeEvent<Id> {
        let snapshot = self.selection_snapshot_from(state, &state.selection);

        TreeEvent {
            state_change: None,
            kind: TreeEventKind::ContextRequested(ContextRequest {
                target: ContextTarget::Empty,
                selection: snapshot,
                position: ContextPosition::Pointer(position),
                invocation: ContextInvocation::SecondaryClick,
            }),
        }
    }

    pub(crate) fn context_request_tree_event_for_keyboard(
        &self,
        state: &TreeState<Id>,
    ) -> Option<TreeEvent<Id>> {
        let focused = state.selection.focused.clone()?;
        let is_selected = state.is_selected(&focused);

        let state_change = if !is_selected {
            match self.context_selection_behavior {
                ContextSelectionBehavior::SelectTargetIfUnselected => {
                    let mut selection = state.selection.clone();
                    selection.selected.clear();
                    selection.selected.insert(focused.clone());
                    selection.anchor = Some(focused.clone());
                    Some(TreeStateChange::SetSelection(selection))
                }
                ContextSelectionBehavior::PreserveSelection => None,
                ContextSelectionBehavior::FocusOnly => None,
            }
        } else {
            None
        };

        let effective_selection = match &state_change {
            Some(TreeStateChange::SetSelection(sel)) => sel,
            _ => &state.selection,
        };
        let snapshot = self.selection_snapshot_from(state, effective_selection);

        Some(TreeEvent {
            state_change,
            kind: TreeEventKind::ContextRequested(ContextRequest {
                target: ContextTarget::Item(focused),
                selection: snapshot,
                position: ContextPosition::FocusedItem,
                invocation: ContextInvocation::Keyboard,
            }),
        })
    }

    pub(crate) fn context_request_at_tree_event(
        &self,
        state: &TreeState<Id>,
        y: f32,
        position: Point,
    ) -> Option<Message> {
        let event = if let Some(id) = self.row_id_at(state, y) {
            self.context_request_tree_event_for_node(
                state,
                &id,
                position,
                self.is_disabled_visible_node(state, &id),
            )?
        } else {
            self.context_request_tree_event_for_empty_space(state, position)
        };

        self.on_event.as_ref().map(|on_event| on_event(event))
    }

    pub(crate) fn context_request_event_for_keyboard(
        &self,
        state: &TreeState<Id>,
    ) -> Option<Message> {
        let event = self.context_request_tree_event_for_keyboard(state)?;
        self.on_event.as_ref().map(|on_event| on_event(event))
    }

    pub(crate) fn transfer_payload(
        &self,
        state: &TreeState<Id>,
        initiating: Option<&Id>,
    ) -> Option<CollectionTransferPayload<Id>> {
        let entries = visible_entries(&self.nodes, state);
        let mut visible_ids = Vec::new();
        let mut parent_by_id = Vec::new();
        let mut selected_visible = BTreeSet::new();

        for entry in &entries {
            let VisibleTreeEntry::Row(row) = entry else {
                continue;
            };

            parent_by_id.push((row.id.clone(), row.parent.clone()));

            if row.disabled {
                continue;
            }

            visible_ids.push(row.id.clone());
            if state.selection.selected.contains(&row.id) {
                selected_visible.insert(row.id.clone());
            }
        }

        if selected_visible.is_empty() {
            return None;
        }

        if initiating.is_some_and(|id| !selected_visible.contains(id)) {
            return None;
        }

        let payload = CollectionTransferPayload::from_visible_order_with_parents(
            visible_ids,
            &selected_visible,
            parent_by_id,
        );

        (!payload.ids.is_empty()).then_some(payload)
    }

    pub(crate) fn paste_target(&self, state: &TreeState<Id>) -> TreePasteTarget<Id> {
        let entries = visible_entries(&self.nodes, state);

        if let Some(focused) = &state.selection.focused {
            if is_visible_enabled_branch(&entries, focused) {
                return TreePasteTarget::Into(focused.clone());
            }
        }

        for entry in &entries {
            let VisibleTreeEntry::Row(row) = entry else {
                continue;
            };

            if !row.disabled && row.expanded.is_some() && state.selection.selected.contains(&row.id)
            {
                return TreePasteTarget::Into(row.id.clone());
            }
        }

        TreePasteTarget::Root
    }

    pub(crate) fn is_disabled_visible_node(&self, state: &TreeState<Id>, id: &Id) -> bool {
        visible_entries(&self.nodes, state)
            .into_iter()
            .any(|entry| {
                matches!(
                    entry,
                    VisibleTreeEntry::Row(row) if row.id == *id && row.disabled
                )
            })
    }

    pub(crate) fn handle_escape_tree(&self, state: &TreeState<Id>) -> Option<TreeEvent<Id>> {
        match &state.transfer {
            Transfer::Dragging { .. } => Some(TreeEvent {
                state_change: Some(TreeStateChange::SetTransfer(Transfer::None)),
                kind: TreeEventKind::StateChanged,
            }),
            Transfer::Clipboard { operation, .. } if *operation == TransferOperation::Move => {
                Some(TreeEvent {
                    state_change: Some(TreeStateChange::SetTransfer(Transfer::None)),
                    kind: TreeEventKind::StateChanged,
                })
            }
            _ => None,
        }
    }

    pub(crate) fn handle_copy_tree(&self, state: &TreeState<Id>) -> Option<TreeEvent<Id>> {
        let payload = self.transfer_payload(state, state.selection.focused.as_ref())?;

        Some(TreeEvent {
            state_change: None,
            kind: TreeEventKind::CopyRequested(payload),
        })
    }

    pub(crate) fn handle_cut_tree(&self, state: &TreeState<Id>) -> Option<TreeEvent<Id>> {
        let payload = self.transfer_payload(state, state.selection.focused.as_ref())?;

        Some(TreeEvent {
            state_change: Some(TreeStateChange::SetTransfer(Transfer::Clipboard {
                payload: payload.clone(),
                operation: TransferOperation::Move,
            })),
            kind: TreeEventKind::CutRequested(payload),
        })
    }

    pub(crate) fn handle_paste_tree(&self, state: &TreeState<Id>) -> Option<TreeEvent<Id>> {
        Some(TreeEvent {
            state_change: None,
            kind: TreeEventKind::PasteRequested(self.paste_target(state)),
        })
    }
}
