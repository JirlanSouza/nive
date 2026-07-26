use nive_ui::widgets::RailSide;
use serde::{Deserialize, Serialize};

/// Named workbench regions used by fixed-region shells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WorkbenchRegion {
    /// Optional top toolbar or command strip.
    Toolbar,
    /// Left panel host.
    Left,
    /// Central document area.
    Center,
    /// Right panel host.
    Right,
    /// Bottom panel host.
    Bottom,
    /// Optional footer/status area.
    Status,
}

impl WorkbenchRegion {
    /// Returns whether this region can host workbench panels.
    pub const fn is_panel_region(self) -> bool {
        matches!(self, Self::Left | Self::Right | Self::Bottom)
    }

    /// Returns the window edge a side rail hugs for this region, when it hosts one.
    #[must_use]
    pub const fn rail_side(self) -> Option<RailSide> {
        match self {
            Self::Left => Some(RailSide::Left),
            Self::Right => Some(RailSide::Right),
            Self::Toolbar | Self::Center | Self::Bottom | Self::Status => None,
        }
    }
}

/// Region sizes for the fixed left/center/right/bottom shell.
///
/// Every field is the size of its own region in logical pixels, never a share of
/// the layout around it. The center region takes what is left over, so the side
/// and bottom regions keep their size when the shell is resized and one region's
/// divider never disturbs another.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorkbenchPaneSizes {
    /// Width of the left panel region.
    pub left: f32,
    /// Width of the right panel region.
    pub right: f32,
    /// Height of the bottom panel region.
    pub bottom: f32,
}

impl WorkbenchPaneSizes {
    /// Builds region sizes, normalizing every value.
    pub fn new(left: f32, right: f32, bottom: f32) -> Self {
        Self {
            left: Self::normalize(left),
            right: Self::normalize(right),
            bottom: Self::normalize(bottom),
        }
    }

    /// Normalizes a region size to a finite, non-negative length.
    ///
    /// Usable minimums live in [`crate::WorkbenchPaneConstraints`]; this only
    /// rejects degenerate values.
    pub fn normalize(size: f32) -> f32 {
        if size.is_finite() {
            size.max(0.0)
        } else {
            0.0
        }
    }

    /// Applies a region-specific size.
    pub fn with_region_size(mut self, region: WorkbenchRegion, size: f32) -> Self {
        self.set_region_size(region, size);
        self
    }

    /// Sets a region-specific size.
    pub fn set_region_size(&mut self, region: WorkbenchRegion, size: f32) {
        let size = Self::normalize(size);
        match region {
            WorkbenchRegion::Left => self.left = size,
            WorkbenchRegion::Right => self.right = size,
            WorkbenchRegion::Bottom => self.bottom = size,
            WorkbenchRegion::Toolbar | WorkbenchRegion::Center | WorkbenchRegion::Status => {}
        }
    }

    /// Returns the size used by a panel region.
    pub fn region_size(self, region: WorkbenchRegion) -> Option<f32> {
        match region {
            WorkbenchRegion::Left => Some(self.left),
            WorkbenchRegion::Right => Some(self.right),
            WorkbenchRegion::Bottom => Some(self.bottom),
            WorkbenchRegion::Toolbar | WorkbenchRegion::Center | WorkbenchRegion::Status => None,
        }
    }
}

impl Default for WorkbenchPaneSizes {
    fn default() -> Self {
        Self {
            left: 240.0,
            right: 300.0,
            bottom: 240.0,
        }
    }
}
