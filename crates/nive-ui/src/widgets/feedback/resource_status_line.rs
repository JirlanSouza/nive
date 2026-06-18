use iced::{
    widget::{row, Space},
    Alignment, Length,
};

use crate::{
    theme::{self, GapRole, ToneRole},
    widgets::{text, LoadingIndicator},
    Element,
};

use super::{ErrorFeedbackAction, ErrorFeedbackActionRow, ResourceStatusPresentation};

pub struct ResourceStatusLine<'a, Message> {
    state: ResourceStatusLineState<'a>,
    actions: Vec<ErrorFeedbackAction<'a, Message>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceStatusLineState<'a> {
    Idle,
    Refreshing { label: &'a str },
    StaleError { message: &'a str },
}

impl<'a, Message> ResourceStatusLine<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn idle() -> Self {
        Self {
            state: ResourceStatusLineState::Idle,
            actions: Vec::new(),
        }
    }

    pub fn refreshing(label: &'a str) -> Self {
        Self {
            state: ResourceStatusLineState::Refreshing { label },
            actions: Vec::new(),
        }
    }

    pub fn stale_error(message: &'a str) -> Self {
        Self {
            state: ResourceStatusLineState::StaleError { message },
            actions: Vec::new(),
        }
    }

    pub fn from_presentation(
        presentation: &impl ResourceStatusPresentation,
        refreshing_label: &'a str,
        error_message: &'a str,
    ) -> Self {
        if presentation.is_refreshing() && presentation.has_value() {
            Self::refreshing(refreshing_label)
        } else if presentation.error().is_some() && presentation.has_value() {
            Self::stale_error(error_message)
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
            ResourceStatusLineState::Idle => Space::new().width(Length::Shrink).into(),
            ResourceStatusLineState::Refreshing { label } => {
                LoadingIndicator::new().neutral().xs().label(label).into()
            }
            ResourceStatusLineState::StaleError { message } => {
                let mut content =
                    row![text::caption(message).style(theme::text::tone(ToneRole::Danger))]
                        .spacing(theme::gap(GapRole::Related))
                        .align_y(Alignment::Center)
                        .width(Length::Shrink);

                if !self.actions.is_empty() {
                    content = content.push(ErrorFeedbackActionRow::new(self.actions).xs());
                }

                content.into()
            }
        }
    }
}

impl<'a, Message> From<ResourceStatusLine<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(status: ResourceStatusLine<'a, Message>) -> Self {
        status.into_element()
    }
}

#[cfg(test)]
mod resource_status_line_tests {
    use super::*;

    struct Presentation {
        refreshing: bool,
        has_value: bool,
        has_error: bool,
    }

    impl ResourceStatusPresentation for Presentation {
        fn is_refreshing(&self) -> bool {
            self.refreshing
        }

        fn has_value(&self) -> bool {
            self.has_value
        }

        fn error(&self) -> Option<&dyn super::super::ErrorPresentation> {
            self.has_error.then_some(&TestError)
        }
    }

    struct TestError;

    impl super::super::ErrorPresentation for TestError {
        fn summary(&self) -> &str {
            "summary"
        }

        fn detail(&self) -> &str {
            "detail"
        }
    }

    #[test]
    fn presentation_with_cached_loading_value_is_refreshing() {
        let status = ResourceStatusLine::<()>::from_presentation(
            &Presentation {
                refreshing: true,
                has_value: true,
                has_error: false,
            },
            "Refreshing",
            "Failed",
        );

        assert_eq!(
            status.state,
            ResourceStatusLineState::Refreshing {
                label: "Refreshing"
            }
        );
    }

    #[test]
    fn presentation_with_cached_error_is_stale_error() {
        let status = ResourceStatusLine::<()>::from_presentation(
            &Presentation {
                refreshing: false,
                has_value: true,
                has_error: true,
            },
            "Refreshing",
            "Failed",
        );

        assert_eq!(
            status.state,
            ResourceStatusLineState::StaleError { message: "Failed" }
        );
    }
}
