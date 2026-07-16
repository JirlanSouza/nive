use std::borrow::Cow;

use iced::{
    border::Radius,
    widget::{container, row, text, Space},
    Alignment, Background, Border, Length, Shadow,
};

use crate::theme::{ControlSize, ToneRole};
use crate::Element;

use super::style as theme_feedback;

pub struct Spinner<'a> {
    label: Option<Cow<'a, str>>,
    tone: ToneRole,
    size: ControlSize,
}

impl<'a> Spinner<'a> {
    pub fn new() -> Self {
        Self {
            label: None,
            tone: ToneRole::Accent,
            size: ControlSize::Sm,
        }
    }

    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn tone(mut self, tone: ToneRole) -> Self {
        self.tone = tone;
        self
    }

    pub fn neutral(self) -> Self {
        self.tone(ToneRole::Neutral)
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

    pub fn danger(self) -> Self {
        self.tone(ToneRole::Danger)
    }

    pub fn accent(self) -> Self {
        self.tone(ToneRole::Accent)
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

    fn into_element<Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let metrics = theme_feedback::loading_metrics(self.size);
        let dot = loading_indicator(self.tone, metrics.indicator_size);

        if let Some(label) = self.label {
            row![
                dot,
                text(label)
                    .size(metrics.label_size)
                    .style(theme_feedback::loading_label_style())
                    .shaping(text::Shaping::Auto)
            ]
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .into()
        } else {
            dot
        }
    }
}

fn loading_indicator<'a, Message>(tone: ToneRole, diameter: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    container(Space::new().width(Length::Fixed(diameter)))
        .style(move |theme: &crate::theme::Theme| container::Style {
            background: Some(Background::Color(theme.tone(tone).color)),
            border: Border {
                radius: Radius::new(diameter / 2.0),
                ..Border::default()
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        })
        .width(Length::Fixed(diameter))
        .height(Length::Fixed(diameter))
        .into()
}

impl<'a> Default for Spinner<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> From<Spinner<'a>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(indicator: Spinner<'a>) -> Self {
        indicator.into_element()
    }
}
