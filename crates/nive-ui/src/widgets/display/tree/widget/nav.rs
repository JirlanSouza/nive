use crate::interaction::keyboard::TypeAhead;
use crate::interaction::{ClickModifiers, Selection, SelectionMode};

use super::super::event::{TreeEvent, TreeEventKind};
use super::super::state::{TreeState, TreeStateChange};
use super::super::visible::{visible_entries, VisibleTreeEntry};
use super::selection::{batch_or_single, selection_for_click};
use super::Tree;

pub(super) fn focusable_row_indices<Id>(entries: &[VisibleTreeEntry<'_, Id>]) -> Vec<usize> {
    entries
        .iter()
        .enumerate()
        .filter_map(|(i, entry)| match entry {
            VisibleTreeEntry::Row(_) => Some(i),
            VisibleTreeEntry::Loading(_) => None,
        })
        .collect()
}

pub(super) fn resolve_focus_in_visible<Id>(
    entries: &[VisibleTreeEntry<'_, Id>],
    focusable: &[usize],
    state: &TreeState<Id>,
) -> (usize, bool)
where
    Id: Clone + Ord,
{
    if let Some(focused) = &state.selection.focused {
        if let Some(idx) = entries.iter().position(|e| match e {
            VisibleTreeEntry::Row(row) => &row.id == focused,
            _ => false,
        }) {
            return (idx, false);
        }
    }

    for &idx in focusable {
        if let VisibleTreeEntry::Row(row) = &entries[idx] {
            if state.selection.selected.contains(&row.id) {
                return (idx, true);
            }
        }
    }

    (focusable[0], true)
}

pub(super) fn is_visible_enabled_branch<Id>(entries: &[VisibleTreeEntry<'_, Id>], id: &Id) -> bool
where
    Id: Clone + Ord,
{
    entries.iter().any(|entry| {
        matches!(
            entry,
            VisibleTreeEntry::Row(row)
                if &row.id == id && !row.disabled && row.expanded.is_some()
        )
    })
}

pub(super) fn focus_only_selection<Id>(state: &TreeState<Id>, id: Id) -> Selection<Id>
where
    Id: Clone + Ord,
{
    let mut selection = state.selection.clone();
    selection.focused = Some(id);
    selection
}

/// Finds the next visible, non-disabled row after `current_idx` (wrapping)
/// whose label matches `buffer` case-insensitively.
pub(super) fn type_ahead_target<Id>(
    entries: &[VisibleTreeEntry<'_, Id>],
    focusable: &[usize],
    current_idx: usize,
    buffer: &str,
) -> Option<usize> {
    let start_pos = focusable
        .iter()
        .position(|&i| i == current_idx)
        .map(|pos| (pos + 1) % focusable.len())
        .unwrap_or(0);

    (0..focusable.len()).find_map(|offset| {
        let pos = (start_pos + offset) % focusable.len();
        let idx = focusable[pos];
        match &entries[idx] {
            VisibleTreeEntry::Row(row)
                if !row.disabled && TypeAhead::matches(&row.label, buffer) =>
            {
                Some(idx)
            }
            _ => None,
        }
    })
}

impl<'a, Id, Message> Tree<'a, Id, Message>
where
    Id: Clone + Ord + 'a,
    Message: Clone + 'a,
{
    pub(crate) fn handle_key_down_tree(
        &self,
        state: &TreeState<Id>,
        modifiers: ClickModifiers,
    ) -> Option<TreeEvent<Id>> {
        let entries = visible_entries(&self.nodes, state);
        let focusable = focusable_row_indices(&entries);

        if focusable.is_empty() {
            return None;
        }

        let (start_idx, recovered) = resolve_focus_in_visible(&entries, &focusable, state);
        let current_pos = focusable.iter().position(|&i| i == start_idx).unwrap_or(0);
        let next_pos = (current_pos + 1).min(focusable.len() - 1);

        self.focus_move_event(state, &entries, focusable[next_pos], recovered, modifiers)
    }

    pub(crate) fn handle_key_up_tree(
        &self,
        state: &TreeState<Id>,
        modifiers: ClickModifiers,
    ) -> Option<TreeEvent<Id>> {
        let entries = visible_entries(&self.nodes, state);
        let focusable = focusable_row_indices(&entries);

        if focusable.is_empty() {
            return None;
        }

        let (start_idx, recovered) = resolve_focus_in_visible(&entries, &focusable, state);
        let current_pos = focusable.iter().position(|&i| i == start_idx).unwrap_or(0);
        let next_pos = current_pos.saturating_sub(1);

        self.focus_move_event(state, &entries, focusable[next_pos], recovered, modifiers)
    }

    pub(crate) fn handle_key_right_tree(&self, state: &TreeState<Id>) -> Option<TreeEvent<Id>> {
        let entries = visible_entries(&self.nodes, state);
        let focusable = focusable_row_indices(&entries);

        if focusable.is_empty() {
            return None;
        }

        let (idx, recovered) = resolve_focus_in_visible(&entries, &focusable, state);
        let VisibleTreeEntry::Row(row) = &entries[idx] else {
            return None;
        };
        let id = row.id.clone();

        match row.expanded {
            Some(false) => Some(self.expand_toggle_event(state, id, true, Some(false), recovered)),
            Some(true) => {
                let current_pos = focusable.iter().position(|&i| i == idx)?;
                let child_idx = focusable
                    .get(current_pos + 1)
                    .copied()
                    .filter(|&child_idx| {
                        matches!(
                            &entries[child_idx],
                            VisibleTreeEntry::Row(child) if child.parent.as_ref() == Some(&id)
                        )
                    });

                match child_idx {
                    Some(child_idx) => self.focus_move_event(
                        state,
                        &entries,
                        child_idx,
                        recovered,
                        ClickModifiers::new(true, false),
                    ),
                    None if recovered => Some(self.recovery_only_event(state, id)),
                    None => None,
                }
            }
            None if recovered => Some(self.recovery_only_event(state, id)),
            None => None,
        }
    }

    pub(crate) fn handle_key_left_tree(&self, state: &TreeState<Id>) -> Option<TreeEvent<Id>> {
        let entries = visible_entries(&self.nodes, state);
        let focusable = focusable_row_indices(&entries);

        if focusable.is_empty() {
            return None;
        }

        let (idx, recovered) = resolve_focus_in_visible(&entries, &focusable, state);
        let VisibleTreeEntry::Row(row) = &entries[idx] else {
            return None;
        };
        let id = row.id.clone();

        if row.expanded == Some(true) {
            return Some(self.expand_toggle_event(state, id, false, Some(true), recovered));
        }

        match &row.parent {
            Some(parent_id) => {
                let parent_idx = entries.iter().position(
                    |entry| matches!(entry, VisibleTreeEntry::Row(r) if &r.id == parent_id),
                )?;
                self.focus_move_event(
                    state,
                    &entries,
                    parent_idx,
                    recovered,
                    ClickModifiers::new(true, false),
                )
            }
            None if recovered => Some(self.recovery_only_event(state, id)),
            None => None,
        }
    }

    pub(crate) fn handle_key_home_tree(
        &self,
        state: &TreeState<Id>,
        modifiers: ClickModifiers,
    ) -> Option<TreeEvent<Id>> {
        let entries = visible_entries(&self.nodes, state);
        let focusable = focusable_row_indices(&entries);

        if focusable.is_empty() {
            return None;
        }

        let (_, recovered) = resolve_focus_in_visible(&entries, &focusable, state);

        self.focus_move_event(state, &entries, focusable[0], recovered, modifiers)
    }

    pub(crate) fn handle_key_end_tree(
        &self,
        state: &TreeState<Id>,
        modifiers: ClickModifiers,
    ) -> Option<TreeEvent<Id>> {
        let entries = visible_entries(&self.nodes, state);
        let focusable = focusable_row_indices(&entries);

        if focusable.is_empty() {
            return None;
        }

        let (_, recovered) = resolve_focus_in_visible(&entries, &focusable, state);
        let last = *focusable.last().expect("focusable is non-empty");

        self.focus_move_event(state, &entries, last, recovered, modifiers)
    }

    pub(crate) fn handle_key_page_down_tree(
        &self,
        state: &TreeState<Id>,
        modifiers: ClickModifiers,
    ) -> Option<TreeEvent<Id>> {
        let entries = visible_entries(&self.nodes, state);
        let focusable = focusable_row_indices(&entries);

        if focusable.is_empty() {
            return None;
        }

        let (start_idx, recovered) = resolve_focus_in_visible(&entries, &focusable, state);
        let current_pos = focusable.iter().position(|&i| i == start_idx).unwrap_or(0);
        let next_pos = (current_pos + self.page_rows).min(focusable.len() - 1);

        self.focus_move_event(state, &entries, focusable[next_pos], recovered, modifiers)
    }

    pub(crate) fn handle_key_page_up_tree(
        &self,
        state: &TreeState<Id>,
        modifiers: ClickModifiers,
    ) -> Option<TreeEvent<Id>> {
        let entries = visible_entries(&self.nodes, state);
        let focusable = focusable_row_indices(&entries);

        if focusable.is_empty() {
            return None;
        }

        let (start_idx, recovered) = resolve_focus_in_visible(&entries, &focusable, state);
        let current_pos = focusable.iter().position(|&i| i == start_idx).unwrap_or(0);
        let next_pos = current_pos.saturating_sub(self.page_rows);

        self.focus_move_event(state, &entries, focusable[next_pos], recovered, modifiers)
    }

    pub(crate) fn handle_type_ahead_tree(
        &self,
        state: &TreeState<Id>,
        buffer: &str,
    ) -> Option<TreeEvent<Id>> {
        if !self.type_ahead || buffer.is_empty() {
            return None;
        }

        let entries = visible_entries(&self.nodes, state);
        let focusable = focusable_row_indices(&entries);

        if focusable.is_empty() {
            return None;
        }

        let (start_idx, recovered) = resolve_focus_in_visible(&entries, &focusable, state);
        let target_idx = type_ahead_target(&entries, &focusable, start_idx, buffer)?;

        self.focus_move_event(state, &entries, target_idx, recovered, ClickModifiers::NONE)
    }

    fn focus_move_event(
        &self,
        state: &TreeState<Id>,
        entries: &[VisibleTreeEntry<'_, Id>],
        target_entry_idx: usize,
        recovered: bool,
        modifiers: ClickModifiers,
    ) -> Option<TreeEvent<Id>> {
        let VisibleTreeEntry::Row(target_row) = &entries[target_entry_idx] else {
            return None;
        };

        let target_id = target_row.id.clone();
        let target_disabled = target_row.disabled;

        if !recovered && state.selection.focused.as_ref() == Some(&target_id) {
            return None;
        }

        let selection = if modifiers.primary || target_disabled {
            focus_only_selection(state, target_id)
        } else if modifiers.shift && self.selection_mode == SelectionMode::Multiple {
            selection_for_click(
                state,
                target_id,
                self.selection_mode,
                ClickModifiers::new(false, true),
                &self.nodes,
            )?
        } else {
            selection_for_click(
                state,
                target_id,
                self.selection_mode,
                ClickModifiers::NONE,
                &self.nodes,
            )?
        };

        Some(TreeEvent {
            state_change: Some(TreeStateChange::SetSelection(selection)),
            kind: TreeEventKind::StateChanged,
        })
    }

    /// Emits a state change that only records recovered focus, used when a
    /// navigation key has no other effect (e.g. Right on a leaf) but the
    /// previously focused ID needed to be resolved to a visible row.
    fn recovery_only_event(&self, state: &TreeState<Id>, id: Id) -> TreeEvent<Id> {
        TreeEvent {
            state_change: Some(TreeStateChange::SetSelection(focus_only_selection(
                state, id,
            ))),
            kind: TreeEventKind::StateChanged,
        }
    }

    /// Builds the expansion state change for Left/Right, folding in a
    /// recovered focus set when the previously focused ID was stale.
    fn expand_toggle_event(
        &self,
        state: &TreeState<Id>,
        id: Id,
        target_expanded: bool,
        prior_expanded_hint: Option<bool>,
        recovered: bool,
    ) -> TreeEvent<Id> {
        let mut changes = vec![TreeStateChange::SetExpanded {
            id: id.clone(),
            expanded: target_expanded,
        }];

        if recovered {
            changes.push(TreeStateChange::SetSelection(focus_only_selection(
                state,
                id.clone(),
            )));
        }

        TreeEvent {
            state_change: Some(batch_or_single(changes)),
            kind: self.expand_event_kind(id, prior_expanded_hint),
        }
    }
}
