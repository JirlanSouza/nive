//! Neutral presentation and interaction contracts shared by Nive layers.
//!
//! `nive-core` is the lowest layer of the [Nive](https://github.com/JirlanSouza/nive)
//! framework. It has **zero dependencies**, including no dependency on `iced`,
//! and defines dependency-free contracts for shared presentation status and
//! immutable application actions without depending on any concrete widget,
//! runtime state machine, toolkit event, or platform type.
//!
//! `nive-ui` re-exports these contracts so its feedback widgets can render
//! any type that implements them; `nive-runtime` implements them for its
//! concrete state types (`UserFacingError`, `Resource<T>`, `Operation<C>`,
//! `ToastItem`). Neither crate defines its own copy.
//!
//! # Charter
//!
//! `nive-core` exists to fix an inverted ownership boundary, not to become a
//! general shared-types dumping ground. A type belongs here only if it is:
//!
//! - a neutral presentation or interaction contract consumed by more than one layer, and
//! - free of any `iced`, widget, runtime lifecycle, or platform dependency.
//!
//! Concrete runtime types (`Resource`, `Operation`, `Toast`, `UserFacingError`,
//! UI vocabulary (`ToastPosition`, widgets, theme), and opinionated
//! helpers (tone-to-color mapping, error formatting) do **not** belong here —
//! they stay in the layer that owns them.
//!
//! # Status
//!
//! Part of Nive **v0.1.0**, a pre-publication alpha. Public APIs may change
//! before 1.0.

pub mod actions;
pub mod presentation;

pub use actions::{
    Action, ActionId, ActionMap, DuplicateActionId, NamedShortcutKey, ShortcutBinding, ShortcutKey,
    ShortcutMap, ShortcutModifiers,
};
pub use presentation::{
    AnnouncementPoliteness, ErrorPresentation, OperationStatusPresentation,
    ResourceStatusPresentation, ToastPresentation, ToastTone,
};
