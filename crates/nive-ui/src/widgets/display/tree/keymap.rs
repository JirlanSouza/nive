use iced::{
    keyboard::{self, key::Named, Key},
    Event,
};

use crate::interaction::ClickModifiers;

/// Directional/boundary navigation keys recognized by the focused tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TreeNavKey {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
}

/// Keyboard-driven action recognized by the focused tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TreeKeyAction {
    Navigate(TreeNavKey, ClickModifiers),
    Clipboard(TreeClipboardAction),
    Context,
    TypeAhead(char),
    Escape,
}

/// Clipboard intent keys recognized by the focused tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TreeClipboardAction {
    Copy,
    Cut,
    Paste,
}

/// Maps a raw keyboard event to the tree action it represents, if any.
///
/// Only the keys owned by keyboard navigation and type-ahead are recognized
/// here; activation, rename, and context-menu keys are resolved elsewhere.
pub(super) fn key_action(event: &Event) -> Option<TreeKeyAction> {
    let Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        modifiers,
        text,
        repeat: false,
        ..
    }) = event
    else {
        return None;
    };

    let click_modifiers = ClickModifiers::new(modifiers.command(), modifiers.shift());

    if modifiers.command() {
        match key {
            Key::Character(ch) if ch.eq_ignore_ascii_case("c") => {
                return Some(TreeKeyAction::Clipboard(TreeClipboardAction::Copy));
            }
            Key::Character(ch) if ch.eq_ignore_ascii_case("x") => {
                return Some(TreeKeyAction::Clipboard(TreeClipboardAction::Cut));
            }
            Key::Character(ch) if ch.eq_ignore_ascii_case("v") => {
                return Some(TreeKeyAction::Clipboard(TreeClipboardAction::Paste));
            }
            _ => {}
        }
    }

    if matches!(key, Key::Named(Named::ContextMenu))
        || matches!(key, Key::Named(Named::F10)) && modifiers.shift()
    {
        return Some(TreeKeyAction::Context);
    }

    let nav_key = match key {
        Key::Named(Named::ArrowUp) => Some(TreeNavKey::Up),
        Key::Named(Named::ArrowDown) => Some(TreeNavKey::Down),
        Key::Named(Named::ArrowLeft) => Some(TreeNavKey::Left),
        Key::Named(Named::ArrowRight) => Some(TreeNavKey::Right),
        Key::Named(Named::Home) => Some(TreeNavKey::Home),
        Key::Named(Named::End) => Some(TreeNavKey::End),
        Key::Named(Named::PageUp) => Some(TreeNavKey::PageUp),
        Key::Named(Named::PageDown) => Some(TreeNavKey::PageDown),
        _ => None,
    };

    if let Some(nav_key) = nav_key {
        return Some(TreeKeyAction::Navigate(nav_key, click_modifiers));
    }

    if matches!(key, Key::Named(Named::Escape)) {
        return Some(TreeKeyAction::Escape);
    }

    if modifiers.command() || modifiers.alt() {
        return None;
    }

    let ch = text
        .as_ref()
        .and_then(|text| text.chars().next())
        .filter(|ch| !ch.is_control())?;

    Some(TreeKeyAction::TypeAhead(ch))
}

#[cfg(test)]
mod tree_keymap_tests {
    use super::*;
    use iced::keyboard::{
        key::{Code, Physical},
        Location, Modifiers,
    };

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

    #[test]
    fn arrow_keys_map_to_navigation() {
        assert_eq!(
            key_action(&key_pressed(
                Key::Named(Named::ArrowDown),
                Modifiers::NONE,
                None
            )),
            Some(TreeKeyAction::Navigate(
                TreeNavKey::Down,
                ClickModifiers::NONE
            ))
        );
        assert_eq!(
            key_action(&key_pressed(
                Key::Named(Named::ArrowUp),
                Modifiers::NONE,
                None
            )),
            Some(TreeKeyAction::Navigate(
                TreeNavKey::Up,
                ClickModifiers::NONE
            ))
        );
        assert_eq!(
            key_action(&key_pressed(
                Key::Named(Named::ArrowLeft),
                Modifiers::NONE,
                None
            )),
            Some(TreeKeyAction::Navigate(
                TreeNavKey::Left,
                ClickModifiers::NONE
            ))
        );
        assert_eq!(
            key_action(&key_pressed(
                Key::Named(Named::ArrowRight),
                Modifiers::NONE,
                None
            )),
            Some(TreeKeyAction::Navigate(
                TreeNavKey::Right,
                ClickModifiers::NONE
            ))
        );
    }

    #[test]
    fn boundary_and_page_keys_map_to_navigation() {
        assert_eq!(
            key_action(&key_pressed(Key::Named(Named::Home), Modifiers::NONE, None)),
            Some(TreeKeyAction::Navigate(
                TreeNavKey::Home,
                ClickModifiers::NONE
            ))
        );
        assert_eq!(
            key_action(&key_pressed(Key::Named(Named::End), Modifiers::NONE, None)),
            Some(TreeKeyAction::Navigate(
                TreeNavKey::End,
                ClickModifiers::NONE
            ))
        );
        assert_eq!(
            key_action(&key_pressed(
                Key::Named(Named::PageUp),
                Modifiers::NONE,
                None
            )),
            Some(TreeKeyAction::Navigate(
                TreeNavKey::PageUp,
                ClickModifiers::NONE
            ))
        );
        assert_eq!(
            key_action(&key_pressed(
                Key::Named(Named::PageDown),
                Modifiers::NONE,
                None
            )),
            Some(TreeKeyAction::Navigate(
                TreeNavKey::PageDown,
                ClickModifiers::NONE
            ))
        );
    }

    #[test]
    fn shift_modifier_is_captured_for_range_selection() {
        assert_eq!(
            key_action(&key_pressed(
                Key::Named(Named::ArrowDown),
                Modifiers::SHIFT,
                None
            )),
            Some(TreeKeyAction::Navigate(
                TreeNavKey::Down,
                ClickModifiers::new(false, true)
            ))
        );
    }

    #[test]
    fn escape_maps_to_escape_action() {
        assert_eq!(
            key_action(&key_pressed(
                Key::Named(Named::Escape),
                Modifiers::NONE,
                None
            )),
            Some(TreeKeyAction::Escape)
        );
    }

    #[test]
    fn printable_character_maps_to_type_ahead() {
        assert_eq!(
            key_action(&key_pressed(
                Key::Character("r".into()),
                Modifiers::NONE,
                Some("r")
            )),
            Some(TreeKeyAction::TypeAhead('r'))
        );
    }

    #[test]
    fn command_modifier_suppresses_type_ahead() {
        assert_eq!(
            key_action(&key_pressed(
                Key::Character("z".into()),
                Modifiers::COMMAND,
                Some("z")
            )),
            None
        );
    }

    #[test]
    fn command_clipboard_keys_map_to_clipboard_actions() {
        assert_eq!(
            key_action(&key_pressed(
                Key::Character("c".into()),
                Modifiers::COMMAND,
                Some("c")
            )),
            Some(TreeKeyAction::Clipboard(TreeClipboardAction::Copy))
        );
        assert_eq!(
            key_action(&key_pressed(
                Key::Character("x".into()),
                Modifiers::COMMAND,
                Some("x")
            )),
            Some(TreeKeyAction::Clipboard(TreeClipboardAction::Cut))
        );
        assert_eq!(
            key_action(&key_pressed(
                Key::Character("v".into()),
                Modifiers::COMMAND,
                Some("v")
            )),
            Some(TreeKeyAction::Clipboard(TreeClipboardAction::Paste))
        );
    }

    #[test]
    fn context_menu_keys_map_to_context_action() {
        assert_eq!(
            key_action(&key_pressed(
                Key::Named(Named::ContextMenu),
                Modifiers::NONE,
                None
            )),
            Some(TreeKeyAction::Context)
        );
        assert_eq!(
            key_action(&key_pressed(Key::Named(Named::F10), Modifiers::SHIFT, None)),
            Some(TreeKeyAction::Context)
        );
    }

    #[test]
    fn repeated_key_press_is_ignored() {
        let event = Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(Named::ArrowDown),
            modified_key: Key::Named(Named::ArrowDown),
            physical_key: Physical::Code(Code::ArrowDown),
            location: Location::Standard,
            modifiers: Modifiers::NONE,
            text: None,
            repeat: true,
        });

        assert_eq!(key_action(&event), None);
    }

    #[test]
    fn key_release_is_ignored() {
        let event = Event::Keyboard(keyboard::Event::KeyReleased {
            key: Key::Named(Named::ArrowDown),
            modified_key: Key::Named(Named::ArrowDown),
            physical_key: Physical::Code(Code::ArrowDown),
            location: Location::Standard,
            modifiers: Modifiers::NONE,
        });

        assert_eq!(key_action(&event), None);
    }
}
