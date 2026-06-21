use crate::{
    widgets::{EmptyState, IconName},
    Element,
};

use super::{ErrorFeedbackAction, ErrorFeedbackActionRow};

pub struct ErrorEmptyState<'a, Message> {
    title: &'a str,
    description: Option<&'a str>,
    icon: Option<IconName>,
    actions: Vec<ErrorFeedbackAction<'a, Message>>,
}

impl<'a, Message> ErrorEmptyState<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            description: None,
            icon: None,
            actions: Vec::new(),
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
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
        let mut empty_state = EmptyState::new(self.title);

        if let Some(description) = self.description {
            empty_state = empty_state.description(description);
        }

        if let Some(icon) = self.icon {
            empty_state = empty_state.icon(icon);
        }

        if !self.actions.is_empty() {
            empty_state = empty_state.action(ErrorFeedbackActionRow::new(self.actions));
        }

        empty_state.into()
    }
}

impl<'a, Message> From<ErrorEmptyState<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(feedback: ErrorEmptyState<'a, Message>) -> Self {
        feedback.into_element()
    }
}
