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
    inherit_color: bool,
    indicator_size: Option<f32>,
}

impl<'a> Spinner<'a> {
    pub fn new() -> Self {
        Self {
            label: None,
            tone: ToneRole::Accent,
            size: ControlSize::Sm,
            inherit_color: false,
            indicator_size: None,
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

    pub(crate) fn inherit_color(mut self) -> Self {
        self.inherit_color = true;
        self
    }

    pub(crate) fn custom_size(mut self, size: f32) -> Self {
        self.indicator_size = Some(size);
        self
    }

    fn into_element<Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let metrics = theme_feedback::loading_metrics(self.size);
        let indicator_size = self.indicator_size.unwrap_or(metrics.indicator_size);
        let dot = if self.inherit_color {
            inherited_loading_indicator(indicator_size)
        } else {
            loading_indicator(self.tone, indicator_size)
        };

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

fn inherited_loading_indicator<'a, Message>(diameter: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    container(
        text("●")
            .size(diameter)
            .line_height(text::LineHeight::Relative(1.0)),
    )
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .width(Length::Fixed(diameter))
    .height(Length::Fixed(diameter))
    .into()
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

#[cfg(test)]
mod spinner_tests {
    use super::*;

    #[test]
    fn button_spinner_can_inherit_foreground_at_the_form_metric_size() {
        let size = crate::theme::form_control_metrics(ControlSize::Lg).icon_size;
        let spinner = Spinner::new().inherit_color().custom_size(size);

        assert!(spinner.inherit_color);
        assert_eq!(spinner.indicator_size, Some(size));
    }
}
