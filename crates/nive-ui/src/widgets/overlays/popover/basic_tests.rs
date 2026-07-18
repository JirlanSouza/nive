use super::*;
use crate::test_support::WidgetHarness;
use iced::{
    keyboard::{self, key},
    mouse, touch, Event, Point, Size,
};

#[test]
fn width_shortcuts_map_to_popover_width_variants() {
    assert_eq!(
        empty_popover().width_px(240.0).width,
        PopoverWidth::Fixed(240.0)
    );
    assert_eq!(
        empty_popover().match_anchor_width().width,
        PopoverWidth::MatchAnchor
    );
    assert_eq!(
        empty_popover().at_least_anchor_width().width,
        PopoverWidth::AtLeastAnchor
    );
    assert_eq!(empty_popover().content_width().width, PopoverWidth::Content);
}

#[test]
fn defaults_own_standard_surface_geometry_and_retain_anchor_focus() {
    let popover = empty_popover();

    assert_eq!(popover.gap, 4.0);
    assert_eq!(popover.inset, PopoverInset::Standard);
    assert_eq!(popover.focus_policy, PopoverFocusPolicy::RetainAnchor);
}

#[test]
fn inset_values_are_exact() {
    assert_eq!(PopoverInset::Standard.value(), 12.0);
    assert_eq!(PopoverInset::Compact.value(), 8.0);
    assert_eq!(PopoverInset::EdgeToEdge.value(), 0.0);
}

#[test]
fn surface_style_owns_one_pixel_eight_pixel_radius_frame() {
    let style = surface_style(&crate::theme::Theme::Dark);

    assert_eq!(style.border.width, 1.0);
    assert_eq!(style.border.radius, Radius::new(8.0));
}

#[test]
fn escape_requests_one_controlled_dismissal() {
    let mut harness = popover_harness(Some("dismiss"));
    let result = harness
        .update_overlay(key_pressed(key::Named::Escape, key::Code::Escape))
        .expect("open Popover overlay");

    assert_eq!(result.messages, vec!["dismiss"]);
    assert!(result.captured);

    let repeated = harness
        .update_overlay(key_pressed(key::Named::Escape, key::Code::Escape))
        .expect("open Popover overlay");
    assert!(repeated.messages.is_empty());
    assert!(repeated.captured);
}

#[test]
fn outside_mouse_and_touch_request_one_dismissal() {
    let mut mouse_harness = popover_harness(Some("mouse"));
    mouse_harness.set_cursor(Point::new(300.0, 180.0));
    let mouse = mouse_harness
        .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .expect("open Popover overlay");
    assert_eq!(mouse.messages, vec!["mouse"]);
    assert!(mouse.captured);

    let mut touch_harness = popover_harness(Some("touch"));
    let touch = touch_harness
        .update_overlay(Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(1),
            position: Point::new(300.0, 180.0),
        }))
        .expect("open Popover overlay");
    assert_eq!(touch.messages, vec!["touch"]);
    assert!(touch.captured);
}

#[test]
fn callback_absence_does_not_capture_escape_or_outside_press() {
    let mut harness = popover_harness(None);
    let escape = harness
        .update_overlay(key_pressed(key::Named::Escape, key::Code::Escape))
        .expect("open Popover overlay");
    assert!(escape.messages.is_empty());
    assert!(!escape.captured);

    harness.set_cursor(Point::new(300.0, 180.0));
    let outside = harness
        .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .expect("open Popover overlay");
    assert!(outside.messages.is_empty());
    assert!(!outside.captured);
}

fn popover_harness(message: Option<&'static str>) -> WidgetHarness<'static, &'static str> {
    let anchor = iced::widget::Space::new().width(40).height(24);
    let content = iced::widget::Space::new().width(120).height(60);
    let popover = Popover::new(anchor)
        .content(content)
        .open(true)
        .on_dismiss_maybe(message);
    WidgetHarness::new(popover.into(), Size::new(320.0, 200.0))
}

fn key_pressed(named: key::Named, code: key::Code) -> Event {
    let key = keyboard::Key::Named(named);
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: key::Physical::Code(code),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat: false,
    })
}

fn empty_popover() -> Popover<'static, ()> {
    Popover::new(iced::widget::Space::new())
}
