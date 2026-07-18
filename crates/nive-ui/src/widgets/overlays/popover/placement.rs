use iced::{advanced::layout, Length, Rectangle, Size, Vector};

use crate::widgets::overlays::anchored_overlay::SAFE_VIEWPORT_MARGIN;
#[cfg(test)]
use crate::widgets::overlays::anchored_overlay::{resolve_geometry, GeometryInput};
#[cfg(test)]
use iced::Point;

pub use crate::widgets::overlays::anchored_overlay::{
    PopoverCollision, PopoverPlacement, PopoverWidth,
};

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

#[cfg(test)]
pub(crate) fn resolve_position(
    anchor: Rectangle,
    content_size: Size,
    viewport: Size,
    placement: PopoverPlacement,
    collision: PopoverCollision,
    gap: f32,
) -> Point {
    resolve_geometry(GeometryInput {
        anchor,
        viewport: Rectangle::with_size(viewport),
        intrinsic_content: content_size,
        placement,
        collision,
        width: PopoverWidth::Fixed(content_size.width),
        gap,
    })
    .frame
    .position()
}

pub fn translated_bounds(bounds: Rectangle, translation: Vector) -> Rectangle {
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

#[cfg(test)]
mod placement_tests;
