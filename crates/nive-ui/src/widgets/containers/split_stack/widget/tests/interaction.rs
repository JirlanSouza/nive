use iced::{mouse, Event, Size};

use crate::interaction::Orientation;

use super::super::super::SplitSizing;
use super::support::{Harness, Message};

const VIEWPORT: Size = Size::new(1400.0, 600.0);

fn sides() -> Vec<SplitSizing> {
    vec![
        SplitSizing::Fixed(280.0),
        SplitSizing::Fill,
        SplitSizing::Fixed(320.0),
    ]
}

fn minimums() -> Vec<f32> {
    vec![160.0, 240.0, 160.0]
}

fn harness() -> Harness {
    Harness::new(sides(), minimums(), VIEWPORT)
}

fn resizes(messages: &[Message]) -> Vec<super::super::super::SplitResize> {
    messages
        .iter()
        .filter_map(|message| match message {
            Message::Resize(resize) => Some(*resize),
            Message::Collapse(_) | Message::Pane(_) => None,
        })
        .collect()
}

#[test]
fn layout_interleaves_panes_and_dividers() {
    let harness = harness();

    assert_eq!(harness.lengths(), vec![280.0, 798.0, 320.0]);
    // Two one-pixel dividers sit between the three panes.
    assert_eq!(harness.divider_bounds(0).width, 1.0);
    assert_eq!(harness.divider_bounds(1).width, 1.0);
    assert_eq!(
        harness.pane_bounds(0).x + harness.pane_bounds(0).width,
        harness.divider_bounds(0).x
    );
    assert_eq!(harness.divider_bounds(0).x + 1.0, harness.pane_bounds(1).x);
}

#[test]
fn dragging_a_divider_leaves_every_non_adjacent_pane_untouched() {
    for divider in 0..2 {
        let untouched = if divider == 0 { 2 } else { 0 };
        let mut moved_at_least_once = false;

        for step in 0..=60 {
            let delta = -600.0 + 20.0 * step as f32;
            let mut harness = harness();
            let before = harness.pane_bounds(untouched);
            let adjacent_before = harness.lengths()[divider];

            let messages = harness.drag(divider, delta);
            let proposals = resizes(&messages);
            assert!(
                !proposals.is_empty(),
                "divider {divider} emitted nothing at delta {delta}"
            );

            for resize in proposals {
                assert_eq!(resize.divider, divider);
                harness.apply(resize);
            }

            assert_eq!(
                harness.pane_bounds(untouched),
                before,
                "divider {divider} at delta {delta} moved pane {untouched}"
            );
            moved_at_least_once |= harness.lengths()[divider] != adjacent_before;
        }

        assert!(
            moved_at_least_once,
            "divider {divider} never actually resized its own pane"
        );
    }
}

#[test]
fn a_divider_stops_at_the_neighbour_minimum_without_spilling() {
    let mut harness = harness();

    for resize in resizes(&harness.drag(0, 5_000.0)) {
        harness.apply(resize);
    }

    let lengths = harness.lengths();
    assert_eq!(lengths[1], 240.0, "centre did not stop at its minimum");
    assert_eq!(lengths[2], 320.0, "right pane gave way to a distant drag");
    assert_eq!(lengths.iter().sum::<f32>(), 1398.0);
}

#[test]
fn a_drag_started_on_one_divider_ignores_the_other() {
    let mut harness = harness();
    let start = harness.divider_hit_point(0);
    let _ = harness.update(Event::Mouse(mouse::Event::CursorMoved { position: start }));
    let _ = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));

    // Cross the threshold, then move the pointer far past the second divider.
    let anchor = iced::Point::new(start.x + 8.0, start.y);
    let _ = harness.update(Event::Mouse(mouse::Event::CursorMoved { position: anchor }));
    let far = iced::Point::new(anchor.x + 900.0, anchor.y);
    let messages = harness.update(Event::Mouse(mouse::Event::CursorMoved { position: far }));

    let resizes = resizes(&messages);
    assert!(!resizes.is_empty(), "the drag emitted nothing");
    assert!(
        resizes.iter().all(|resize| resize.divider == 0),
        "the gesture jumped to another divider: {resizes:?}"
    );
}

#[test]
fn growing_the_container_grows_only_the_filling_pane() {
    let mut harness = harness();
    assert_eq!(harness.lengths(), vec![280.0, 798.0, 320.0]);

    harness.resize_container(Size::new(1920.0, 600.0));

    assert_eq!(harness.lengths(), vec![280.0, 1318.0, 320.0]);
}

#[test]
fn the_hit_target_is_wider_than_the_one_pixel_seam() {
    let mut harness = harness();
    let divider = harness.divider_bounds(0);
    let hit = harness.hit_bounds(0);

    assert_eq!(divider.width, 1.0);
    assert!(
        hit.width > divider.width,
        "hit target did not widen: {hit:?}"
    );

    // A point inside the hit target but outside the seam still resizes.
    let point = iced::Point::new(hit.x + 1.0, divider.center_y());
    assert!(!divider.contains(point));
    let _ = harness.update(Event::Mouse(mouse::Event::CursorMoved { position: point }));

    assert_eq!(
        harness.mouse_interaction(),
        mouse::Interaction::ResizingColumn
    );
}

#[test]
fn a_locked_or_display_only_stack_claims_no_gesture() {
    for (locked, with_callback) in [(true, true), (false, false)] {
        let mut harness = Harness::configured(
            Orientation::Horizontal,
            sides(),
            minimums(),
            VIEWPORT,
            locked,
            with_callback,
        );

        assert!(resizes(&harness.drag(0, 200.0)).is_empty());
        assert_eq!(harness.mouse_interaction(), mouse::Interaction::None);
    }
}

#[test]
fn a_vertical_stack_resizes_along_its_own_axis() {
    let mut harness = Harness::configured(
        Orientation::Vertical,
        vec![SplitSizing::Fill, SplitSizing::Fixed(200.0)],
        vec![100.0, 80.0],
        Size::new(600.0, 900.0),
        false,
        true,
    );

    for resize in resizes(&harness.drag(0, -120.0)) {
        harness.apply(resize);
    }

    assert_eq!(harness.lengths(), vec![579.0, 320.0]);
}
