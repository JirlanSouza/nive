use iced::keyboard;

use nive_core::ActionMap;

pub use nive_core::{
    NamedShortcutKey, ShortcutBinding, ShortcutKey, ShortcutMap, ShortcutModifiers,
};

pub(crate) fn action_message_for_event<M: Clone>(
    actions: &ActionMap<M>,
    event: &keyboard::Event,
) -> Option<M> {
    let (key, modifiers) = shortcut_parts(event)?;

    actions
        .iter()
        .find(|action| {
            action.is_enabled()
                && action
                    .shortcut_binding()
                    .is_some_and(|binding| binding.matches(key, modifiers))
        })
        .and_then(nive_core::Action::activate)
}

pub(crate) fn shortcut_message_for_event<M: Clone>(
    shortcuts: &ShortcutMap<M>,
    event: &keyboard::Event,
) -> Option<M> {
    let (key, modifiers) = shortcut_parts(event)?;

    shortcuts
        .iter()
        .find(|(binding, _)| binding.matches(key, modifiers))
        .map(|(_, message)| message.clone())
}

fn shortcut_parts(event: &keyboard::Event) -> Option<(ShortcutKey, ShortcutModifiers)> {
    let keyboard::Event::KeyPressed {
        key,
        modifiers,
        repeat: false,
        ..
    } = event
    else {
        return None;
    };

    let key = match key {
        keyboard::Key::Character(character) => {
            ShortcutKey::Character(character.chars().next()?.to_ascii_lowercase())
        }
        keyboard::Key::Named(named) => ShortcutKey::Named(named_key(*named)?),
        keyboard::Key::Unidentified => return None,
    };

    Some((key, modifiers_from_iced(*modifiers)))
}

fn modifiers_from_iced(modifiers: keyboard::Modifiers) -> ShortcutModifiers {
    let mut neutral = ShortcutModifiers::NONE;

    if modifiers.control() {
        neutral |= ShortcutModifiers::CONTROL;
    }
    if modifiers.alt() {
        neutral |= ShortcutModifiers::ALT;
    }
    if modifiers.shift() {
        neutral |= ShortcutModifiers::SHIFT;
    }
    if modifiers.logo() {
        neutral |= ShortcutModifiers::LOGO;
    }

    neutral
}

fn named_key(named: keyboard::key::Named) -> Option<NamedShortcutKey> {
    use keyboard::key::Named;

    Some(match named {
        Named::Enter => NamedShortcutKey::Enter,
        Named::Tab => NamedShortcutKey::Tab,
        Named::Space => NamedShortcutKey::Space,
        Named::Escape => NamedShortcutKey::Escape,
        Named::Backspace => NamedShortcutKey::Backspace,
        Named::Delete => NamedShortcutKey::Delete,
        Named::ArrowUp => NamedShortcutKey::ArrowUp,
        Named::ArrowDown => NamedShortcutKey::ArrowDown,
        Named::ArrowLeft => NamedShortcutKey::ArrowLeft,
        Named::ArrowRight => NamedShortcutKey::ArrowRight,
        Named::Home => NamedShortcutKey::Home,
        Named::End => NamedShortcutKey::End,
        Named::PageUp => NamedShortcutKey::PageUp,
        Named::PageDown => NamedShortcutKey::PageDown,
        Named::F1 => NamedShortcutKey::F1,
        Named::F2 => NamedShortcutKey::F2,
        Named::F3 => NamedShortcutKey::F3,
        Named::F4 => NamedShortcutKey::F4,
        Named::F5 => NamedShortcutKey::F5,
        Named::F6 => NamedShortcutKey::F6,
        Named::F7 => NamedShortcutKey::F7,
        Named::F8 => NamedShortcutKey::F8,
        Named::F9 => NamedShortcutKey::F9,
        Named::F10 => NamedShortcutKey::F10,
        Named::F11 => NamedShortcutKey::F11,
        Named::F12 => NamedShortcutKey::F12,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::keyboard::key::{Code, Physical};
    use iced::keyboard::Location;

    fn key_pressed(
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
        repeat: bool,
    ) -> keyboard::Event {
        keyboard::Event::KeyPressed {
            modified_key: key.clone(),
            key,
            physical_key: Physical::Code(Code::KeyK),
            location: Location::Standard,
            modifiers,
            text: None,
            repeat,
        }
    }

    #[test]
    fn matches_character_binding_and_rejects_repeat() {
        let shortcuts = ShortcutMap::new().bind(
            ShortcutBinding::character('k', ShortcutModifiers::CONTROL),
            "open",
        );
        let event = key_pressed(
            keyboard::Key::Character("K".into()),
            keyboard::Modifiers::CTRL,
            false,
        );
        let repeated = key_pressed(
            keyboard::Key::Character("k".into()),
            keyboard::Modifiers::CTRL,
            true,
        );

        assert_eq!(shortcut_message_for_event(&shortcuts, &event), Some("open"));
        assert_eq!(shortcut_message_for_event(&shortcuts, &repeated), None);
    }

    #[test]
    fn matches_supported_named_binding() {
        let shortcuts = ShortcutMap::new().bind(
            ShortcutBinding::named(NamedShortcutKey::Tab, ShortcutModifiers::NONE),
            "next",
        );
        let event = key_pressed(
            keyboard::Key::Named(keyboard::key::Named::Tab),
            keyboard::Modifiers::NONE,
            false,
        );

        assert_eq!(shortcut_message_for_event(&shortcuts, &event), Some("next"));
    }

    #[test]
    fn logical_primary_matches_platform_modifier() {
        let shortcuts =
            ShortcutMap::new().bind(ShortcutBinding::primary_character('k'), "commands");
        let platform_modifier = if cfg!(target_os = "macos") {
            keyboard::Modifiers::COMMAND
        } else {
            keyboard::Modifiers::CTRL
        };
        let event = key_pressed(
            keyboard::Key::Character("k".into()),
            platform_modifier,
            false,
        );

        assert_eq!(
            shortcut_message_for_event(&shortcuts, &event),
            Some("commands")
        );
    }
}
