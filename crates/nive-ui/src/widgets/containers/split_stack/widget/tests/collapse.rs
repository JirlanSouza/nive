use iced::{mouse, Event, Size};

use super::super::super::{SplitCollapse, SplitSizing};
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

/// Sides collapsible, centre not — the workbench arrangement.
fn harness() -> Harness {
    Harness::new(sides(), minimums(), VIEWPORT).collapsible(vec![true, false, true], None)
}

fn collapses(messages: &[Message]) -> Vec<SplitCollapse> {
    messages
        .iter()
        .filter_map(|message| match message {
            Message::Collapse(collapse) => Some(*collapse),
            Message::Resize(_) | Message::Pane(_) => None,
        })
        .collect()
}

#[test]
fn dragging_past_the_minimum_proposes_a_collapse_with_the_pre_drag_length() {
    let mut harness = harness();
    // The left pane has 120 of slack above its 160 minimum, so -120 lands on it
    // and the next 32 are over-travel.
    let messages = harness.drag(0, -160.0);
    let collapses = collapses(&messages);

    assert_eq!(
        collapses,
        vec![SplitCollapse {
            divider: 0,
            pane: 0,
            restore: 280.0,
        }]
    );
}

#[test]
fn the_trailing_side_collapses_from_its_own_divider() {
    let mut harness = harness();
    // Divider 1 borders the centre and the right pane; the right pane has 160
    // of slack, so +192 clears the threshold.
    let messages = harness.drag(1, 192.0);

    assert_eq!(
        collapses(&messages),
        vec![SplitCollapse {
            divider: 1,
            pane: 2,
            restore: 320.0,
        }]
    );
}

#[test]
fn stopping_short_of_the_threshold_proposes_nothing() {
    // Exactly at the minimum, and then just under the 32px threshold.
    for delta in [-120.0, -151.0] {
        let mut stack = harness();

        assert!(
            collapses(&stack.drag(0, delta)).is_empty(),
            "delta {delta} proposed a collapse"
        );
    }
}

#[test]
fn a_collapse_is_proposed_once_per_drag() {
    let mut harness = harness();
    let anchor = harness.begin_drag(0);

    let first = collapses(&harness.move_by(anchor, -200.0));
    let second = collapses(&harness.move_by(anchor, -400.0));
    let third = collapses(&harness.move_by(anchor, -800.0));

    assert_eq!(first.len(), 1);
    assert!(
        second.is_empty() && third.is_empty(),
        "the collapse repeated: {second:?} {third:?}"
    );
}

#[test]
fn a_pane_that_is_not_collapsible_never_collapses() {
    // Only the centre is marked, and no divider can push the centre outward.
    let centre_only =
        || Harness::new(sides(), minimums(), VIEWPORT).collapsible(vec![false, true, false], None);

    assert!(collapses(&centre_only().drag(0, -600.0)).is_empty());
    assert!(collapses(&centre_only().drag(1, 600.0)).is_empty());
}

#[test]
fn without_the_callback_no_drag_collapses() {
    let mut stack = Harness::new(sides(), minimums(), VIEWPORT);

    assert!(collapses(&stack.drag(0, -600.0)).is_empty());
}

#[test]
fn the_threshold_is_configurable() {
    let tight = || {
        Harness::new(sides(), minimums(), VIEWPORT).collapsible(vec![true, false, true], Some(4.0))
    };

    // A 4px threshold collapses far earlier than the 32px default would.
    assert!(collapses(&tight().drag(0, -121.0)).is_empty());
    assert_eq!(collapses(&tight().drag(0, -126.0)).len(), 1);
}

#[test]
fn a_drag_survives_leaving_and_re_entering_the_window() {
    let mut harness = harness();
    let anchor = harness.begin_drag(0);

    let _ = harness.move_by(anchor, -40.0);
    assert!(harness.state().drag.is_some(), "the drag never started");

    let left = harness.cursor_left();
    assert!(
        !left
            .iter()
            .any(|message| matches!(message, Message::Resize(_) | Message::Collapse(_))),
        "leaving the window proposed a change: {left:?}"
    );
    assert!(
        harness.state().drag.is_some(),
        "leaving the window dropped the drag"
    );

    let _ = harness.cursor_entered();
    let resumed = harness.move_by(anchor, -80.0);

    assert!(
        resumed
            .iter()
            .any(|message| matches!(message, Message::Resize(resize) if resize.divider == 0)),
        "the drag did not resume: {resumed:?}"
    );
}

#[test]
fn a_press_with_no_cursor_position_does_not_end_the_drag() {
    let mut harness = harness();
    let anchor = harness.begin_drag(0);
    let _ = harness.move_by(anchor, -40.0);

    // The pointer is outside the window, so its position is unresolvable.
    let _ = harness.cursor_left();
    harness.clear_cursor();
    let _ = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));

    assert!(
        harness.state().drag.is_some(),
        "an unresolvable press ended the drag"
    );
}
