use std::borrow::Cow;

use crate::IconRef;

/// Data for one side rail entry.
///
/// `SideRailItem` carries item identity and metadata only. Activation is
/// configured on [`super::SideRail`] with `on_select` or `on_select_maybe`.
/// The label is the visible rotated label and accessible label. An optional
/// icon renders upright while only the label rotates.
///
/// An item carries no count or status marker. The rail is sized for a rotated
/// label, which leaves no room for one, so quantity and status belong to the
/// panel the item selects. `NavigationRail` is the wider variant that carries
/// them.
#[derive(Debug, Clone)]
pub struct SideRailItem<'a, Id> {
    pub(super) id: Id,
    pub(super) label: Cow<'a, str>,
    pub(super) icon: Option<IconRef>,
    pub(super) selected: bool,
    pub(super) disabled: bool,
    pub(super) tooltip: Option<Cow<'a, str>>,
}

impl<'a, Id> SideRailItem<'a, Id> {
    /// Builds an item with mandatory identity and visible/accessibility label.
    pub fn new(id: Id, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            selected: false,
            disabled: false,
            tooltip: None,
        }
    }

    /// Returns the item identity.
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Returns the visible and accessible item label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Sets the upright icon rendered at the icon end of the item.
    pub fn icon(mut self, icon: impl Into<IconRef>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets the selected visual state. Selection remains app-owned.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Disables pointer and keyboard activation.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the tooltip shown for this item.
    ///
    /// Explicit tooltips override the truncated-label fallback.
    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}
