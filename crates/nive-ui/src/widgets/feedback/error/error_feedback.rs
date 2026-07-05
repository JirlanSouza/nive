use iced::{
    widget::{column, row},
    Alignment, Length,
};

use crate::{
    theme::{self, ControlSize, GapRole},
    widgets::{
        controls::button,
        primitives::{text, IconRole},
    },
    Element,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorFeedbackCommandRole {
    Primary,
    Secondary,
}

pub enum ErrorFeedbackAction<'a, Message> {
    Command {
        label: &'a str,
        on_press: Message,
        role: ErrorFeedbackCommandRole,
        disabled: bool,
    },
    Details {
        on_press: Message,
    },
    Dismiss {
        on_press: Message,
    },
    Custom(Element<'a, Message>),
}

pub struct ErrorFeedbackActionRow<'a, Message> {
    actions: Vec<ErrorFeedbackAction<'a, Message>>,
    size: ControlSize,
}

pub struct ErrorFeedback<'a, Message> {
    title: &'a str,
    description: Option<&'a str>,
    message: Option<&'a str>,
    actions: Vec<ErrorFeedbackAction<'a, Message>>,
}

impl<'a, Message> ErrorFeedbackAction<'a, Message> {
    pub fn retry(label: &'a str, on_press: Message) -> Self {
        Self::Command {
            label,
            on_press,
            role: ErrorFeedbackCommandRole::Secondary,
            disabled: false,
        }
    }

    pub fn primary_retry(label: &'a str, on_press: Message) -> Self {
        Self::Command {
            label,
            on_press,
            role: ErrorFeedbackCommandRole::Primary,
            disabled: false,
        }
    }

    pub fn details(on_press: Message) -> Self {
        Self::Details { on_press }
    }

    pub fn dismiss(on_press: Message) -> Self {
        Self::Dismiss { on_press }
    }

    pub fn custom(action: impl Into<Element<'a, Message>>) -> Self {
        Self::Custom(action.into())
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        if let Self::Command {
            disabled: state, ..
        } = &mut self
        {
            *state = disabled;
        }

        self
    }

    fn into_element(self, size: ControlSize) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        match self {
            Self::Command {
                label,
                on_press,
                role,
                disabled,
            } => match role {
                ErrorFeedbackCommandRole::Primary => button::primary(label)
                    .size(size)
                    .shrink()
                    .disabled(disabled)
                    .on_press(on_press)
                    .into(),
                ErrorFeedbackCommandRole::Secondary => button::secondary(label)
                    .size(size)
                    .shrink()
                    .disabled(disabled)
                    .on_press(on_press)
                    .into(),
            },
            Self::Details { on_press } => button::link("Details...")
                .size(size)
                .shrink()
                .on_press(on_press)
                .into(),
            Self::Dismiss { on_press } => button::icon(IconRole::WindowClose)
                .xs()
                .tooltip("Dismiss")
                .on_press(on_press)
                .into(),
            Self::Custom(action) => action,
        }
    }
}

impl<'a, Message> From<Element<'a, Message>> for ErrorFeedbackAction<'a, Message> {
    fn from(action: Element<'a, Message>) -> Self {
        Self::Custom(action)
    }
}

impl<'a, Message> ErrorFeedbackActionRow<'a, Message> {
    pub fn new(actions: impl IntoIterator<Item = ErrorFeedbackAction<'a, Message>>) -> Self {
        Self {
            actions: actions.into_iter().collect(),
            size: ControlSize::Sm,
        }
    }

    pub fn xs(mut self) -> Self {
        self.size = ControlSize::Xs;
        self
    }

    pub fn sm(mut self) -> Self {
        self.size = ControlSize::Sm;
        self
    }

    pub fn push(mut self, action: impl Into<ErrorFeedbackAction<'a, Message>>) -> Self {
        self.actions.push(action.into());
        self
    }

    fn into_element(self) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        let actions = self
            .actions
            .into_iter()
            .map(|action| action.into_element(self.size))
            .collect::<Vec<_>>();

        row(actions)
            .spacing(theme::gap(GapRole::Related))
            .align_y(Alignment::Center)
            .into()
    }
}

impl<'a, Message> From<ErrorFeedbackActionRow<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(actions: ErrorFeedbackActionRow<'a, Message>) -> Self {
        actions.into_element()
    }
}

impl<'a, Message> ErrorFeedback<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            description: None,
            message: None,
            actions: Vec::new(),
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn message(mut self, message: &'a str) -> Self {
        self.message = Some(message);
        self
    }

    pub fn body(self, body: &'a str) -> Self {
        self.message(body)
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
        let mut content = column![text::label_strong(self.title)]
            .spacing(theme::gap(GapRole::Tight))
            .width(Length::Fill);

        if let Some(description) = self.description {
            content = content.push(text::caption(description));
        }

        if let Some(message) = self.message {
            content = content.push(text::caption(message).width(Length::Fill));
        }

        if !self.actions.is_empty() {
            content = content.push(ErrorFeedbackActionRow::new(self.actions));
        }

        content.into()
    }
}

impl<'a, Message> From<ErrorFeedback<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(feedback: ErrorFeedback<'a, Message>) -> Self {
        feedback.into_element()
    }
}
