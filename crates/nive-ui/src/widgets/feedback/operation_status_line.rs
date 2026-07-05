use iced::{
    widget::{container, Space},
    Length,
};

use crate::{widgets::Spinner, Element};

use super::{ErrorFeedbackAction, ErrorStatusLine, OperationStatusPresentation};

pub struct OperationStatusLine<'a, Message> {
    state: OperationStatusLineState<'a>,
    actions: Vec<ErrorFeedbackAction<'a, Message>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperationStatusLineState<'a> {
    Idle,
    Running { label: &'a str },
    Error { message: &'a str },
}

impl<'a, Message> OperationStatusLine<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn idle() -> Self {
        Self {
            state: OperationStatusLineState::Idle,
            actions: Vec::new(),
        }
    }

    pub fn running(label: &'a str) -> Self {
        Self {
            state: OperationStatusLineState::Running { label },
            actions: Vec::new(),
        }
    }

    pub fn error(message: &'a str) -> Self {
        Self {
            state: OperationStatusLineState::Error { message },
            actions: Vec::new(),
        }
    }

    pub fn from_presentation(
        presentation: &impl OperationStatusPresentation,
        running_label: &'a str,
        error_message: &'a str,
    ) -> Self {
        if presentation.is_running() {
            Self::running(running_label)
        } else if presentation.error().is_some() {
            Self::error(error_message)
        } else {
            Self::idle()
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
        match self.state {
            OperationStatusLineState::Idle => Space::new().width(Length::Fill).into(),
            OperationStatusLineState::Running { label } => {
                container(Spinner::new().neutral().xs().label(label))
                    .width(Length::Fill)
                    .into()
            }
            OperationStatusLineState::Error { message } => {
                ErrorStatusLine::new(message).actions(self.actions).into()
            }
        }
    }
}

impl<'a, Message> From<OperationStatusLine<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(status: OperationStatusLine<'a, Message>) -> Self {
        status.into_element()
    }
}
