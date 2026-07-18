use iced::{Point, Rectangle, Size};

use crate::widgets::overlays::anchored_overlay::{
    content_limits, resolve_geometry, translated_bounds, GeometryInput, PopoverCollision,
    PopoverPlacement, PopoverWidth,
};

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

#[cfg(test)]
mod placement_tests;
