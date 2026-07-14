use std::collections::BTreeSet;

use crate::interaction::{Selection, Transfer};

use super::{
    event::TreeEvent,
    node::{TreeChildren, TreeNode},
    transfer::TreeDropTarget,
};

/// App-owned interaction state for a rendered tree.
///
/// `Tree` reads this state but does not persist hidden expansion, selection,
/// focus, anchor, or transfer state internally. Apps should call
/// [`TreeState::apply`] once for every emitted tree event before handling
/// domain-specific event kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeState<Id> {
    /// IDs of expanded branch nodes.
    pub expanded: BTreeSet<Id>,
    /// Selection, keyboard focus, and range-selection anchor.
    pub selection: Selection<Id>,
    /// Clipboard or drag feedback owned by tree interactions.
    pub transfer: Transfer<Id, TreeDropTarget<Id>>,
}

/// State delta emitted by tree interactions.
///
/// This enum is non-exhaustive; app matches should include a wildcard arm.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TreeStateChange<Id> {
    /// Set a branch expansion state.
    SetExpanded {
        /// Branch ID to update.
        id: Id,
        /// Whether the branch should be expanded.
        expanded: bool,
    },
    /// Replace the full selection/focus/anchor state.
    SetSelection(Selection<Id>),
    /// Replace transfer feedback state.
    SetTransfer(Transfer<Id, TreeDropTarget<Id>>),
    /// Clear selected IDs and anchor while preserving focus.
    ClearSelection,
    /// Apply multiple state changes in order.
    Batch(Vec<TreeStateChange<Id>>),
}

impl<Id> Default for TreeState<Id> {
    fn default() -> Self {
        Self {
            expanded: BTreeSet::new(),
            selection: Selection::default(),
            transfer: Transfer::None,
        }
    }
}

impl<Id> TreeState<Id>
where
    Id: Clone + Ord,
{
    /// Applies the state delta carried by a tree event, if any.
    pub fn apply(&mut self, event: &TreeEvent<Id>) {
        if let Some(change) = &event.state_change {
            self.apply_change(change);
        }
    }

    /// Applies a tree state change.
    pub fn apply_change(&mut self, change: &TreeStateChange<Id>) {
        match change {
            TreeStateChange::SetExpanded { id, expanded } => {
                self.set_expanded(id.clone(), *expanded);
            }
            TreeStateChange::SetSelection(selection) => {
                self.selection = selection.clone();
            }
            TreeStateChange::SetTransfer(transfer) => {
                self.transfer = transfer.clone();
            }
            TreeStateChange::ClearSelection => {
                self.clear_selection();
            }
            TreeStateChange::Batch(changes) => {
                for change in changes {
                    self.apply_change(change);
                }
            }
        }
    }

    /// Sets whether a branch is expanded.
    pub fn set_expanded(&mut self, id: Id, expanded: bool) {
        if expanded {
            self.expanded.insert(id);
        } else {
            self.expanded.remove(&id);
        }
    }

    /// Expands a branch.
    pub fn expand(&mut self, id: Id) {
        self.set_expanded(id, true);
    }

    /// Collapses a branch.
    pub fn collapse(&mut self, id: &Id) {
        self.expanded.remove(id);
    }

    /// Toggles a branch and returns the new expansion state.
    pub fn toggle_expanded(&mut self, id: Id) -> bool {
        let expanded = !self.is_expanded(&id);
        self.set_expanded(id, expanded);
        expanded
    }

    /// Replaces selection with one selected and focused ID.
    pub fn select_only(&mut self, id: Id) {
        self.selection.selected.clear();
        self.selection.selected.insert(id.clone());
        self.selection.focused = Some(id.clone());
        self.selection.anchor = Some(id);
    }

    /// Replaces selected IDs while leaving focus and anchor unchanged.
    pub fn select_many(&mut self, ids: impl IntoIterator<Item = Id>) {
        self.selection.selected = ids.into_iter().collect();
    }

    /// Clears selected IDs and anchor while preserving focus.
    pub fn clear_selection(&mut self) {
        self.selection.selected.clear();
        self.selection.anchor = None;
    }

    /// Sets the focused node.
    pub fn set_focused(&mut self, id: Option<Id>) {
        self.selection.focused = id;
    }

    /// Returns the focused node ID.
    pub fn focused(&self) -> Option<&Id> {
        self.selection.focused.as_ref()
    }

    /// Replaces transfer feedback state.
    pub fn set_transfer(&mut self, transfer: Transfer<Id, TreeDropTarget<Id>>) {
        self.transfer = transfer;
    }

    /// Returns whether a branch is expanded.
    pub fn is_expanded(&self, id: &Id) -> bool {
        self.expanded.contains(id)
    }

    /// Returns whether a node is selected.
    pub fn is_selected(&self, id: &Id) -> bool {
        self.selection.selected.contains(id)
    }

    /// Returns whether `id` holds model focus (see [`Self::focused`]).
    pub fn is_focused(&self, id: &Id) -> bool {
        self.selection.focused.as_ref() == Some(id)
    }

    /// Expands all loaded ancestors of `id` and focuses it when found.
    pub fn reveal(&mut self, roots: &[TreeNode<'_, Id>], id: &Id) -> bool {
        let mut ancestors = Vec::new();
        if !find_ancestor_path(roots, id, &mut ancestors) {
            return false;
        }

        self.expanded.extend(ancestors);
        self.selection.focused = Some(id.clone());
        true
    }

    /// Prunes IDs from expansion, selection, focus, anchor, and transfer state.
    pub fn retain_ids(&mut self, keep: impl Fn(&Id) -> bool) {
        self.expanded.retain(|id| keep(id));
        self.selection.selected.retain(|id| keep(id));

        if self.selection.focused.as_ref().is_some_and(|id| !keep(id)) {
            self.selection.focused = None;
        }

        if self.selection.anchor.as_ref().is_some_and(|id| !keep(id)) {
            self.selection.anchor = None;
        }

        retain_transfer_ids(&mut self.transfer, keep);
    }
}

fn find_ancestor_path<Id>(nodes: &[TreeNode<'_, Id>], target: &Id, path: &mut Vec<Id>) -> bool
where
    Id: Clone + Ord,
{
    for node in nodes {
        if node.id() == target {
            return true;
        }

        let Some(TreeChildren::Loaded(children)) = node.children() else {
            continue;
        };

        path.push(node.id().clone());
        if find_ancestor_path(children, target, path) {
            return true;
        }
        path.pop();
    }

    false
}

fn retain_transfer_ids<Id>(
    transfer: &mut Transfer<Id, TreeDropTarget<Id>>,
    keep: impl Fn(&Id) -> bool,
) where
    Id: Clone + Ord,
{
    match transfer {
        Transfer::None => {}
        Transfer::Clipboard { payload, .. } => {
            payload.ids.retain(|id| keep(id));
            payload.root_ids.retain(|id| keep(id));
            if payload.ids.is_empty() && payload.root_ids.is_empty() {
                *transfer = Transfer::None;
            }
        }
        Transfer::Dragging {
            payload, target, ..
        } => {
            payload.ids.retain(|id| keep(id));
            payload.root_ids.retain(|id| keep(id));
            if target
                .as_ref()
                .is_some_and(|target| !target_ids_kept(target, &keep))
            {
                *target = None;
            }
            if payload.ids.is_empty() && payload.root_ids.is_empty() {
                *transfer = Transfer::None;
            }
        }
    }
}

fn target_ids_kept<Id>(target: &TreeDropTarget<Id>, keep: impl Fn(&Id) -> bool) -> bool {
    match target {
        TreeDropTarget::Before(id) | TreeDropTarget::After(id) | TreeDropTarget::Into(id) => {
            keep(id)
        }
        TreeDropTarget::Root => true,
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
