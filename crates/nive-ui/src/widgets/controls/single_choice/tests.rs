use iced::{keyboard::key, touch, Point};

use super::*;
use crate::test_support::WidgetHarness;
use crate::widgets::controls::choice_test_support::{
    key_pressed, key_released, pointer_click, pointer_move, pointer_press, pointer_release,
    touch_lift, touch_press,
};

fn checkbox(message: Option<&'static str>) -> Element<'static, &'static str> {
    SingleChoice::new(
        SingleChoiceKind::Checkbox,
        SingleChoiceLayout::Leading,
        Cow::Borrowed("Choice"),
        ChoicePersistentState::Unselected,
    )
    .on_activate(message)
    .into()
}

#[test]
fn pointer_touch_and_space_activate_once() {
    let mut pointer = WidgetHarness::new(checkbox(Some("toggle")), Size::new(240.0, 80.0));
    assert_eq!(
        pointer_click(&mut pointer, Point::new(8.0, 8.0)),
        ["toggle"]
    );
    assert!(pointer.state::<SingleChoiceState>().focus.is_active());
    assert!(!pointer
        .state::<SingleChoiceState>()
        .focus
        .is_focus_visible());

    let mut touch = WidgetHarness::new(checkbox(Some("toggle")), Size::new(240.0, 80.0));
    assert!(touch.update(touch_press(1, Point::new(8.0, 8.0))).captured);
    assert_eq!(
        touch.update(touch_lift(1, Point::new(8.0, 8.0))).messages,
        ["toggle"]
    );

    let id = widget::Id::new("choice");
    let choice: Element<'_, &'static str> = SingleChoice::new(
        SingleChoiceKind::Checkbox,
        SingleChoiceLayout::Leading,
        Cow::Borrowed("Choice"),
        ChoicePersistentState::Unselected,
    )
    .id(Some(id.clone()))
    .on_activate(Some("toggle"))
    .into();
    let mut keyboard = WidgetHarness::new(choice, Size::new(240.0, 80.0));
    keyboard.focus(id);
    assert!(keyboard
        .state::<SingleChoiceState>()
        .focus
        .is_focus_visible());
    assert!(keyboard
        .update(key_pressed(key::Named::Space, key::Code::Space))
        .messages
        .is_empty());
    assert_eq!(
        keyboard
            .update(key_released(key::Named::Space, key::Code::Space))
            .messages,
        ["toggle"]
    );
}

#[test]
fn release_outside_and_lost_touch_cancel_activation() {
    let mut pointer = WidgetHarness::new(checkbox(Some("toggle")), Size::new(240.0, 80.0));
    pointer.set_cursor(Point::new(8.0, 8.0));
    pointer.update(pointer_press());
    pointer.update(pointer_move(Point::new(220.0, 70.0)));
    pointer.set_cursor(Point::new(220.0, 70.0));
    assert!(pointer.update(pointer_release()).messages.is_empty());

    let mut touch = WidgetHarness::new(checkbox(Some("toggle")), Size::new(240.0, 80.0));
    touch.update(touch_press(7, Point::new(8.0, 8.0)));
    assert!(touch
        .update(Event::Touch(touch::Event::FingerLost {
            id: touch::Finger(7),
            position: Point::new(220.0, 70.0),
        }))
        .messages
        .is_empty());
}

#[test]
fn display_only_and_disabled_have_no_focus_or_activation() {
    let mut display = WidgetHarness::new(checkbox(None), Size::new(240.0, 80.0));
    assert_eq!(
        display.focused_count(),
        operation::focusable::Count::default()
    );
    assert!(pointer_click(&mut display, Point::new(8.0, 8.0)).is_empty());

    let disabled: Element<'_, &'static str> = SingleChoice::new(
        SingleChoiceKind::Switch,
        SingleChoiceLayout::Leading,
        Cow::Borrowed("Switch"),
        ChoicePersistentState::Selected,
    )
    .disabled(true)
    .on_activate(Some("toggle"))
    .into();
    let mut disabled = WidgetHarness::new(disabled, Size::new(240.0, 80.0));
    assert_eq!(
        disabled.focused_count(),
        operation::focusable::Count::default()
    );
    assert!(pointer_click(&mut disabled, Point::new(8.0, 8.0)).is_empty());
}
