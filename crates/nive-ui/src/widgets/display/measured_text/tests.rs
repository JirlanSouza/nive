use super::projection::{tooltip_source, Projection};
use super::*;
use crate::test_support::WidgetHarness;
use crate::widgets::controls::choice_test_support::pointer_move;
use iced::{Point, Size};

fn grapheme_width(value: &str) -> f32 {
    value.graphemes(true).count() as f32
}

#[test]
fn end_projection_preserves_graphemes_and_fallback_ladder() {
    assert_eq!(
        project("Ame\u{301}lie", EllipsisStrategy::End, 4.0, grapheme_width),
        Projection {
            visible: "Ame\u{301}…".to_string(),
            truncated: true,
        }
    );
    assert_eq!(
        project("long", EllipsisStrategy::End, 1.0, grapheme_width).visible,
        "…"
    );
    assert_eq!(
        project("long", EllipsisStrategy::End, 0.0, grapheme_width).visible,
        ""
    );
}

#[test]
fn middle_projection_uses_balance_then_longer_prefix_tie_break() {
    assert_eq!(
        project("abcdef", EllipsisStrategy::Middle, 5.0, grapheme_width).visible,
        "ab…ef"
    );
    assert_eq!(
        project("abcde", EllipsisStrategy::Middle, 4.0, grapheme_width).visible,
        "ab…e"
    );
}

#[test]
fn complete_and_empty_values_retain_exact_content() {
    assert_eq!(
        project("", EllipsisStrategy::End, 0.0, grapheme_width),
        Projection {
            visible: String::new(),
            truncated: false,
        }
    );
    assert_eq!(
        project("界", EllipsisStrategy::Middle, 1.0, grapheme_width),
        Projection {
            visible: "界".to_string(),
            truncated: false,
        }
    );
}

#[test]
fn unicode_cases_never_split_graphemes() {
    let cases = [
        ("e\u{301}clair", 2.0, "e\u{301}…"),
        ("👩‍💻studio", 2.0, "👩‍💻…"),
        ("界面", 3.0, "界…"),
    ];

    for (source, width, expected) in cases {
        assert_eq!(
            project(source, EllipsisStrategy::End, width, |value| {
                value
                    .graphemes(true)
                    .map(|grapheme| {
                        if grapheme.contains(['界', '面']) {
                            2.0
                        } else {
                            1.0
                        }
                    })
                    .sum()
            })
            .visible,
            expected
        );
    }
}

#[test]
fn measured_layout_state_recomputes_wide_narrow_wide_and_closes_tooltip() {
    let original = "build-2026.07.15";
    let mut state = State::default();

    let wide = project(original, EllipsisStrategy::Middle, 32.0, grapheme_width);
    update_state(&mut state, wide, Some(32.0));
    assert_eq!(state.projection, original);
    assert_eq!(tooltip_source(&state, original), None);

    let narrow = project(original, EllipsisStrategy::Middle, 7.0, grapheme_width);
    update_state(&mut state, narrow, Some(7.0));
    assert_eq!(tooltip_source(&state, original), Some(original));
    assert_ne!(state.projection, original);

    let restored = project(original, EllipsisStrategy::Middle, 32.0, grapheme_width);
    update_state(&mut state, restored, Some(32.0));
    assert_eq!(state.projection, original);
    assert_eq!(tooltip_source(&state, original), None);
}

#[test]
fn exact_fit_is_complete_but_sub_ellipsis_width_is_truncated() {
    assert!(!project("exact", EllipsisStrategy::End, 5.0, grapheme_width).truncated);
    let sub_ellipsis = project("x", EllipsisStrategy::End, 0.5, grapheme_width);
    assert!(sub_ellipsis.truncated);
    assert!(sub_ellipsis.visible.is_empty());
}

/// A window resize reaches widgets through `UserInterface::relayout`, which
/// re-runs `layout` against the retained tree **without** a `diff` pass.
/// Truncation flips the child from a bare `Text` to a `Tooltip`-wrapped one,
/// so every crossing of that threshold has to reconcile the child tree
/// inside `layout` itself — nothing else will do it.
#[test]
fn crossing_the_truncation_threshold_on_resize_keeps_the_child_tree_valid() {
    let label = || -> Element<'static, ()> {
        MeasuredText::new_inherited(
            "A deliberately long constrained label",
            EllipsisStrategy::End,
            TypographyRole::ControlStrong,
        )
        .into()
    };
    let mut harness = WidgetHarness::new(label(), Size::new(800.0, 40.0));
    harness.draw();

    for width in [40.0, 800.0, 40.0, 24.0, 800.0] {
        harness.relayout(Size::new(width, 40.0));
        harness.draw();
    }
}

#[test]
fn truncated_tooltip_keeps_one_tree_through_event_overlay_and_draw() {
    let label = || -> Element<'static, ()> {
        MeasuredText::new_inherited(
            "A deliberately long constrained label",
            EllipsisStrategy::End,
            TypographyRole::ControlStrong,
        )
        .max_width(48.0)
        .into()
    };
    let mut harness = WidgetHarness::new(label(), Size::new(48.0, 40.0));
    let point = Point::new(4.0, 8.0);

    harness.set_cursor(point);
    harness.update(pointer_move(point));
    harness.draw();
    assert!(harness.draw_overlay());

    harness.replace(label());
    harness.set_cursor(point);
    harness.update(pointer_move(point));
    harness.draw();
    assert!(harness.draw_overlay());
}
