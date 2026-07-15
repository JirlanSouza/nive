mod builders;

use std::borrow::Cow;

use nive_ui::{
    theme::ToneRole,
    widgets::{RailSide, VerticalRailBadge},
    Element, IconRole,
};

use crate::layout::WorkbenchRegion;

/// Selector placement for panels in a host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PanelSelectorPlacement {
    /// Do not render a selector.
    Hidden,
    /// Render a compact side rail.
    SideRail,
    /// Render compact tabs in a shared host header.
    HeaderTabs,
}

impl PanelSelectorPlacement {
    /// Returns the default selector placement for a workbench region.
    pub const fn default_for_region(region: WorkbenchRegion) -> Self {
        match region {
            WorkbenchRegion::Left | WorkbenchRegion::Right => Self::SideRail,
            WorkbenchRegion::Bottom => Self::HeaderTabs,
            WorkbenchRegion::Toolbar | WorkbenchRegion::Center | WorkbenchRegion::Status => {
                Self::Hidden
            }
        }
    }
}

/// How a panel host presents its region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum PanelHostMode {
    /// Selector plus the active panel (default docked presentation).
    #[default]
    Docked,
    /// Region collapsed to its selector rail only.
    Collapsed,
    /// A single panel maximized, exposing an un-maximize affordance.
    Maximized,
}

/// App action rendered in a panel header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelAction<'a, ActionId> {
    pub(super) id: ActionId,
    pub(super) label: Option<Cow<'a, str>>,
    pub(super) icon: Option<IconRole>,
    pub(super) accessible_label: Cow<'a, str>,
    pub(super) disabled: bool,
}

/// Generic app-provided panel hosted by a workbench region.
pub struct WorkbenchPanel<'a, PanelId, ActionId, Message> {
    pub(super) id: PanelId,
    pub(super) title: Cow<'a, str>,
    pub(super) icon: Option<IconRole>,
    pub(super) badge: Option<Cow<'a, str>>,
    pub(super) status: Option<ToneRole>,
    pub(super) content: Element<'a, Message>,
    pub(super) actions: Vec<PanelAction<'a, ActionId>>,
    pub(super) visible: bool,
    pub(super) disabled: bool,
    pub(super) collapsible: bool,
    pub(super) restorable: bool,
    pub(super) maximizable: bool,
    pub(super) closable: bool,
    pub(super) tooltip: Option<Cow<'a, str>>,
}

/// State owned by a panel host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbenchPanelHostState<PanelId> {
    /// Host region.
    pub region: WorkbenchRegion,
    /// Active panel id.
    pub active_panel: Option<PanelId>,
    /// Selector placement.
    pub selector: PanelSelectorPlacement,
    /// Whether selecting the already active rail item collapses the host.
    pub collapse_on_active_click: bool,
    /// Host presentation mode.
    pub mode: PanelHostMode,
}

/// Semantic panel event emitted by the workbench.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorkbenchPanelEvent<PanelId, ActionId> {
    /// Panel selection was requested.
    Selected {
        /// Host region.
        region: WorkbenchRegion,
        /// Panel id.
        panel_id: PanelId,
    },
    /// App action was activated.
    Action {
        /// Host region.
        region: WorkbenchRegion,
        /// Panel id.
        panel_id: PanelId,
        /// App action id.
        action_id: ActionId,
    },
    /// Host collapse was requested.
    CollapseRequested {
        /// Host region.
        region: WorkbenchRegion,
        /// Panel id that originated the request.
        panel_id: PanelId,
    },
    /// Host restore was requested.
    RestoreRequested {
        /// Host region.
        region: WorkbenchRegion,
        /// Panel id that should become active after restore.
        panel_id: PanelId,
    },
    /// Panel maximize was requested.
    MaximizeRequested {
        /// Host region.
        region: WorkbenchRegion,
        /// Panel id.
        panel_id: PanelId,
    },
    /// Panel restore from maximized state was requested.
    PanelRestoreRequested {
        /// Host region.
        region: WorkbenchRegion,
        /// Panel id.
        panel_id: PanelId,
    },
    /// Panel close was requested.
    CloseRequested {
        /// Host region.
        region: WorkbenchRegion,
        /// Panel id.
        panel_id: PanelId,
    },
}

/// Compact side-rail item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelRailItem<'a, PanelId> {
    pub(super) id: PanelId,
    pub(super) icon: IconRole,
    pub(super) label: Cow<'a, str>,
    pub(super) badge: Option<VerticalRailBadge<'a>>,
    pub(super) selected: bool,
    pub(super) disabled: bool,
}

/// Compact side rail for left and right panel hosts.
///
/// If no mapper is configured with [`PanelRail::on_select`], rail items render
/// from metadata but remain inert.
pub struct PanelRail<'a, PanelId, Message> {
    pub(super) side: RailSide,
    pub(super) items: Vec<PanelRailItem<'a, PanelId>>,
    pub(super) on_select: Option<Box<dyn Fn(PanelId) -> Message + 'a>>,
}

/// Metadata for one bottom header tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BottomHeaderTab<'a, PanelId> {
    /// Panel id.
    pub panel_id: PanelId,
    /// Visible label.
    pub label: Cow<'a, str>,
    /// Optional icon.
    pub icon: Option<IconRole>,
    /// Optional badge or count.
    pub badge: Option<Cow<'a, str>>,
    /// Optional status tone.
    pub status: Option<ToneRole>,
    /// Whether the tab is disabled.
    pub disabled: bool,
    /// Optional truncation tooltip.
    pub tooltip: Option<Cow<'a, str>>,
}

/// Standard workbench panel header bar.
pub struct PanelHeaderBar<'a, PanelId, ActionId> {
    pub(super) region: WorkbenchRegion,
    pub(super) panel_id: PanelId,
    pub(super) title: Cow<'a, str>,
    pub(super) tooltip: Option<Cow<'a, str>>,
    pub(super) icon: Option<IconRole>,
    pub(super) badge: Option<Cow<'a, str>>,
    pub(super) status: Option<ToneRole>,
    pub(super) actions: Vec<PanelAction<'a, ActionId>>,
    pub(super) collapsible: bool,
    pub(super) restorable: bool,
    pub(super) maximizable: bool,
    pub(super) closable: bool,
}
