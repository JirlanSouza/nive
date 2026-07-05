use std::collections::BTreeSet;

use crate::interaction::{ClickModifiers, Selection, SelectionMode};

use super::super::{TreeChildren, TreeNode};
use super::{visible_entries, TreeState, TreeStateChange, VisibleTreeEntry};

pub(super) fn selection_for_click<Id>(
    state: &TreeState<Id>,
    id: Id,
    mode: SelectionMode,
    modifiers: ClickModifiers,
    nodes: &[TreeNode<'_, Id>],
) -> Option<Selection<Id>>
where
    Id: Clone + Ord,
{
    let mut selection = state.selection.clone();
    selection.focused = Some(id.clone());

    match mode {
        SelectionMode::None => {
            selection.selected.clear();
            selection.anchor = None;
        }
        SelectionMode::Single => {
            selection.selected.clear();
            selection.selected.insert(id.clone());
            selection.anchor = Some(id);
        }
        SelectionMode::Multiple => {
            if modifiers.shift {
                let anchor = selection.anchor.clone().unwrap_or_else(|| id.clone());
                let range = visible_range_ids(nodes, state, &anchor, &id);
                selection.selected = range;
            } else if modifiers.primary {
                if selection.selected.contains(&id) {
                    selection.selected.remove(&id);
                } else {
                    selection.selected.insert(id.clone());
                }
            } else {
                selection.selected.clear();
                selection.selected.insert(id.clone());
                selection.anchor = Some(id);
            }
        }
    }

    Some(selection)
}

pub(super) fn visible_range_ids<Id>(
    nodes: &[TreeNode<'_, Id>],
    state: &TreeState<Id>,
    from: &Id,
    to: &Id,
) -> BTreeSet<Id>
where
    Id: Clone + Ord,
{
    let entries = visible_entries(nodes, state);
    let selectable: Vec<&Id> = entries
        .iter()
        .filter_map(|entry| match entry {
            VisibleTreeEntry::Row(row) if !row.disabled => Some(&row.id),
            _ => None,
        })
        .collect();

    let from_pos = selectable.iter().position(|id| *id == from);
    let to_pos = selectable.iter().position(|id| *id == to);

    let (Some(start_idx), Some(end_idx)) = (from_pos, to_pos) else {
        return BTreeSet::new();
    };

    let (lo, hi) = if start_idx <= end_idx {
        (start_idx, end_idx)
    } else {
        (end_idx, start_idx)
    };

    selectable[lo..=hi].iter().map(|id| (*id).clone()).collect()
}

pub(super) fn batch_or_single<Id>(changes: Vec<TreeStateChange<Id>>) -> TreeStateChange<Id> {
    let mut changes = changes.into_iter();
    let first = changes.next().expect("state change");
    match changes.next() {
        Some(second) => {
            let mut batch = vec![first, second];
            batch.extend(changes);
            TreeStateChange::Batch(batch)
        }
        None => first,
    }
}

pub(super) fn is_deferred_branch<Id>(nodes: &[TreeNode<'_, Id>], id: &Id) -> bool
where
    Id: Clone + Ord,
{
    for node in nodes {
        if node.id() == id {
            return matches!(node.children(), Some(TreeChildren::Deferred));
        }

        if let Some(TreeChildren::Loaded(children)) = node.children() {
            if is_deferred_branch(children, id) {
                return true;
            }
        }
    }

    false
}
