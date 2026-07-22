use iced::{
    advanced::{clipboard, input_method},
    keyboard::{
        self,
        key::{Code, Named, Physical},
        Key, Location, Modifiers,
    },
    mouse,
    widget::Column,
    Event, Point, Size,
};

use super::*;
use crate::test_support::{named_probe, WidgetHarness};
use crate::theme::{ThemeBuilder, ThemeDensity, ThemeMode};

fn command_modifier() -> Modifiers {
    if cfg!(target_os = "macos") {
        Modifiers::LOGO
    } else {
        Modifiers::CTRL
    }
}

fn key_pressed(key: Key, modifiers: Modifiers, text: Option<&str>) -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: Physical::Code(Code::KeyA),
        location: Location::Standard,
        modifiers,
        text: text.map(Into::into),
        repeat: false,
    })
}

fn focus<Message>(harness: &mut WidgetHarness<'_, Message>) {
    harness.set_cursor(Point::new(8.0, 8.0));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
}

#[test]
fn input_model_retains_id_semantic_name_and_capability() {
    let id = Id::new("account-name");
    let input = Input::<String>::new("Account", "Ada")
        .id(id.clone())
        .semantic_name(String::from("Account name"))
        .read_only(true)
        .lg();

    assert_eq!(input.widget_id(), Some(&id));
    assert_eq!(input.semantic_name_value(), Some("Account name"));
    assert!(input.is_read_only());
    assert_eq!(input.control_size(), ControlSize::Lg);
}

#[test]
fn callback_absence_is_read_only_while_callback_presence_is_editable() {
    assert!(Input::<String>::new("Name", "Ada").is_read_only());
    assert!(!Input::new("Name", "Ada")
        .on_change(|value| value)
        .is_read_only());
}

#[test]
fn editable_input_publishes_changes_and_read_only_drops_them() {
    let editable: Element<'_, String> = Input::new("Name", "Ada").on_change(|value| value).into();
    let mut editable = WidgetHarness::new(editable, Size::new(240.0, 80.0));
    focus(&mut editable);
    let changed = editable.update(key_pressed(
        Key::Character("x".into()),
        Modifiers::NONE,
        Some("x"),
    ));
    assert_eq!(changed.messages.len(), 1);

    let read_only: Element<'_, String> = Input::new("Name", "Ada")
        .on_change(|value| value)
        .read_only(true)
        .into();
    let mut read_only = WidgetHarness::new(read_only, Size::new(240.0, 80.0));
    focus(&mut read_only);
    let blocked = read_only.update(key_pressed(
        Key::Character("x".into()),
        Modifiers::NONE,
        Some("x"),
    ));
    assert!(blocked.messages.is_empty());
    assert!(blocked.captured);
}

#[test]
fn read_only_retains_selection_copy_and_non_mutating_submit() {
    let input: Element<'_, &'static str> = Input::new("Name", "Ada")
        .read_only(true)
        .on_submit("submit")
        .into();
    let mut harness = WidgetHarness::new(input, Size::new(240.0, 80.0));
    focus(&mut harness);
    harness.update(Event::Keyboard(keyboard::Event::ModifiersChanged(
        command_modifier(),
    )));

    harness.update(key_pressed(
        Key::Character("a".into()),
        command_modifier(),
        None,
    ));
    harness.update(key_pressed(
        Key::Character("c".into()),
        command_modifier(),
        None,
    ));
    let submitted = harness.update(key_pressed(Key::Named(Named::Enter), Modifiers::NONE, None));

    assert_eq!(harness.clipboard(clipboard::Kind::Standard), Some("Ada"));
    assert_eq!(submitted.messages, vec!["submit"]);
}

#[test]
fn read_only_never_requests_an_enabled_input_method() {
    let input: Element<'_, String> = Input::new("Name", "Ada").read_only(true).into();
    let mut harness = WidgetHarness::new(input, Size::new(240.0, 80.0));
    focus(&mut harness);

    let redraw = harness.update(Event::Window(iced::window::Event::RedrawRequested(
        iced::time::Instant::now(),
    )));

    assert!(!redraw.input_method_enabled);
}

#[test]
fn caller_owned_id_supports_programmatic_focus_and_disabled_blocks_it() {
    let id = Id::new("focus-target");
    let input: Element<'_, String> = Input::new("Name", "Ada")
        .id(id.clone())
        .on_change(|value| value)
        .into();
    let mut harness = WidgetHarness::new(input, Size::new(240.0, 80.0));

    harness.focus(id);

    assert_eq!(harness.focused_count().focused, Some(0));

    let disabled_id = Id::new("disabled-target");
    let disabled: Element<'_, String> = Input::new("Name", "Ada")
        .id(disabled_id.clone())
        .disabled(true)
        .into();
    let mut disabled = WidgetHarness::new(disabled, Size::new(240.0, 80.0));
    disabled.focus(disabled_id);

    assert_eq!(disabled.focused_count().focused, None);
}

#[test]
fn tab_continues_after_the_last_pointer_focused_input_is_blurred() {
    let ids = [Id::new("first"), Id::new("second"), Id::new("third")];
    let second_visual_focus = Rc::new(Cell::new(false));
    let column = Column::new()
        .push(
            Input::new("Value", "")
                .id(ids[0].clone())
                .on_change(|value| value),
        )
        .push(
            Input::new("Value", "")
                .id(ids[1].clone())
                .track_focus(Rc::clone(&second_visual_focus))
                .on_change(|value| value),
        )
        .push(
            Input::new("Value", "")
                .id(ids[2].clone())
                .on_change(|value| value),
        );
    let mut harness = WidgetHarness::new(
        crate::accessibility::FocusRoot::new(column).into(),
        Size::new(240.0, 140.0),
    );

    harness.set_cursor(Point::new(20.0, 42.0));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert_eq!(harness.focused_ids(), [ids[1].clone()]);

    harness.set_cursor(Point::new(20.0, 120.0));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert!(!second_visual_focus.get());

    harness.focus_next();
    assert_eq!(harness.focused_ids(), [ids[2].clone()]);
}

#[test]
fn tab_continues_after_an_unidentified_pointer_focused_input_is_blurred() {
    let next = Id::new("next-after-input");
    let column = Column::new()
        .push(crate::widgets::button::primary("Before").on_press("before"))
        .push(Input::new("Value", "").on_change(|_| "changed"))
        .push(
            crate::widgets::button::primary("After")
                .id(next.clone())
                .on_press("after"),
        );
    let mut harness = WidgetHarness::new(
        crate::accessibility::FocusRoot::new(iced::widget::scrollable(column)).into(),
        Size::new(240.0, 160.0),
    );

    harness.set_cursor(Point::new(20.0, 42.0));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert_eq!(
        harness
            .managed_focus()
            .entries
            .iter()
            .filter(|entry| entry.active)
            .count(),
        1
    );

    harness.set_cursor(Point::new(220.0, 150.0));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert_eq!(
        harness
            .managed_focus()
            .entries
            .iter()
            .filter(|entry| entry.anchor_only)
            .count(),
        1
    );

    harness.focus_next();
    assert_eq!(harness.focused_ids(), [next]);
}

#[test]
fn every_density_and_size_keeps_the_form_control_height() {
    let cases = [
        (ThemeDensity::Compact, [20.0, 24.0, 28.0, 32.0]),
        (ThemeDensity::Standard, [24.0, 28.0, 32.0, 36.0]),
        (ThemeDensity::Comfortable, [28.0, 32.0, 36.0, 40.0]),
    ];
    let sizes = [
        ControlSize::Xs,
        ControlSize::Sm,
        ControlSize::Md,
        ControlSize::Lg,
    ];

    for (density, heights) in cases {
        let theme = ThemeBuilder::new("Input metrics", ThemeMode::Light)
            .density(density)
            .build();
        let _guard = crate::theme::testing::ThemeTestGuard::activate(theme);

        for (size, height) in sizes.into_iter().zip(heights) {
            let input: Element<'_, String> = Input::new("Name", "Ada")
                .size(size)
                .on_change(|value| value)
                .into();
            let harness = WidgetHarness::new(input, Size::new(240.0, 80.0));

            assert_eq!(harness.bounds().height, height);
        }
    }
}

#[test]
fn named_fill_input_reflows_wide_narrow_wide_without_height_growth() {
    let long_value = "A long single-line value that must scroll instead of wrapping";
    let input: Element<'_, String> = named_probe(
        "input",
        Input::new("Name", long_value).on_change(|value| value),
    );
    let mut harness = WidgetHarness::new(input, Size::new(320.0, 80.0));

    assert_eq!(harness.named_bounds("input").expect("wide").width, 320.0);
    assert_eq!(harness.named_bounds("input").expect("wide").height, 28.0);

    harness.relayout(Size::new(96.0, 80.0));
    assert_eq!(harness.named_bounds("input").expect("narrow").width, 96.0);
    assert_eq!(harness.named_bounds("input").expect("narrow").height, 28.0);

    harness.relayout(Size::new(320.0, 80.0));
    assert_eq!(
        harness.named_bounds("input").expect("wide again").width,
        320.0
    );
    assert_eq!(
        harness.named_bounds("input").expect("wide again").height,
        28.0
    );
}

#[test]
fn fixed_and_embedded_inputs_keep_finite_named_bounds() {
    let fixed: Element<'_, String> = named_probe(
        "fixed",
        Input::new("Name", "Ada")
            .width(Length::Fixed(120.0))
            .on_change(|value| value),
    );
    let mut fixed = WidgetHarness::new(fixed, Size::new(320.0, 80.0));
    assert_eq!(fixed.named_bounds("fixed").expect("fixed").width, 120.0);
    assert_eq!(fixed.named_bounds("fixed").expect("fixed").height, 28.0);

    let shrink: Element<'_, String> = named_probe(
        "shrink",
        Input::new("Name", "Ada")
            .shrink_width()
            .on_change(|value| value),
    );
    let mut shrink = WidgetHarness::new(shrink, Size::new(320.0, 80.0));
    let shrink_bounds = shrink.named_bounds("shrink").expect("shrink");
    assert!(shrink_bounds.width.is_finite());
    assert!(shrink_bounds.width > 0.0);
    assert!(shrink_bounds.width <= 320.0);
    assert_eq!(shrink_bounds.height, 28.0);

    let embedded: Element<'_, String> = named_probe(
        "embedded",
        Input::new("Name", "Ada")
            .embedded()
            .on_change(|value| value),
    );
    let mut embedded = WidgetHarness::new(embedded, Size::new(180.0, 80.0));
    assert_eq!(
        embedded.named_bounds("embedded").expect("embedded").height,
        28.0
    );
}

#[test]
fn read_only_blocks_delete_cut_paste_and_ime_before_native_update() {
    let input: Element<'_, String> = Input::new("Name", "Ada")
        .read_only(true)
        .on_change(|value| value)
        .into();
    let mut harness = WidgetHarness::new(input, Size::new(240.0, 80.0));
    focus(&mut harness);
    harness.set_clipboard(clipboard::Kind::Standard, "Grace");

    for event in [
        key_pressed(Key::Named(Named::Backspace), Modifiers::NONE, None),
        key_pressed(Key::Named(Named::Delete), Modifiers::NONE, None),
        key_pressed(Key::Character("x".into()), command_modifier(), None),
        key_pressed(Key::Character("v".into()), command_modifier(), None),
        Event::InputMethod(input_method::Event::Opened),
        Event::InputMethod(input_method::Event::Preedit("draft".into(), None)),
        Event::InputMethod(input_method::Event::Commit("committed".into())),
    ] {
        let result = harness.update(event);
        assert!(result.messages.is_empty());
        assert!(result.captured);
    }

    assert_eq!(harness.clipboard(clipboard::Kind::Standard), Some("Grace"));
}

#[test]
fn secure_read_only_input_preserves_native_copy_restriction() {
    let input: Element<'_, String> = Input::new("Password", "secret")
        .read_only(true)
        .secure(true)
        .into();
    let mut harness = WidgetHarness::new(input, Size::new(240.0, 80.0));
    focus(&mut harness);
    harness.update(Event::Keyboard(keyboard::Event::ModifiersChanged(
        command_modifier(),
    )));
    harness.update(key_pressed(
        Key::Character("a".into()),
        command_modifier(),
        None,
    ));
    harness.update(key_pressed(
        Key::Character("c".into()),
        command_modifier(),
        None,
    ));

    assert_eq!(harness.clipboard(clipboard::Kind::Standard), None);
}

#[test]
fn disabled_input_suppresses_focus_mutation_and_submit() {
    let id = Id::new("disabled-input");
    let input: Element<'_, String> = Input::new("Name", "Ada")
        .id(id.clone())
        .disabled(true)
        .on_change(|value| value)
        .on_submit("submit".into())
        .into();
    let mut harness = WidgetHarness::new(input, Size::new(240.0, 80.0));
    focus(&mut harness);
    harness.focus(id);

    assert_eq!(harness.focused_count().focused, None);
    assert!(harness
        .update(key_pressed(
            Key::Character("x".into()),
            Modifiers::NONE,
            Some("x"),
        ))
        .messages
        .is_empty());
    assert!(harness
        .update(key_pressed(Key::Named(Named::Enter), Modifiers::NONE, None,))
        .messages
        .is_empty());
}

#[test]
fn outside_pointer_blur_is_published_exactly_once() {
    let input: Element<'_, &'static str> = Input::new("Name", "Ada")
        .read_only(true)
        .on_blur("blur")
        .into();
    let mut harness = WidgetHarness::new(input, Size::new(240.0, 80.0));
    focus(&mut harness);
    harness.set_cursor(Point::new(40.0, 60.0));

    let first = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    let second = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));

    assert_eq!(first.messages, vec!["blur"]);
    assert!(second.messages.is_empty());
}

#[test]
fn focused_to_disabled_rebuild_clears_focus_without_blur_message() {
    let id = Id::new("rebuild-input");
    let focused: Element<'static, &'static str> = Input::new("Name", "Ada")
        .id(id.clone())
        .read_only(true)
        .on_blur("blur")
        .into();
    let mut harness = WidgetHarness::new(focused, Size::new(240.0, 80.0));
    harness.focus(id.clone());
    assert_eq!(harness.focused_count().focused, Some(0));

    let disabled: Element<'static, &'static str> = Input::new("Name", "Ada")
        .id(id)
        .disabled(true)
        .on_blur("blur")
        .into();
    harness.replace(disabled);

    assert_eq!(harness.focused_count().focused, None);
}
