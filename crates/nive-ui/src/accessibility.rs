//! Keyboard navigation and the standalone managed-focus boundary.
//!
//! [`FocusRoot`] is an explicit opt-in for applications that use `nive-ui`
//! without `nive-runtime`. Wrap the final content once per window. Runtime
//! applications already receive exactly one root automatically and must not
//! add another one.
//!
//! The root is layout- and paint-neutral. It coordinates one logical
//! sequential anchor across base content and nested overlays while Iced
//! remains responsible for traversal order. Custom widget authors integrate
//! through [`crate::advanced::focus::FocusState`].

pub(crate) mod focus_root;

pub use focus_root::FocusRoot;

pub use crate::focus_trap::{direction_from_event, direction_from_keyboard_event, FocusDirection};
