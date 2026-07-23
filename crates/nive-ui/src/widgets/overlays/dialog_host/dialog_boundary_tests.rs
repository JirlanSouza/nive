use super::{DialogHost, DialogInitialFocus};
use crate::{test_support::WidgetHarness, widgets::button, Element};
use iced::{keyboard, mouse, touch, Event, Point, Size};

const VIEWPORT: Size = Size::new(400.0, 300.0);

fn harness_with_dialog(
    on_backdrop: Option<&'static str>,
    on_escape: Option<&'static str>,
) -> WidgetHarness<'static, &'static str> {
    let base = button::primary("Base action")
        .id(iced::widget::Id::new("base-action"))
        .on_press("base-pressed");
    let dialog_content = button::primary("Dialog action")
        .id(iced::widget::Id::new("dialog-action"))
        .on_press("dialog-pressed");

    let host = DialogHost::new(base).dialog(
        dialog_content,
        on_backdrop,
        on_escape,
        DialogInitialFocus::default(),
    );
    WidgetHarness::new(Element::from(host), VIEWPORT)
}

fn escape(repeat: bool) -> Event {
    let key = keyboard::Key::Named(keyboard::key::Named::Escape);
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Escape),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat,
    })
}

#[test]
fn primary_press_outside_the_dialog_publishes_backdrop_exactly_once() {
    let mut harness = harness_with_dialog(Some("backdrop"), Some("escape"));
    harness.set_cursor(Point::new(5.0, 5.0));

    let result = harness
        .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .expect("open dialog overlay");

    assert_eq!(result.messages, vec!["backdrop"]);
    assert!(result.captured);
}

#[test]
fn touch_press_outside_the_dialog_publishes_backdrop() {
    let mut harness = harness_with_dialog(Some("backdrop"), None);
    harness.set_cursor(Point::new(5.0, 5.0));

    let result = harness
        .update_overlay(Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(0),
            position: Point::new(5.0, 5.0),
        }))
        .expect("open dialog overlay");

    assert_eq!(result.messages, vec!["backdrop"]);
}

#[test]
fn secondary_press_outside_the_dialog_does_not_dismiss() {
    let mut harness = harness_with_dialog(Some("backdrop"), None);
    harness.set_cursor(Point::new(5.0, 5.0));

    let result = harness
        .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Right,
        )))
        .expect("open dialog overlay");

    assert!(result.messages.is_empty());
    assert!(result.captured, "outside input must still be consumed");
}

#[test]
fn press_inside_the_dialog_does_not_dismiss() {
    let mut harness = harness_with_dialog(Some("backdrop"), None);
    let bounds = harness.overlay_bounds().expect("open dialog overlay");
    harness.set_cursor(Point::new(bounds.center_x(), bounds.center_y()));

    let result = harness
        .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .expect("open dialog overlay");

    assert!(result.messages.is_empty());
}

#[test]
fn outside_press_without_a_configured_backdrop_route_is_still_consumed_silently() {
    let mut harness = harness_with_dialog(None, None);
    harness.set_cursor(Point::new(5.0, 5.0));

    let result = harness
        .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .expect("open dialog overlay");

    assert!(result.messages.is_empty());
    assert!(result.captured);
}

#[test]
fn escape_publishes_the_configured_message_exactly_once() {
    let mut harness = harness_with_dialog(None, Some("escape"));

    let result = harness
        .update_overlay(escape(false))
        .expect("open dialog overlay");

    assert_eq!(result.messages, vec!["escape"]);
}

#[test]
fn repeated_escape_publishes_nothing() {
    let mut harness = harness_with_dialog(None, Some("escape"));

    let result = harness
        .update_overlay(escape(true))
        .expect("open dialog overlay");

    assert!(result.messages.is_empty());
}

#[test]
fn escape_without_a_configured_route_publishes_nothing_but_is_captured() {
    let mut harness = harness_with_dialog(None, None);

    let result = harness
        .update_overlay(escape(false))
        .expect("open dialog overlay");

    assert!(result.messages.is_empty());
    assert!(result.captured);
}

#[test]
fn base_content_is_inert_to_pointer_input_while_a_dialog_is_open() {
    let mut harness = harness_with_dialog(None, None);
    let base_bounds = harness.bounds();
    harness.set_cursor(Point::new(base_bounds.x + 5.0, base_bounds.y + 5.0));

    let pressed = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    let released = harness.update(Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    )));

    assert!(pressed.messages.is_empty());
    assert!(released.messages.is_empty());
}

#[test]
fn external_focus_operation_does_not_reach_base_content_while_open() {
    let base_action = iced::widget::Id::new("base-action");
    let mut harness = harness_with_dialog(None, None);

    harness.focus(base_action.clone());

    assert!(!harness
        .managed_focus()
        .entries
        .iter()
        .any(|entry| entry.id.as_ref() == Some(&base_action) && entry.active));
}

#[test]
fn dialog_content_still_activates_normally() {
    let mut harness = harness_with_dialog(None, None);
    let bounds = harness.overlay_bounds().expect("open dialog overlay");
    harness.set_cursor(Point::new(bounds.center_x(), bounds.center_y()));

    harness
        .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .expect("open dialog overlay");
    let released = harness
        .update_overlay(Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        )))
        .expect("open dialog overlay");

    assert_eq!(released.messages, vec!["dialog-pressed"]);
}

// --- 2.9: safe-viewport-margin clamping through the real host pipeline
// ---
//
// `DialogOverlay`'s own `safe_max_size` arithmetic and `Dialog`'s own
// Limits-clamping are each unit-tested in isolation (`dialog_host_tests`
// and `dialog_widget_tests`); these two prove the two compose correctly
// through `DialogHost` with a real header/body/footer Dialog rather than
// a single button. Fixed header/footer sizing and body-only overflow are
// already covered by `dialog_widget_tests`
// (`header_and_footer_are_positioned_before_and_after_the_body`,
// `total_height_never_exceeds_the_limits_max_height`) and are not
// duplicated here.

fn harness_with_tall_dialog(viewport: Size) -> WidgetHarness<'static, &'static str> {
    use crate::widgets::Dialog;

    let base = button::primary("Base action")
        .id(iced::widget::Id::new("base-action"))
        .on_press("base-pressed");
    let long_body =
        iced::widget::column((0..200).map(|i| iced::widget::text(format!("Line {i}")).into()));
    let dialog = Dialog::<&'static str>::new(long_body)
        .header(iced::widget::text("Title"))
        .footer(iced::widget::text("Footer"));

    let host = DialogHost::new(base).dialog(dialog, None, None, DialogInitialFocus::default());
    WidgetHarness::new(Element::from(host), viewport)
}

#[test]
fn dialog_clamps_to_the_safe_margin_and_height_cap_on_a_narrow_low_viewport() {
    let viewport = Size::new(300.0, 500.0);
    let mut harness = harness_with_tall_dialog(viewport);

    let bounds = harness.overlay_bounds().expect("open dialog overlay");

    // Width: safe margin only (720px Lg target would need it, but this
    // is the default Sm target at 420px, still wider than the
    // 300 - 32 = 268px safe cap).
    assert!(bounds.width <= viewport.width - 32.0 + f32::EPSILON);
    // Height: min(0.80 * 500, 500 - 32) = min(400, 468) = 400, not the
    // long body's natural (uncapped) height.
    assert!(bounds.height <= 400.0 + f32::EPSILON);
    // Centered within the viewport, never off-screen.
    assert!(bounds.x >= 0.0 && bounds.y >= 0.0);
    assert!(bounds.x + bounds.width <= viewport.width + f32::EPSILON);
    assert!(bounds.y + bounds.height <= viewport.height + f32::EPSILON);
}

#[test]
fn dialog_layout_stays_non_negative_below_the_safe_margin() {
    // Smaller than 2 * SAFE_VIEWPORT_MARGIN (32px) on both axes: the
    // safe area collapses to zero rather than going negative.
    let viewport = Size::new(20.0, 20.0);
    let mut harness = harness_with_tall_dialog(viewport);

    let bounds = harness.overlay_bounds().expect("open dialog overlay");

    assert!(bounds.width >= 0.0);
    assert!(bounds.height >= 0.0);
    assert!(bounds.x >= 0.0 && bounds.y >= 0.0);
}
