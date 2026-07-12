mod ratios;
#[cfg(test)]
mod tests;

pub use ratios::{WorkbenchLayout, WorkbenchRegion, WorkbenchSplitRatios};

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// A maximized panel identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaximizedPanel<PanelId> {
    /// Region that owns the maximized panel.
    pub region: WorkbenchRegion,
    /// App-provided panel id.
    pub panel_id: PanelId,
}

impl<PanelId> MaximizedPanel<PanelId> {
    /// Builds a maximized panel identity.
    pub fn new(region: WorkbenchRegion, panel_id: PanelId) -> Self {
        Self { region, panel_id }
    }
}

/// View-state snapshot used to restore a maximized panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkbenchRestoreSnapshot<DocumentId, PanelId> {
    /// Split ratios before maximize.
    pub split_ratios: WorkbenchSplitRatios,
    /// Collapsed regions before maximize.
    pub collapsed_regions: BTreeSet<WorkbenchRegion>,
    /// Active document before maximize.
    pub active_document: Option<DocumentId>,
    /// Active panel per region before maximize.
    pub active_panels: BTreeMap<WorkbenchRegion, PanelId>,
    /// Panel ordering per region before maximize.
    pub panel_order: BTreeMap<WorkbenchRegion, Vec<PanelId>>,
}

/// Durable app-owned shell state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkbenchLayoutState<DocumentId = String, PanelId = String> {
    split_ratios: WorkbenchSplitRatios,
    collapsed_regions: BTreeSet<WorkbenchRegion>,
    maximized: Option<MaximizedPanel<PanelId>>,
    restore_snapshot: Option<WorkbenchRestoreSnapshot<DocumentId, PanelId>>,
    active_document: Option<DocumentId>,
    active_panels: BTreeMap<WorkbenchRegion, PanelId>,
    panel_order: BTreeMap<WorkbenchRegion, Vec<PanelId>>,
}

/// Layout-state transition returned by [`WorkbenchLayoutState`] mutators.
///
/// The same type is used for shell layout events when a rendered widget asks
/// the app to feed updated controlled state back into the shell. In the current
/// shell event path only [`WorkbenchLayoutChange::SplitRatioChanged`] is
/// emitted directly; the collapse, restore, maximize, active panel, and active
/// document variants are returned by state mutators and can also be reached
/// through [`crate::shell::WorkbenchEvent::apply_to`].
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum WorkbenchLayoutChange<DocumentId, PanelId> {
    /// A controlled split ratio changed.
    SplitRatioChanged {
        /// Affected region.
        region: WorkbenchRegion,
        /// New ratio, already clamped to workbench bounds.
        ratio: f32,
    },
    /// A region should collapse.
    RegionCollapsed(WorkbenchRegion),
    /// A region should restore.
    RegionRestored(WorkbenchRegion),
    /// Active document should change.
    ActiveDocumentChanged(Option<DocumentId>),
    /// Active panel should change.
    ActivePanelChanged {
        /// Panel host region.
        region: WorkbenchRegion,
        /// Active panel id.
        panel_id: PanelId,
    },
    /// A panel should maximize.
    PanelMaximized(MaximizedPanel<PanelId>),
    /// The current maximized panel should restore.
    PanelRestored,
}

impl<DocumentId, PanelId> Default for WorkbenchLayoutState<DocumentId, PanelId> {
    fn default() -> Self {
        Self {
            split_ratios: WorkbenchSplitRatios::default(),
            collapsed_regions: BTreeSet::new(),
            maximized: None,
            restore_snapshot: None,
            active_document: None,
            active_panels: BTreeMap::new(),
            panel_order: BTreeMap::new(),
        }
    }
}

impl<DocumentId, PanelId> WorkbenchLayoutState<DocumentId, PanelId> {
    /// Builds a default layout state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns current split ratios.
    pub const fn split_ratios(&self) -> WorkbenchSplitRatios {
        self.split_ratios
    }

    /// Returns collapsed regions.
    pub fn collapsed_regions(&self) -> &BTreeSet<WorkbenchRegion> {
        &self.collapsed_regions
    }

    /// Returns whether a region is collapsed.
    pub fn is_collapsed(&self, region: WorkbenchRegion) -> bool {
        self.collapsed_regions.contains(&region)
    }

    /// Returns the maximized panel, if any.
    pub fn maximized(&self) -> Option<&MaximizedPanel<PanelId>> {
        self.maximized.as_ref()
    }

    /// Returns the restore snapshot, if a panel is maximized.
    pub fn restore_snapshot(&self) -> Option<&WorkbenchRestoreSnapshot<DocumentId, PanelId>> {
        self.restore_snapshot.as_ref()
    }

    /// Returns the active document id.
    pub fn active_document(&self) -> Option<&DocumentId> {
        self.active_document.as_ref()
    }

    /// Sets the active document id and returns the updated state.
    pub fn with_active_document(mut self, id: DocumentId) -> Self {
        self.active_document = Some(id);
        self
    }

    /// Sets the active document id.
    pub fn set_active_document(
        &mut self,
        id: Option<DocumentId>,
    ) -> WorkbenchLayoutChange<DocumentId, PanelId>
    where
        DocumentId: Clone,
    {
        self.active_document = id.clone();
        WorkbenchLayoutChange::ActiveDocumentChanged(id)
    }

    /// Returns the active panel for a region.
    pub fn active_panel(&self, region: WorkbenchRegion) -> Option<&PanelId> {
        self.active_panels.get(&region)
    }

    /// Returns panel order for a region.
    pub fn panel_order(&self, region: WorkbenchRegion) -> Option<&[PanelId]> {
        self.panel_order.get(&region).map(Vec::as_slice)
    }

    /// Returns all active panels.
    pub fn active_panels(&self) -> &BTreeMap<WorkbenchRegion, PanelId> {
        &self.active_panels
    }

    /// Returns all panel order entries.
    pub fn panel_orders(&self) -> &BTreeMap<WorkbenchRegion, Vec<PanelId>> {
        &self.panel_order
    }

    /// Applies a controlled split-ratio change.
    pub fn set_split_ratio(
        &mut self,
        region: WorkbenchRegion,
        ratio: f32,
    ) -> WorkbenchLayoutChange<DocumentId, PanelId> {
        let ratio = WorkbenchSplitRatios::clamp(ratio);
        self.split_ratios.set_region_ratio(region, ratio);
        WorkbenchLayoutChange::SplitRatioChanged { region, ratio }
    }

    /// Collapses a panel region.
    pub fn collapse_region(
        &mut self,
        region: WorkbenchRegion,
    ) -> Option<WorkbenchLayoutChange<DocumentId, PanelId>> {
        if region.is_panel_region() && self.collapsed_regions.insert(region) {
            Some(WorkbenchLayoutChange::RegionCollapsed(region))
        } else {
            None
        }
    }

    /// Restores a collapsed panel region.
    pub fn restore_region(
        &mut self,
        region: WorkbenchRegion,
    ) -> Option<WorkbenchLayoutChange<DocumentId, PanelId>> {
        if self.collapsed_regions.remove(&region) {
            Some(WorkbenchLayoutChange::RegionRestored(region))
        } else {
            None
        }
    }

    /// Sets active panel for a region.
    pub fn set_active_panel(
        &mut self,
        region: WorkbenchRegion,
        panel_id: PanelId,
    ) -> Option<WorkbenchLayoutChange<DocumentId, PanelId>>
    where
        PanelId: Clone + PartialEq,
    {
        if !region.is_panel_region() {
            return None;
        }

        if self.active_panels.get(&region) == Some(&panel_id) {
            return None;
        }

        self.active_panels.insert(region, panel_id.clone());
        Some(WorkbenchLayoutChange::ActivePanelChanged { region, panel_id })
    }

    /// Sets visible panel order for a region.
    pub fn set_panel_order(&mut self, region: WorkbenchRegion, order: Vec<PanelId>) {
        if region.is_panel_region() {
            self.panel_order.insert(region, order);
        }
    }

    /// Maximizes a panel and stores a view-state restore snapshot.
    pub fn maximize_panel(
        &mut self,
        region: WorkbenchRegion,
        panel_id: PanelId,
    ) -> WorkbenchLayoutChange<DocumentId, PanelId>
    where
        DocumentId: Clone,
        PanelId: Clone,
    {
        if self.restore_snapshot.is_none() {
            self.restore_snapshot = Some(WorkbenchRestoreSnapshot {
                split_ratios: self.split_ratios,
                collapsed_regions: self.collapsed_regions.clone(),
                active_document: self.active_document.clone(),
                active_panels: self.active_panels.clone(),
                panel_order: self.panel_order.clone(),
            });
        }

        let maximized = MaximizedPanel::new(region, panel_id);
        self.maximized = Some(maximized.clone());
        WorkbenchLayoutChange::PanelMaximized(maximized)
    }

    /// Restores the layout saved when a panel was maximized.
    pub fn restore_maximized(&mut self) -> Option<WorkbenchLayoutChange<DocumentId, PanelId>> {
        let snapshot = self.restore_snapshot.take()?;
        self.split_ratios = snapshot.split_ratios;
        self.collapsed_regions = snapshot.collapsed_regions;
        self.active_document = snapshot.active_document;
        self.active_panels = snapshot.active_panels;
        self.panel_order = snapshot.panel_order;
        self.maximized = None;
        Some(WorkbenchLayoutChange::PanelRestored)
    }
}
