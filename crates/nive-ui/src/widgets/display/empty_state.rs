use iced::{
    widget::{column, container, text, Space},
    Alignment, Border, Length, Shadow,
};

use crate::theme::{self, text as theme_text, SpaceStep, TextRole, TypographyRole};
use crate::widgets::feedback::Spinner;
use crate::widgets::primitives::{icon, IconName};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq)]
struct EmptyStateMetrics {
    gap: f32,
    icon_size: f32,
    title_size: f32,
    description_size: f32,
}

const VERTICAL_OFFSET_TOP_FILL_PORTION: u16 = 6;
const VERTICAL_OFFSET_BOTTOM_FILL_PORTION: u16 = 14;

pub struct EmptyState<'a, Message> {
    title: &'a str,
    description: Option<&'a str>,
    icon: Option<IconName>,
    loading: bool,
    action: Option<Element<'a, Message>>,
}

impl<'a, Message> EmptyState<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            description: None,
            icon: None,
            loading: false,
            action: None,
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

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.action = Some(action.into());
        self
    }

    fn into_container(self) -> container::Container<'a, Message, crate::theme::Theme> {
        let metrics = metrics();
        let icon: Element<'a, Message> = if self.loading {
            Spinner::new().md().into()
        } else {
            container(icon::new(self.icon.unwrap_or(IconName::Inbox)).size(metrics.icon_size))
                .style(icon_container_style())
                .into()
        };

        let mut state_content = column![
            icon,
            text(self.title)
                .size(metrics.title_size)
                .style(title_style())
        ]
        .spacing(metrics.gap)
        .align_x(Alignment::Center);

        if let Some(description) = self.description {
            state_content = state_content.push(
                text(description)
                    .size(metrics.description_size)
                    .style(description_style()),
            );
        }

        if let Some(action) = self.action {
            state_content = state_content.push(action);
        }

        let content = column![
            Space::new().height(Length::FillPortion(VERTICAL_OFFSET_TOP_FILL_PORTION)),
            state_content,
            Space::new().height(Length::FillPortion(VERTICAL_OFFSET_BOTTOM_FILL_PORTION)),
        ]
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content).width(Length::Fill).height(Length::Fill)
    }
}

impl<'a, Message> From<EmptyState<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(empty_state: EmptyState<'a, Message>) -> Self {
        empty_state.into_container().into()
    }
}

fn metrics() -> EmptyStateMetrics {
    EmptyStateMetrics {
        gap: theme::space(SpaceStep::Md),
        icon_size: 48.0,
        title_size: theme::typography(TypographyRole::Title).size,
        description_size: theme::typography(TypographyRole::Body).size,
    }
}

fn icon_container_style() -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| container::Style {
        text_color: Some(theme.text(TextRole::Muted).color.scale_alpha(0.6)),
        background: None,
        border: Border::default(),
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

fn title_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    theme_text::style(TextRole::Primary)
}

fn description_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    theme_text::style(TextRole::Muted)
}

#[cfg(test)]
mod empty_state_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn title_style_uses_app_primary_text() {
        let theme = Theme::Dark;

        assert_eq!(
            title_style()(&theme).color,
            Some(theme.text(TextRole::Primary).color)
        );
    }
}
