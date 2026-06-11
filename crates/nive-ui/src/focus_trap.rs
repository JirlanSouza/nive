use iced::{
    advanced::widget::{operation, Operation},
    keyboard::{self, key::Named, Key, Modifiers},
    Event,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    Next,
    Previous,
}

impl FocusDirection {
    pub fn operate(self, mut target: impl FnMut(&mut dyn Operation)) {
        let mut operation: Box<dyn Operation> = match self {
            Self::Next => Box::new(operation::focusable::focus_next::<()>()),
            Self::Previous => Box::new(operation::focusable::focus_previous::<()>()),
        };

        loop {
            target(operation.as_mut());

            match operation.finish() {
                operation::Outcome::Chain(next) => {
                    operation = next;
                }
                operation::Outcome::None | operation::Outcome::Some(()) => break,
            }
        }
    }
}

pub fn direction_from_event(event: &Event) -> Option<FocusDirection> {
    let Event::Keyboard(event) = event else {
        return None;
    };

    direction_from_keyboard_event(event)
}

pub fn direction_from_keyboard_event(event: &keyboard::Event) -> Option<FocusDirection> {
    let keyboard::Event::KeyPressed {
        key: Key::Named(Named::Tab),
        modifiers,
        ..
    } = event
    else {
        return None;
    };

    direction_from_modifiers(*modifiers)
}

fn direction_from_modifiers(modifiers: Modifiers) -> Option<FocusDirection> {
    if modifiers.control() || modifiers.alt() || modifiers.logo() {
        return None;
    }

    if modifiers.shift() {
        Some(FocusDirection::Previous)
    } else {
        Some(FocusDirection::Next)
    }
}

#[cfg(test)]
mod focus_trap_tests {
    use super::*;
    use iced::advanced::widget::operation::Focusable;
    use iced::keyboard::{
        key::{Code, Physical},
        Location,
    };
    use iced::Rectangle;

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
            direction_from_keyboard_event(&event),
            Some(FocusDirection::Next)
        );
    }

    #[test]
    fn shift_tab_moves_focus_backward() {
        let event = key_pressed(Key::Named(Named::Tab), Modifiers::SHIFT);

        assert_eq!(
            direction_from_keyboard_event(&event),
            Some(FocusDirection::Previous)
        );
    }

    #[test]
    fn modified_tab_keeps_application_shortcuts_available() {
        let event = key_pressed(Key::Named(Named::Tab), Modifiers::CTRL);

        assert_eq!(direction_from_keyboard_event(&event), None);
    }

    #[test]
    fn non_tab_keys_are_ignored() {
        let event = key_pressed(Key::Named(Named::Enter), Modifiers::NONE);

        assert_eq!(direction_from_keyboard_event(&event), None);
    }

    #[test]
    fn focus_next_executes_chained_focus_operation() {
        let mut controls = [TestFocusable::default(), TestFocusable::default()];

        FocusDirection::Next.operate(|operation| {
            for control in &mut controls {
                operation.focusable(None, Rectangle::default(), control);
            }
        });

        assert!(controls[0].focused);
        assert!(!controls[1].focused);

        FocusDirection::Next.operate(|operation| {
            for control in &mut controls {
                operation.focusable(None, Rectangle::default(), control);
            }
        });

        assert!(!controls[0].focused);
        assert!(controls[1].focused);
    }

    #[derive(Default)]
    struct TestFocusable {
        focused: bool,
    }

    impl Focusable for TestFocusable {
        fn is_focused(&self) -> bool {
            self.focused
        }

        fn focus(&mut self) {
            self.focused = true;
        }

        fn unfocus(&mut self) {
            self.focused = false;
        }
    }
}
