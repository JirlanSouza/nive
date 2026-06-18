use iced::{keyboard, Subscription, Task};

use nive_ui::focus_trap::{self, FocusDirection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardNavigation {
    FocusNext,
    FocusPrevious,
}

impl KeyboardNavigation {
    pub fn task<Message>(self) -> Task<Message> {
        match self {
            Self::FocusNext => iced::widget::operation::focus_next(),
            Self::FocusPrevious => iced::widget::operation::focus_previous(),
        }
    }
}

pub fn keyboard_navigation_subscription() -> Subscription<KeyboardNavigation> {
    keyboard::listen().filter_map(navigation_from_event)
}

fn navigation_from_event(event: keyboard::Event) -> Option<KeyboardNavigation> {
    focus_trap::direction_from_keyboard_event(&event).map(KeyboardNavigation::from)
}

impl From<FocusDirection> for KeyboardNavigation {
    fn from(direction: FocusDirection) -> Self {
        match direction {
            FocusDirection::Next => Self::FocusNext,
            FocusDirection::Previous => Self::FocusPrevious,
        }
    }
}

#[cfg(test)]
mod keyboard_navigation_tests {
    use super::*;
    use iced::keyboard::{
        key::{Code, Named, Physical},
        Key, Location, Modifiers,
    };

    fn key_pressed(key: Key, modifiers: Modifiers) -> keyboard::Event {
        keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Code(Code::Tab),
            location: Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        }
    }

    #[test]
    fn tab_moves_focus_forward() {
        let event = key_pressed(Key::Named(Named::Tab), Modifiers::NONE);

        assert_eq!(
            navigation_from_event(event),
            Some(KeyboardNavigation::FocusNext)
        );
    }

    #[test]
    fn shift_tab_moves_focus_backward() {
        let event = key_pressed(Key::Named(Named::Tab), Modifiers::SHIFT);

        assert_eq!(
            navigation_from_event(event),
            Some(KeyboardNavigation::FocusPrevious)
        );
    }

    #[test]
    fn modified_tab_keeps_product_shortcuts_available() {
        let event = key_pressed(Key::Named(Named::Tab), Modifiers::CTRL);

        assert_eq!(navigation_from_event(event), None);
    }

    #[test]
    fn non_tab_keys_are_ignored() {
        let event = key_pressed(Key::Named(Named::Enter), Modifiers::NONE);

        assert_eq!(navigation_from_event(event), None);
    }
}
