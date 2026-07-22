use super::*;

#[test]
fn product_shortcut_routes_to_unscoped_app_message() {
    let actions = ActionMap::new();
    let shortcuts = ShortcutMap::new().bind(
        ShortcutBinding::character('K', ShortcutModifiers::CONTROL),
        TestMessage::Shortcut,
    );
    let event = key_pressed(
        keyboard::Key::Character("k".into()),
        keyboard::Modifiers::CTRL,
        false,
    );

    assert!(matches!(
        shortcut_message_from_event::<TestApp, NoProbe>(&actions, &shortcuts, event),
        Some(NiveMessage::App {
            window_id: None,
            source: MessageSource::Action,
            message: TestMessage::Shortcut
        })
    ));
}

#[test]
fn repeated_product_shortcut_keypress_is_ignored() {
    let actions = ActionMap::new();
    let shortcuts = ShortcutMap::new().bind(
        ShortcutBinding::character('k', ShortcutModifiers::CONTROL),
        TestMessage::Shortcut,
    );
    let event = key_pressed(
        keyboard::Key::Character("k".into()),
        keyboard::Modifiers::CTRL,
        true,
    );

    assert!(shortcut_message_from_event::<TestApp, NoProbe>(&actions, &shortcuts, event).is_none());
}

#[test]
fn action_shortcut_routes_before_legacy_shortcut() {
    let actions = ActionMap::new().action(
        Action::new("test.action", "Test action", TestMessage::Action)
            .shortcut(ShortcutBinding::character('k', ShortcutModifiers::CONTROL)),
    );
    let shortcuts = ShortcutMap::new().bind(
        ShortcutBinding::character('k', ShortcutModifiers::CONTROL),
        TestMessage::LegacyShortcut,
    );
    let event = key_pressed(
        keyboard::Key::Character("k".into()),
        keyboard::Modifiers::CTRL,
        false,
    );

    assert!(matches!(
        shortcut_message_from_event::<TestApp, NoProbe>(&actions, &shortcuts, event),
        Some(NiveMessage::App {
            window_id: None,
            source: MessageSource::Action,
            message: TestMessage::Action
        })
    ));
}

#[test]
fn disabled_action_shortcut_does_not_dispatch() {
    let actions = ActionMap::new().action(
        Action::new("test.action", "Test action", TestMessage::Action)
            .shortcut(ShortcutBinding::character('k', ShortcutModifiers::CONTROL))
            .disabled(),
    );
    let shortcuts = ShortcutMap::new();
    let event = key_pressed(
        keyboard::Key::Character("k".into()),
        keyboard::Modifiers::CTRL,
        false,
    );

    assert!(shortcut_message_from_event::<TestApp, NoProbe>(&actions, &shortcuts, event).is_none());
}

#[test]
fn framework_shortcut_wins_product_conflict() {
    let actions = ActionMap::new();
    let shortcuts = ShortcutMap::new().bind(
        ShortcutBinding::named(NamedShortcutKey::Tab, ShortcutModifiers::NONE),
        TestMessage::Shortcut,
    );
    let event = key_pressed(
        keyboard::Key::Named(keyboard::key::Named::Tab),
        keyboard::Modifiers::NONE,
        false,
    );

    assert!(matches!(
        shortcut_message_from_event::<TestApp, NoProbe>(&actions, &shortcuts, event),
        Some(NiveMessage::Core(CoreMessage::KeyboardNavigation(
            KeyboardNavigation::FocusNext
        )))
    ));
}

#[test]
fn framework_shortcut_wins_action_conflict() {
    let actions = ActionMap::new().action(
        Action::new("test.action", "Test action", TestMessage::Shortcut).shortcut(
            ShortcutBinding::named(NamedShortcutKey::Tab, ShortcutModifiers::NONE),
        ),
    );
    let shortcuts = ShortcutMap::new();
    let event = key_pressed(
        keyboard::Key::Named(keyboard::key::Named::Tab),
        keyboard::Modifiers::NONE,
        false,
    );

    assert!(matches!(
        shortcut_message_from_event::<TestApp, NoProbe>(&actions, &shortcuts, event),
        Some(NiveMessage::Core(CoreMessage::KeyboardNavigation(
            KeyboardNavigation::FocusNext
        )))
    ));
}
