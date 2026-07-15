use iced::{
    advanced::{renderer, Renderer},
    Background, Color, Rectangle, Shadow,
};

use crate::interaction::Orientation;
use crate::theme::BorderRole;

use super::super::helpers::{
    focus_seam_color, grip_style, seam_color, visible_grip_bounds, visual_seam_bounds,
    SplitPaneMetrics,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DividerVisualState {
    Inert,
    Idle,
    Hovered,
    Engaged,
}

pub(super) fn resolve_visual_state(
    interactive: bool,
    dragged_or_focused: bool,
    hovered: bool,
) -> DividerVisualState {
    if !interactive {
        DividerVisualState::Inert
    } else if dragged_or_focused {
        DividerVisualState::Engaged
    } else if hovered {
        DividerVisualState::Hovered
    } else {
        DividerVisualState::Idle
    }
}

pub(super) fn draw_grip(
    renderer: &mut iced::Renderer,
    theme: &crate::theme::Theme,
    bounds: Rectangle,
    orientation: Orientation,
    metrics: SplitPaneMetrics,
    state: DividerVisualState,
) {
    let seam = visual_seam_bounds(bounds, orientation, metrics);
    let seam_color = match state {
        DividerVisualState::Inert | DividerVisualState::Idle => {
            seam_color(theme, BorderRole::Subtle)
        }
        DividerVisualState::Hovered => seam_color(theme, BorderRole::Strong),
        DividerVisualState::Engaged => focus_seam_color(theme),
    };
    renderer.fill_quad(
        renderer::Quad {
            bounds: seam,
            border: iced::Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        seam_color,
    );

    if matches!(state, DividerVisualState::Inert | DividerVisualState::Idle) {
        return;
    }

    let grip = grip_style(theme);
    let visible = visible_grip_bounds(bounds, orientation, metrics);

    renderer.fill_quad(
        renderer::Quad {
            bounds: visible,
            border: grip.border,
            shadow: Shadow::default(),
            snap: true,
        },
        if matches!(state, DividerVisualState::Engaged) {
            focus_seam_color(theme)
        } else {
            background_color(grip.background)
        },
    );
}

fn background_color(background: Option<Background>) -> Color {
    match background {
        Some(Background::Color(color)) => color,
        _ => Color::TRANSPARENT,
    }
}
