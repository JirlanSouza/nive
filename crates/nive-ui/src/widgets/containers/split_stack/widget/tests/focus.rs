use iced::{keyboard::key::Named, mouse, Event, Size};

use crate::interaction::Orientation;

use super::super::super::SplitSizing;
use super::support::{Harness, Message};

const VIEWPORT: Size = Size::new(1400.0, 600.0);

fn stack_of(panes: usize) -> Harness {
    let sizing = (0..panes)
        .map(|index| {
            if index == panes - 1 {
                SplitSizing::Fill
            } else {
                SplitSizing::Fixed(200.0)
            }
        })
        .collect();

    Harness::new(sizing, vec![80.0; panes], VIEWPORT)
}

fn focus_divider(harness: &mut Harness, divider: usize) {
    let point = harness.divider_hit_point(divider);
    let _ = harness.update(Event::Mouse(mouse::Event::CursorMoved { position: point }));
    let _ = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
}

#[test]
fn pane_count_does_not_change_the_number_of_focus_targets() {
    for panes in 2..=4 {
        let mut harness = stack_of(panes);

        assert_eq!(
            harness.focusable_bounds().len(),
            1,
            "{panes} panes registered more than one focus target"
        );
    }
}

#[test]
fn the_focus_target_follows_the_roving_divider() {
    let mut harness = stack_of(4);
    focus_divider(&mut harness, 0);
    let first = harness.focusable_bounds();

    let _ = harness.press_key(Named::ArrowDown);
    let second = harness.focusable_bounds();

    assert_eq!(harness.state().focused_divider, 1);
    assert_eq!(second.len(), 1);
    assert_ne!(first[0], second[0], "the focus target did not move");
}

#[test]
fn cross_axis_arrows_rove_without_resizing() {
    let mut harness = stack_of(4);
    focus_divider(&mut harness, 0);

    let messages = harness.press_key(Named::ArrowDown);

    assert_eq!(harness.state().focused_divider, 1);
    assert!(
        !messages.iter().any(|m| matches!(m, Message::Resize(_))),
        "roving emitted a resize"
    );
}

#[test]
fn roving_clamps_at_both_ends() {
    let mut harness = stack_of(4);
    focus_divider(&mut harness, 0);

    let _ = harness.press_key(Named::ArrowUp);
    assert_eq!(harness.state().focused_divider, 0);

    let _ = harness.press_key(Named::End);
    assert_eq!(harness.state().focused_divider, 2);

    let _ = harness.press_key(Named::ArrowDown);
    assert_eq!(harness.state().focused_divider, 2);

    let _ = harness.press_key(Named::Home);
    assert_eq!(harness.state().focused_divider, 0);
}

#[test]
fn main_axis_arrows_resize_the_focused_divider() {
    let mut harness = stack_of(4);
    focus_divider(&mut harness, 1);

    let messages = harness.press_key(Named::ArrowRight);
    let resize = messages
        .iter()
        .find_map(|message| match message {
            Message::Resize(resize) => Some(*resize),
            Message::Collapse(_) | Message::Pane(_) => None,
        })
        .expect("arrow key emitted no resize");

    assert_eq!(resize.divider, 1);
    assert!(resize.leading > 200.0, "the divider did not move forward");
}

#[test]
fn a_locked_stack_registers_no_focus_target() {
    let mut harness = Harness::configured(
        Orientation::Horizontal,
        vec![SplitSizing::Fixed(200.0), SplitSizing::Fill],
        vec![80.0, 80.0],
        VIEWPORT,
        true,
        true,
    );

    assert!(harness.focusable_bounds().is_empty());
}
