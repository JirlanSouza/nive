use iced::Length;

use crate::{
    widgets::{
        primitives::text, Dialog, DialogActionFooter, DialogHeader, DialogSize,
        DialogTerminalAction,
    },
    Element,
};

pub struct ErrorDetailsDialog<'a, Message> {
    detail: &'a str,
    on_close: Option<Message>,
}

impl<'a, Message> ErrorDetailsDialog<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(detail: &'a str) -> Self {
        Self {
            detail,
            on_close: None,
        }
    }

    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let mut dialog = Dialog::new(text::code(self.detail).width(Length::Fill))
            .size(DialogSize::Lg)
            .header(
                DialogHeader::new("Error details")
                    .description("This information can help diagnose the problem."),
            );

        if let Some(on_close) = self.on_close {
            dialog = dialog.footer(DialogActionFooter::new(DialogTerminalAction::primary(
                "Close", on_close,
            )));
        }

        dialog.into()
    }
}

impl<'a, Message> From<ErrorDetailsDialog<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(dialog: ErrorDetailsDialog<'a, Message>) -> Self {
        dialog.into_element()
    }
}

#[cfg(test)]
mod error_details_dialog_tests {
    use super::*;
    use iced::{keyboard, Size};

    #[test]
    fn requests_lg_size_in_an_unconstrained_viewport() {
        let node = crate::test_support::layout(
            ErrorDetailsDialog::<()>::new("short trace").into(),
            Size::new(2000.0, 2000.0),
        );

        assert_eq!(node.size().width, 720.0);
    }

    #[test]
    fn absent_close_omits_the_footer_slot() {
        let node = crate::test_support::layout(
            ErrorDetailsDialog::<()>::new("short trace").into(),
            Size::new(2000.0, 2000.0),
        );

        // Header + body only: no footer slot, and no extra nested-Panel
        // level around the body content.
        assert_eq!(node.children().len(), 2);
    }

    #[test]
    fn configured_close_adds_exactly_one_footer_slot() {
        let node = crate::test_support::layout(
            ErrorDetailsDialog::new("short trace")
                .on_close("close")
                .into(),
            Size::new(2000.0, 2000.0),
        );

        assert_eq!(node.children().len(), 3);
    }

    #[test]
    fn long_diagnostic_content_clamps_total_height_and_keeps_header_and_footer_fixed() {
        let long_detail = "line\n".repeat(400);
        let node = crate::test_support::layout(
            ErrorDetailsDialog::new(long_detail.as_str())
                .on_close("close")
                .into(),
            Size::new(720.0, 400.0),
        );

        assert!(node.size().height <= 400.0 + f32::EPSILON);
        let children = node.children();
        assert_eq!(children.len(), 3, "header, body, footer");
        // Header stays at the top and footer stays fixed after the body,
        // regardless of how much the body content overflowed.
        assert_eq!(children[0].bounds().y, 0.0);
        assert!(children[2].bounds().y >= children[1].bounds().y + children[1].bounds().height);
    }

    #[test]
    fn low_viewport_clamps_the_frame_inside_the_safe_margin() {
        let node = crate::test_support::layout(
            ErrorDetailsDialog::<()>::new("short trace").into(),
            Size::new(720.0, 80.0),
        );

        assert!(node.size().height <= 80.0);
    }

    #[test]
    fn close_action_publishes_the_configured_message_exactly_once_on_enter() {
        let key = keyboard::Key::Named(keyboard::key::Named::Enter);
        let event = iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Enter),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::NONE,
            text: None,
            repeat: false,
        });

        let messages = crate::test_support::event_messages(
            ErrorDetailsDialog::new("short trace")
                .on_close("close")
                .into(),
            Size::new(720.0, 600.0),
            event,
        );

        assert_eq!(messages, vec!["close"]);
    }
}
