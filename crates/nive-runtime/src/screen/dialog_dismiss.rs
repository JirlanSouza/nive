use iced::{keyboard, Event};

/// Composable Dialog dismissal policy: independent, optional messages for
/// backdrop activation and Escape. `Default`/[`DialogDismiss::none`]
/// configures neither route. [`DialogRequest::dismiss`](super::DialogRequest::dismiss)
/// replaces the complete policy; [`with_backdrop`](Self::with_backdrop) and
/// [`with_escape`](Self::with_escape) each replace only their own route and
/// preserve the other, fixing the previous exhaustive-enum shape where
/// chaining `dismiss_on_backdrop(...)` after `dismiss_on_escape(...)` (or
/// vice versa) silently discarded the first configured route.
///
/// Non-exhaustive so a future dismissal route can be added without becoming
/// a breaking change for downstream code that only uses the documented
/// constructors, builders, and accessors:
///
/// ```compile_fail
/// use nive_runtime::DialogDismiss;
///
/// let _ = DialogDismiss::OnBackdrop(());
/// ```
///
/// ```compile_fail
/// use nive_runtime::DialogDismiss;
///
/// let _ = DialogDismiss {
///     backdrop: Some(()),
///     escape: None,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct DialogDismiss<Message> {
    backdrop: Option<Message>,
    escape: Option<Message>,
}

impl<Message> DialogDismiss<Message> {
    /// No dismissal route configured. Equivalent to `Default::default()`.
    pub fn none() -> Self {
        Self {
            backdrop: None,
            escape: None,
        }
    }

    /// Only backdrop activation dismisses.
    pub fn backdrop(message: Message) -> Self {
        Self {
            backdrop: Some(message),
            escape: None,
        }
    }

    /// Only Escape dismisses.
    pub fn escape(message: Message) -> Self {
        Self {
            backdrop: None,
            escape: Some(message),
        }
    }

    /// Both backdrop activation and Escape dismiss with the same message.
    pub fn backdrop_or_escape(message: Message) -> Self
    where
        Message: Clone,
    {
        Self {
            backdrop: Some(message.clone()),
            escape: Some(message),
        }
    }

    /// Replaces only the backdrop route, preserving the Escape route.
    pub fn with_backdrop(mut self, message: Message) -> Self {
        self.backdrop = Some(message);
        self
    }

    /// Replaces only the Escape route, preserving the backdrop route.
    pub fn with_escape(mut self, message: Message) -> Self {
        self.escape = Some(message);
        self
    }
}

impl<Message: Clone> DialogDismiss<Message> {
    pub fn on_backdrop(&self) -> Option<Message> {
        self.backdrop.clone()
    }

    pub fn on_escape(&self) -> Option<Message> {
        self.escape.clone()
    }
}

impl<Message> DialogDismiss<Message> {
    pub fn map<T>(self, map_message: impl Fn(Message) -> T + Copy) -> DialogDismiss<T> {
        DialogDismiss {
            backdrop: self.backdrop.map(map_message),
            escape: self.escape.map(map_message),
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

    #[test]
    fn none_configures_neither_route() {
        let dismiss = DialogDismiss::<u8>::none();

        assert_eq!(dismiss.on_backdrop(), None);
        assert_eq!(dismiss.on_escape(), None);
        assert_eq!(dismiss, DialogDismiss::default());
    }

    #[test]
    fn backdrop_or_escape_returns_message_for_both_paths() {
        let dismiss = DialogDismiss::backdrop_or_escape(7_u8);

        assert_eq!(dismiss.on_backdrop(), Some(7));
        assert_eq!(dismiss.on_escape(), Some(7));
    }

    #[test]
    fn independent_builders_preserve_distinct_routes() {
        let dismiss = DialogDismiss::none()
            .with_backdrop("backdrop")
            .with_escape("cancel");

        assert_eq!(dismiss.on_backdrop(), Some("backdrop"));
        assert_eq!(dismiss.on_escape(), Some("cancel"));
    }

    #[test]
    fn with_backdrop_does_not_erase_an_existing_escape_route() {
        let dismiss = DialogDismiss::escape("cancel").with_backdrop("backdrop");

        assert_eq!(dismiss.on_backdrop(), Some("backdrop"));
        assert_eq!(dismiss.on_escape(), Some("cancel"));
    }

    #[test]
    fn with_escape_does_not_erase_an_existing_backdrop_route() {
        let dismiss = DialogDismiss::backdrop("backdrop").with_escape("cancel");

        assert_eq!(dismiss.on_backdrop(), Some("backdrop"));
        assert_eq!(dismiss.on_escape(), Some("cancel"));
    }

    #[test]
    fn distinct_messages_are_supported_on_both_routes() {
        let dismiss = DialogDismiss::backdrop(1_u8).with_escape(2_u8);

        assert_eq!(dismiss.on_backdrop(), Some(1));
        assert_eq!(dismiss.on_escape(), Some(2));
    }

    #[test]
    fn map_maps_both_configured_routes_exactly_once() {
        let dismiss = DialogDismiss::backdrop(1_u8).with_escape(2_u8);
        let mapped = dismiss.map(|value| value * 10);

        assert_eq!(mapped.on_backdrop(), Some(10));
        assert_eq!(mapped.on_escape(), Some(20));
    }

    #[test]
    fn map_preserves_an_absent_route_as_absent() {
        let dismiss = DialogDismiss::backdrop(1_u8);
        let mapped = dismiss.map(|value| value * 10);

        assert_eq!(mapped.on_backdrop(), Some(10));
        assert_eq!(mapped.on_escape(), None);
    }

    #[test]
    fn escape_key_event_is_detected() {
        let event = Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            modified_key: keyboard::Key::Named(keyboard::key::Named::Escape),
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Escape),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::default(),
            text: None,
            repeat: false,
        });

        assert!(is_escape_key_press(&event));
    }
}
