use iced::{keyboard::key, Point, Size};

use iced::advanced::widget::operation;

use super::*;
use crate::test_support::WidgetHarness;
use crate::widgets::controls::choice_test_support::{key_pressed, pointer_click};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Message {
    Selected(Mode),
}

fn options() -> [SegmentedOption<'static, Mode>; 3] {
    [
        SegmentedOption::new(Mode::First, "First"),
        SegmentedOption::new(Mode::Second, "Second"),
        SegmentedOption::new(Mode::Third, "Third").disabled(true),
    ]
}

#[test]
fn typed_default_and_linked_models_render() {
    let default: Element<'_, Message> =
        SegmentedControl::new("Mode", Mode::First, options()).into();
    let linked: Element<'_, Message> = SegmentedControl::new("Mode", Mode::First, options())
        .linked()
        .into();

    assert!(
        WidgetHarness::new(default, Size::new(320.0, 80.0))
            .bounds()
            .width
            > 0.0
    );
    assert!(
        WidgetHarness::new(linked, Size::new(320.0, 80.0))
            .bounds()
            .width
            > 0.0
    );
}

#[test]
fn fill_width_uses_equal_tracks_and_exact_form_height() {
    let control: Element<'_, Message> = SegmentedControl::new("Mode", Mode::First, options())
        .fill_width()
        .into();
    let harness = WidgetHarness::new(control, Size::new(301.0, 80.0));

    assert_eq!(harness.bounds().size(), Size::new(301.0, 28.0));
}

#[test]
fn invalid_models_are_finite_and_noninteractive() {
    let zero: Element<'_, Message> = SegmentedControl::new(
        "Mode",
        Mode::First,
        std::iter::empty::<SegmentedOption<'_, Mode>>(),
    )
    .fill_width()
    .on_select(Message::Selected)
    .into();
    let duplicate: Element<'_, Message> = SegmentedControl::new(
        "Mode",
        Mode::First,
        [
            SegmentedOption::new(Mode::First, "First"),
            SegmentedOption::new(Mode::First, "Duplicate"),
        ],
    )
    .on_select(Message::Selected)
    .into();
    let mut zero = WidgetHarness::new(zero, Size::new(240.0, 80.0));
    let mut duplicate = WidgetHarness::new(duplicate, Size::new(240.0, 80.0));

    assert_eq!(zero.bounds().size(), Size::new(240.0, 28.0));
    assert!(zero.focusable_ids().is_empty());
    assert!(duplicate.focusable_ids().is_empty());
}

#[test]
fn pointer_and_keyboard_publish_changed_values_once() {
    let control = || -> Element<'static, Message> {
        SegmentedControl::new("Mode", Mode::First, options())
            .id(iced::widget::Id::new("segments"))
            .fill_width()
            .on_select(Message::Selected)
            .into()
    };
    let mut pointer = WidgetHarness::new(control(), Size::new(300.0, 80.0));
    assert_eq!(
        pointer_click(&mut pointer, Point::new(150.0, 14.0)),
        [Message::Selected(Mode::Second)]
    );
    assert!(pointer.state::<SegmentedState>().focus.is_active());
    assert!(!pointer.state::<SegmentedState>().focus.is_focus_visible());

    let id = iced::widget::Id::new("segments");
    let keyboard_control: Element<'_, Message> =
        SegmentedControl::new("Mode", Mode::First, options())
            .id(id.clone())
            .on_select(Message::Selected)
            .into();
    let mut keyboard = WidgetHarness::new(keyboard_control, Size::new(300.0, 80.0));
    keyboard.focus(id);
    assert!(keyboard.state::<SegmentedState>().focus.is_focus_visible());
    assert_eq!(
        keyboard
            .update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight))
            .messages,
        [Message::Selected(Mode::Second)]
    );
    assert!(keyboard
        .update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight))
        .messages
        .is_empty());
}

#[test]
fn linked_radius_only_rounds_the_outer_corners() {
    let radius = 6.0;

    assert_eq!(
        segment_radius(SegmentedControlVariant::Linked, 0, 3, radius),
        Radius::default().left(radius)
    );
    assert_eq!(
        segment_radius(SegmentedControlVariant::Linked, 1, 3, radius),
        Radius::default()
    );
    assert_eq!(
        segment_radius(SegmentedControlVariant::Linked, 2, 3, radius),
        Radius::default().right(radius)
    );
}

#[test]
fn selected_activation_is_a_no_op() {
    let control: Element<'_, Message> = SegmentedControl::new("Mode", Mode::First, options())
        .fill_width()
        .on_select(Message::Selected)
        .into();
    let mut harness = WidgetHarness::new(control, Size::new(300.0, 80.0));

    assert!(pointer_click(&mut harness, Point::new(25.0, 14.0)).is_empty());
}

#[test]
fn two_and_five_option_counts_are_valid() {
    let two = SegmentedControl::<_, Message>::new(
        "Mode",
        Mode::First,
        [
            SegmentedOption::new(Mode::First, "First"),
            SegmentedOption::new(Mode::Second, "Second"),
        ],
    );
    let five = SegmentedControl::<_, Message>::new(
        "Mode",
        Mode::First,
        [
            SegmentedOption::new(Mode::First, "First"),
            SegmentedOption::new(Mode::Second, "Second"),
            SegmentedOption::new(Mode::Third, "Third"),
            SegmentedOption::new(Mode::Fourth, "Fourth"),
            SegmentedOption::new(Mode::Fifth, "Fifth"),
        ],
    );

    assert!(two.model_valid());
    assert!(five.model_valid());
}

#[test]
fn focused_value_reconciles_after_option_reorder() {
    let id = iced::widget::Id::new("reordered-segments");
    let initial: Element<'static, Message> = SegmentedControl::new("Mode", Mode::First, options())
        .id(id.clone())
        .on_select(Message::Selected)
        .into();
    let mut harness = WidgetHarness::new(initial, Size::new(300.0, 80.0));
    harness.focus(id.clone());
    assert_eq!(
        harness
            .update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight))
            .messages,
        [Message::Selected(Mode::Second)]
    );

    let reordered: Element<'static, Message> = SegmentedControl::new(
        "Mode",
        Mode::Second,
        [
            SegmentedOption::new(Mode::Second, "Second"),
            SegmentedOption::new(Mode::First, "First"),
            SegmentedOption::new(Mode::Third, "Third").disabled(true),
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
        [Message::Selected(Mode::First)]
    );
}

#[test]
fn keyboard_reentry_uses_current_selection_instead_of_last_clicked_item() {
    let id = iced::widget::Id::new("programmatic-selection");
    let control = |selected| -> Element<'static, Message> {
        SegmentedControl::new("Mode", selected, options())
            .id(id.clone())
            .fill_width()
            .on_select(Message::Selected)
            .into()
    };
    let mut harness = WidgetHarness::new(control(Mode::First), Size::new(300.0, 80.0));

    assert_eq!(
        pointer_click(&mut harness, Point::new(150.0, 14.0)),
        [Message::Selected(Mode::Second)]
    );
    assert_eq!(harness.state::<SegmentedState>().focused_index, Some(1));

    harness.operate(&mut operation::focusable::unfocus());
    harness.replace(control(Mode::First));
    harness.focus(id);

    let state = harness.state::<SegmentedState>();
    assert!(state.focus.is_active());
    assert!(state.focus.is_focus_visible());
    assert_eq!(state.focused_index, None);
}

#[test]
fn truncated_items_survive_draw_click_and_rebuild_lifecycle() {
    let control = |selected| -> Element<'static, Message> {
        SegmentedControl::new(
            "Constrained mode",
            selected,
            [
                SegmentedOption::new(Mode::First, "A deliberately long first mode"),
                SegmentedOption::new(Mode::Second, "A deliberately long second mode"),
            ],
        )
        .width(120)
        .on_select(Message::Selected)
        .into()
    };
    let mut harness = WidgetHarness::new(control(Mode::First), Size::new(120.0, 80.0));

    harness.set_cursor(Point::new(90.0, 14.0));
    harness.draw();
    assert_eq!(
        pointer_click(&mut harness, Point::new(90.0, 14.0)),
        [Message::Selected(Mode::Second)]
    );

    harness.replace(control(Mode::Second));
    harness.set_cursor(Point::new(30.0, 14.0));
    harness.draw();
}
