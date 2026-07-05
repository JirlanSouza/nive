use iced::{
    keyboard::{self, key},
    Event,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AutocompleteNavigation {
    Previous,
    Next,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AutocompleteKeyAction {
    Navigate(AutocompleteNavigation),
    SelectHighlighted,
    Dismiss,
}

pub(super) fn key_action(event: &Event) -> Option<AutocompleteKeyAction> {
    match event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::ArrowUp),
            ..
        }) => Some(AutocompleteKeyAction::Navigate(
            AutocompleteNavigation::Previous,
        )),
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::ArrowDown),
            ..
        }) => Some(AutocompleteKeyAction::Navigate(
            AutocompleteNavigation::Next,
        )),
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::Enter),
            repeat: false,
            ..
        }) => Some(AutocompleteKeyAction::SelectHighlighted),
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::Escape),
            repeat: false,
            ..
        }) => Some(AutocompleteKeyAction::Dismiss),
        _ => None,
    }
}

pub(super) fn navigate_highlight(
    current: Option<usize>,
    item_count: usize,
    direction: AutocompleteNavigation,
) -> Option<usize> {
    if item_count == 0 {
        return None;
    }

    match (direction, current.filter(|index| *index < item_count)) {
        (AutocompleteNavigation::Next, Some(index)) => Some((index + 1) % item_count),
        (AutocompleteNavigation::Next, None) => Some(0),
        (AutocompleteNavigation::Previous, Some(0)) => Some(item_count - 1),
        (AutocompleteNavigation::Previous, Some(index)) => Some(index - 1),
        (AutocompleteNavigation::Previous, None) => Some(item_count - 1),
    }
}

#[cfg(test)]
mod keyboard_tests {
    use iced::keyboard::{
        key::{Code, Named, Physical},
        Key, Location, Modifiers,
    };

    use super::*;

    fn key_pressed(key: Key, repeat: bool) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: Physical::Code(Code::Enter),
            location: Location::Standard,
            modifiers: Modifiers::NONE,
            text: None,
            repeat,
        })
    }

    #[test]
    fn arrow_up_maps_to_previous_navigation() {
        assert_eq!(
            key_action(&key_pressed(Key::Named(Named::ArrowUp), false)),
            Some(AutocompleteKeyAction::Navigate(
                AutocompleteNavigation::Previous
            ))
        );
    }

    #[test]
    fn arrow_down_maps_to_next_navigation() {
        assert_eq!(
            key_action(&key_pressed(Key::Named(Named::ArrowDown), false)),
            Some(AutocompleteKeyAction::Navigate(
                AutocompleteNavigation::Next
            ))
        );
    }

    #[test]
    fn enter_maps_to_select_highlighted() {
        assert_eq!(
            key_action(&key_pressed(Key::Named(Named::Enter), false)),
            Some(AutocompleteKeyAction::SelectHighlighted)
        );
    }

    #[test]
    fn repeated_enter_does_not_select() {
        assert_eq!(
            key_action(&key_pressed(Key::Named(Named::Enter), true)),
            None
        );
    }

    #[test]
    fn escape_maps_to_dismiss() {
        assert_eq!(
            key_action(&key_pressed(Key::Named(Named::Escape), false)),
            Some(AutocompleteKeyAction::Dismiss)
        );
    }

    #[test]
    fn next_navigation_moves_highlight_forward() {
        assert_eq!(
            navigate_highlight(Some(0), 3, AutocompleteNavigation::Next),
            Some(1)
        );
    }

    #[test]
    fn previous_navigation_wraps_highlight() {
        assert_eq!(
            navigate_highlight(Some(0), 3, AutocompleteNavigation::Previous),
            Some(2)
        );
    }

    #[test]
    fn navigation_without_items_has_no_highlight() {
        assert_eq!(
            navigate_highlight(Some(0), 0, AutocompleteNavigation::Next),
            None
        );
    }
}
