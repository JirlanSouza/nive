//! Reusable visual design system for Rust/Iced desktop applications.
//!
//! `nive-ui` owns the design tokens, semantic theme contracts, and reusable
//! primitive widgets that are independent of product-specific domain logic. It
//! is the lowest layer of the Nive framework and depends only on `iced`.
//!
//! # Scope
//!
//! - `tokens` — color, spacing, radius, shadow, and typography constants.
//! - `theme` — semantic role enums, framework theme data (`Nive Light` / `Nive
//!   Dark`), active-theme accessors, and iced `Catalog` implementations.
//! - `widgets` — reusable primitive widgets (buttons, cards, fields, dialogs,
//!   toasts, feedback, metadata, animation, and more).
//! - `focus_trap` — Tab/Shift+Tab focus cycling helpers for overlays.
//! - `BootstrapView` — generic startup loading/failure template.
//! - `DialogHost` and `ToastHost` — modal and toast overlay composition.
//!
//! `nive-ui` does not depend on `nive-runtime` or any application crate; it is
//! a lower layer. Presentation contracts such as `ToastPresentation` keep
//! runtime types out of the UI crate.
//!
//! # Public API
//!
//! Application and screen code should prefer `nive_ui::prelude`, the crate
//! root, `nive_ui::theme`, and `nive_ui::widgets`. These facades expose the
//! shared `Element`/`Renderer` aliases, theme builders/catalogs, common Iced
//! layout primitives, and reusable widget contracts.
//!
//! Lower-level widget and theme submodules remain public for advanced
//! composition, styling, and focused tests. Generic app code should avoid
//! depending on private host internals or product-specific assumptions.
//!
//! # Status
//!
//! Part of Nive **v0.1.0**, a beta release. Public APIs may change before 1.0.
//! See `docs/components.md` for contract details.

pub mod bootstrap;
mod dialog_host;
pub mod focus_trap;
pub mod prelude;
pub mod theme;
mod toast_host;
pub mod tokens;
pub mod widgets;

pub use bootstrap::{BootstrapError, BootstrapView};
pub use dialog_host::DialogHost;
pub use theme::{Theme, ThemeBuilder, ThemeCatalog, ThemeData, ThemeId};
pub use toast_host::{ToastHost, ToastPosition, ToastPresentation, ToastTone};
pub use tokens::color;
pub use tokens::radius;
pub use tokens::shadow;
pub use tokens::spacing;
pub use tokens::typography;
pub use widgets::Separator;

pub type Renderer = iced::Renderer;
pub type Element<'a, Message> = iced::Element<'a, Message, Theme, Renderer>;
pub use iced::{advanced, border, widget};
pub use iced::{
    Alignment, Background, Border, Color, Length, Padding, Point, Radians, Rectangle, Shadow, Size,
    Vector,
};
