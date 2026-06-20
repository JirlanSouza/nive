use iced::{
    widget::{column, container, row, Space},
    Alignment, Length,
};

use crate::{
    theme::{self, ControlSize, GapRole},
    Element,
};

pub struct OperationActionGroup<'a, Message> {
    status: Option<Element<'a, Message>>,
    actions: Vec<Element<'a, Message>>,
}

impl<'a, Message> OperationActionGroup<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new() -> Self {
        Self {
            status: None,
            actions: Vec::new(),
        }
    }

    pub fn status(mut self, status: impl Into<Element<'a, Message>>) -> Self {
        self.status = Some(status.into());
        self
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
        let status = self
            .status
            .unwrap_or_else(|| Space::new().width(Length::Fill).into());
        let status_slot = container(status)
            .align_y(Alignment::Center)
            .height(Length::Fixed(
                theme::control_metrics(ControlSize::Sm).height,
            ))
            .width(Length::Fill);
        let actions = row(self.actions)
            .spacing(theme::gap(GapRole::Related))
            .width(Length::Fill);

        column![status_slot, actions]
            .spacing(theme::gap(GapRole::Tight))
            .width(Length::Fill)
            .into()
    }
}

impl<'a, Message> Default for OperationActionGroup<'a, Message>
where
    Message: Clone + 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> From<OperationActionGroup<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(group: OperationActionGroup<'a, Message>) -> Self {
        group.into_element()
    }
}
