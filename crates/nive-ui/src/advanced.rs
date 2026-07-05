//! Internals for authoring custom widgets, mirroring the split that
//! `iced::advanced` makes between application code and widget authors.
//!
//! Nothing here is needed to assemble screens from `nive_ui::widgets`; it
//! exists for widget implementations inside and outside this crate that need
//! to share layout math, border/style helpers, or `Shell` propagation.

pub mod control_group;
pub mod control_style;
pub mod pressable;
pub mod shell_relay;
