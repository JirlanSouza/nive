//! Experimental devtools simulator support.
//!
//! These APIs are useful for framework diagnostics and development builds, but
//! the simulator internals are experimental before Nive 1.0. Production app
//! templates should not depend on this module directly.

pub mod command;
pub mod command_input;
mod helpers;
pub mod host;
pub mod panel;
pub mod probe;
pub mod types;
pub mod view;

pub use command::{DevtoolsRowId, SimulateAction, SimulateResult};
pub use helpers::join_path;
pub use host::{apply_simulate, collect_snapshot, DevtoolsApp};
pub use panel::{
    DevtoolsConfig, DevtoolsHostState, DevtoolsPanelEffect, DevtoolsPanelMessage,
    DevtoolsPanelState, DevtoolsPanelTab, DevtoolsWindowSpec,
};
pub use types::{
    DevtoolStateSnapshot, RegistryEntry, RegistryStatus, SimulatorCapabilities, SimulatorEntry,
    SimulatorKind, DEFAULT_CAPABILITY_HINT, INPUT_CAPABILITY_HINT, REFRESH_CAPABILITY_HINT,
    SAMPLE_CAPABILITY_HINT,
};
pub use view::devtools_window;
