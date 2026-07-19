mod geometry;
mod identity;
mod overlay;
#[allow(dead_code)]
pub(crate) mod scroll;

use iced::{advanced::layout, Length, Rectangle, Size, Vector};

pub(crate) use geometry::{resolve_geometry, GeometryInput, SAFE_VIEWPORT_MARGIN};
pub(crate) use identity::{expose_node, OverlayIdentity, OverlayNodeState};
pub(crate) use overlay::AnchoredOverlay;
pub(crate) type ColorInputCompatOverlay<'a, 'b, LocalMessage, Message, OnMessage> =
    AnchoredOverlay<'a, 'b, LocalMessage, Message, OnMessage>;
pub use geometry::{PopoverCollision, PopoverPlacement, PopoverWidth};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// How keyboard focus behaves when a Popover opens.
///
/// These policies consume the enclosing shared logical-focus root. They do not
/// expose or create a popup-local focus coordinator. Focus restoration uses an
/// internal opaque target and never treats a public widget id as restoration
/// identity.
pub enum PopoverFocusPolicy {
    /// Keep actual focus on the logical anchor. This is the default.
    #[default]
    RetainAnchor,
    /// Focus the first enabled descendant and let ordinary Tab traversal leave.
    ///
    /// A traversal exit requests dismissal only when the Popover has dismissal
    /// capability and does not restore over the traversal destination.
    FocusFirst,
    /// Focus the first enabled descendant and cycle Tab traversal inside.
    ///
    /// Escape, outside press, or owned activation may conditionally restore the
    /// captured anchor after the application supplies the closed state.
    Trap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PopoverDismissalCause {
    RestoreAnchor,
    TraversalExit,
}

pub(crate) fn content_limits(
    width: PopoverWidth,
    anchor: Rectangle,
    viewport: Size,
) -> layout::Limits {
    let safe_width = (finite_nonnegative(viewport.width) - SAFE_VIEWPORT_MARGIN * 2.0).max(0.0);
    let anchor_width = finite_nonnegative(anchor.width).min(safe_width);
    let limits = layout::Limits::new(
        Size::ZERO,
        Size::new(safe_width, finite_nonnegative(viewport.height)),
    )
    .width(Length::Shrink);

    match width {
        PopoverWidth::Content => limits,
        PopoverWidth::MatchAnchor => limits.width(anchor_width),
        PopoverWidth::AtLeastAnchor => limits.min_width(anchor_width),
        PopoverWidth::Fixed(width) => limits.width(finite_nonnegative(width).min(safe_width)),
    }
}

pub(crate) fn translated_bounds(bounds: Rectangle, translation: Vector) -> Rectangle {
    let translation_x = if translation.x.is_finite() {
        translation.x
    } else {
        0.0
    };
    let translation_y = if translation.y.is_finite() {
        translation.y
    } else {
        0.0
    };

    Rectangle {
        x: if bounds.x.is_finite() {
            bounds.x + translation_x
        } else {
            translation_x
        },
        y: if bounds.y.is_finite() {
            bounds.y + translation_y
        } else {
            translation_y
        },
        width: finite_nonnegative(bounds.width),
        height: finite_nonnegative(bounds.height),
    }
}

fn finite_nonnegative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
