use iced::{keyboard, Event};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogDismiss<Message> {
    Blocked,
    Backdrop(Message),
    Escape(Message),
    BackdropOrEscape(Message),
}

impl<Message> Default for DialogDismiss<Message> {
    fn default() -> Self {
        Self::Blocked
    }
}

impl<Message: Clone> DialogDismiss<Message> {
    pub fn on_backdrop(&self) -> Option<Message> {
        match self {
            Self::Backdrop(message) | Self::BackdropOrEscape(message) => Some(message.clone()),
            Self::Blocked | Self::Escape(_) => None,
        }
    }

    pub fn on_escape(&self) -> Option<Message> {
        match self {
            Self::Escape(message) | Self::BackdropOrEscape(message) => Some(message.clone()),
            Self::Blocked | Self::Backdrop(_) => None,
        }
    }
}

impl<Message> DialogDismiss<Message> {
    pub fn map<T>(self, map_message: impl Fn(Message) -> T + Copy) -> DialogDismiss<T> {
        match self {
            Self::Blocked => DialogDismiss::Blocked,
            Self::Backdrop(message) => DialogDismiss::Backdrop(map_message(message)),
            Self::Escape(message) => DialogDismiss::Escape(map_message(message)),
            Self::BackdropOrEscape(message) => {
                DialogDismiss::BackdropOrEscape(map_message(message))
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
        let dismiss = DialogDismiss::BackdropOrEscape(7_u8);

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
