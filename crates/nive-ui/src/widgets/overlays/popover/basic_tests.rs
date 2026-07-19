use super::*;
use crate::test_support::{event_probe, WidgetHarness};
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
fn public_policy_builders_preserve_every_variant() {
    let placements = [
        PopoverPlacement::TopStart,
        PopoverPlacement::TopCenter,
        PopoverPlacement::TopEnd,
        PopoverPlacement::RightStart,
        PopoverPlacement::RightCenter,
        PopoverPlacement::RightEnd,
        PopoverPlacement::BottomStart,
        PopoverPlacement::BottomCenter,
        PopoverPlacement::BottomEnd,
        PopoverPlacement::LeftStart,
        PopoverPlacement::LeftCenter,
        PopoverPlacement::LeftEnd,
    ];
    for placement in placements {
        assert_eq!(empty_popover().placement(placement).placement, placement);
    }

    for collision in [
        PopoverCollision::None,
        PopoverCollision::Shift,
        PopoverCollision::Flip,
        PopoverCollision::FlipAndShift,
    ] {
        assert_eq!(empty_popover().collision(collision).collision, collision);
    }

    for inset in [
        PopoverInset::Standard,
        PopoverInset::Compact,
        PopoverInset::EdgeToEdge,
    ] {
        assert_eq!(empty_popover().inset(inset).inset, inset);
    }

    for focus_policy in [
        PopoverFocusPolicy::RetainAnchor,
        PopoverFocusPolicy::FocusFirst,
        PopoverFocusPolicy::Trap,
    ] {
        assert_eq!(
            empty_popover().focus_policy(focus_policy).focus_policy,
            focus_policy
        );
    }
}

#[test]
fn popover_surface_resolves_from_custom_theme() {
    use crate::theme::{ThemeBuilder, ThemeMode};

    let theme = ThemeBuilder::new("Popover test", ThemeMode::Dark)
        .app_background(iced::Color::from_rgb8(12, 18, 24))
        .build();
    let style = surface_style(&theme);

    assert!(style.background.is_some());
    assert!(style.text_color.is_some());
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

#[test]
fn leaf_activation_relays_once_and_controlled_close_is_silent() {
    let anchor = iced::widget::Space::new().width(40).height(24);
    let popover = Popover::new(anchor)
        .content(event_probe("leaf"))
        .open(true)
        .on_dismiss("dismiss");
    let mut harness = WidgetHarness::new(popover.into(), Size::new(320.0, 200.0));
    let overlay = harness.overlay_bounds().expect("open Popover overlay");
    harness.set_cursor(overlay.center());

    let activated = harness
        .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .expect("open Popover overlay");

    assert_eq!(activated.messages, vec!["leaf"]);
    assert!(activated.captured);

    let closed = Popover::new(iced::widget::Space::new().width(40).height(24))
        .content(event_probe("leaf"))
        .open(false)
        .on_dismiss("dismiss");
    harness.replace(closed.into());

    assert!(!harness.has_overlay());
}

#[test]
fn programmatic_open_and_close_publish_no_lifecycle_messages() {
    let closed = Popover::new(iced::widget::Space::new().width(40).height(24))
        .content(event_probe("content"))
        .open(false)
        .on_dismiss("dismiss");
    let mut harness = WidgetHarness::new(closed.into(), Size::new(320.0, 200.0));
    assert!(!harness.has_overlay());

    let opened = Popover::new(iced::widget::Space::new().width(40).height(24))
        .content(event_probe("content"))
        .open(true)
        .on_dismiss("dismiss");
    harness.replace(opened.into());
    assert!(harness.has_overlay());

    let closed_again = Popover::new(iced::widget::Space::new().width(40).height(24))
        .content(event_probe("content"))
        .open(false)
        .on_dismiss("dismiss");
    harness.replace(closed_again.into());
    assert!(!harness.has_overlay());
}

#[test]
fn wide_narrow_wide_relayout_stays_finite_and_recovers_width() {
    let popover: Popover<'static, ()> =
        Popover::new(iced::widget::Space::new().width(40).height(24))
            .content(iced::widget::Space::new().width(260).height(80))
            .width_px(280.0)
            .open(true);
    let mut harness = WidgetHarness::new(popover.into(), Size::new(480.0, 260.0));

    let wide = harness.overlay_bounds().expect("wide overlay");
    harness.relayout(Size::new(120.0, 140.0));
    let narrow = harness.overlay_bounds().expect("narrow overlay");
    harness.relayout(Size::new(480.0, 260.0));
    let restored = harness.overlay_bounds().expect("restored overlay");

    assert!(narrow.width.is_finite() && narrow.height.is_finite());
    assert!(narrow.width >= 0.0 && narrow.height >= 0.0);
    assert!(narrow.width <= 104.0);
    assert_eq!(restored.width, wide.width);
    assert_eq!(restored.height, wide.height);
}

#[test]
fn low_height_overlay_is_bounded_by_safe_viewport() {
    let popover: Popover<'static, ()> =
        Popover::new(iced::widget::Space::new().width(40).height(24))
            .content(iced::widget::Space::new().width(180).height(600))
            .open(true);
    let mut harness = WidgetHarness::new(popover.into(), Size::new(240.0, 90.0));

    let bounds = harness.overlay_bounds().expect("bounded overlay");

    assert!(bounds.height.is_finite());
    assert!(bounds.height <= 74.0);
    assert!(bounds.y >= 8.0);
    assert!(bounds.y + bounds.height <= 82.0);
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
