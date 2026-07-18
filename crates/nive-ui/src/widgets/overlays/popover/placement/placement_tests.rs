use iced::{Point, Rectangle, Size};

use super::*;

fn anchor() -> Rectangle {
    Rectangle::new(Point::new(40.0, 50.0), Size::new(80.0, 24.0))
}

fn content() -> Size {
    Size::new(100.0, 60.0)
}

#[test]
fn bottom_start_places_content_below_anchor_start() {
    let position = resolve_position(
        anchor(),
        content(),
        Size::new(300.0, 300.0),
        PopoverPlacement::BottomStart,
        PopoverCollision::None,
        8.0,
    );

    assert_eq!(position, Point::new(40.0, 82.0));
}

#[test]
fn bottom_center_places_content_below_anchor_center() {
    let position = resolve_position(
        anchor(),
        content(),
        Size::new(300.0, 300.0),
        PopoverPlacement::BottomCenter,
        PopoverCollision::None,
        8.0,
    );

    assert_eq!(position, Point::new(30.0, 82.0));
}

#[test]
fn top_end_places_content_above_anchor_end() {
    let position = resolve_position(
        anchor(),
        content(),
        Size::new(300.0, 300.0),
        PopoverPlacement::TopEnd,
        PopoverCollision::None,
        8.0,
    );

    assert_eq!(position, Point::new(20.0, 8.0));
}

#[test]
fn flip_and_shift_uses_opposite_side_when_main_axis_overflows() {
    let anchor = Rectangle::new(Point::new(40.0, 250.0), Size::new(80.0, 24.0));

    let position = resolve_position(
        anchor,
        content(),
        Size::new(300.0, 300.0),
        PopoverPlacement::BottomStart,
        PopoverCollision::FlipAndShift,
        8.0,
    );

    assert_eq!(position, Point::new(40.0, 182.0));
}

#[test]
fn shift_keeps_content_inside_viewport_without_flipping() {
    let position = resolve_position(
        anchor(),
        content(),
        Size::new(90.0, 90.0),
        PopoverPlacement::BottomStart,
        PopoverCollision::Shift,
        8.0,
    );

    assert_eq!(position, Point::new(8.0, 82.0));
}

#[test]
fn match_anchor_width_sets_fixed_width() {
    let limits = content_limits(PopoverWidth::MatchAnchor, anchor(), Size::new(300.0, 300.0));

    assert_eq!(limits.min().width, 80.0);
    assert_eq!(limits.max().width, 80.0);
}

#[test]
fn at_least_anchor_width_sets_minimum_width() {
    let limits = content_limits(
        PopoverWidth::AtLeastAnchor,
        anchor(),
        Size::new(300.0, 300.0),
    );

    assert_eq!(limits.min().width, 80.0);
    assert_eq!(limits.max().width, 284.0);
}

#[test]
fn translated_bounds_sanitizes_translation_and_dimensions() {
    let translated = translated_bounds(
        Rectangle {
            x: 10.0,
            y: 20.0,
            width: -40.0,
            height: f32::NAN,
        },
        iced::Vector::new(5.0, f32::INFINITY),
    );

    assert_eq!(
        translated,
        Rectangle::new(Point::new(15.0, 20.0), Size::ZERO)
    );
}
