use iced::{widget::container, Background, Border, Color, Rectangle, Shadow, Size};

use crate::interaction::Orientation;
use crate::theme::{control_metrics, BorderRole, ControlMetrics, ControlSize};

use super::{state::SnapConfig, SplitPaneConstraints};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SplitPaneMetrics {
    pub visual_thickness: f32,
    pub layout_thickness: f32,
    pub hit_thickness: f32,
    pub grip_length: f32,
    pub grip_thickness: f32,
}

pub(super) fn metrics(size: ControlSize) -> SplitPaneMetrics {
    metrics_for_control(control_metrics(size))
}

pub(super) fn metrics_for_control(control: ControlMetrics) -> SplitPaneMetrics {
    SplitPaneMetrics {
        visual_thickness: 1.0,
        layout_thickness: 1.0,
        hit_thickness: 12.0_f32.max(control.icon_size),
        grip_length: control.height,
        grip_thickness: 1.0,
    }
}

pub(super) fn normalize_ratio(ratio: f32) -> f32 {
    if ratio.is_finite() {
        ratio.clamp(0.05, 0.95)
    } else {
        0.5
    }
}

pub(super) fn normalize_minimum(minimum: f32) -> f32 {
    if minimum.is_finite() {
        minimum.max(0.0)
    } else {
        0.0
    }
}

pub(super) fn clamp_ratio(ratio: f32, constraints: SplitPaneConstraints, available: f32) -> f32 {
    let constraints = constraints.normalized();
    let ratio = normalize_ratio(ratio);
    let available = normalize_minimum(available);
    if available <= 0.0 {
        return normalize_ratio(ratio);
    }

    let min_total = constraints.leading_min + constraints.trailing_min;

    if min_total >= available && min_total > 0.0 {
        return normalize_ratio(constraints.leading_min / min_total);
    }

    ratio.clamp(
        minimum_ratio(constraints, available),
        maximum_ratio(constraints, available),
    )
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

pub(super) fn minimum_ratio(constraints: SplitPaneConstraints, available: f32) -> f32 {
    let constraints = constraints.normalized();
    let available = normalize_minimum(available);
    if available <= 0.0 {
        return 0.05;
    }

    let min_total = constraints.leading_min + constraints.trailing_min;

    if min_total >= available && min_total > 0.0 {
        return normalize_ratio(constraints.leading_min / min_total);
    }

    (constraints.leading_min / available).clamp(0.05, 0.95)
}

pub(super) fn maximum_ratio(constraints: SplitPaneConstraints, available: f32) -> f32 {
    let constraints = constraints.normalized();
    let available = normalize_minimum(available);
    if available <= 0.0 {
        return 0.95;
    }

    let min_total = constraints.leading_min + constraints.trailing_min;

    if min_total >= available && min_total > 0.0 {
        return normalize_ratio(constraints.leading_min / min_total);
    }

    (1.0 - constraints.trailing_min / available).clamp(0.05, 0.95)
}

pub(super) fn apply_snap(ratio: f32, snap: Option<&SnapConfig>, minimum: f32, maximum: f32) -> f32 {
    let Some(snap) = snap else {
        return ratio;
    };

    if snap.threshold <= 0.0 {
        return ratio;
    }

    snap.points
        .iter()
        .copied()
        .filter(|point| *point >= minimum && *point <= maximum)
        .filter_map(|point| {
            let distance = (ratio - point).abs();

            (distance <= snap.threshold).then_some((distance, point))
        })
        .min_by(|(left, _), (right, _)| left.total_cmp(right))
        .map_or(ratio, |(_, point)| point)
}

pub(super) fn visible_grip_bounds(
    divider_bounds: Rectangle,
    orientation: Orientation,
    metrics: SplitPaneMetrics,
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
    metrics: SplitPaneMetrics,
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
    metrics: SplitPaneMetrics,
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

pub(super) fn pane_sizes(
    orientation: Orientation,
    cross: f32,
    leading_length: f32,
    trailing_length: f32,
) -> (Size, Size) {
    (
        orientation.size(leading_length, cross),
        orientation.size(trailing_length, cross),
    )
}

pub(super) fn main_length(orientation: Orientation, size: Size) -> f32 {
    orientation.main_length(size)
}

pub(super) fn cross_length(orientation: Orientation, size: Size) -> f32 {
    orientation.cross_length(size)
}

#[cfg(test)]
mod split_pane_helper_tests {
    use super::*;

    #[test]
    fn minimum_ratio_uses_leading_constraint() {
        let constraints = SplitPaneConstraints::new(200.0, 100.0);

        assert_eq!(minimum_ratio(constraints, 1000.0), 0.2);
    }

    #[test]
    fn maximum_ratio_uses_trailing_constraint() {
        let constraints = SplitPaneConstraints::new(200.0, 300.0);

        assert_eq!(maximum_ratio(constraints, 1000.0), 0.7);
    }

    #[test]
    fn impossible_constraints_share_available_space_by_constraint_ratio() {
        let constraints = SplitPaneConstraints::new(300.0, 100.0);

        assert_eq!(minimum_ratio(constraints, 200.0), 0.75);
        assert_eq!(maximum_ratio(constraints, 200.0), 0.75);
        assert_eq!(clamp_ratio(0.1, constraints, 200.0), 0.75);
    }

    #[test]
    fn apply_snap_uses_near_valid_point() {
        let snap = SnapConfig::new(0.05, vec![0.25, 0.5, 0.75]);

        assert_eq!(apply_snap(0.48, Some(&snap), 0.1, 0.9), 0.5);
    }

    #[test]
    fn apply_snap_ignores_points_beyond_threshold() {
        let snap = SnapConfig::new(0.05, vec![0.25, 0.5, 0.75]);

        assert_eq!(apply_snap(0.42, Some(&snap), 0.1, 0.9), 0.42);
    }

    #[test]
    fn apply_snap_ignores_points_outside_constraints() {
        let snap = SnapConfig::new(0.05, vec![0.25]);

        assert_eq!(apply_snap(0.24, Some(&snap), 0.3, 0.9), 0.24);
    }

    #[test]
    fn invalid_constraints_and_candidates_always_resolve_finitely() {
        let constraints = SplitPaneConstraints::new(f32::NAN, f32::INFINITY);

        assert_eq!(
            constraints.normalized(),
            SplitPaneConstraints::new(0.0, 0.0)
        );
        assert_eq!(clamp_ratio(f32::NAN, constraints, 100.0), 0.5);
        assert_eq!(
            clamp_ratio(0.2, SplitPaneConstraints::new(-20.0, -1.0), 100.0),
            0.2
        );
        assert_eq!(clamp_ratio(0.7, constraints, 0.0), 0.7);
        assert_eq!(clamp_ratio(0.7, constraints, f32::NAN), 0.7);
    }

    #[test]
    fn impossible_and_zero_available_constraints_are_deterministic() {
        let constraints = SplitPaneConstraints::new(300.0, 100.0);

        assert_eq!(clamp_ratio(0.1, constraints, 200.0), 0.75);
        assert_eq!(clamp_ratio(0.9, constraints, 200.0), 0.75);
        assert_eq!(clamp_ratio(0.0, constraints, 0.0), 0.05);
    }
}
