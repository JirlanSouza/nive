mod color_math;
mod coverage;
mod marker;
mod surface;

use iced::{Color, Rectangle};

use super::control_state::{ControlState, SurfaceCacheKey};

use self::{
    color_math::{alpha_surface_color, hue_surface_color, saturation_value_color},
    surface::draw_cached_surface,
};

pub(super) use self::marker::{bounded_marker_center, draw_marker};

pub(super) fn draw_saturation_value_surface(
    renderer: &mut iced::Renderer,
    state: &ControlState,
    bounds: Rectangle,
    hue: f32,
) {
    draw_cached_surface(
        renderer,
        state,
        bounds,
        SurfaceCacheKey::saturation_value(hue),
        |u, v, _x, _y| saturation_value_color(hue, u, v),
    );
}

pub(super) fn draw_hue_surface(
    renderer: &mut iced::Renderer,
    state: &ControlState,
    bounds: Rectangle,
) {
    draw_cached_surface(
        renderer,
        state,
        bounds,
        SurfaceCacheKey::Hue,
        |_u, v, _x, _y| hue_surface_color(v),
    );
}

pub(super) fn draw_alpha_surface(
    renderer: &mut iced::Renderer,
    state: &ControlState,
    bounds: Rectangle,
    color: Color,
) {
    draw_cached_surface(
        renderer,
        state,
        bounds,
        SurfaceCacheKey::alpha(color),
        |_u, v, x, y| alpha_surface_color(color, v, x, y),
    );
}
