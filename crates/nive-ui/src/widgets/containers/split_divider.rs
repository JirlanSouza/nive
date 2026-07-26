use iced::{
    advanced::{mouse, renderer, Renderer},
    widget::container,
    Background, Border, Color, Rectangle, Shadow,
};

use crate::interaction::Orientation;
use crate::theme::{control_metrics, BorderRole, ControlMetrics, ControlSize};

/// Pixel metrics shared by every splitter divider.
///
/// The visual and layout seam stays one logical pixel at every control size;
/// only the resize hit target and grip length scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SplitDividerMetrics {
    pub visual_thickness: f32,
    pub layout_thickness: f32,
    pub hit_thickness: f32,
    pub grip_length: f32,
    pub grip_thickness: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DividerVisualState {
    Inert,
    Idle,
    Hovered,
    Engaged,
}

pub(super) fn metrics(size: ControlSize) -> SplitDividerMetrics {
    metrics_for_control(control_metrics(size))
}

pub(super) fn metrics_for_control(control: ControlMetrics) -> SplitDividerMetrics {
    SplitDividerMetrics {
        visual_thickness: 1.0,
        layout_thickness: 1.0,
        hit_thickness: 12.0_f32.max(control.icon_size),
        grip_length: control.height,
        grip_thickness: 1.0,
    }
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

pub(super) fn resize_interaction(orientation: Orientation) -> mouse::Interaction {
    match orientation {
        Orientation::Horizontal => mouse::Interaction::ResizingColumn,
        Orientation::Vertical => mouse::Interaction::ResizingRow,
    }
}

pub(super) fn visible_grip_bounds(
    divider_bounds: Rectangle,
    orientation: Orientation,
    metrics: SplitDividerMetrics,
) -> Rectangle {
    match orientation {
        Orientation::Horizontal => {
            let height = metrics.grip_length.min(divider_bounds.height);

            Rectangle {
                x: divider_bounds.center_x() - metrics.grip_thickness / 2.0,
                y: divider_bounds.center_y() - height / 2.0,
                width: metrics.grip_thickness,
                height,
            }
        }
        Orientation::Vertical => {
            let width = metrics.grip_length.min(divider_bounds.width);

            Rectangle {
                x: divider_bounds.center_x() - width / 2.0,
                y: divider_bounds.center_y() - metrics.grip_thickness / 2.0,
                width,
                height: metrics.grip_thickness,
            }
        }
    }
}

pub(super) fn visual_seam_bounds(
    divider_bounds: Rectangle,
    orientation: Orientation,
    metrics: SplitDividerMetrics,
) -> Rectangle {
    match orientation {
        Orientation::Horizontal => Rectangle {
            x: divider_bounds.center_x() - metrics.visual_thickness / 2.0,
            y: divider_bounds.y,
            width: metrics.visual_thickness,
            height: divider_bounds.height,
        },
        Orientation::Vertical => Rectangle {
            x: divider_bounds.x,
            y: divider_bounds.center_y() - metrics.visual_thickness / 2.0,
            width: divider_bounds.width,
            height: metrics.visual_thickness,
        },
    }
}

pub(super) fn hit_bounds(
    divider_bounds: Rectangle,
    container_bounds: Rectangle,
    orientation: Orientation,
    metrics: SplitDividerMetrics,
) -> Rectangle {
    let expanded = match orientation {
        Orientation::Horizontal => Rectangle {
            x: divider_bounds.center_x() - metrics.hit_thickness / 2.0,
            y: divider_bounds.y,
            width: metrics.hit_thickness,
            height: divider_bounds.height,
        },
        Orientation::Vertical => Rectangle {
            x: divider_bounds.x,
            y: divider_bounds.center_y() - metrics.hit_thickness / 2.0,
            width: divider_bounds.width,
            height: metrics.hit_thickness,
        },
    };

    expanded
        .intersection(&container_bounds)
        .unwrap_or(Rectangle::default())
}

pub(super) fn draw_grip(
    renderer: &mut iced::Renderer,
    theme: &crate::theme::Theme,
    bounds: Rectangle,
    orientation: Orientation,
    metrics: SplitDividerMetrics,
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
            border: Border::default(),
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

pub(super) fn grip_style(theme: &crate::theme::Theme) -> container::Style {
    let border = theme.border(BorderRole::Strong);

    container::Style {
        text_color: None,
        background: Some(Background::Color(border.color)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 1.0.into(),
        },
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

pub(super) fn focus_seam_color(theme: &crate::theme::Theme) -> Color {
    theme.border(BorderRole::Focus).color
}

pub(super) fn seam_color(theme: &crate::theme::Theme, role: BorderRole) -> Color {
    theme.border(role).color
}

fn background_color(background: Option<Background>) -> Color {
    match background {
        Some(Background::Color(color)) => color,
        _ => Color::TRANSPARENT,
    }
}
