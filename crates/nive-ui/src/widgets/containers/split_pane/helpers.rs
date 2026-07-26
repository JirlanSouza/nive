use iced::Size;

use crate::interaction::Orientation;

use super::super::split_divider;
use super::state::SnapConfig;
use super::SplitPaneConstraints;

pub(super) use split_divider::{
    draw_grip, hit_bounds, metrics, resize_interaction, resolve_visual_state, SplitDividerMetrics,
};
#[cfg(test)]
pub(super) use split_divider::{
    focus_seam_color, metrics_for_control, visible_grip_bounds, visual_seam_bounds,
    DividerVisualState,
};

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
