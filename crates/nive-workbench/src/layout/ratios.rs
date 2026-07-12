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
}

/// Split ratios for the fixed left/center/right/bottom shell.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorkbenchSplitRatios {
    /// Width share of the left panel when left and center are split.
    pub left: f32,
    /// Width share of the leading content when splitting content from right.
    pub right: f32,
    /// Height share of the upper content when splitting content from bottom.
    pub bottom: f32,
}

impl WorkbenchSplitRatios {
    /// Minimum accepted split ratio.
    pub const MIN: f32 = 0.1;
    /// Maximum accepted split ratio.
    pub const MAX: f32 = 0.9;

    /// Builds split ratios, clamping every ratio to the supported range.
    pub fn new(left: f32, right: f32, bottom: f32) -> Self {
        Self {
            left: Self::clamp(left),
            right: Self::clamp(right),
            bottom: Self::clamp(bottom),
        }
    }

    /// Clamps a ratio to the workbench-supported range.
    pub fn clamp(ratio: f32) -> f32 {
        if ratio.is_nan() {
            0.5
        } else {
            ratio.clamp(Self::MIN, Self::MAX)
        }
    }

    /// Applies a region-specific split ratio.
    pub fn with_region_ratio(mut self, region: WorkbenchRegion, ratio: f32) -> Self {
        self.set_region_ratio(region, ratio);
        self
    }

    /// Sets a region-specific split ratio.
    pub fn set_region_ratio(&mut self, region: WorkbenchRegion, ratio: f32) {
        let ratio = Self::clamp(ratio);
        match region {
            WorkbenchRegion::Left => self.left = ratio,
            WorkbenchRegion::Right => self.right = ratio,
            WorkbenchRegion::Bottom => self.bottom = ratio,
            WorkbenchRegion::Toolbar | WorkbenchRegion::Center | WorkbenchRegion::Status => {}
        }
    }

    /// Returns the split ratio used by a panel region.
    pub fn region_ratio(self, region: WorkbenchRegion) -> Option<f32> {
        match region {
            WorkbenchRegion::Left => Some(self.left),
            WorkbenchRegion::Right => Some(self.right),
            WorkbenchRegion::Bottom => Some(self.bottom),
            WorkbenchRegion::Toolbar | WorkbenchRegion::Center | WorkbenchRegion::Status => None,
        }
    }
}

impl Default for WorkbenchSplitRatios {
    fn default() -> Self {
        Self {
            left: 0.22,
            right: 0.78,
            bottom: 0.72,
        }
    }
}

/// Static shell layout options independent from app/domain data.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkbenchLayout {
    /// Current fixed-region split ratios.
    pub split_ratios: WorkbenchSplitRatios,
}

impl WorkbenchLayout {
    /// Builds a layout from split ratios.
    pub const fn new(split_ratios: WorkbenchSplitRatios) -> Self {
        Self { split_ratios }
    }
}
