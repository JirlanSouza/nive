use super::{command_palette_filter, CommandPalette, CommandPaletteItem};
use crate::test_support::WidgetHarness;
use crate::Element;
use iced::{keyboard, mouse, Event, Point, Size};

const VIEWPORT: Size = Size::new(800.0, 600.0);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Message {
    QueryChanged(String),
    Dismissed,
    Activated(&'static str),
}

fn key(named: keyboard::key::Named) -> Event {
    let key = keyboard::Key::Named(named);
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat: false,
    })
}

fn items() -> Vec<CommandPaletteItem<'static, Message>> {
    vec![
        CommandPaletteItem::new("open", "Open project", Message::Activated("open")),
        CommandPaletteItem::new("save", "Save project", Message::Activated("save")).disabled(true),
        CommandPaletteItem::new("close", "Close project", Message::Activated("close")),
    ]
}

fn harness(query: &'static str) -> WidgetHarness<'static, Message> {
    let base = iced::widget::text("base");
    let palette = CommandPalette::new(base)
        .open(true)
        .query(query)
        .items(items())
        .on_query_change(Message::QueryChanged)
        .on_dismiss(Message::Dismissed);
    WidgetHarness::new(Element::from(palette), VIEWPORT)
}

#[test]
fn closed_palette_renders_only_the_base_content() {
    let base = iced::widget::text("base");
    let palette: Element<'static, Message> = CommandPalette::new(base).open(false).into();
    let mut harness = WidgetHarness::new(palette, VIEWPORT);

    assert!(!harness.has_overlay());
}

#[test]
fn arrow_down_moves_highlight_to_the_next_eligible_row_skipping_disabled() {
    let mut harness = harness("");

    // Opening resolves the highlight to the first eligible row ("open").
    let initial = harness
        .update_overlay(key(keyboard::key::Named::Enter))
        .expect("open palette overlay");
    assert_eq!(initial.messages, vec![Message::Activated("open")]);

    // ArrowDown moves to "close", skipping the disabled "save" row entirely.
    harness
        .update_overlay(key(keyboard::key::Named::ArrowDown))
        .expect("open palette overlay");
    let after_first = harness
        .update_overlay(key(keyboard::key::Named::Enter))
        .expect("open palette overlay");
    assert_eq!(after_first.messages, vec![Message::Activated("close")]);

    // A further ArrowDown wraps back to "open".
    harness
        .update_overlay(key(keyboard::key::Named::ArrowDown))
        .expect("open palette overlay");
    let after_second = harness
        .update_overlay(key(keyboard::key::Named::Enter))
        .expect("open palette overlay");
    assert_eq!(after_second.messages, vec![Message::Activated("open")]);
}

#[test]
fn arrow_up_wraps_to_the_last_eligible_row() {
    let mut harness = harness("");

    harness
        .update_overlay(key(keyboard::key::Named::ArrowUp))
        .expect("open palette overlay");
    let result = harness
        .update_overlay(key(keyboard::key::Named::Enter))
        .expect("open palette overlay");

    assert_eq!(result.messages, vec![Message::Activated("close")]);
}

#[test]
fn enter_on_a_disabled_row_publishes_nothing() {
    let base = iced::widget::text("base");
    let single_disabled =
        vec![
            CommandPaletteItem::new("save", "Save project", Message::Activated("save"))
                .disabled(true),
        ];
    let palette = CommandPalette::new(base)
        .open(true)
        .items(single_disabled)
        .on_dismiss(Message::Dismissed);
    let mut harness = WidgetHarness::new(Element::from(palette), VIEWPORT);

    let result = harness
        .update_overlay(key(keyboard::key::Named::Enter))
        .expect("open palette overlay");

    assert!(result.messages.is_empty());
}

#[test]
fn escape_dismisses_exactly_once() {
    let mut harness = harness("");

    let escape = Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(keyboard::key::Named::Escape),
        modified_key: keyboard::Key::Named(keyboard::key::Named::Escape),
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Escape),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat: false,
    });

    let result = harness
        .update_overlay(escape)
        .expect("open palette overlay");

    assert_eq!(result.messages, vec![Message::Dismissed]);
}

#[test]
fn outside_press_dismisses_through_the_shared_kernel() {
    let mut harness = harness("");
    harness.set_cursor(Point::new(5.0, 5.0));

    let result = harness
        .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .expect("open palette overlay");

    assert_eq!(result.messages, vec![Message::Dismissed]);
}

#[test]
fn base_content_is_inert_while_the_palette_is_open() {
    let base = crate::widgets::button::primary("Base")
        .id(iced::widget::Id::new("base"))
        .on_press(Message::Activated("base"));
    let palette = CommandPalette::new(base)
        .open(true)
        .items(items())
        .on_dismiss(Message::Dismissed);
    let mut harness = WidgetHarness::new(Element::from(palette), VIEWPORT);
    let base_bounds = harness.bounds();
    harness.set_cursor(Point::new(base_bounds.x + 2.0, base_bounds.y + 2.0));

    let result = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));

    assert!(result.messages.is_empty());
}

#[test]
fn filter_reflects_query_change_through_the_provided_matcher() {
    let items = items();
    let visible = command_palette_filter("close", &items);

    assert_eq!(visible, vec![2]);
}

#[test]
fn typed_text_reaches_the_search_input_instead_of_being_swallowed() {
    let mut harness = harness("");

    let key = keyboard::Key::Character("a".into());
    let event = Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: Some("a".into()),
        repeat: false,
    });

    let result = harness.update_overlay(event).expect("open palette overlay");

    assert_eq!(result.messages, vec![Message::QueryChanged("a".to_owned())]);
}

#[test]
fn caret_keys_do_not_move_the_palette_highlight() {
    let mut harness = harness("");

    for named in [
        keyboard::key::Named::Home,
        keyboard::key::Named::End,
        keyboard::key::Named::ArrowLeft,
        keyboard::key::Named::ArrowRight,
    ] {
        harness
            .update_overlay(key(named))
            .expect("open palette overlay");
    }

    // The highlight must still be on the first eligible row ("open"): none
    // of Home/End/Left/Right are palette navigation keys, so they must have
    // reached the Input's own caret handling instead of moving highlight.
    let result = harness
        .update_overlay(key(keyboard::key::Named::Enter))
        .expect("open palette overlay");
    assert_eq!(result.messages, vec![Message::Activated("open")]);
}

#[test]
fn palette_width_clamps_to_a_narrow_viewport() {
    let base = iced::widget::text("base");
    let palette = CommandPalette::new(base)
        .open(true)
        .items(items())
        .on_dismiss(Message::Dismissed);
    let mut harness = WidgetHarness::new(Element::from(palette), Size::new(320.0, 600.0));

    let bounds = harness.overlay_bounds().expect("open palette overlay");

    assert!(bounds.width <= 320.0);
    assert!(bounds.x >= 0.0);
}

#[test]
fn palette_renders_near_the_top_and_stays_on_screen_in_a_low_viewport() {
    let base = iced::widget::text("base");
    let palette = CommandPalette::new(base)
        .open(true)
        .items(items())
        .on_dismiss(Message::Dismissed);
    let mut harness = WidgetHarness::new(Element::from(palette), Size::new(800.0, 200.0));

    let bounds = harness.overlay_bounds().expect("open palette overlay");

    assert!(bounds.y >= 0.0);
    assert!(bounds.y + bounds.height <= 200.0 + f32::EPSILON);
}
