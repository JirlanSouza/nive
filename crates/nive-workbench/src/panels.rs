mod header;
mod host;
mod model;
mod rail;
#[cfg(test)]
mod tests;

pub(crate) use host::panel_host_with_size;
pub use host::{
    bottom_panel_slot, logs_panel_slot, operations_panel_slot, output_panel_slot, panel_host,
};
pub use model::PanelRail;
pub use model::{
    BottomHeaderTab, PanelAction, PanelHeaderBar, PanelHostMode, PanelRailItem,
    PanelSelectorPlacement, WorkbenchPanel, WorkbenchPanelEvent, WorkbenchPanelHostState,
};
pub use nive_ui::widgets::RailSide;
