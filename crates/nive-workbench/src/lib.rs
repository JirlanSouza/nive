//! Professional fixed-region workbench shell for Nive applications.
//!
//! `nive-workbench` sits above `nive-ui`: it composes existing widgets into a
//! desktop shell vocabulary with document tabs, generic panel hosts, compact
//! side rails, bottom header tabs, status bars, command palette hosting, and
//! serializable layout/session state.
//!
//! The crate owns only shell and view state. Applications continue to own
//! domain state, side effects, persistence location, command execution,
//! resources, operations, native-window behavior, and final message routing.
//! Workbench interactions are emitted as semantic events so apps can map them
//! into product-specific messages at one boundary.
//!
//! Runtime integration is optional. Default features expose the shell, layout,
//! document, panel, diagnostics, status, command palette host, and session APIs
//! without requiring lifecycle types from `nive-runtime`. Enable the `runtime`
//! feature for adapters to runtime concepts such as action-map-backed command
//! palette rows.
//!
//! ```
//! use nive_workbench::prelude::*;
//!
//! #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
//! enum Doc {
//!     Readme,
//! }
//!
//! let state = WorkbenchLayoutState::<Doc, &'static str>::default()
//!     .with_active_document(Doc::Readme);
//! let documents = [WorkbenchDocument::new(Doc::Readme, "README.md").closable(true)];
//! let _tabs = DocumentArea::new(state.active_document().cloned(), documents)
//!     .on_event(WorkbenchEvent::<Doc, &'static str, &'static str>::Document)
//!     .view();
//! ```
//!
//! Build a fixed-region shell from app-owned state and content:
//!
//! ```
//! use iced::widget::{text, Space};
//! use nive_workbench::prelude::*;
//!
//! #[derive(Clone)]
//! enum Message {
//!     Workbench(WorkbenchEvent<&'static str, &'static str, &'static str>),
//! }
//!
//! let state =
//!     WorkbenchLayoutState::<&'static str, &'static str>::default()
//!         .with_active_document("overview");
//! let _shell = WorkbenchShell::new(state, Message::Workbench)
//!     .left_panels([WorkbenchPanel::new("files", "Files", Space::new())])
//!     .documents([WorkbenchDocument::new("overview", "Overview.md")])
//!     .document_content(text("App-owned document content"))
//!     .view();
//! ```
//!
//! Host arbitrary custom panels in any panel region:
//!
//! ```
//! use iced::widget::Space;
//! use nive_workbench::prelude::*;
//!
//! #[derive(Clone)]
//! enum Message {
//!     Panel(WorkbenchPanelEvent<&'static str, &'static str>),
//! }
//!
//! let host = WorkbenchPanelHostState::new(WorkbenchRegion::Left)
//!     .active_panel("files");
//! let panel = WorkbenchPanel::new("files", "Files", Space::new())
//!     .action(PanelAction::icon("refresh", nive_ui::IconRole::ViewRefresh, "Refresh"));
//! let _view = panel_host(host, [panel], Message::Panel);
//! ```
//!
//! Host command palette interaction state while command execution remains
//! app-owned:
//!
//! ```
//! use nive_workbench::prelude::*;
//!
//! #[derive(Clone)]
//! enum Message {
//!     Palette(WorkbenchCommandPaletteEvent<&'static str>),
//! }
//!
//! let state = CommandPaletteState::new();
//! let commands = [WorkbenchCommand::new("file.save", "Save")];
//! let _palette = WorkbenchCommandPalette::new(&state, &commands, Message::Palette)
//!     .view();
//! ```

pub mod commands;
pub mod documents;
pub mod explorer;
pub mod inspector;
pub mod layout;
pub mod panels;
pub mod prelude;
pub mod problems;
#[cfg(feature = "runtime")]
pub mod runtime;
pub mod session;
pub mod shell;
pub mod status;

pub use commands::*;
pub use documents::*;
pub use explorer::*;
pub use inspector::*;
pub use layout::*;
pub use panels::*;
pub use problems::*;
#[cfg(feature = "runtime")]
pub use runtime::*;
pub use session::*;
pub use shell::*;
pub use status::*;
