//! Professional fixed-region workbench shell for Nive applications.
//!
//! `nive-workbench` sits above `nive-ui`: it composes existing widgets into a
//! desktop shell vocabulary with document tabs, generic panel hosts, compact
//! side rails, bottom header tabs, status bars, and serializable
//! layout/session state. Command palette hosting is the canonical
//! `nive_ui::widgets::CommandPalette`; this crate provides no bespoke host.
//!
//! The crate owns only shell and view state. Applications continue to own
//! domain state, side effects, persistence location, command execution,
//! resources, operations, native-window behavior, and final message routing.
//! Workbench interactions are emitted as semantic events so apps can map them
//! into product-specific messages at one boundary.
//!
//! Runtime integration is optional. Default features expose the shell, layout,
//! document, panel, diagnostics, status, and session APIs without requiring
//! lifecycle types from `nive-runtime`. Enable the `runtime` feature for
//! adapters to runtime concepts such as action-map-backed command palette
//! items.
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
//! use nive_ui::widgets::Toolbar;
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
//!     .chrome_size(ControlSize::Sm)
//!     .toolbar(Toolbar::new())
//!     .left_panels([WorkbenchPanel::new("files", "Files", Space::new())])
//!     .documents([WorkbenchDocument::new("overview", "Overview.md")])
//!     .document_content(text("App-owned document content"))
//!     .status(StatusBar::new())
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
//! Command palette hosting is the canonical `nive_ui::widgets::CommandPalette`;
//! `nive-workbench` provides no bespoke host and, with the `runtime` feature,
//! [`action_palette_items`] projects a shared `nive_core::ActionMap` directly
//! into its items:
//!
//! ```
//! use nive_ui::widgets::{CommandPalette, CommandPaletteItem};
//!
//! #[derive(Clone)]
//! enum Message {
//!     QueryChanged(String),
//!     Dismissed,
//!     Save,
//! }
//!
//! let items = [CommandPaletteItem::new("file.save", "Save", Message::Save)];
//! let _palette = CommandPalette::new(iced::widget::text("Base"))
//!     .open(true)
//!     .items(items)
//!     .on_query_change(Message::QueryChanged)
//!     .on_dismiss(Message::Dismissed);
//! ```

pub mod documents;
pub mod explorer;
pub mod inspector;
pub mod layout;
mod layout_probe;
pub mod panels;
pub mod prelude;
pub mod problems;
#[cfg(feature = "runtime")]
pub mod runtime;
pub mod session;
pub mod shell;
pub mod status;

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
