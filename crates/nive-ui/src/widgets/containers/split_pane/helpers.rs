use iced::{widget::container, Background, Border, Color, Rectangle, Shadow, Size};

use crate::interaction::Orientation;
use crate::theme::{self, BorderRole, SpaceStep, SurfaceRole};

use super::{state::SnapConfig, SplitPaneConstraints};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SplitPaneMetrics {
    pub handle_size: f32,
    pub grip_length: f32,
    pub grip_thickness: f32,
}

pub(super) fn metrics() -> SplitPaneMetrics {
    SplitPaneMetrics {
        handle_size: theme::space(SpaceStep::Md),
        grip_length: theme::space(SpaceStep::Xxl),
        grip_thickness: 1.0,
    }
}

pub(super) fn normalize_ratio(ratio: f32) -> f32 {
    ratio.clamp(0.05, 0.95)
}

pub(super) fn clamp_ratio(ratio: f32, constraints: SplitPaneConstraints, available: f32) -> f32 {
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

pub(super) fn handle_style(theme: &crate::theme::Theme, role: SurfaceRole) -> container::Style {
    let surface = theme.surface(role);

    container::Style {
        text_color: Some(surface.foreground),
        background: Some(Background::Color(surface.background)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        shadow: Shadow::default(),
        ..container::Style::default()
    }
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

pub(super) fn minimum_ratio(constraints: SplitPaneConstraints, available: f32) -> f32 {
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
    handle_bounds: Rectangle,
    orientation: Orientation,
    metrics: SplitPaneMetrics,
) -> Rectangle {
    match orientation {
        Orientation::Horizontal => {
            let height = metrics.grip_length.min(handle_bounds.height);

            Rectangle {
                x: handle_bounds.center_x() - metrics.grip_thickness / 2.0,
                y: handle_bounds.center_y() - height / 2.0,
                width: metrics.grip_thickness,
                height,
            }
        }
        Orientation::Vertical => {
            let width = metrics.grip_length.min(handle_bounds.width);

            Rectangle {
                x: handle_bounds.center_x() - width / 2.0,
                y: handle_bounds.center_y() - metrics.grip_thickness / 2.0,
                width,
                height: metrics.grip_thickness,
            }
        }
    }
}

pub(super) fn pane_sizes(
    orientation: Orientation,
    cross: f32,
    leading_length: f32,
    trailing_length: f32,
) -> (Size, Size) {
    match orientation {
        Orientation::Horizontal => (
            Size::new(leading_length, cross),
            Size::new(trailing_length, cross),
        ),
        Orientation::Vertical => (
            Size::new(cross, leading_length),
            Size::new(cross, trailing_length),
        ),
    }
}

pub(super) fn main_length(orientation: Orientation, size: Size) -> f32 {
    match orientation {
        Orientation::Horizontal => size.width,
        Orientation::Vertical => size.height,
    }
}

pub(super) fn cross_length(orientation: Orientation, size: Size) -> f32 {
    match orientation {
        Orientation::Horizontal => size.height,
        Orientation::Vertical => size.width,
    }
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
}
