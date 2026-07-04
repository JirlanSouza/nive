use std::collections::BTreeSet;

/// Modifier keys held during a click interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClickModifiers {
    /// Primary modifier (Cmd on macOS, Ctrl elsewhere).
    pub primary: bool,
    /// Shift modifier.
    pub shift: bool,
}

impl ClickModifiers {
    /// No modifiers held.
    pub const NONE: Self = Self {
        primary: false,
        shift: false,
    };

    /// Creates modifiers from individual flags.
    pub fn new(primary: bool, shift: bool) -> Self {
        Self { primary, shift }
    }
}

/// Selection policy used by collection widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    /// Rows can be focused, but no selected IDs are stored.
    None,
    /// At most one ID is selected.
    #[default]
    Single,
    /// Multiple IDs can be selected with range and additive interactions.
    Multiple,
}

/// App-owned selection state shared by collection widgets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection<Id> {
    /// Selected IDs. Widgets that emit snapshots convert this set to visible
    /// order.
    pub selected: BTreeSet<Id>,
    /// Keyboard-focused ID, if any.
    pub focused: Option<Id>,
    /// Range-selection anchor ID, if any.
    pub anchor: Option<Id>,
}

impl<Id> Default for Selection<Id> {
    fn default() -> Self {
        Self {
            selected: BTreeSet::new(),
            focused: None,
            anchor: None,
        }
    }
}

/// Visible-order selection snapshot carried by context requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSnapshot<Id> {
    /// Selected IDs in widget-visible order.
    pub selected: Vec<Id>,
    /// Focused ID at the time of the request.
    pub focused: Option<Id>,
    /// Range-selection anchor at the time of the request.
    pub anchor: Option<Id>,
}

impl<Id> Default for SelectionSnapshot<Id> {
    fn default() -> Self {
        Self {
            selected: Vec::new(),
            focused: None,
            anchor: None,
        }
    }
}

impl<Id> SelectionSnapshot<Id>
where
    Id: Clone + Ord,
{
    /// Builds a snapshot from visible IDs and the current selection.
    pub fn from_visible_order(
        visible_ids: impl IntoIterator<Item = Id>,
        selection: &Selection<Id>,
    ) -> Self {
        Self {
            selected: visible_ids
                .into_iter()
                .filter(|id| selection.selected.contains(id))
                .collect(),
            focused: selection.focused.clone(),
            anchor: selection.anchor.clone(),
        }
    }
}

#[cfg(test)]
mod selection_tests {
    use super::*;

    #[test]
    fn snapshot_preserves_visible_order() {
        let selection = Selection {
            selected: [3, 1].into_iter().collect(),
            focused: Some(3),
            anchor: Some(1),
        };

        let snapshot = SelectionSnapshot::from_visible_order([1, 2, 3], &selection);

        assert_eq!(snapshot.selected, vec![1, 3]);
        assert_eq!(snapshot.focused, Some(3));
        assert_eq!(snapshot.anchor, Some(1));
    }
}
