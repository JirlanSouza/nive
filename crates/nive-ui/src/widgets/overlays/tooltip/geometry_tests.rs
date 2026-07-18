use std::time::Duration;

use iced::{
    advanced::widget::operation,
    keyboard::{self, key},
    widget::{column, container, Id},
    Event, Point, Size,
};

use super::*;
use crate::{
    test_support::WidgetHarness,
    widgets::{button, Input},
};

#[test]
fn every_preferred_side_keeps_the_four_pixel_gap() {
    let start = iced::time::Instant::now();
    for placement in [
        TooltipPlacement::Top,
        TooltipPlacement::Right,
        TooltipPlacement::Bottom,
        TooltipPlacement::Left,
    ] {
        let tooltip: Element<'_, ()> = container(
            Tooltip::new(iced::widget::Space::new().width(20).height(20), "Help")
                .placement(placement)
                .delay(Duration::ZERO)
                .at(start)
                .intent(true, false),
        )
        .padding(100)
        .into();
        let mut harness = WidgetHarness::new(tooltip, Size::new(400.0, 300.0));
        harness.update(redraw(start));
        let bounds = harness.overlay_bounds().expect("visible Tooltip");

        match placement {
            TooltipPlacement::Top => assert_eq!(bounds.y + bounds.height, 96.0),
            TooltipPlacement::Right => assert_eq!(bounds.x, 124.0),
            TooltipPlacement::Bottom => assert_eq!(bounds.y, 124.0),
            TooltipPlacement::Left => assert_eq!(bounds.x + bounds.width, 96.0),
        }
    }
}

#[test]
fn long_text_wraps_within_the_tooltip_and_safe_viewport_caps() {
    let start = iced::time::Instant::now();
    let tooltip: Element<'_, ()> = Tooltip::new(
        iced::widget::Space::new().width(20).height(20),
        "A deliberately long tooltip label that must wrap instead of widening beyond its compact desktop disclosure limit.",
    )
    .delay(Duration::ZERO)
    .at(start)
    .intent(true, false)
    .into();
    let mut harness = WidgetHarness::new(tooltip, Size::new(240.0, 180.0));
    harness.update(redraw(start));
    let bounds = harness.overlay_bounds().expect("visible Tooltip");

    assert!(bounds.width <= 224.0);
    assert!(bounds.height > 24.0);
}

#[test]
fn focus_reveals_and_escape_suppresses_until_intent_leaves() {
    let start = iced::time::Instant::now();
    let id = Id::unique();
    let tooltip: Element<'_, ()> = Tooltip::new(Input::new("Anchor", "").id(id.clone()), "Help")
        .delay(Duration::ZERO)
        .at(start)
        .into();
    let mut harness = WidgetHarness::new(tooltip, Size::new(320.0, 120.0));
    harness.focus(id);
    harness.update(redraw(start));
    assert_eq!(visible_keys(&mut harness).len(), 1);

    harness.update(key_pressed(key::Named::Escape, key::Code::Escape));
    assert!(visible_keys(&mut harness).is_empty());
}

#[test]
fn disabled_anchor_remains_pointer_explainable_without_focusability() {
    let start = iced::time::Instant::now();
    let anchor = button::secondary("Unavailable").on_press(()).disabled(true);
    let tooltip: Element<'_, ()> = Tooltip::new(anchor, "Not available in this state")
        .delay(Duration::ZERO)
        .at(start)
        .into();
    let mut harness = WidgetHarness::new(tooltip, Size::new(320.0, 120.0));
    harness.set_cursor(Point::new(10.0, 10.0));
    harness.update(redraw(start));

    assert_eq!(harness.focused_widgets(), 0);
    assert_eq!(visible_keys(&mut harness).len(), 1);
}

#[test]
fn collision_flips_and_shifts_inside_the_safe_viewport() {
    let start = iced::time::Instant::now();
    let tooltip: Element<'_, ()> = column![
        iced::widget::Space::new().height(160),
        Tooltip::new(
            iced::widget::Space::new().width(20).height(20),
            "A wide tooltip that must remain inside the safe viewport",
        )
        .delay(Duration::ZERO)
        .at(start)
        .intent(true, false),
    ]
    .into();
    let mut harness = WidgetHarness::new(tooltip, Size::new(220.0, 200.0));
    harness.update(redraw(start));
    let bounds = harness.overlay_bounds().expect("visible Tooltip");

    assert!(bounds.y + bounds.height <= 156.0);
    assert!(bounds.x >= 8.0);
    assert!(bounds.x + bounds.width <= 212.0);
}

#[test]
fn pointer_leave_closes_without_intercepting_anchor_events() {
    let start = iced::time::Instant::now();
    let tooltip: Element<'_, ()> = Tooltip::new(
        iced::widget::Space::new().width(20).height(20),
        "Pointer help",
    )
    .delay(Duration::ZERO)
    .at(start)
    .into();
    let mut harness = WidgetHarness::new(tooltip, Size::new(320.0, 120.0));
    harness.set_cursor(Point::new(10.0, 10.0));
    let entered = harness.update(redraw(start));
    assert!(!entered.captured);
    assert_eq!(visible_keys(&mut harness).len(), 1);

    harness.set_cursor(Point::new(100.0, 100.0));
    let left = harness.update(redraw(start + Duration::from_millis(1)));
    assert!(!left.captured);
    assert!(visible_keys(&mut harness).is_empty());
}

fn redraw(now: iced::time::Instant) -> Event {
    Event::Window(iced::window::Event::RedrawRequested(now))
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

fn visible_keys(harness: &mut WidgetHarness<'_, ()>) -> Vec<u64> {
    struct VisibleKeys {
        index: u64,
        visible: Vec<u64>,
    }

    impl operation::Operation for VisibleKeys {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
            operate(self);
        }

        fn custom(
            &mut self,
            _id: Option<&iced::advanced::widget::Id>,
            _bounds: iced::Rectangle,
            state: &mut dyn std::any::Any,
        ) {
            if let Some(state) = state.downcast_ref::<widget::TooltipState>() {
                self.index += 1;
                if state.visible {
                    self.visible.push(self.index);
                }
            }
        }
    }

    let mut keys = VisibleKeys {
        index: 0,
        visible: Vec::new(),
    };
    harness.operate(&mut keys);
    keys.visible
}

#[test]
fn real_pointer_intent_is_observed_without_capturing_activation() {
    let start = iced::time::Instant::now();
    let tooltip: Element<'_, ()> = Tooltip::new(
        iced::widget::Space::new().width(20).height(20),
        "Pointer help",
    )
    .delay(Duration::ZERO)
    .at(start)
    .into();
    let mut harness = WidgetHarness::new(tooltip, Size::new(320.0, 120.0));
    harness.set_cursor(Point::new(10.0, 10.0));
    let result = harness.update(redraw(start));

    assert!(!result.captured);
    assert_eq!(visible_keys(&mut harness).len(), 1);
}
