use iced::{keyboard, Event, Size};

use super::*;

fn enter(repeat: bool) -> Event {
    let key = keyboard::Key::Named(keyboard::key::Named::Enter);
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Enter),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat,
    })
}

fn messages(footer: DialogActionFooter<'static, &'static str>, event: Event) -> Vec<&'static str> {
    crate::test_support::event_messages(footer.into(), Size::new(900.0, 200.0), event)
}

#[test]
fn unconsumed_enter_activates_the_enabled_primary_action() {
    let footer = DialogActionFooter::with_one(
        DialogAction::cancel("Cancel", "cancel"),
        DialogTerminalAction::primary("Save", "save"),
    );

    assert_eq!(messages(footer, enter(false)), vec!["save"]);
}

#[test]
fn repeated_enter_publishes_nothing() {
    let footer = DialogActionFooter::new(DialogTerminalAction::primary("Save", "save"));

    assert!(messages(footer, enter(true)).is_empty());
}

#[test]
fn disabled_primary_publishes_nothing_on_enter() {
    let footer =
        DialogActionFooter::new(DialogTerminalAction::primary("Save", "save").disabled(true));

    assert!(messages(footer, enter(false)).is_empty());
}

#[test]
fn destructive_terminal_is_never_an_implicit_enter_default() {
    let footer = DialogActionFooter::new(DialogTerminalAction::destructive("Delete", "delete"));

    assert!(messages(footer, enter(false)).is_empty());
}
