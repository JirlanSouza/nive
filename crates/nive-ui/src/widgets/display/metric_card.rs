use iced::{
    widget::{column, text},
    Alignment, Length,
};

use crate::{
    theme::{self, GapRole, TypographyRole},
    widgets::primitives::text::{text_muted, text_secondary},
    Element,
};

pub struct MetricCard<'a, Message> {
    label: &'a str,
    value: i128,
    _marker: std::marker::PhantomData<Message>,
}

impl<'a, Message: Clone + 'a> MetricCard<'a, Message> {
    pub fn new(label: &'a str, value: impl Into<i128>) -> Self {
        Self {
            label,
            value: value.into(),
            _marker: std::marker::PhantomData,
        }
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = metrics();
        let value = if self.value > 0 {
            self.value.to_string()
        } else {
            "--".to_string()
        };

        column![
            text(value).size(metrics.value_size).style(value_style()),
            text(self.label)
                .size(metrics.label_size)
                .style(label_style()),
        ]
        .spacing(metrics.gap)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .into()
    }
}

impl<'a, Message> From<MetricCard<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(metric: MetricCard<'a, Message>) -> Self {
        metric.into_element()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Metrics {
    gap: f32,
    value_size: f32,
    label_size: f32,
}

fn metrics() -> Metrics {
    Metrics {
        gap: theme::gap(GapRole::Tight),
        value_size: theme::typography(TypographyRole::Title).size,
        label_size: theme::typography(TypographyRole::BodySmall).size,
    }
}

fn value_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    text_secondary()
}

fn label_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    text_muted()
}
