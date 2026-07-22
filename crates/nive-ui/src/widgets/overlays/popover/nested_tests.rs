use iced::{
    keyboard::{self, key},
    mouse, touch, Event, Point, Size,
};

use super::*;
use crate::test_support::WidgetHarness;

#[test]
fn escape_is_owned_by_the_innermost_open_popover() {
    let mut harness = nested_harness(true);
    let child = harness
        .update_nested_overlay(escape())
        .expect("nested Popover chain");
    assert_eq!(child.messages, vec!["child-dismiss"]);
    assert!(child.captured);

    harness.replace(nested_popovers(false));
    let parent = harness
        .update_nested_overlay(escape())
        .expect("parent Popover");
    assert_eq!(parent.messages, vec!["parent-dismiss"]);
    assert!(parent.captured);
}

#[test]
fn outside_press_dismisses_only_the_innermost_level() {
    let mut harness = nested_harness(true);
    harness.set_cursor(Point::new(315.0, 195.0));

    let child = harness
        .update_nested_overlay(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .expect("nested Popover chain");
    assert_eq!(child.messages, vec!["child-dismiss"]);
    assert!(child.captured);

    harness.replace(nested_popovers(false));
    harness.set_cursor(Point::new(315.0, 195.0));
    let parent = harness
        .update_nested_overlay(Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: Point::new(315.0, 195.0),
        }))
        .expect("parent Popover");
    assert_eq!(parent.messages, vec!["parent-dismiss"]);
    assert!(parent.captured);
}

#[test]
fn passive_child_hit_region_blocks_parent_dismissal_and_click_through() {
    let mut harness = nested_harness(true);
    let bounds = harness.nested_overlay_bounds();
    assert_eq!(bounds.len(), 2);
    let child = bounds[1];
    harness.set_cursor(child.center());

    let press = harness
        .update_nested_overlay(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .expect("nested Popover chain");
    assert!(press.messages.is_empty());
    assert!(press.captured);
}

#[test]
fn trigger_reactivation_is_left_to_the_controlled_anchor() {
    let mut harness = nested_harness(false);
    harness.set_cursor(Point::new(10.0, 10.0));

    let press = harness
        .update_nested_overlay(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .expect("parent Popover");
    assert!(press.messages.is_empty());
    assert!(!press.captured);
}

fn nested_harness(child_open: bool) -> WidgetHarness<'static, &'static str> {
    WidgetHarness::new(nested_popovers(child_open), Size::new(320.0, 200.0))
}

fn nested_popovers(child_open: bool) -> crate::Element<'static, &'static str> {
    let child = Popover::new(iced::widget::Space::new().width(80).height(24))
        .content(iced::widget::Space::new().width(90).height(40))
        .open(child_open)
        .on_dismiss("child-dismiss");

    Popover::new(iced::widget::Space::new().width(60).height(24))
        .content(child)
        .open(true)
        .on_dismiss("parent-dismiss")
        .into()
}

fn escape() -> Event {
    let key = keyboard::Key::Named(key::Named::Escape);
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Code(key::Code::Escape),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat: false,
    })
}
