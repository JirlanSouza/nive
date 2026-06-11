use iced::{
    advanced::{graphics::geometry::Renderer as _, Renderer as _},
    widget::canvas,
    Color, Point, Rectangle, Size, Vector,
};

use crate::theme::{BorderRole, Theme};

use super::super::metrics::MARKER_SIZE;
use super::coverage;

const MARKER_MARGIN: f32 = 1.0;
const FOCUS_RING_OUTSET: f32 = 1.0;
const FOCUS_MARGIN: f32 = 1.0;
const PIXEL_SIZE: f32 = 1.0;
const COVERAGE_SAMPLES: usize = 8;

pub(in crate::widgets::color_picker::controls) fn draw_marker(
    renderer: &mut iced::Renderer,
    theme: &Theme,
    center: Point,
    inner_color: Color,
    focused: bool,
) {
    let size = marker_size(focused);
    let top_left = marker_top_left(center, size);

    renderer.with_translation(Vector::new(top_left.x, top_left.y), |renderer| {
        let mut frame = canvas::Frame::new(renderer, size);
        let center = Point::new(size.width / 2.0, size.height / 2.0);

        if focused {
            draw_disk(
                &mut frame,
                center,
                focus_ring_radius(),
                focus_ring_color(theme),
            );
        }

        draw_disk(&mut frame, center, MARKER_SIZE / 2.0, Color::WHITE);
        draw_disk(&mut frame, center, MARKER_SIZE / 3.5, inner_color);

        renderer.draw_geometry(frame.into_geometry());
    });
}

fn draw_disk(frame: &mut canvas::Frame<iced::Renderer>, center: Point, radius: f32, color: Color) {
    let size = frame.size();
    let cols = (size.width / PIXEL_SIZE).ceil().max(0.0) as usize;
    let rows = (size.height / PIXEL_SIZE).ceil().max(0.0) as usize;

    for row in 0..rows {
        for col in 0..cols {
            let x = col as f32 * PIXEL_SIZE;
            let y = row as f32 * PIXEL_SIZE;
            let pixel_size = Size::new(
                PIXEL_SIZE.min(size.width - x),
                PIXEL_SIZE.min(size.height - y),
            );

            if pixel_size.width <= 0.0 || pixel_size.height <= 0.0 {
                continue;
            }

            let top_left = Point::new(x, y);
            let coverage =
                coverage::circle_coverage(center, radius, top_left, pixel_size, COVERAGE_SAMPLES);

            if coverage <= 0.0 {
                continue;
            }

            let mut color = color;
            color.a = (color.a * coverage).clamp(0.0, 1.0);

            frame.fill_rectangle(top_left, pixel_size, color);
        }
    }
}

fn focus_ring_color(theme: &Theme) -> Color {
    theme.border(BorderRole::Focus).color
}

fn outer_marker_radius() -> f32 {
    MARKER_SIZE / 2.0 + MARKER_MARGIN
}

fn focus_ring_radius() -> f32 {
    outer_marker_radius() + FOCUS_RING_OUTSET
}

fn marker_size(focused: bool) -> Size {
    let diameter = if focused {
        MARKER_SIZE + (MARKER_MARGIN + FOCUS_RING_OUTSET + FOCUS_MARGIN) * 2.0
    } else {
        MARKER_SIZE + MARKER_MARGIN * 2.0
    };

    Size::new(diameter, diameter)
}

fn marker_top_left(center: Point, size: Size) -> Point {
    Point::new(center.x - size.width / 2.0, center.y - size.height / 2.0)
}

pub(in crate::widgets::color_picker::controls) fn bounded_marker_center(
    bounds: Rectangle,
    x: f32,
    y: f32,
) -> Point {
    Point::new(
        clamp_marker_coordinate(x, bounds.x, bounds.width),
        clamp_marker_coordinate(y, bounds.y, bounds.height),
    )
}

fn clamp_marker_coordinate(value: f32, start: f32, length: f32) -> f32 {
    let radius = outer_marker_radius().min(length / 2.0);

    value.clamp(start + radius, start + length - radius)
}

#[cfg(test)]
mod marker_tests {
    use super::*;

    #[test]
    fn bounded_marker_center_keeps_marker_inside_control() {
        let bounds = Rectangle::new(Point::new(10.0, 20.0), Size::new(12.0, 136.0));
        let x = bounds.center_x();
        let y_margin = outer_marker_radius();

        assert_eq!(
            bounded_marker_center(bounds, bounds.x, bounds.y),
            Point::new(x, bounds.y + y_margin)
        );
        assert_eq!(
            bounded_marker_center(bounds, bounds.x + bounds.width, bounds.y + bounds.height),
            Point::new(x, bounds.y + bounds.height - y_margin)
        );
    }

    #[test]
    fn bounded_marker_center_handles_tiny_controls() {
        let bounds = Rectangle::new(Point::new(10.0, 20.0), Size::new(4.0, 4.0));

        assert_eq!(
            bounded_marker_center(bounds, bounds.x, bounds.y),
            Point::new(12.0, 22.0)
        );
    }

    #[test]
    fn focused_marker_frame_stays_centered_on_value_position() {
        let center = Point::new(20.0, 30.0);
        let size = marker_size(true);
        let top_left = marker_top_left(center, size);

        assert_eq!(
            Point::new(
                top_left.x + size.width / 2.0,
                top_left.y + size.height / 2.0
            ),
            center
        );
    }

    #[test]
    fn unfocused_marker_frame_includes_outer_margin() {
        assert_eq!(marker_size(false).width / 2.0, outer_marker_radius());
    }

    #[test]
    fn focused_marker_ring_stays_inside_frame() {
        assert!(focus_ring_radius() > outer_marker_radius());
        assert!(focus_ring_radius() < marker_size(true).width / 2.0);
    }
}
