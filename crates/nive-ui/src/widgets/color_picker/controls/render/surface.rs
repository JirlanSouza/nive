use iced::{
    advanced::{graphics::geometry::Renderer as _, Renderer as _},
    widget::canvas,
    Color, Point, Rectangle, Size, Vector,
};

use super::super::{
    control_state::{ControlState, SurfaceCacheKey},
    metrics::CONTROL_RADIUS,
};
use super::coverage;

const PIXEL_SIZE: f32 = 1.0;
const COVERAGE_SAMPLES: usize = 4;

pub(super) fn draw_cached_surface(
    renderer: &mut iced::Renderer,
    state: &ControlState,
    bounds: Rectangle,
    key: SurfaceCacheKey,
    color_at: impl Fn(f32, f32, f32, f32) -> Color,
) {
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return;
    }

    let size = bounds.size();

    renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
        let geometry = state.surface_cache(key).draw(renderer, size, |frame| {
            draw_masked_pixels(frame, size, &color_at);
        });

        renderer.draw_geometry(geometry);
    });
}

fn draw_masked_pixels(
    frame: &mut canvas::Frame<iced::Renderer>,
    size: Size,
    color_at: &impl Fn(f32, f32, f32, f32) -> Color,
) {
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
            let coverage = coverage::rounded_rect_coverage(
                size,
                top_left,
                pixel_size,
                CONTROL_RADIUS,
                COVERAGE_SAMPLES,
            );

            if coverage <= 0.0 {
                continue;
            }

            let center = Point::new(x + pixel_size.width / 2.0, y + pixel_size.height / 2.0);
            let u = (center.x / size.width).clamp(0.0, 1.0);
            let v = (center.y / size.height).clamp(0.0, 1.0);
            let mut color = color_at(u, v, center.x, center.y);
            color.a = (color.a * coverage).clamp(0.0, 1.0);

            frame.fill_rectangle(top_left, pixel_size, color);
        }
    }
}
