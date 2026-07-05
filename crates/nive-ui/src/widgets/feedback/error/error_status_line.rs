use iced::{widget::row, Alignment, Length};

use crate::{
    theme::{self, GapRole, ToneRole},
    widgets::primitives::text,
    Element,
};

use super::{ErrorFeedbackAction, ErrorFeedbackActionRow};

pub struct ErrorStatusLine<'a, Message> {
    message: &'a str,
    actions: Vec<ErrorFeedbackAction<'a, Message>>,
}

impl<'a, Message> ErrorStatusLine<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            actions: Vec::new(),
        }
    }

    pub fn action(mut self, action: impl Into<ErrorFeedbackAction<'a, Message>>) -> Self {
        self.actions.push(action.into());
        self
    }

    pub fn actions(
        mut self,
        actions: impl IntoIterator<Item = ErrorFeedbackAction<'a, Message>>,
    ) -> Self {
        self.actions.extend(actions);
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let mut content =
            row![text::caption(self.message).style(theme::text::tone(ToneRole::Danger))]
                .spacing(theme::gap(GapRole::Related))
                .align_y(Alignment::Center)
                .width(Length::Fill);

        if !self.actions.is_empty() {
            content = content.push(ErrorFeedbackActionRow::new(self.actions));
        }

        content.into()
    }
}

impl<'a, Message> From<ErrorStatusLine<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(status: ErrorStatusLine<'a, Message>) -> Self {
        status.into_element()
    }
}
