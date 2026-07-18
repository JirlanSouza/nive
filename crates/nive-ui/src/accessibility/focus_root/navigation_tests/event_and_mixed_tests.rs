use super::*;

#[test]
fn root_events_keep_origin_visibility_and_window_state_distinct() {
    let ids = [Id::unique(), Id::unique(), Id::unique()];
    let mut harness = harness(&ids);
    let button = harness.focusable_bounds(&ids[0]).expect("button bounds");
    harness.set_cursor(button.center());
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    let pointer_button = harness.managed_focus();
    assert!(pointer_button.entries[0].active);
    assert!(!pointer_button.entries[0].visible);

    let input = harness.focusable_bounds(&ids[1]).expect("input bounds");
    harness.update(Event::Touch(touch::Event::FingerPressed {
        id: touch::Finger(2),
        position: input.center(),
    }));
    let touch_input = harness.managed_focus();
    assert!(touch_input.entries[1].active);
    assert!(touch_input.entries[1].visible);

    harness.update(Event::Window(iced::window::Event::Unfocused));
    let inactive = harness.managed_focus();
    assert!(inactive.entries[1].anchor_only);
    assert!(!inactive.entries[1].visible);
    harness.update(Event::Window(iced::window::Event::Focused));
    assert!(harness.managed_focus().entries[1].anchor_only);

    harness.update(Event::Keyboard(keyboard::Event::ModifiersChanged(
        keyboard::Modifiers::default(),
    )));
    harness.focus(ids[2].clone());
    let keyboard_button = harness.managed_focus();
    assert!(keyboard_button.entries[2].active);
    assert!(keyboard_button.entries[2].visible);
}

#[test]
fn independent_roots_do_not_share_focus_origin_or_window_transitions() {
    let first_ids = [Id::unique(), Id::unique(), Id::unique()];
    let second_ids = [Id::unique(), Id::unique(), Id::unique()];
    let mut first = harness(&first_ids);
    let mut second = harness(&second_ids);
    first.focus(first_ids[1].clone());
    second.focus(second_ids[1].clone());
    let first_root = first.managed_focus().entries[0].root_identity;
    let second_root = second.managed_focus().entries[0].root_identity;
    assert_ne!(first_root, second_root);

    first.set_cursor(Point::new(160.0, 140.0));
    first.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert!(first.managed_focus().entries[1].anchor_only);
    assert!(second.managed_focus().entries[1].active);

    first.focus_next();
    assert!(first.managed_focus().entries[2].active);
    assert!(second.managed_focus().entries[1].active);

    first.update(Event::Window(iced::window::Event::Unfocused));
    assert!(first.managed_focus().entries[2].anchor_only);
    assert!(second.managed_focus().entries[1].active);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MixedChoice {
    First,
    Second,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MixedMessage {
    Pressed,
    Changed,
    Selected(MixedChoice),
}

fn mixed_element(
    ids: &[Id; 3],
    reverse: bool,
    radio_enabled: bool,
    include_radio: bool,
) -> Element<'static, MixedMessage> {
    let button: Element<'static, MixedMessage> = button::primary("Action")
        .id(ids[0].clone())
        .on_press(MixedMessage::Pressed)
        .into();
    let input: Element<'static, MixedMessage> = Input::new("Value", "")
        .id(ids[2].clone())
        .on_change(|_| MixedMessage::Changed)
        .into();
    let radio = include_radio.then(|| {
        RadioGroup::new(
            "Choice",
            None,
            [
                RadioOption::new(MixedChoice::First, "First"),
                RadioOption::new(MixedChoice::Second, "Second"),
            ],
        )
        .id(ids[1].clone())
        .disabled(!radio_enabled)
        .on_select(MixedMessage::Selected)
        .into()
    });

    let mut children = if reverse {
        vec![input, button]
    } else {
        vec![button, input]
    };
    if let Some(radio) = radio {
        children.insert(1, radio);
    }

    FocusRoot::new(iced::widget::Column::with_children(children).spacing(4)).into()
}

#[test]
fn mixed_real_widgets_keep_one_target_across_origins_orders_and_rebuilds() {
    for reverse in [false, true] {
        let ids = [Id::unique(), Id::unique(), Id::unique()];
        let mut harness = WidgetHarness::new(
            mixed_element(&ids, reverse, true, true),
            Size::new(320.0, 220.0),
        );
        let radio_bounds = harness.focusable_bounds(&ids[1]).expect("radio bounds");
        let radio_position = Point::new(
            radio_bounds.x + 8.0,
            radio_bounds.y + radio_bounds.height - 8.0,
        );

        harness.focus(ids[1].clone());
        let targeted = harness.managed_focus();
        assert_eq!(
            targeted.entries.iter().filter(|entry| entry.active).count(),
            1
        );
        assert!(targeted
            .entries
            .iter()
            .any(|entry| { entry.id.as_ref() == Some(&ids[1]) && entry.active && entry.visible }));

        harness.set_cursor(radio_position);
        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        assert!(harness
            .managed_focus()
            .entries
            .iter()
            .any(|entry| { entry.id.as_ref() == Some(&ids[1]) && entry.active && !entry.visible }));

        harness.set_cursor(Point::new(300.0, 210.0));
        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        assert!(harness
            .managed_focus()
            .entries
            .iter()
            .any(|entry| { entry.id.as_ref() == Some(&ids[1]) && entry.anchor_only }));

        harness.update(Event::Keyboard(keyboard::Event::ModifiersChanged(
            keyboard::Modifiers::default(),
        )));
        harness.focus_next();
        let next = if reverse { &ids[0] } else { &ids[2] };
        assert!(harness
            .managed_focus()
            .entries
            .iter()
            .any(|entry| { entry.id.as_ref() == Some(next) && entry.active && entry.visible }));

        harness.update(Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(9),
            position: radio_position,
        }));
        assert!(harness
            .managed_focus()
            .entries
            .iter()
            .any(|entry| { entry.id.as_ref() == Some(&ids[1]) && entry.active && !entry.visible }));

        harness.replace(mixed_element(&ids, reverse, false, true));
        assert!(!harness
            .managed_focus()
            .entries
            .iter()
            .any(|entry| entry.active || entry.anchor_only));
        harness.focus_next();
        let first = if reverse { &ids[2] } else { &ids[0] };
        assert!(harness
            .managed_focus()
            .entries
            .iter()
            .any(|entry| entry.id.as_ref() == Some(first) && entry.active));

        harness.replace(mixed_element(&ids, reverse, true, true));
        harness.focus(ids[1].clone());
        harness.replace(mixed_element(&ids, reverse, true, false));
        assert!(!harness
            .managed_focus()
            .entries
            .iter()
            .any(|entry| entry.active || entry.anchor_only));
    }
}
