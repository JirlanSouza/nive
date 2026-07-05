use iced::{advanced::layout, Length, Point, Rectangle, Size, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopoverPlacement {
    TopStart,
    TopCenter,
    TopEnd,
    RightStart,
    RightCenter,
    RightEnd,
    #[default]
    BottomStart,
    BottomCenter,
    BottomEnd,
    LeftStart,
    LeftCenter,
    LeftEnd,
}

impl PopoverPlacement {
    fn flipped(self) -> Self {
        match self {
            Self::TopStart => Self::BottomStart,
            Self::TopCenter => Self::BottomCenter,
            Self::TopEnd => Self::BottomEnd,
            Self::RightStart => Self::LeftStart,
            Self::RightCenter => Self::LeftCenter,
            Self::RightEnd => Self::LeftEnd,
            Self::BottomStart => Self::TopStart,
            Self::BottomCenter => Self::TopCenter,
            Self::BottomEnd => Self::TopEnd,
            Self::LeftStart => Self::RightStart,
            Self::LeftCenter => Self::RightCenter,
            Self::LeftEnd => Self::RightEnd,
        }
    }

    fn side(self) -> Side {
        match self {
            Self::TopStart | Self::TopCenter | Self::TopEnd => Side::Top,
            Self::RightStart | Self::RightCenter | Self::RightEnd => Side::Right,
            Self::BottomStart | Self::BottomCenter | Self::BottomEnd => Side::Bottom,
            Self::LeftStart | Self::LeftCenter | Self::LeftEnd => Side::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PopoverWidth {
    #[default]
    Content,
    MatchAnchor,
    AtLeastAnchor,
    Fixed(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopoverCollision {
    #[default]
    FlipAndShift,
    Flip,
    Shift,
    None,
}

impl PopoverCollision {
    fn flips(self) -> bool {
        matches!(self, Self::FlipAndShift | Self::Flip)
    }

    fn shifts(self) -> bool {
        matches!(self, Self::FlipAndShift | Self::Shift)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

pub(crate) fn content_limits(
    width: PopoverWidth,
    anchor: Rectangle,
    viewport: Size,
) -> layout::Limits {
    let limits = layout::Limits::new(Size::ZERO, viewport).width(Length::Shrink);

    match width {
        PopoverWidth::Content => limits,
        PopoverWidth::MatchAnchor => limits.width(anchor.width),
        PopoverWidth::AtLeastAnchor => limits.min_width(anchor.width),
        PopoverWidth::Fixed(width) => limits.width(width.max(0.0)),
    }
}

pub(crate) fn resolve_position(
    anchor: Rectangle,
    content_size: Size,
    viewport: Size,
    placement: PopoverPlacement,
    collision: PopoverCollision,
    gap: f32,
) -> Point {
    let viewport = Rectangle::with_size(viewport);
    let mut position = place(anchor, content_size, placement, gap);

    if collision.flips() {
        let flipped = placement.flipped();
        let flipped_position = place(anchor, content_size, flipped, gap);

        if should_flip(
            position,
            flipped_position,
            content_size,
            viewport,
            placement.side(),
        ) {
            position = flipped_position;
        }
    }

    if collision.shifts() {
        position = shift_into_viewport(position, content_size, viewport);
    }

    position
}

fn place(anchor: Rectangle, size: Size, placement: PopoverPlacement, gap: f32) -> Point {
    let gap = gap.max(0.0);

    match placement {
        PopoverPlacement::TopStart => top(anchor, size, gap, align_start(anchor.x)),
        PopoverPlacement::TopCenter => top(anchor, size, gap, align_center_x(anchor, size)),
        PopoverPlacement::TopEnd => top(anchor, size, gap, align_end_x(anchor, size)),
        PopoverPlacement::RightStart => right(anchor, gap, align_start(anchor.y)),
        PopoverPlacement::RightCenter => right(anchor, gap, align_center_y(anchor, size)),
        PopoverPlacement::RightEnd => right(anchor, gap, align_end_y(anchor, size)),
        PopoverPlacement::BottomStart => bottom(anchor, gap, align_start(anchor.x)),
        PopoverPlacement::BottomCenter => bottom(anchor, gap, align_center_x(anchor, size)),
        PopoverPlacement::BottomEnd => bottom(anchor, gap, align_end_x(anchor, size)),
        PopoverPlacement::LeftStart => left(anchor, size, gap, align_start(anchor.y)),
        PopoverPlacement::LeftCenter => left(anchor, size, gap, align_center_y(anchor, size)),
        PopoverPlacement::LeftEnd => left(anchor, size, gap, align_end_y(anchor, size)),
    }
}

fn top(anchor: Rectangle, size: Size, gap: f32, x: f32) -> Point {
    Point::new(x, anchor.y - size.height - gap)
}

fn right(anchor: Rectangle, gap: f32, y: f32) -> Point {
    Point::new(anchor.x + anchor.width + gap, y)
}

fn bottom(anchor: Rectangle, gap: f32, x: f32) -> Point {
    Point::new(x, anchor.y + anchor.height + gap)
}

fn left(anchor: Rectangle, size: Size, gap: f32, y: f32) -> Point {
    Point::new(anchor.x - size.width - gap, y)
}

fn align_start(value: f32) -> f32 {
    value
}

fn align_center_x(anchor: Rectangle, size: Size) -> f32 {
    anchor.x + (anchor.width - size.width) / 2.0
}

fn align_end_x(anchor: Rectangle, size: Size) -> f32 {
    anchor.x + anchor.width - size.width
}

fn align_center_y(anchor: Rectangle, size: Size) -> f32 {
    anchor.y + (anchor.height - size.height) / 2.0
}

fn align_end_y(anchor: Rectangle, size: Size) -> f32 {
    anchor.y + anchor.height - size.height
}

fn should_flip(
    position: Point,
    flipped_position: Point,
    size: Size,
    viewport: Rectangle,
    side: Side,
) -> bool {
    let current = main_axis_overflow(position, size, viewport, side);
    let flipped = main_axis_overflow(flipped_position, size, viewport, opposite(side));

    current > 0.0 && flipped < current
}

fn main_axis_overflow(position: Point, size: Size, viewport: Rectangle, side: Side) -> f32 {
    match side {
        Side::Top => (viewport.y - position.y).max(0.0),
        Side::Right => (position.x + size.width - (viewport.x + viewport.width)).max(0.0),
        Side::Bottom => (position.y + size.height - (viewport.y + viewport.height)).max(0.0),
        Side::Left => (viewport.x - position.x).max(0.0),
    }
}

fn opposite(side: Side) -> Side {
    match side {
        Side::Top => Side::Bottom,
        Side::Right => Side::Left,
        Side::Bottom => Side::Top,
        Side::Left => Side::Right,
    }
}

fn shift_into_viewport(position: Point, size: Size, viewport: Rectangle) -> Point {
    let max_x = viewport.x + (viewport.width - size.width).max(0.0);
    let max_y = viewport.y + (viewport.height - size.height).max(0.0);

    Point::new(
        position.x.clamp(viewport.x, max_x),
        position.y.clamp(viewport.y, max_y),
    )
}

pub fn translated_bounds(bounds: Rectangle, translation: Vector) -> Rectangle {
    Rectangle {
        x: bounds.x + translation.x,
        y: bounds.y + translation.y,
        ..bounds
    }
}

#[cfg(test)]
mod placement_tests;
