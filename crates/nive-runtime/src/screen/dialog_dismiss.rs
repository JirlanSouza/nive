use iced::{keyboard, Event};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DialogDismiss<Message> {
    #[default]
    None,
    OnEscape(Message),
    OnBackdrop(Message),
    OnBackdropOrEscape(Message),
}

impl<Message: Clone> DialogDismiss<Message> {
    pub fn on_backdrop(&self) -> Option<Message> {
        match self {
            Self::OnBackdrop(message) | Self::OnBackdropOrEscape(message) => Some(message.clone()),
            Self::None | Self::OnEscape(_) => None,
        }
    }

    pub fn on_escape(&self) -> Option<Message> {
        match self {
            Self::OnEscape(message) | Self::OnBackdropOrEscape(message) => Some(message.clone()),
            Self::None | Self::OnBackdrop(_) => None,
        }
    }
}

impl<Message> DialogDismiss<Message> {
    pub fn map<T>(self, map_message: impl Fn(Message) -> T + Copy) -> DialogDismiss<T> {
        match self {
            Self::None => DialogDismiss::None,
            Self::OnBackdrop(message) => DialogDismiss::OnBackdrop(map_message(message)),
            Self::OnEscape(message) => DialogDismiss::OnEscape(map_message(message)),
            Self::OnBackdropOrEscape(message) => {
                DialogDismiss::OnBackdropOrEscape(map_message(message))
            }
        }
    }
}

pub fn is_escape_key_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            ..
        })
    )
}

#[cfg(test)]
mod dialog_dismiss_tests {
    use super::*;
    use iced::keyboard::{self, Modifiers};

    #[test]
    fn backdrop_or_escape_returns_message_for_both_paths() {
        let dismiss = DialogDismiss::OnBackdropOrEscape(7_u8);

        assert_eq!(dismiss.on_backdrop(), Some(7));
        assert_eq!(dismiss.on_escape(), Some(7));
    }

    #[test]
    fn escape_key_event_is_detected() {
        let event = Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            modified_key: keyboard::Key::Named(keyboard::key::Named::Escape),
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Escape),
            location: keyboard::Location::Standard,
            modifiers: Modifiers::default(),
            text: None,
            repeat: false,
        });

        assert!(is_escape_key_press(&event));
    }
}
