//! Canonical Dialog anatomy: [`Dialog`], [`DialogHeader`], [`DialogFooter`],
//! typed [`DialogAction`]/[`DialogTerminalAction`]/[`DialogActionFooter`],
//! and [`DialogSize`]. Hosting, the semantic backdrop, event routing, and
//! focus lifecycle live in [`super::DialogHost`]; the declarative runtime
//! request/dismissal policy lives in `nive_runtime::DialogRequest` and
//! `nive_runtime::DialogDismiss`.
//!
//! # Anatomy
//!
//! A canonical Dialog is `Dialog::new(body)` plus optional `.header(...)`,
//! `.footer(...)`, and `.size(DialogSize)`. The frame, insets, seams, and
//! internal body viewport are private: Dialog exposes no generic `Length`,
//! raw width/height, raw padding, raw radius, or `ControlSize` builder.
//!
//! ```
//! use nive_ui::prelude::*;
//!
//! #[derive(Clone)]
//! enum Message {
//!     Cancel,
//!     Save,
//! }
//!
//! let _dialog: Element<'_, Message> = Dialog::new(text("Body content"))
//!     .size(DialogSize::Md)
//!     .header(DialogHeader::new("Title").description("Supporting copy"))
//!     .footer(DialogActionFooter::with_one(
//!         DialogAction::cancel("Cancel", Message::Cancel),
//!         DialogTerminalAction::primary("Save", Message::Save),
//!     ))
//!     .into();
//! ```
//!
//! # Platform limits
//!
//! This family implements widget-event modality, logical-focus containment,
//! retained semantic copy, and semantic visual roles — it does not claim
//! platform features Iced 0.14 does not expose. In particular:
//!
//! - No native dialog role, `aria-modal`, title/description relationships,
//!   busy/live announcements, or accessibility-tree inertness are claimed
//!   before an AccessKit bridge exists; keyboard trapping and widget-level
//!   inertness are implemented and tested today.
//! - The frame paints the semantic large radius, but Iced 0.14 container
//!   clipping is rectangular — there is no true rounded descendant mask.
//! - Action placement is physical LTR and direction-ready, not full RTL.
//! - The modal capture boundary suppresses ordinary ignored-event listeners
//!   (`keyboard::listen()`, the runtime shortcut map), but a deliberately
//!   raw `event::listen_raw` subscription can still observe captured input
//!   by Iced design.
//! - Mount, Scrim appearance, state changes, and unmount are immediate: no
//!   duration, easing, timer, or retained exit subtree exists yet.
//!
//! # Private implementation surface
//!
//! Frame/geometry, action-layout, and focus-marker internals are not part
//! of the public API:
//!
//! ```compile_fail
//! use nive_ui::widgets::overlays::dialog::widget::Dialog;
//! ```
//!
//! ```compile_fail
//! use nive_ui::widgets::overlays::dialog::footer::{
//!     DialogActionFooterWidget, TerminalActionMarker, TerminalActionTag,
//! };
//! ```
//!
//! ```compile_fail
//! use nive_ui::prelude::*;
//! let dialog: Dialog<'_, ()> = Dialog::new(text("Body"));
//! let _ = dialog.width(iced::Length::Fill);
//! ```
//!
//! ```compile_fail
//! use nive_ui::prelude::*;
//! let _footer: DialogActionFooter<'_, ()> =
//!     DialogActionFooter::new(DialogAction::cancel("Cancel", ()));
//! ```

mod footer;
mod header;
mod size;
mod widget;

pub(crate) use footer::TerminalActionTag;
pub use footer::{
    DialogAction, DialogActionFooter, DialogActionFooterError, DialogActionRole, DialogFooter,
    DialogTerminalAction,
};
pub use header::DialogHeader;
pub use size::DialogSize;
pub use widget::Dialog;
