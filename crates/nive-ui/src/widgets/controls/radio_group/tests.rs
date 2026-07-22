use iced::{keyboard::key, Point, Size};

use super::*;
use crate::test_support::WidgetHarness;
use crate::widgets::controls::choice_test_support::{key_pressed, pointer_click};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    First,
    Second,
    Third,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Message {
    Selected(Choice),
}

fn options() -> [RadioOption<'static, Choice>; 3] {
    [
        RadioOption::new(Choice::First, "First"),
        RadioOption::new(Choice::Second, "Second").description("A longer description"),
        RadioOption::new(Choice::Third, "Third").disabled(true),
    ]
}

#[test]
fn generic_values_none_and_explicit_none_render() {
    let unselected: Element<'_, Message> = RadioGroup::new("Choice", None, options()).into();
    let explicit_none: Element<'_, Message> = RadioGroup::new(
        "Choice",
        Some(Choice::None),
        [RadioOption::new(Choice::None, "No preference")],
    )
    .into();

    assert!(
        WidgetHarness::new(unselected, Size::new(320.0, 240.0))
            .bounds()
            .height
            > 0.0
    );
    assert!(
        WidgetHarness::new(explicit_none, Size::new(320.0, 240.0))
            .bounds()
            .height
            > 0.0
    );
}

#[test]
fn vertical_and_horizontal_wrap_are_finite() {
    let vertical: Element<'_, Message> = RadioGroup::new("Choice", None, options()).into();
    let wrapped: Element<'_, Message> = RadioGroup::new("Choice", None, options())
        .layout(RadioGroupLayout::HorizontalWrap)
        .into();
    let vertical = WidgetHarness::new(vertical, Size::new(160.0, 400.0));
    let wrapped = WidgetHarness::new(wrapped, Size::new(160.0, 400.0));

    assert!(vertical.bounds().size().width.is_finite());
    assert!(wrapped.bounds().size().height.is_finite());
}

#[test]
fn duplicate_values_fall_back_to_display_only() {
    let duplicate: Element<'_, Message> = RadioGroup::new(
        "Duplicate",
        None,
        [
            RadioOption::new(Choice::First, "First"),
            RadioOption::new(Choice::First, "Again"),
        ],
    )
    .on_select(Message::Selected)
    .into();
    let mut harness = WidgetHarness::new(duplicate, Size::new(320.0, 240.0));

    assert!(harness.focusable_ids().is_empty());
    assert!(pointer_click(&mut harness, Point::new(8.0, 60.0)).is_empty());
}

#[test]
fn group_has_one_focus_entry_and_arrows_skip_disabled_options() {
    let id = widget::Id::new("radio-group");
    let group: Element<'_, Message> = RadioGroup::new("Choice", Some(Choice::First), options())
        .id(id.clone())
        .on_select(Message::Selected)
        .into();
    let mut harness = WidgetHarness::new(group, Size::new(320.0, 240.0));

    assert_eq!(harness.focusable_ids(), std::slice::from_ref(&id));
    harness.focus(id);
    assert!(harness
        .state_at::<RadioGroupState>(&[1])
        .focus
        .is_focus_visible());
    assert_eq!(
        harness
            .update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight))
            .messages,
        [Message::Selected(Choice::Second)]
    );
    assert!(harness
        .update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight))
        .messages
        .is_empty());
}

#[test]
fn selected_option_activation_is_a_no_op() {
    let group: Element<'_, Message> = RadioGroup::new("Choice", Some(Choice::First), options())
        .on_select(Message::Selected)
        .into();
    let mut harness = WidgetHarness::new(group, Size::new(320.0, 240.0));

    assert!(pointer_click(&mut harness, Point::new(8.0, 60.0)).is_empty());
    assert!(harness.state_at::<RadioGroupState>(&[1]).focus.is_active());
    assert!(!harness
        .state_at::<RadioGroupState>(&[1])
        .focus
        .is_focus_visible());
}

#[test]
fn focused_value_reconciles_after_option_reorder() {
    let id = widget::Id::new("reordered-radio-group");
    let initial: Element<'static, Message> =
        RadioGroup::new("Choice", Some(Choice::First), options())
            .id(id.clone())
            .on_select(Message::Selected)
            .into();
    let mut harness = WidgetHarness::new(initial, Size::new(320.0, 240.0));
    harness.focus(id.clone());
    assert_eq!(
        harness
            .update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight))
            .messages,
        [Message::Selected(Choice::Second)]
    );

    let reordered: Element<'static, Message> = RadioGroup::new(
        "Choice",
        Some(Choice::Second),
        [
            RadioOption::new(Choice::Second, "Second"),
            RadioOption::new(Choice::First, "First"),
            RadioOption::new(Choice::Third, "Third").disabled(true),
        ],
    )
    .id(id)
    .on_select(Message::Selected)
    .into();
    harness.replace(reordered);

    assert_eq!(
        harness
            .update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight))
            .messages,
        [Message::Selected(Choice::First)]
    );
}
