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
//! - `widgets` — reusable widgets grouped by taxonomy (`primitives`,
//!   `controls`, `display`, `containers`, `navigation`, `overlays`, and
//!   `feedback`), plus the flat widget facade. `DialogHost` and `ToastHost`
//!   live under `widgets::overlays`.
//! - `icons` — semantic `IconRole` values, resolved `IconGlyph` bytes,
//!   theme-owned `IconCatalog` values, and the `IconSource` contract used by
//!   generated app `IconSymbol` enums.
//! - `advanced` — internals for authoring custom widgets (layout math,
//!   border/style helpers, `Shell` propagation), mirroring `iced::advanced`.
//!   Not needed to assemble screens from `widgets`.
//! - `layout`, `graphics`, and `accessibility` — focused facades for layout
//!   surfaces, visual assets, and keyboard/focus affordances.
//! - `focus_trap` — Tab/Shift+Tab focus cycling helpers for overlays.
//! - `BootstrapView` — generic startup loading/failure template.
//!
//! `nive-ui` does not depend on `nive-runtime` or any application crate; it is
//! a lower layer. Presentation contracts such as `ToastPresentation` are
//! defined in `nive-core` (zero dependencies) and re-exported here, which
//! keeps runtime types out of the UI crate while letting feedback widgets
//! render any type that implements them.
//!
//! # Public API
//!
//! Application and screen code should prefer `nive_ui::prelude`, the crate
//! root, `nive_ui::theme`, and `nive_ui::widgets`. These facades expose the
//! shared `Element`/`Renderer` aliases, theme builders/catalogs, common Iced
//! layout primitives, icon roles/catalogs, and reusable widget contracts.
//!
//! Lower-level widget and theme submodules remain public for advanced
//! composition, styling, and focused tests. Generic app code should avoid
//! depending on private host internals or product-specific assumptions.
//!
//! # Status
//!
//! Part of Nive **v0.1.0**, a beta release. Public APIs may change before 1.0.
//! See `docs/components.md` for contract details.

pub mod accessibility;
pub mod advanced;
pub mod animation;
pub mod bootstrap;
pub mod focus_trap;
pub mod graphics;
pub mod icons;
pub mod interaction;
pub mod layout;
pub mod prelude;
pub mod theme;
pub mod tokens;

macro_rules! impl_layout_builders {
    () => {};
    (width_opt $(, $rest:ident)*) => {
        /// Sets the widget width.
        pub fn width(mut self, width: impl Into<Length>) -> Self {
            self.width = Some(width.into());
            self
        }

        crate::impl_layout_builders!($($rest),*);
    };
    (width_direct $(, $rest:ident)*) => {
        /// Sets the widget width.
        pub fn width(mut self, width: impl Into<Length>) -> Self {
            self.width = width.into();
            self
        }

        crate::impl_layout_builders!($($rest),*);
    };
    (height_opt $(, $rest:ident)*) => {
        /// Sets the widget height.
        pub fn height(mut self, height: impl Into<Length>) -> Self {
            self.height = Some(height.into());
            self
        }

        crate::impl_layout_builders!($($rest),*);
    };
    (height_direct $(, $rest:ident)*) => {
        /// Sets the widget height.
        pub fn height(mut self, height: impl Into<Length>) -> Self {
            self.height = height.into();
            self
        }

        crate::impl_layout_builders!($($rest),*);
    };
    (fill_width_opt $(, $rest:ident)*) => {
        /// Sets the widget width to fill available space.
        pub fn fill_width(mut self) -> Self {
            self.width = Some(Length::Fill);
            self
        }

        crate::impl_layout_builders!($($rest),*);
    };
    (fill_width_direct $(, $rest:ident)*) => {
        /// Sets the widget width to fill available space.
        pub fn fill_width(mut self) -> Self {
            self.width = Length::Fill;
            self
        }

        crate::impl_layout_builders!($($rest),*);
    };
    (fill_height_opt $(, $rest:ident)*) => {
        /// Sets the widget height to fill available space.
        pub fn fill_height(mut self) -> Self {
            self.height = Some(Length::Fill);
            self
        }

        crate::impl_layout_builders!($($rest),*);
    };
    (fill_height_direct $(, $rest:ident)*) => {
        /// Sets the widget height to fill available space.
        pub fn fill_height(mut self) -> Self {
            self.height = Length::Fill;
            self
        }

        crate::impl_layout_builders!($($rest),*);
    };
    (fill_opt $(, $rest:ident)*) => {
        /// Sets widget width and height to fill available space.
        pub fn fill(mut self) -> Self {
            self.width = Some(Length::Fill);
            self.height = Some(Length::Fill);
            self
        }

        crate::impl_layout_builders!($($rest),*);
    };
    (fill_direct $(, $rest:ident)*) => {
        /// Sets widget width and height to fill available space.
        pub fn fill(mut self) -> Self {
            self.width = Length::Fill;
            self.height = Length::Fill;
            self
        }

        crate::impl_layout_builders!($($rest),*);
    };
    (fill_direct_height_opt $(, $rest:ident)*) => {
        /// Sets widget width and height to fill available space.
        pub fn fill(mut self) -> Self {
            self.width = Length::Fill;
            self.height = Some(Length::Fill);
            self
        }

        crate::impl_layout_builders!($($rest),*);
    };
    (shrink_width_opt $(, $rest:ident)*) => {
        /// Sets the widget width to shrink to its content.
        pub fn shrink_width(mut self) -> Self {
            self.width = Some(Length::Shrink);
            self
        }

        crate::impl_layout_builders!($($rest),*);
    };
    (shrink_width_direct $(, $rest:ident)*) => {
        /// Sets the widget width to shrink to its content.
        pub fn shrink_width(mut self) -> Self {
            self.width = Length::Shrink;
            self
        }

        crate::impl_layout_builders!($($rest),*);
    };
}

pub(crate) use impl_layout_builders;

pub mod widgets;

pub use bootstrap::{BootstrapError, BootstrapView};
pub use icons::{IconCatalog, IconCatalogEntry, IconGlyph, IconRole, IconSource};
pub use theme::{Theme, ThemeBuilder, ThemeCatalog, ThemeData, ThemeDensity, ThemeId};
pub use tokens::color;
pub use tokens::radius;
pub use tokens::shadow;
pub use tokens::spacing;
pub use tokens::typography;

pub type Renderer = iced::Renderer;
pub type Element<'a, Message> = iced::Element<'a, Message, Theme, Renderer>;
pub use iced::{border, widget};
pub use iced::{
    Alignment, Background, Border, Color, Length, Padding, Point, Radians, Rectangle, Shadow, Size,
    Vector,
};
