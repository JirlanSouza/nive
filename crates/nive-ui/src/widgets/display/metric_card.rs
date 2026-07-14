use iced::{
    widget::{column, text},
    Alignment, Length,
};

use crate::{
    theme::{self, GapRole, TextStyle, TypographyRole},
    widgets::primitives::text::{text_primary, text_secondary},
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
        let value = format_metric_value(self.value);

        column![
            text(value)
                .font(metrics.value_style.font)
                .size(metrics.value_style.size)
                .line_height(metrics.value_style.line_height)
                .style(value_style()),
            text(self.label)
                .font(metrics.label_style.font)
                .size(metrics.label_style.size)
                .line_height(metrics.label_style.line_height)
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
    value_style: TextStyle,
    label_style: TextStyle,
}

fn metrics() -> Metrics {
    Metrics {
        gap: theme::gap(GapRole::Tight),
        value_style: theme::typography(TypographyRole::Title),
        label_style: theme::typography(TypographyRole::BodySmall),
    }
}

fn value_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    text_primary()
}

fn label_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    text_secondary()
}

fn format_metric_value(value: i128) -> String {
    value.to_string()
}

#[cfg(test)]
mod metric_card_tests {
    use super::*;

    #[test]
    fn formats_zero_as_a_valid_metric_value() {
        assert_eq!(format_metric_value(0), "0");
    }

    #[test]
    fn value_uses_primary_text_role_at_the_title_style() {
        let theme = theme::active();
        let metrics = metrics();

        assert_eq!(metrics.value_style, theme.typography(TypographyRole::Title));
        assert_eq!(
            value_style()(&theme).color,
            Some(theme.text(crate::theme::TextRole::Primary).color)
        );
    }
}
