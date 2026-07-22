use iced::{
    advanced::mouse,
    keyboard::{self, key, key::Named},
    widget::Space,
    window, Event, Point, Rectangle, Size,
};

use super::helpers::{move_highlight, option_bounds, typeahead_match};
use super::*;
use crate::{
    test_support::WidgetHarness,
    widgets::controls::{choice_test_support::key_pressed, Select},
    widgets::navigation::menu::{MENU_LIST_INSET, MENU_ROW_HEIGHT},
};

fn options() -> Vec<SelectOption<'static, u8>> {
    vec![
        SelectOption::new(1, "Alpha"),
        SelectOption::new(2, "Beta").disabled(true),
        SelectOption::new(3, "Bravo"),
    ]
}

#[test]
fn bounded_navigation_skips_disabled_options() {
    let options = options();

    assert_eq!(move_highlight(&options, Some(0), 1, true), Some(2));
    assert_eq!(move_highlight(&options, Some(2), 1, true), Some(2));
    assert_eq!(move_highlight(&options, Some(2), -1, true), Some(0));
    assert_eq!(move_highlight(&options, Some(0), -1, true), Some(0));
}

#[test]
fn typeahead_wraps_only_its_search_pass() {
    let options = options();

    assert_eq!(typeahead_match(&options, Some(2), "a", true), Some(0));
    assert_eq!(typeahead_match(&options, Some(0), "br", true), Some(2));
    assert_eq!(typeahead_match(&options, Some(0), "be", true), None);
}

#[test]
fn row_geometry_has_one_four_pixel_list_inset() {
    let bounds = Rectangle::new(Point::new(10.0, 20.0), Size::new(200.0, 92.0));

    assert_eq!(
        option_bounds(bounds, 1, 3),
        Some(Rectangle::new(
            Point::new(14.0, 52.0),
            Size::new(192.0, 28.0)
        ))
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    Opened,
    Selected(u8),
    Closed,
}

fn interactive_select(selected: Option<u8>, id: Id) -> Element<'static, Message> {
    Select::new(options(), selected)
        .id(id)
        .on_select(Message::Selected)
        .on_open(Message::Opened)
        .on_close(Message::Closed)
        .into()
}

#[test]
fn closed_keyboard_activation_opens_without_committing_and_has_one_focus_target() {
    for (named, code) in [
        (Named::Enter, key::Code::Enter),
        (Named::Space, key::Code::Space),
        (Named::ArrowDown, key::Code::ArrowDown),
        (Named::ArrowUp, key::Code::ArrowUp),
    ] {
        let id = Id::unique();
        let mut harness = WidgetHarness::new(
            interactive_select(Some(1), id.clone()),
            Size::new(240.0, 160.0),
        );
        assert_eq!(harness.focusable_ids(), vec![id.clone()]);
        harness.focus(id);

        let result = harness.update(key_pressed(named, code));

        assert_eq!(result.messages, vec![Message::Opened]);
        assert!(result.captured);
        assert!(harness.state_at::<SelectState>(&[0]).open);
        assert!(harness.has_overlay());
    }
}

#[test]
fn popup_navigation_is_bounded_skips_disabled_and_commits_before_close() {
    let id = Id::unique();
    let mut harness = WidgetHarness::new(
        interactive_select(Some(1), id.clone()),
        Size::new(240.0, 160.0),
    );
    harness.focus(id);
    harness.update(key_pressed(Named::Enter, key::Code::Enter));

    let moved = harness
        .update_overlay(key_pressed(Named::ArrowDown, key::Code::ArrowDown))
        .expect("open Select overlay");
    assert!(moved.messages.is_empty());
    let committed = harness
        .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
        .expect("open Select overlay");

    assert_eq!(
        committed.messages,
        vec![Message::Selected(3), Message::Closed]
    );
    assert!(!harness.state_at::<SelectState>(&[0]).open);
}

#[test]
fn committing_the_current_value_closes_without_republishing_selection() {
    let id = Id::unique();
    let mut harness = WidgetHarness::new(
        interactive_select(Some(1), id.clone()),
        Size::new(240.0, 160.0),
    );
    harness.focus(id);
    harness.update(key_pressed(Named::Enter, key::Code::Enter));

    let result = harness
        .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
        .expect("open Select overlay");

    assert_eq!(result.messages, vec![Message::Closed]);
}

#[test]
fn escape_closes_and_tab_closes_without_capturing_traversal() {
    for (named, code, captured) in [
        (Named::Escape, key::Code::Escape, true),
        (Named::Tab, key::Code::Tab, false),
    ] {
        let id = Id::unique();
        let mut harness = WidgetHarness::new(
            interactive_select(Some(1), id.clone()),
            Size::new(240.0, 160.0),
        );
        harness.focus(id);
        harness.update(key_pressed(Named::Enter, key::Code::Enter));

        let result = harness
            .update_overlay(key_pressed(named, code))
            .expect("open Select overlay");

        assert_eq!(result.messages, vec![Message::Closed]);
        assert_eq!(result.captured, captured);
        assert!(!harness.state_at::<SelectState>(&[0]).open);
    }
}

#[test]
fn popup_is_at_least_as_wide_as_its_trigger() {
    let id = Id::unique();
    let select: Element<'static, Message> = Select::new(options(), Some(1))
        .id(id.clone())
        .width(Length::Fixed(180.0))
        .on_select(Message::Selected)
        .on_open(Message::Opened)
        .on_close(Message::Closed)
        .into();
    let mut harness = WidgetHarness::new(select, Size::new(240.0, 160.0));
    harness.focus(id);
    harness.update(key_pressed(Named::Enter, key::Code::Enter));

    let popup = harness.overlay_bounds().expect("open Select overlay");

    assert!(popup.width >= harness.bounds().width);
    assert!(popup.x >= 8.0);
    assert!(popup.x + popup.width <= 232.0);
}

#[test]
fn disabling_an_existing_select_clears_focus_and_open_ownership() {
    let id = Id::unique();
    let mut harness = WidgetHarness::new(
        interactive_select(Some(1), id.clone()),
        Size::new(240.0, 160.0),
    );
    harness.focus(id.clone());
    harness.update(key_pressed(Named::Enter, key::Code::Enter));
    assert!(harness.state_at::<SelectState>(&[0]).open);

    let disabled: Element<'static, Message> = Select::new(options(), Some(1))
        .id(id)
        .disabled(true)
        .on_select(Message::Selected)
        .into();
    harness.replace(disabled);

    assert!(!harness.state_at::<SelectState>(&[0]).open);
    assert_eq!(harness.focused_widgets(), 0);
    assert!(harness.focusable_ids().is_empty());
}

#[test]
fn pointer_and_touch_commit_the_option_before_the_close_notification() {
    for touch_input in [false, true] {
        let id = Id::unique();
        let mut harness = WidgetHarness::new(
            interactive_select(Some(1), id.clone()),
            Size::new(240.0, 160.0),
        );
        harness.focus(id);
        harness.update(key_pressed(Named::Enter, key::Code::Enter));
        let popup = harness.overlay_bounds().expect("open Select overlay");
        let point = Point::new(
            popup.x + MENU_LIST_INSET + 12.0,
            popup.y + MENU_LIST_INSET + MENU_ROW_HEIGHT * 2.0 + 14.0,
        );

        let (pressed, released) = if touch_input {
            (
                Event::Touch(touch::Event::FingerPressed {
                    id: touch::Finger(7),
                    position: point,
                }),
                Event::Touch(touch::Event::FingerLifted {
                    id: touch::Finger(7),
                    position: point,
                }),
            )
        } else {
            harness.set_cursor(point);
            (
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            )
        };
        let press = harness
            .update_overlay(pressed)
            .expect("open Select overlay");
        assert!(press.messages.is_empty());
        let release = harness
            .update_overlay(released)
            .expect("open Select overlay");

        assert_eq!(
            release.messages,
            vec![Message::Selected(3), Message::Closed]
        );
    }
}

#[test]
fn programmatic_rebuild_is_silent_and_preserves_the_open_session() {
    let id = Id::unique();
    let mut harness = WidgetHarness::new(
        interactive_select(Some(1), id.clone()),
        Size::new(240.0, 160.0),
    );
    harness.focus(id.clone());
    assert_eq!(
        harness
            .update(key_pressed(Named::Enter, key::Code::Enter))
            .messages,
        vec![Message::Opened]
    );

    harness.replace(interactive_select(Some(3), id));

    assert!(harness.state_at::<SelectState>(&[0]).open);
    assert!(harness.has_overlay());
}

#[test]
fn callback_absence_is_display_only_without_disabled_focus_or_hover_capability() {
    let select: Element<'static, Message> = Select::new(options(), Some(1)).into();
    let mut harness = WidgetHarness::new(select, Size::new(240.0, 160.0));
    harness.set_cursor(Point::new(40.0, 16.0));

    assert!(harness.focusable_ids().is_empty());
    assert_eq!(harness.mouse_interaction(), mouse::Interaction::None);
    assert!(harness
        .update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left
        )))
        .messages
        .is_empty());
    assert!(!harness.has_overlay());
}

#[test]
fn empty_model_opens_with_finite_explanatory_content_and_cannot_commit() {
    let id = Id::unique();
    let select: Element<'static, Message> = Select::new(Vec::new(), None::<u8>)
        .id(id.clone())
        .on_select(Message::Selected)
        .on_open(Message::Opened)
        .on_close(Message::Closed)
        .into();
    let mut harness = WidgetHarness::new(select, Size::new(240.0, 120.0));
    harness.focus(id);
    assert_eq!(
        harness
            .update(key_pressed(Named::Enter, key::Code::Enter))
            .messages,
        vec![Message::Opened]
    );

    let bounds = harness.overlay_bounds().expect("empty Select overlay");
    assert!(bounds.x.is_finite());
    assert!(bounds.y.is_finite());
    assert!(bounds.width.is_finite() && bounds.width > 0.0);
    assert!(bounds.height.is_finite() && bounds.height >= MENU_ROW_HEIGHT);
    let commit = harness
        .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
        .expect("empty Select overlay");
    assert!(commit.messages.is_empty());
    assert!(harness.state_at::<SelectState>(&[0]).open);
}

#[test]
fn duplicate_values_are_diagnosable_and_every_row_is_nonactivating() {
    let id = Id::unique();
    let duplicate_options = vec![
        SelectOption::new(1_u8, "First"),
        SelectOption::new(1_u8, "Duplicate"),
    ];
    let model = Select::<_, Message>::new(duplicate_options.clone(), None);
    assert!(!model.has_unique_values());
    let select: Element<'static, Message> = Select::new(duplicate_options, None)
        .id(id.clone())
        .on_select(Message::Selected)
        .on_open(Message::Opened)
        .on_close(Message::Closed)
        .into();
    let mut harness = WidgetHarness::new(select, Size::new(240.0, 120.0));
    harness.focus(id);
    harness.update(key_pressed(Named::Enter, key::Code::Enter));

    let bounds = harness.overlay_bounds().expect("duplicate Select overlay");
    assert!(bounds.width.is_finite() && bounds.height.is_finite());
    let activation = harness
        .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
        .expect("duplicate Select overlay");
    assert!(activation.messages.is_empty());
    assert!(harness.state_at::<SelectState>(&[0]).open);
}

#[test]
fn missing_selected_value_recovers_through_first_enabled_without_inventing_state() {
    let id = Id::unique();
    let mut harness = WidgetHarness::new(
        interactive_select(Some(99), id.clone()),
        Size::new(240.0, 160.0),
    );
    harness.focus(id);
    harness.update(key_pressed(Named::Enter, key::Code::Enter));

    let result = harness
        .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
        .expect("open Select overlay");

    assert_eq!(result.messages, vec![Message::Selected(1), Message::Closed]);
}

#[test]
fn home_end_and_typeahead_drive_the_persistent_open_overlay() {
    for (selected, navigation, expected) in [
        (
            1,
            key_pressed(Named::End, key::Code::End),
            Message::Selected(3),
        ),
        (
            3,
            key_pressed(Named::Home, key::Code::Home),
            Message::Selected(1),
        ),
        (1, text_key("br"), Message::Selected(3)),
    ] {
        let id = Id::unique();
        let mut harness = WidgetHarness::new(
            interactive_select(Some(selected), id.clone()),
            Size::new(240.0, 160.0),
        );
        harness.focus(id);
        harness.update(key_pressed(Named::Enter, key::Code::Enter));
        harness
            .update_overlay(navigation)
            .expect("open Select overlay");

        let result = harness
            .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
            .expect("open Select overlay");

        assert_eq!(result.messages, vec![expected, Message::Closed]);
    }
}

#[test]
fn initial_highlight_is_ensured_visible_in_a_low_viewport() {
    let options = (0_u8..24)
        .map(|value| SelectOption::new(value, format!("Option {value}")))
        .collect::<Vec<_>>();
    let id = Id::unique();
    let select: Element<'static, Message> = Select::new(options, Some(23))
        .id(id.clone())
        .width(Length::Fixed(180.0))
        .on_select(Message::Selected)
        .into();
    let mut harness = WidgetHarness::new(select, Size::new(220.0, 90.0));
    harness.focus(id);
    harness.update(key_pressed(Named::Enter, key::Code::Enter));

    harness
        .update_overlay(Event::Window(window::Event::RedrawRequested(
            iced::time::Instant::now(),
        )))
        .expect("open Select overlay");

    assert!(harness
        .overlay_scroll_offsets()
        .iter()
        .any(|offset| offset.y.abs() > f32::EPSILON));
}

#[test]
fn popup_flips_and_remains_safe_when_the_trigger_is_near_the_bottom() {
    let id = Id::unique();
    let content: Element<'static, Message> = iced::widget::column![
        Space::new().height(Length::Fixed(58.0)),
        Select::new(options(), Some(1))
            .id(id.clone())
            .on_select(Message::Selected),
    ]
    .into();
    let mut harness = WidgetHarness::new(content, Size::new(240.0, 100.0));
    let anchor = harness.focusable_bounds(&id).expect("Select trigger");
    harness.focus(id);
    harness.update(key_pressed(Named::Enter, key::Code::Enter));

    let popup = harness.overlay_bounds().expect("flipped Select overlay");

    assert!(popup.y < anchor.y);
    assert!(popup.y >= 8.0);
    assert!(popup.y + popup.height <= 92.0);
}

#[test]
fn diff_reorder_preserves_the_highlighted_option_by_visible_identity() {
    let id = Id::unique();
    let mut harness = WidgetHarness::new(
        interactive_select(Some(1), id.clone()),
        Size::new(240.0, 160.0),
    );
    harness.focus(id.clone());
    harness.update(key_pressed(Named::Enter, key::Code::Enter));
    harness
        .update_overlay(key_pressed(Named::ArrowDown, key::Code::ArrowDown))
        .expect("open Select overlay");

    let reordered: Element<'static, Message> = Select::new(
        vec![
            SelectOption::new(1, "Alpha"),
            SelectOption::new(3, "Bravo"),
            SelectOption::new(2, "Beta").disabled(true),
        ],
        Some(1),
    )
    .id(id)
    .on_select(Message::Selected)
    .on_open(Message::Opened)
    .on_close(Message::Closed)
    .into();
    harness.replace(reordered);

    let result = harness
        .update_overlay(key_pressed(Named::Enter, key::Code::Enter))
        .expect("reconciled Select overlay");
    assert_eq!(result.messages, vec![Message::Selected(3), Message::Closed]);
}

#[test]
fn closed_wheel_and_command_wheel_never_mutate_or_open_selection() {
    let id = Id::unique();
    let mut harness = WidgetHarness::new(
        interactive_select(Some(1), id.clone()),
        Size::new(240.0, 160.0),
    );
    harness.focus(id);

    assert!(harness
        .update(Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
        }))
        .messages
        .is_empty());
    harness.update(Event::Keyboard(keyboard::Event::ModifiersChanged(
        keyboard::Modifiers::COMMAND,
    )));
    assert!(harness
        .update(Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: 40.0 },
        }))
        .messages
        .is_empty());
    assert!(!harness.state_at::<SelectState>(&[0]).open);
}

#[test]
fn wide_narrow_wide_relayout_keeps_open_geometry_finite_and_safe() {
    let id = Id::unique();
    let select: Element<'static, Message> = Select::new(options(), Some(1))
        .id(id.clone())
        .width(Length::Fixed(300.0))
        .on_select(Message::Selected)
        .into();
    let mut harness = WidgetHarness::new(select, Size::new(400.0, 180.0));
    harness.focus(id);
    harness.update(key_pressed(Named::Enter, key::Code::Enter));

    for viewport in [
        Size::new(400.0, 180.0),
        Size::new(220.0, 100.0),
        Size::new(400.0, 180.0),
    ] {
        harness.relayout(viewport);
        let popup = harness.overlay_bounds().expect("open Select overlay");
        assert!(popup.x.is_finite() && popup.y.is_finite());
        assert!(popup.width.is_finite() && popup.height.is_finite());
        assert!(popup.x >= 8.0);
        assert!(popup.x + popup.width <= viewport.width - 8.0);
    }
}

fn text_key(value: &str) -> Event {
    let key = keyboard::Key::Character(value.into());
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Code(key::Code::KeyB),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: Some(value.into()),
        repeat: false,
    })
}
