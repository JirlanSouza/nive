use iced::{
    widget::{column, container, row, text},
    Alignment, Length, Padding,
};

use crate::theme::{ControlSize, ToneRole};
use crate::Element;

use super::style as theme_feedback;
use crate::widgets::primitives::tone_dot::tone_dot;

/// Inline feedback callout.
///
/// Use `danger()` for error/failure status tone. Destructive action semantics
/// belong to actionable widgets such as `Button`.
pub struct InlineAlert<'a, Message> {
    title: &'a str,
    body: Option<&'a str>,
    tone: ToneRole,
    size: ControlSize,
    action: Option<Element<'a, Message>>,
}

impl<'a, Message> InlineAlert<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            body: None,
            tone: ToneRole::Info,
            size: ControlSize::Sm,
            action: None,
        }
    }

    pub fn body(mut self, body: &'a str) -> Self {
        self.body = Some(body);
        self
    }

    pub fn tone(mut self, tone: ToneRole) -> Self {
        self.tone = tone;
        self
    }

    pub fn neutral(self) -> Self {
        self.tone(ToneRole::Neutral)
    }

    pub fn accent(self) -> Self {
        self.tone(ToneRole::Accent)
    }

    pub fn info(self) -> Self {
        self.tone(ToneRole::Info)
    }

    pub fn success(self) -> Self {
        self.tone(ToneRole::Success)
    }

    pub fn warning(self) -> Self {
        self.tone(ToneRole::Warning)
    }

    /// Applies the danger status tone.
    pub fn danger(self) -> Self {
        self.tone(ToneRole::Danger)
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    pub fn lg(self) -> Self {
        self.size(ControlSize::Lg)
    }

    pub fn action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.action = Some(action.into());
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_feedback::inline_alert_metrics(self.size);
        let indicator = tone_dot(self.tone, metrics.indicator_size);
        let mut text_content = column![text(self.title)
            .size(metrics.title_size)
            .style(theme_feedback::title_style(self.tone))
            .shaping(text::Shaping::Auto)]
        .spacing(metrics.gap / 2.0);

        if let Some(body) = self.body {
            text_content = text_content.push(
                text(body)
                    .size(metrics.body_size)
                    .style(theme_feedback::body_style())
                    .shaping(text::Shaping::Auto),
            );
        }

        let mut content = row![indicator, text_content.width(Length::Fill)]
            .spacing(metrics.gap)
            .align_y(Alignment::Start)
            .width(Length::Fill);

        if let Some(action) = self.action {
            content = content.push(action);
        }

        container(content)
            .style(theme_feedback::inline_alert_style(
                self.tone,
                metrics.radius,
            ))
            .padding(Padding::new(metrics.padding))
            .width(Length::Fill)
            .into()
    }
}

impl<'a, Message> From<InlineAlert<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(alert: InlineAlert<'a, Message>) -> Self {
        alert.into_element()
    }
}
