use iced::{widget::Space, Point, Rectangle, Size};

use crate::interaction::Orientation;
use crate::theme::{BorderRole, ControlSize, Theme, ThemeDensity, ThemeMode};

use super::super::super::helpers::{
    focus_seam_color, metrics, metrics_for_control, visible_grip_bounds, visual_seam_bounds,
};
use super::super::super::SplitPane;
use super::support::{Harness, ORIGIN};

const CONTROL_SIZES: [ControlSize; 4] = [
    ControlSize::Xs,
    ControlSize::Sm,
    ControlSize::Md,
    ControlSize::Lg,
];

fn pane() -> SplitPane<'static, ()> {
    SplitPane::new(Space::new(), Space::new())
}

#[test]
fn defaults_to_small_and_exposes_standard_size_builders() {
    assert_eq!(pane().size, ControlSize::Sm);
    assert_eq!(pane().xs().size, ControlSize::Xs);
    assert_eq!(pane().sm().size, ControlSize::Sm);
    assert_eq!(pane().md().size, ControlSize::Md);
    assert_eq!(pane().lg().size, ControlSize::Lg);
    assert_eq!(pane().size(ControlSize::Lg).size, ControlSize::Lg);
}

#[test]
fn metrics_resolve_one_pixel_geometry_and_control_derived_interaction() {
    for density in ThemeDensity::ALL {
        let theme = Theme::builder("Split pane metrics", ThemeMode::Dark)
            .density(density)
            .build();

        for size in CONTROL_SIZES {
            let metrics = metrics_for_control(theme.control_metrics(size));
            let control = theme.control_metrics(size);

            assert_eq!(metrics.visual_thickness, 1.0);
            assert_eq!(metrics.layout_thickness, 1.0);
            assert_eq!(metrics.grip_thickness, 1.0);
            assert_eq!(metrics.grip_length, control.height);
            assert_eq!(metrics.hit_thickness, 12.0_f32.max(control.icon_size));
        }
    }
}

#[test]
fn both_orientations_reserve_and_render_one_pixel_dividers() {
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let harness = Harness::new(
            orientation,
            ControlSize::Sm,
            Size::new(200.0, 120.0),
            0.5,
            false,
        );
        let divider = harness.divider_bounds();
        let leading = harness.child_bounds(0);
        let trailing = harness.child_bounds(2);
        let metrics = harness.metrics();
        let seam = visual_seam_bounds(divider, orientation, metrics);
        let grip = visible_grip_bounds(divider, orientation, metrics);

        match orientation {
            Orientation::Horizontal => {
                assert_eq!(divider.width, 1.0);
                assert_eq!(trailing.x - (leading.x + leading.width), 1.0);
                assert_eq!(seam.width, 1.0);
                assert_eq!(grip.width, 1.0);
                assert_eq!(grip.height, metrics.grip_length);
            }
            Orientation::Vertical => {
                assert_eq!(divider.height, 1.0);
                assert_eq!(trailing.y - (leading.y + leading.height), 1.0);
                assert_eq!(seam.height, 1.0);
                assert_eq!(grip.height, 1.0);
                assert_eq!(grip.width, metrics.grip_length);
            }
        }

        assert_eq!(seam, divider);
    }
}

#[test]
fn runtime_divider_bounds_include_parent_translation() {
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let harness = Harness::new(
            orientation,
            ControlSize::Sm,
            Size::new(200.0, 120.0),
            0.5,
            false,
        );
        let local = harness.local_divider_bounds();
        let translated = harness.divider_bounds();

        assert_eq!(translated.x, local.x + ORIGIN.x);
        assert_eq!(translated.y, local.y + ORIGIN.y);
        assert_eq!(translated.size(), local.size());
    }
}

#[test]
fn hit_bounds_are_centered_on_the_divider_and_clipped_to_the_split_pane() {
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let harness = Harness::new(
            orientation,
            ControlSize::Sm,
            Size::new(200.0, 120.0),
            0.5,
            false,
        );
        let divider = harness.divider_bounds();
        let hit = harness.hit_bounds();
        let metrics = harness.metrics();

        match orientation {
            Orientation::Horizontal => {
                assert_eq!(hit.center_x(), divider.center_x());
                assert_eq!(hit.width, metrics.hit_thickness);
                assert_eq!(hit.y, divider.y);
                assert_eq!(hit.height, divider.height);
            }
            Orientation::Vertical => {
                assert_eq!(hit.center_y(), divider.center_y());
                assert_eq!(hit.height, metrics.hit_thickness);
                assert_eq!(hit.x, divider.x);
                assert_eq!(hit.width, divider.width);
            }
        }

        let bounds = harness.bounds();
        assert!(hit.x >= bounds.x);
        assert!(hit.y >= bounds.y);
        assert!(hit.x + hit.width <= bounds.x + bounds.width);
        assert!(hit.y + hit.height <= bounds.y + bounds.height);
    }

    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let size = match orientation {
            Orientation::Horizontal => Size::new(10.0, 40.0),
            Orientation::Vertical => Size::new(40.0, 10.0),
        };
        let harness = Harness::new(orientation, ControlSize::Sm, size, 0.05, false);
        let hit = harness.hit_bounds();
        let bounds = harness.bounds();

        match orientation {
            Orientation::Horizontal => {
                assert_eq!(hit.x, bounds.x);
                assert!(hit.width < harness.metrics().hit_thickness);
            }
            Orientation::Vertical => {
                assert_eq!(hit.y, bounds.y);
                assert!(hit.height < harness.metrics().hit_thickness);
            }
        }
    }
}

#[test]
fn focusable_bounds_match_the_current_clipped_hit_rectangle() {
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let mut harness = Harness::new(
            orientation,
            ControlSize::Lg,
            Size::new(200.0, 120.0),
            0.5,
            false,
        );

        assert_eq!(harness.focusable_bounds(), vec![harness.hit_bounds()]);
    }
}

#[test]
fn hit_target_size_does_not_change_layout_or_constraint_space() {
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let xs = Harness::new(
            orientation,
            ControlSize::Xs,
            Size::new(200.0, 120.0),
            0.5,
            false,
        );
        let lg = Harness::new(
            orientation,
            ControlSize::Lg,
            Size::new(200.0, 120.0),
            0.5,
            false,
        );

        assert_eq!(xs.state().available_length, lg.state().available_length);
        assert_eq!(xs.divider_bounds(), lg.divider_bounds());
        assert_ne!(xs.metrics().hit_thickness, lg.metrics().hit_thickness);

        let main_length = match orientation {
            Orientation::Horizontal => xs.bounds().width,
            Orientation::Vertical => xs.bounds().height,
        };
        assert_eq!(xs.state().available_length, main_length - 1.0);
    }
}

#[test]
fn focused_indicator_uses_the_one_pixel_seam_and_semantic_focus_color() {
    let theme = Theme::Dark;
    let divider = Rectangle::new(Point::new(20.0, 30.0), Size::new(1.0, 80.0));
    let metrics = metrics(ControlSize::Sm);
    let seam = visual_seam_bounds(divider, Orientation::Horizontal, metrics);
    let hit = Rectangle::new(Point::new(14.5, 30.0), Size::new(12.0, 80.0));

    assert_eq!(seam, divider);
    assert_ne!(seam, hit);
    assert_eq!(
        focus_seam_color(&theme),
        theme.border(BorderRole::Focus).color
    );
}
