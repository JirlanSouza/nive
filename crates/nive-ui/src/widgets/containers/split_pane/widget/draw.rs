use iced::{
    advanced::{renderer, Renderer},
    Background, Color, Rectangle, Shadow,
};

use crate::interaction::Orientation;
use crate::theme::SurfaceRole;

use super::super::helpers::{grip_style, handle_style, metrics, visible_grip_bounds};

pub(super) fn draw_grip(
    renderer: &mut iced::Renderer,
    theme: &crate::theme::Theme,
    bounds: Rectangle,
    orientation: Orientation,
    role: SurfaceRole,
) {
    let handle = handle_style(theme, role);

    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border: handle.border,
            shadow: handle.shadow,
            snap: true,
        },
        background_color(handle.background),
    );

    let grip = grip_style(theme);
    let visible = visible_grip_bounds(bounds, orientation, metrics());

    renderer.fill_quad(
        renderer::Quad {
            bounds: visible,
            border: grip.border,
            shadow: Shadow::default(),
            snap: true,
        },
        background_color(grip.background),
    );
}

fn background_color(background: Option<Background>) -> Color {
    match background {
        Some(Background::Color(color)) => color,
        _ => Color::TRANSPARENT,
    }
}
