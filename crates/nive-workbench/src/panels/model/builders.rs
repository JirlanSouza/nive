use std::borrow::Cow;

use nive_ui::theme::ToneRole;
use nive_ui::{
    widgets::{BadgeContent, StatusIndicator},
    Element, IconRole,
};

use super::{
    PanelAction, PanelHostMode, PanelRailItem, PanelSelectorPlacement, WorkbenchPanel,
    WorkbenchPanelEvent, WorkbenchPanelHostState,
};
use crate::layout::WorkbenchRegion;

impl<'a, ActionId> PanelAction<'a, ActionId> {
    /// Creates an icon-only action with a required accessible label.
    pub fn icon(id: ActionId, icon: IconRole, accessible_label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id,
            label: None,
            icon: Some(icon),
            accessible_label: accessible_label.into(),
            disabled: false,
        }
    }

    /// Creates a text action. The label is also used as the accessible label.
    pub fn text(id: ActionId, label: impl Into<Cow<'a, str>>) -> Self {
        let label = label.into();
        Self {
            id,
            label: Some(label.clone()),
            icon: None,
            accessible_label: label,
            disabled: false,
        }
    }

    /// Creates an icon+text action.
    pub fn icon_text(id: ActionId, icon: IconRole, label: impl Into<Cow<'a, str>>) -> Self {
        let label = label.into();
        Self {
            id,
            label: Some(label.clone()),
            icon: Some(icon),
            accessible_label: label,
            disabled: false,
        }
    }

    /// Sets disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns the action id.
    pub fn id(&self) -> &ActionId {
        &self.id
    }

    /// Returns the accessible label.
    pub fn accessible_label(&self) -> &str {
        self.accessible_label.as_ref()
    }
}

impl<'a, PanelId, ActionId, Message> WorkbenchPanel<'a, PanelId, ActionId, Message> {
    /// Creates a panel with required id, title, and content.
    pub fn new(
        id: PanelId,
        title: impl Into<Cow<'a, str>>,
        content: impl Into<Element<'a, Message>>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            icon: None,
            badge: None,
            status: None,
            content: content.into(),
            actions: Vec::new(),
            visible: true,
            disabled: false,
            collapsible: true,
            restorable: true,
            maximizable: true,
            closable: false,
            tooltip: None,
        }
    }

    /// Returns panel id.
    pub fn id(&self) -> &PanelId {
        &self.id
    }

    /// Returns panel title.
    pub fn title(&self) -> &str {
        self.title.as_ref()
    }

    /// Returns panel actions.
    pub fn panel_actions(&self) -> &[PanelAction<'a, ActionId>] {
        &self.actions
    }

    /// Returns typed badge metadata without reparsing its visible label.
    pub fn badge_content_value(&self) -> Option<&BadgeContent<'a>> {
        self.badge.as_ref()
    }

    /// Returns complete labelled status metadata.
    pub fn status_indicator_value(&self) -> Option<&StatusIndicator<'a>> {
        self.status.as_ref()
    }

    /// Returns visibility.
    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets panel icon.
    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets a textual Status badge, preserving the source-compatible string forwarder.
    ///
    /// Use [`Self::count_badge`] or [`Self::badge_content`] when migrating a
    /// former raw string/Cow field that actually represents a numeric count.
    pub fn badge(mut self, badge: impl Into<Cow<'a, str>>) -> Self {
        self.badge = Some(BadgeContent::Status(badge.into()));
        self
    }

    /// Sets typed panel badge content.
    pub fn badge_content(mut self, badge: BadgeContent<'a>) -> Self {
        self.badge = Some(badge);
        self
    }

    /// Sets a numeric count badge.
    pub fn count_badge(mut self, count: u64) -> Self {
        self.badge = Some(BadgeContent::Count(count));
        self
    }

    /// Sets complete labelled panel status.
    pub fn status_indicator(mut self, status: StatusIndicator<'a>) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets complete labelled panel status from tone and visible text.
    pub fn status_text(self, tone: ToneRole, label: impl Into<Cow<'a, str>>) -> Self {
        self.status_indicator(StatusIndicator::new(tone, label))
    }

    /// Adds one app action.
    pub fn action(mut self, action: PanelAction<'a, ActionId>) -> Self {
        self.actions.push(action);
        self
    }

    /// Replaces app actions.
    pub fn with_actions(
        mut self,
        actions: impl IntoIterator<Item = PanelAction<'a, ActionId>>,
    ) -> Self {
        self.actions = actions.into_iter().collect();
        self
    }

    /// Sets visibility.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Sets disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets collapsible behavior.
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    /// Sets restorable behavior.
    pub fn restorable(mut self, restorable: bool) -> Self {
        self.restorable = restorable;
        self
    }

    /// Sets maximizable behavior.
    pub fn maximizable(mut self, maximizable: bool) -> Self {
        self.maximizable = maximizable;
        self
    }

    /// Sets closable behavior.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Sets truncation/accessibility tooltip.
    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}

impl<PanelId> WorkbenchPanelHostState<PanelId> {
    /// Builds host state for a region.
    pub fn new(region: WorkbenchRegion) -> Self {
        Self {
            region,
            active_panel: None,
            selector: PanelSelectorPlacement::default_for_region(region),
            collapse_on_active_click: false,
            mode: PanelHostMode::Docked,
        }
    }

    /// Sets active panel.
    pub fn active_panel(mut self, panel_id: PanelId) -> Self {
        self.active_panel = Some(panel_id);
        self
    }

    /// Sets selector placement.
    pub fn selector(mut self, selector: PanelSelectorPlacement) -> Self {
        self.selector = selector;
        self
    }

    /// Enables or disables collapse-on-active-click rail behavior.
    pub fn collapse_on_active_click(mut self, enabled: bool) -> Self {
        self.collapse_on_active_click = enabled;
        self
    }

    /// Sets the host presentation mode.
    pub fn mode(mut self, mode: PanelHostMode) -> Self {
        self.mode = mode;
        self
    }

    /// Collapses the host to its selector rail, or restores docked presentation.
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.mode = if collapsed {
            PanelHostMode::Collapsed
        } else {
            PanelHostMode::Docked
        };
        self
    }
}

impl<PanelId> WorkbenchPanelHostState<PanelId>
where
    PanelId: Clone + PartialEq,
{
    /// Returns the semantic event for a rail activation.
    pub fn rail_activation_event<ActionId>(
        &self,
        panel_id: PanelId,
    ) -> WorkbenchPanelEvent<PanelId, ActionId> {
        if matches!(self.mode, PanelHostMode::Collapsed) {
            return WorkbenchPanelEvent::RestoreRequested {
                region: self.region,
                panel_id,
            };
        }

        if self.collapse_on_active_click && self.active_panel.as_ref() == Some(&panel_id) {
            return WorkbenchPanelEvent::CollapseRequested {
                region: self.region,
                panel_id,
            };
        }

        WorkbenchPanelEvent::Selected {
            region: self.region,
            panel_id,
        }
    }
}

impl<PanelId> Default for WorkbenchPanelHostState<PanelId> {
    fn default() -> Self {
        Self::new(WorkbenchRegion::Left)
    }
}

impl<'a, PanelId> PanelRailItem<'a, PanelId> {
    /// Creates a side-rail item with a required accessible label.
    pub fn new(panel_id: PanelId, icon: IconRole, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: panel_id,
            icon,
            label: label.into(),
            selected: false,
            disabled: false,
        }
    }

    /// Sets selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns panel id.
    pub fn id(&self) -> &PanelId {
        &self.id
    }
}
