//! Narrow vertical edge rail widget.
//!
//! `SideRail` is a controlled navigation primitive for professional desktop
//! shells. Applications own selection state; the rail renders each item's
//! `selected` flag independently and maps enabled item activation through
//! rail-level selection callbacks.
//!
//! An item is a rotated label plus an optional icon, and carries no count or
//! status marker. The rail is one chrome height wide — room for a rotated label
//! and nothing beside it — so quantity and status belong to the panel an item
//! selects, the way narrow edge strips behave in professional desktop shells.
//! `NavigationRail` is the wider sibling that carries those markers.

mod content;
mod item;
mod label;
mod layout;
mod style;
mod widget;

#[cfg(test)]
mod side_rail_tests;
#[cfg(test)]
mod widget_tests;

pub use item::SideRailItem;
pub use widget::SideRail;

/// Window edge where a [`SideRail`] is rendered.
///
/// `Left` renders labels counter-clockwise so text reads bottom-to-top. `Right`
/// renders labels clockwise so text reads top-to-bottom. Rotation affects only
/// the label; the icon remains upright, and interaction bounds stay
/// axis-aligned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailSide {
    /// Left window edge; labels read bottom-to-top.
    Left,
    /// Right window edge; labels read top-to-bottom.
    Right,
}
