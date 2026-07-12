use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::layout::{WorkbenchLayoutState, WorkbenchRegion, WorkbenchSplitRatios};

/// Serializable layout sub-session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutSession<DocumentId = String, PanelId = String> {
    /// Durable shell layout state.
    pub state: WorkbenchLayoutState<DocumentId, PanelId>,
}

/// Serializable document sub-session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSession<DocumentId = String> {
    /// Active document id.
    pub active: Option<DocumentId>,
}

/// Serializable panel-region sub-session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelRegionSession<PanelId = String> {
    /// Region represented by this sub-session.
    pub region: WorkbenchRegion,
    /// Whether the region is collapsed.
    pub collapsed: bool,
    /// Active panel in the region.
    pub active: Option<PanelId>,
    /// Visible panel ordering.
    pub order: Vec<PanelId>,
}

/// Serializable panel sub-session for fixed panel hosts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelSession<PanelId = String> {
    /// Left host state.
    pub left: PanelRegionSession<PanelId>,
    /// Right host state.
    pub right: PanelRegionSession<PanelId>,
    /// Bottom host state.
    pub bottom: PanelRegionSession<PanelId>,
}

/// Serializable workbench session.
///
/// This type stores shell/view state only. It intentionally has no app
/// messages, widgets, resources, operations, command effects, files, devices,
/// documents, or other domain payloads.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkbenchSession<DocumentId = String, PanelId = String> {
    /// Layout-level session state.
    pub layout: LayoutSession<DocumentId, PanelId>,
    /// Document-area view state derived from `layout.state`.
    pub documents: DocumentSession<DocumentId>,
    /// Panel-host view state derived from `layout.state`.
    pub panels: PanelSession<PanelId>,
}

impl<DocumentId, PanelId> Default for LayoutSession<DocumentId, PanelId> {
    fn default() -> Self {
        Self {
            state: WorkbenchLayoutState::default(),
        }
    }
}

impl<DocumentId> Default for DocumentSession<DocumentId> {
    fn default() -> Self {
        Self { active: None }
    }
}

impl<PanelId> PanelRegionSession<PanelId> {
    /// Builds a session for a panel region.
    pub fn new(region: WorkbenchRegion) -> Self {
        Self {
            region,
            collapsed: false,
            active: None,
            order: Vec::new(),
        }
    }
}

impl<PanelId> Default for PanelSession<PanelId> {
    fn default() -> Self {
        Self {
            left: PanelRegionSession::new(WorkbenchRegion::Left),
            right: PanelRegionSession::new(WorkbenchRegion::Right),
            bottom: PanelRegionSession::new(WorkbenchRegion::Bottom),
        }
    }
}

impl<DocumentId, PanelId> Default for WorkbenchSession<DocumentId, PanelId> {
    fn default() -> Self {
        Self {
            layout: LayoutSession::default(),
            documents: DocumentSession::default(),
            panels: PanelSession::default(),
        }
    }
}

impl<DocumentId, PanelId> WorkbenchSession<DocumentId, PanelId>
where
    DocumentId: Clone,
    PanelId: Clone,
{
    /// Builds a serializable session from the durable layout state.
    pub fn from_layout_state(state: WorkbenchLayoutState<DocumentId, PanelId>) -> Self {
        let documents = DocumentSession {
            active: state.active_document().cloned(),
        };
        let collapsed: BTreeSet<WorkbenchRegion> = state.collapsed_regions().clone();
        let panels = PanelSession {
            left: region_session(&state, &collapsed, WorkbenchRegion::Left),
            right: region_session(&state, &collapsed, WorkbenchRegion::Right),
            bottom: region_session(&state, &collapsed, WorkbenchRegion::Bottom),
        };

        Self {
            layout: LayoutSession { state },
            documents,
            panels,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SerializedWorkbenchSession<DocumentId = String, PanelId = String> {
    layout: LayoutSession<DocumentId, PanelId>,
}

#[derive(Serialize)]
struct SerializedWorkbenchSessionRef<'a, DocumentId, PanelId> {
    layout: &'a LayoutSession<DocumentId, PanelId>,
}

impl<DocumentId, PanelId> Serialize for WorkbenchSession<DocumentId, PanelId>
where
    DocumentId: Serialize,
    PanelId: Serialize,
    LayoutSession<DocumentId, PanelId>: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedWorkbenchSessionRef {
            layout: &self.layout,
        }
        .serialize(serializer)
    }
}

impl<'de, DocumentId, PanelId> Deserialize<'de> for WorkbenchSession<DocumentId, PanelId>
where
    DocumentId: Deserialize<'de> + Clone,
    PanelId: Deserialize<'de> + Clone,
    LayoutSession<DocumentId, PanelId>: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serialized = SerializedWorkbenchSession::deserialize(deserializer)?;
        Ok(Self::from_layout_state(serialized.layout.state))
    }
}

fn region_session<DocumentId, PanelId>(
    state: &WorkbenchLayoutState<DocumentId, PanelId>,
    collapsed: &BTreeSet<WorkbenchRegion>,
    region: WorkbenchRegion,
) -> PanelRegionSession<PanelId>
where
    PanelId: Clone,
{
    PanelRegionSession {
        region,
        collapsed: collapsed.contains(&region),
        active: state.active_panel(region).cloned(),
        order: state
            .panel_order(region)
            .map_or_else(Vec::new, ToOwned::to_owned),
    }
}

impl<DocumentId, PanelId> From<WorkbenchLayoutState<DocumentId, PanelId>>
    for WorkbenchSession<DocumentId, PanelId>
where
    DocumentId: Clone,
    PanelId: Clone,
{
    fn from(state: WorkbenchLayoutState<DocumentId, PanelId>) -> Self {
        Self::from_layout_state(state)
    }
}

impl WorkbenchSession {
    /// Builds a default string-id session from split ratios.
    pub fn with_split_ratios(split_ratios: WorkbenchSplitRatios) -> Self {
        let mut state = WorkbenchLayoutState::<String, String>::default();
        state.set_split_ratio(WorkbenchRegion::Left, split_ratios.left);
        state.set_split_ratio(WorkbenchRegion::Right, split_ratios.right);
        state.set_split_ratio(WorkbenchRegion::Bottom, split_ratios.bottom);
        Self::from_layout_state(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_round_trips_view_state_without_domain_payloads() {
        let mut state = WorkbenchLayoutState::<String, String>::default();
        state.set_split_ratio(WorkbenchRegion::Left, 0.3);
        state.collapse_region(WorkbenchRegion::Bottom);
        state.set_active_panel(WorkbenchRegion::Left, "files".to_string());
        state.set_panel_order(
            WorkbenchRegion::Left,
            vec!["files".to_string(), "outline".to_string()],
        );

        let session = WorkbenchSession::from_layout_state(state);
        let json = serde_json::to_string(&session).expect("session serializes");
        let restored: WorkbenchSession = serde_json::from_str(&json).expect("session deserializes");

        assert_eq!(restored.layout.state.split_ratios().left, 0.3);
        assert!(restored.layout.state.is_collapsed(WorkbenchRegion::Bottom));
        assert_eq!(
            restored.panels.left.order,
            vec!["files".to_string(), "outline".to_string()]
        );
    }
}
