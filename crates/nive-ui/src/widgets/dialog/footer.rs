use iced::{widget::row, Alignment, Length};

use crate::{
    theme::{self, GapRole},
    Element,
};

pub struct DialogActionFooter<'a, Message> {
    status: Element<'a, Message>,
    actions: Vec<Element<'a, Message>>,
}

impl<'a, Message> DialogActionFooter<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(status: impl Into<Element<'a, Message>>) -> Self {
        Self {
            status: status.into(),
            actions: Vec::new(),
        }
    }

    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.actions.push(action.into());
        self
    }

    pub fn actions(mut self, actions: impl IntoIterator<Item = Element<'a, Message>>) -> Self {
        self.actions.extend(actions);
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let mut content = row![self.status]
            .spacing(theme::gap(GapRole::Related))
            .align_y(Alignment::Center)
            .width(Length::Fill);

        for action in self.actions {
            content = content.push(action);
        }

        content.into()
    }
}

impl<'a, Message> From<DialogActionFooter<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(footer: DialogActionFooter<'a, Message>) -> Self {
        footer.into_element()
    }
}
