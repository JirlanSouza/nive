//! Narrow vertical edge rail widget.
//!
//! `VerticalRail` is a controlled navigation primitive for professional
//! desktop shells. Applications own selection state; the rail renders each
//! item's `selected` flag independently and maps enabled item activation
//! through rail-level selection callbacks. Compact per-item metadata is carried
//! by a semantic badge rendered inside the item.

mod content;
mod item;
mod label;
mod layout;
mod style;
mod widget;

#[cfg(test)]
mod tests;

pub use item::{VerticalRailBadge, VerticalRailItem};
pub use widget::VerticalRail;

/// Window edge where a [`VerticalRail`] is rendered.
///
/// `Left` renders labels counter-clockwise so text reads bottom-to-top. `Right`
/// renders labels clockwise so text reads top-to-bottom. Rotation affects only
/// the label; icon and badge metadata remain upright, and interaction bounds
/// stay axis-aligned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailSide {
    /// Left window edge; labels read bottom-to-top.
    Left,
    /// Right window edge; labels read top-to-bottom.
    Right,
}
