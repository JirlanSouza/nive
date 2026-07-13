//! Common imports for building workbench shells.

pub use nive_ui::theme::{ControlSize, ToneRole};

pub use crate::commands::{
    CommandPaletteState, WorkbenchCommand, WorkbenchCommandPalette, WorkbenchCommandPaletteEvent,
};
pub use crate::documents::{
    DocumentArea, WorkbenchDocument, WorkbenchDocumentDropTarget, WorkbenchDocumentEvent,
};
pub use crate::explorer::{explorer_panel, ExplorerPanel};
pub use crate::inspector::{inspector_panel, InspectorState};
pub use crate::layout::{
    MaximizedPanel, WorkbenchLayout, WorkbenchLayoutChange, WorkbenchLayoutState, WorkbenchRegion,
    WorkbenchRestoreSnapshot, WorkbenchSplitRatios,
};
pub use crate::panels::{
    bottom_panel_slot, logs_panel_slot, operations_panel_slot, output_panel_slot, panel_host,
    BottomHeaderTab, PanelAction, PanelHeaderBar, PanelHostMode, PanelRail, PanelRailItem,
    PanelSelectorPlacement, RailSide, WorkbenchPanel, WorkbenchPanelEvent, WorkbenchPanelHostState,
};
pub use crate::problems::{Problem, ProblemLocation, ProblemSeverity, ProblemsPanel};
pub use crate::session::{
    DocumentSession, LayoutSession, PanelRegionSession, PanelSession, WorkbenchSession,
};
pub use crate::shell::{Workbench, WorkbenchEvent, WorkbenchShell};
pub use crate::status::{StatusBar, StatusItem};

#[cfg(feature = "runtime")]
pub use crate::runtime::action_palette_rows;
