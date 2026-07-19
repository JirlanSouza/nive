use iced::{Point, Rectangle, Size};

pub(crate) const SAFE_VIEWPORT_MARGIN: f32 = 8.0;
const AUTOMATIC_CONTENT_WIDTH_CAP: f32 = 360.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Preferred side and physical alignment for an anchored
/// [`Popover`](crate::widgets::overlays::Popover).
///
/// `Start` and `End` currently mean physical LTR alignment; they are not
/// resolved from a logical text direction. The collision policy may choose the
/// opposite side or shift the alignment without changing this vocabulary.
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
    pub(crate) const fn flipped(self) -> Self {
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

    const fn side(self) -> Side {
        match self {
            Self::TopStart | Self::TopCenter | Self::TopEnd => Side::Top,
            Self::RightStart | Self::RightCenter | Self::RightEnd => Side::Right,
            Self::BottomStart | Self::BottomCenter | Self::BottomEnd => Side::Bottom,
            Self::LeftStart | Self::LeftCenter | Self::LeftEnd => Side::Left,
        }
    }

    const fn alignment(self) -> Alignment {
        match self {
            Self::TopStart | Self::RightStart | Self::BottomStart | Self::LeftStart => {
                Alignment::Start
            }
            Self::TopCenter | Self::RightCenter | Self::BottomCenter | Self::LeftCenter => {
                Alignment::Center
            }
            Self::TopEnd | Self::RightEnd | Self::BottomEnd | Self::LeftEnd => Alignment::End,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
/// Width policy for a Popover's floating frame.
///
/// Automatic content growth is capped at 360px and then clamped to the safe
/// viewport and chosen side. `AtLeastAnchor` preserves a wider safe anchor as
/// its floor. Negative and non-finite fixed values normalize to zero.
pub enum PopoverWidth {
    /// Use intrinsic content width, capped at 360px and safe available width.
    #[default]
    Content,
    /// Match the anchor width, clamped to safe available width.
    MatchAnchor,
    /// Use at least the anchor width while capping only automatic content growth.
    AtLeastAnchor,
    /// Request an exact logical-pixel width before safe-viewport clamping.
    Fixed(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Collision correction applied to a preferred [`PopoverPlacement`].
///
/// Every policy still bounds the frame to the nonnegative space available on
/// its chosen side. Flipping selects a side first; shifting corrects only the
/// perpendicular alignment axis.
pub enum PopoverCollision {
    /// Flip when needed, then shift the chosen side into safe alignment.
    #[default]
    FlipAndShift,
    /// Flip when needed without alignment-axis shifting.
    Flip,
    /// Keep the preferred side and shift only its alignment axis.
    Shift,
    /// Keep the requested side and alignment without flip or shift correction.
    None,
}

impl PopoverCollision {
    const fn flips(self) -> bool {
        matches!(self, Self::FlipAndShift | Self::Flip)
    }

    const fn shifts(self) -> bool {
        matches!(self, Self::FlipAndShift | Self::Shift)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GeometryInput {
    pub(crate) anchor: Rectangle,
    pub(crate) viewport: Rectangle,
    pub(crate) intrinsic_content: Size,
    pub(crate) placement: PopoverPlacement,
    pub(crate) collision: PopoverCollision,
    pub(crate) width: PopoverWidth,
    pub(crate) gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ResolvedGeometry {
    pub(crate) placement: PopoverPlacement,
    pub(crate) safe_viewport: Rectangle,
    pub(crate) available: Rectangle,
    pub(crate) frame: Rectangle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alignment {
    Start,
    Center,
    End,
}

pub(crate) fn resolve_geometry(input: GeometryInput) -> ResolvedGeometry {
    let anchor = sanitize_rectangle(input.anchor);
    let viewport = sanitize_rectangle(input.viewport);
    let intrinsic_content = sanitize_size(input.intrinsic_content);
    let gap = finite_nonnegative(input.gap);
    let safe_viewport = safe_viewport(viewport, SAFE_VIEWPORT_MARGIN);
    let baseline_width = resolve_width(
        input.width,
        safe_viewport.width,
        anchor.width,
        intrinsic_content.width,
    );
    let natural_size = Size::new(baseline_width, intrinsic_content.height);
    let placement = choose_placement(
        input.placement,
        input.collision,
        anchor,
        safe_viewport,
        natural_size,
        gap,
    );
    let available = available_rectangle(placement.side(), anchor, safe_viewport, gap);
    let frame_size = Size::new(
        baseline_width.min(available.width),
        intrinsic_content.height.min(available.height),
    );
    let origin = place(anchor, frame_size, placement, gap);
    let origin = if input.collision.shifts() {
        shift_perpendicular(origin, frame_size, available, placement.side())
    } else {
        origin
    };

    ResolvedGeometry {
        placement,
        safe_viewport,
        available,
        frame: Rectangle::new(origin, frame_size),
    }
}

fn choose_placement(
    preferred: PopoverPlacement,
    collision: PopoverCollision,
    anchor: Rectangle,
    safe_viewport: Rectangle,
    natural_size: Size,
    gap: f32,
) -> PopoverPlacement {
    if !collision.flips() {
        return preferred;
    }

    let opposite = preferred.flipped();
    let preferred_available = available_rectangle(preferred.side(), anchor, safe_viewport, gap);
    if fits(natural_size, preferred_available) {
        return preferred;
    }

    let opposite_available = available_rectangle(opposite.side(), anchor, safe_viewport, gap);
    if fits(natural_size, opposite_available) {
        return opposite;
    }

    if main_axis_extent(opposite_available, opposite.side())
        > main_axis_extent(preferred_available, preferred.side())
    {
        opposite
    } else {
        preferred
    }
}

fn resolve_width(
    width: PopoverWidth,
    safe_width: f32,
    anchor_width: f32,
    content_width: f32,
) -> f32 {
    let automatic_content = content_width.min(AUTOMATIC_CONTENT_WIDTH_CAP);

    match width {
        PopoverWidth::Content => safe_width.min(automatic_content),
        PopoverWidth::MatchAnchor => safe_width.min(anchor_width),
        PopoverWidth::AtLeastAnchor => safe_width.min(anchor_width.max(automatic_content)),
        PopoverWidth::Fixed(width) => safe_width.min(finite_nonnegative(width)),
    }
}

fn safe_viewport(viewport: Rectangle, margin: f32) -> Rectangle {
    let horizontal_inset = margin.min(viewport.width / 2.0);
    let vertical_inset = margin.min(viewport.height / 2.0);

    Rectangle {
        x: viewport.x + horizontal_inset,
        y: viewport.y + vertical_inset,
        width: (viewport.width - margin * 2.0).max(0.0),
        height: (viewport.height - margin * 2.0).max(0.0),
    }
}

fn available_rectangle(
    side: Side,
    anchor: Rectangle,
    safe_viewport: Rectangle,
    gap: f32,
) -> Rectangle {
    let safe_right = safe_viewport.x + safe_viewport.width;
    let safe_bottom = safe_viewport.y + safe_viewport.height;
    let anchor_right = anchor.x + anchor.width;
    let anchor_bottom = anchor.y + anchor.height;

    match side {
        Side::Top => {
            let bottom = (anchor.y - gap).clamp(safe_viewport.y, safe_bottom);
            Rectangle::new(
                safe_viewport.position(),
                Size::new(safe_viewport.width, bottom - safe_viewport.y),
            )
        }
        Side::Right => {
            let left = (anchor_right + gap).clamp(safe_viewport.x, safe_right);
            Rectangle::new(
                Point::new(left, safe_viewport.y),
                Size::new(safe_right - left, safe_viewport.height),
            )
        }
        Side::Bottom => {
            let top = (anchor_bottom + gap).clamp(safe_viewport.y, safe_bottom);
            Rectangle::new(
                Point::new(safe_viewport.x, top),
                Size::new(safe_viewport.width, safe_bottom - top),
            )
        }
        Side::Left => {
            let right = (anchor.x - gap).clamp(safe_viewport.x, safe_right);
            Rectangle::new(
                safe_viewport.position(),
                Size::new(right - safe_viewport.x, safe_viewport.height),
            )
        }
    }
}

fn place(anchor: Rectangle, size: Size, placement: PopoverPlacement, gap: f32) -> Point {
    let perpendicular = match placement.side() {
        Side::Top | Side::Bottom => {
            align(anchor.x, anchor.width, size.width, placement.alignment())
        }
        Side::Right | Side::Left => {
            align(anchor.y, anchor.height, size.height, placement.alignment())
        }
    };

    match placement.side() {
        Side::Top => Point::new(perpendicular, anchor.y - gap - size.height),
        Side::Right => Point::new(anchor.x + anchor.width + gap, perpendicular),
        Side::Bottom => Point::new(perpendicular, anchor.y + anchor.height + gap),
        Side::Left => Point::new(anchor.x - gap - size.width, perpendicular),
    }
}

fn align(anchor_start: f32, anchor_extent: f32, frame_extent: f32, alignment: Alignment) -> f32 {
    match alignment {
        Alignment::Start => anchor_start,
        Alignment::Center => anchor_start + (anchor_extent - frame_extent) / 2.0,
        Alignment::End => anchor_start + anchor_extent - frame_extent,
    }
}

fn shift_perpendicular(origin: Point, size: Size, available: Rectangle, side: Side) -> Point {
    match side {
        Side::Top | Side::Bottom => Point::new(
            clamp_origin(origin.x, size.width, available.x, available.width),
            origin.y,
        ),
        Side::Right | Side::Left => Point::new(
            origin.x,
            clamp_origin(origin.y, size.height, available.y, available.height),
        ),
    }
}

fn clamp_origin(origin: f32, extent: f32, available_origin: f32, available_extent: f32) -> f32 {
    let maximum = available_origin + (available_extent - extent).max(0.0);
    origin.clamp(available_origin, maximum)
}

fn fits(size: Size, available: Rectangle) -> bool {
    size.width <= available.width && size.height <= available.height
}

fn main_axis_extent(available: Rectangle, side: Side) -> f32 {
    match side {
        Side::Top | Side::Bottom => available.height,
        Side::Right | Side::Left => available.width,
    }
}

pub(crate) fn sanitize_rectangle(rectangle: Rectangle) -> Rectangle {
    Rectangle {
        x: finite_or_zero(rectangle.x),
        y: finite_or_zero(rectangle.y),
        width: finite_nonnegative(rectangle.width),
        height: finite_nonnegative(rectangle.height),
    }
}

fn sanitize_size(size: Size) -> Size {
    Size::new(
        finite_nonnegative(size.width),
        finite_nonnegative(size.height),
    )
}

fn finite_nonnegative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(placement: PopoverPlacement) -> GeometryInput {
        GeometryInput {
            anchor: Rectangle::new(Point::new(100.0, 100.0), Size::new(80.0, 24.0)),
            viewport: Rectangle::with_size(Size::new(400.0, 300.0)),
            intrinsic_content: Size::new(120.0, 60.0),
            placement,
            collision: PopoverCollision::None,
            width: PopoverWidth::Content,
            gap: 4.0,
        }
    }

    #[test]
    fn all_placements_keep_the_four_pixel_main_axis_gap() {
        let cases = [
            (PopoverPlacement::TopStart, Point::new(100.0, 36.0)),
            (PopoverPlacement::TopCenter, Point::new(80.0, 36.0)),
            (PopoverPlacement::TopEnd, Point::new(60.0, 36.0)),
            (PopoverPlacement::RightStart, Point::new(184.0, 100.0)),
            (PopoverPlacement::RightCenter, Point::new(184.0, 82.0)),
            (PopoverPlacement::RightEnd, Point::new(184.0, 64.0)),
            (PopoverPlacement::BottomStart, Point::new(100.0, 128.0)),
            (PopoverPlacement::BottomCenter, Point::new(80.0, 128.0)),
            (PopoverPlacement::BottomEnd, Point::new(60.0, 128.0)),
            (PopoverPlacement::LeftStart, Point::new(8.0, 100.0)),
            (PopoverPlacement::LeftCenter, Point::new(8.0, 82.0)),
            (PopoverPlacement::LeftEnd, Point::new(8.0, 64.0)),
        ];

        for (placement, expected) in cases {
            assert_eq!(
                resolve_geometry(input(placement)).frame.position(),
                expected
            );
        }
    }

    #[test]
    fn width_profiles_follow_the_exact_formulas() {
        let cases = [
            (PopoverWidth::Content, 360.0),
            (PopoverWidth::MatchAnchor, 384.0),
            (PopoverWidth::AtLeastAnchor, 384.0),
            (PopoverWidth::Fixed(240.0), 240.0),
            (PopoverWidth::Fixed(-20.0), 0.0),
            (PopoverWidth::Fixed(f32::NAN), 0.0),
        ];

        for (width, expected) in cases {
            let mut case = input(PopoverPlacement::BottomStart);
            case.intrinsic_content.width = 500.0;
            case.anchor.width = 384.0;
            case.viewport.width = 500.0;
            case.width = width;
            assert_eq!(resolve_geometry(case).frame.width, expected);
        }
    }

    #[test]
    fn flip_uses_fit_then_greater_main_axis_and_preserves_ties() {
        let mut case = input(PopoverPlacement::BottomStart);
        case.anchor.y = 250.0;
        case.collision = PopoverCollision::Flip;
        assert_eq!(resolve_geometry(case).placement, PopoverPlacement::TopStart);

        case.anchor.y = 138.0;
        case.intrinsic_content.height = 200.0;
        assert_eq!(
            resolve_geometry(case).placement,
            PopoverPlacement::BottomStart
        );
    }

    #[test]
    fn collision_policies_apply_exact_flip_and_shift_axes() {
        let mut case = input(PopoverPlacement::BottomEnd);
        case.anchor = Rectangle::new(Point::new(0.0, 250.0), Size::new(40.0, 24.0));

        case.collision = PopoverCollision::None;
        let none = resolve_geometry(case);
        assert_eq!(none.placement, PopoverPlacement::BottomEnd);
        assert_eq!(none.frame.x, -80.0);
        assert_eq!(none.frame.height, 14.0);

        case.collision = PopoverCollision::Shift;
        let shift = resolve_geometry(case);
        assert_eq!(shift.placement, PopoverPlacement::BottomEnd);
        assert_eq!(shift.frame.x, 8.0);
        assert_eq!(shift.frame.height, 14.0);

        case.collision = PopoverCollision::Flip;
        let flip = resolve_geometry(case);
        assert_eq!(flip.placement, PopoverPlacement::TopEnd);
        assert_eq!(flip.frame.x, -80.0);
        assert_eq!(flip.frame.height, 60.0);

        case.collision = PopoverCollision::FlipAndShift;
        let flip_and_shift = resolve_geometry(case);
        assert_eq!(flip_and_shift.placement, PopoverPlacement::TopEnd);
        assert_eq!(flip_and_shift.frame.x, 8.0);
        assert_eq!(flip_and_shift.frame.height, 60.0);
    }

    #[test]
    fn horizontal_main_axis_uses_side_width_and_preserves_ties() {
        let mut case = input(PopoverPlacement::RightCenter);
        case.anchor.x = 160.0;
        case.intrinsic_content.width = 300.0;
        case.collision = PopoverCollision::Flip;
        assert_eq!(
            resolve_geometry(case).placement,
            PopoverPlacement::RightCenter
        );

        case.anchor.x = 320.0;
        assert_eq!(
            resolve_geometry(case).placement,
            PopoverPlacement::LeftCenter
        );
    }

    #[test]
    fn shift_only_moves_on_the_perpendicular_axis() {
        let mut case = input(PopoverPlacement::BottomEnd);
        case.anchor.x = 0.0;
        case.collision = PopoverCollision::Shift;
        let geometry = resolve_geometry(case);

        assert_eq!(geometry.frame.x, SAFE_VIEWPORT_MARGIN);
        assert_eq!(geometry.frame.y, 128.0);
    }

    #[test]
    fn left_and_right_width_clamp_to_side_space() {
        let mut case = input(PopoverPlacement::RightStart);
        case.anchor.x = 350.0;
        case.width = PopoverWidth::Fixed(200.0);
        let geometry = resolve_geometry(case);

        assert_eq!(geometry.frame.width, 0.0);
        assert_eq!(geometry.available.width, 0.0);
    }

    #[test]
    fn safe_viewport_and_chosen_rectangle_are_exact() {
        let mut case = input(PopoverPlacement::BottomStart);
        case.viewport = Rectangle::new(Point::new(10.0, 20.0), Size::new(200.0, 100.0));
        case.anchor = Rectangle::new(Point::new(40.0, 50.0), Size::new(80.0, 20.0));
        case.intrinsic_content = Size::new(300.0, 200.0);
        let geometry = resolve_geometry(case);

        assert_eq!(
            geometry.safe_viewport,
            Rectangle::new(Point::new(18.0, 28.0), Size::new(184.0, 84.0))
        );
        assert_eq!(
            geometry.available,
            Rectangle::new(Point::new(18.0, 74.0), Size::new(184.0, 38.0))
        );
        assert_eq!(geometry.frame.size(), Size::new(184.0, 38.0));
        assert_eq!(geometry.frame.position(), Point::new(40.0, 74.0));
    }

    #[test]
    fn omitted_semantic_defaults_are_distinct_from_explicit_zero() {
        assert_eq!(PopoverPlacement::default(), PopoverPlacement::BottomStart);
        assert_eq!(PopoverCollision::default(), PopoverCollision::FlipAndShift);
        assert_eq!(PopoverWidth::default(), PopoverWidth::Content);

        let mut case = input(PopoverPlacement::BottomStart);
        case.gap = -4.0;
        assert_eq!(resolve_geometry(case).frame.y, 124.0);
        case.gap = f32::INFINITY;
        assert_eq!(resolve_geometry(case).frame.y, 124.0);
    }

    #[test]
    fn malformed_and_tiny_geometry_stays_finite_and_nonnegative() {
        let geometry = resolve_geometry(GeometryInput {
            anchor: Rectangle {
                x: f32::NAN,
                y: f32::INFINITY,
                width: -10.0,
                height: f32::NAN,
            },
            viewport: Rectangle::with_size(Size::new(10.0, 6.0)),
            intrinsic_content: Size::new(f32::INFINITY, -30.0),
            placement: PopoverPlacement::BottomStart,
            collision: PopoverCollision::FlipAndShift,
            width: PopoverWidth::Content,
            gap: f32::NAN,
        });

        for value in [
            geometry.safe_viewport.x,
            geometry.safe_viewport.y,
            geometry.safe_viewport.width,
            geometry.safe_viewport.height,
            geometry.frame.x,
            geometry.frame.y,
            geometry.frame.width,
            geometry.frame.height,
        ] {
            assert!(value.is_finite());
            if value == geometry.frame.width
                || value == geometry.frame.height
                || value == geometry.safe_viewport.width
                || value == geometry.safe_viewport.height
            {
                assert!(value >= 0.0);
            }
        }
    }
}
