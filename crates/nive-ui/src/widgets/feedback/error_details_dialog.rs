use iced::{
    widget::{column, row, scrollable, Space},
    Alignment, Length,
};

use crate::{
    theme::{self, GapRole, PaddingRole, SurfaceRole},
    widgets::{button, text, Dialog, DialogFooter, DialogHeader, Panel},
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
        let close = button::primary("Close")
            .sm()
            .shrink()
            .on_press_maybe(self.on_close);

        let body = column![
            DialogHeader::new("Error details")
                .description("This information can help diagnose the problem."),
            scrollable(
                Panel::new(text::code(self.detail).width(Length::Fill))
                    .role(SurfaceRole::Elevated)
                    .padding(theme::padding(PaddingRole::Content))
                    .width(Length::Fill),
            )
            .height(Length::Shrink),
            DialogFooter::new(
                row![Space::new().width(Length::Fill), close]
                    .align_y(Alignment::Center)
                    .width(Length::Fill),
            ),
        ]
        .spacing(theme::gap(GapRole::Section));

        Dialog::new(body).into()
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
