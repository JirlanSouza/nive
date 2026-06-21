//! Nive - A Rust/Iced framework for building desktop applications.
//!
//! This is the umbrella crate that re-exports `nive-ui` and `nive-runtime`
//! for convenient app development.
//!
//! # Quick Start
//!
//! ```ignore
//! use nive::prelude::*;
//!
//! struct MyApp;
//!
//! impl Application for MyApp {
//!     // ... implement Application trait
//! }
//!
//! fn main() -> iced::Result {
//!     nive::run::<MyApp>("My App", ())
//! }
//! ```
//!
//! # Crates
//!
//! - `nive-ui`: visual design system (tokens, theme, widgets, icons)
//! - `nive-runtime`: application lifecycle, window management, feedback, devtools
//!
//! # Status
//!
//! Part of Nive **v0.1.0**, a beta release. Public APIs may change before 1.0.

pub use nive_runtime as runtime;
pub use nive_ui as ui;

pub mod prelude {
    pub use nive_runtime::prelude::*;
    pub use nive_ui::prelude::*;

    pub use nive_runtime::ToastPosition;
    pub use nive_runtime::ToastTone;
    pub use nive_ui::widgets::Icon;
}

pub use nive_runtime::*;
pub use nive_ui::*;

pub use nive_runtime::{ToastPosition, ToastTone};
