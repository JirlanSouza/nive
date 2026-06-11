use std::ops::RangeInclusive;

use iced::{widget::progress_bar as iced_progress_bar, Length};

use super::style as theme_feedback;
use crate::theme::{ControlSize, ToneRole};
use crate::Element;

pub struct ProgressBar {
    range: RangeInclusive<f32>,
    value: f32,
    tone: ToneRole,
    size: ControlSize,
    width: Length,
    height: Option<Length>,
}

impl ProgressBar {
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self {
        Self {
            range,
            value,
            tone: ToneRole::Primary,
            size: ControlSize::Sm,
            width: Length::Fill,
            height: None,
        }
    }

    pub fn percent(value: f32) -> Self {
        Self::new(0.0..=1.0, value)
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
        self.tone(ToneRole::Primary)
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

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = Some(height.into());
        self
    }

    fn into_element<'a, Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let metrics = theme_feedback::progress_metrics(self.size);
        let bar = iced_progress_bar(self.range, self.value)
            .length(self.width)
            .girth(self.height.unwrap_or(Length::Fixed(metrics.height)))
            .style(theme_feedback::progress_style(self.tone, metrics.radius));

        bar.into()
    }
}

impl<'a, Message> From<ProgressBar> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(progress: ProgressBar) -> Self {
        progress.into_element()
    }
}
