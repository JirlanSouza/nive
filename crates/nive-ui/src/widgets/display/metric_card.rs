use std::borrow::Cow;

use iced::{
    widget::{column, rich_text, row, span, text},
    Alignment, Length,
};

use crate::{
    theme::{self, GapRole, TextRole, TextStyle, TypographyRole},
    widgets::primitives::text::text_secondary,
    Element,
};

/// Surface-free label/value composition for dashboard metrics.
///
/// Wrap this widget in [`crate::widgets::Card`] when surface chrome is needed.
pub struct MetricCard<'a, Message> {
    label: Cow<'a, str>,
    value: String,
    unit: Option<Cow<'a, str>>,
    status: Option<Element<'a, Message>>,
    trend: Option<Element<'a, Message>>,
}

impl<'a, Message: Clone + 'a> MetricCard<'a, Message> {
    pub fn new(label: impl Into<Cow<'a, str>>, value: impl ToString) -> Self {
        Self {
            label: label.into(),
            value: value.to_string(),
            unit: None,
            status: None,
            trend: None,
        }
    }

    /// Adds a muted unit on the value's text baseline.
    pub fn unit(mut self, unit: impl Into<Cow<'a, str>>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Adds display-only semantic status content below the value.
    pub fn status(mut self, status: impl Into<Element<'a, Message>>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// Adds display-only trend content separately from status.
    pub fn trend(mut self, trend: impl Into<Element<'a, Message>>) -> Self {
        self.trend = Some(trend.into());
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = metrics();
        let theme = theme::active();
        let mut value_spans: Vec<text::Span<'a, (), iced::Font>> = vec![span(self.value)
            .font(metrics.value_style.font)
            .size(metrics.value_style.size)
            .line_height(metrics.value_style.line_height)
            .color(theme.text(TextRole::Primary).color)];

        if let Some(unit) = self.unit {
            value_spans.push(
                span(format!(" {unit}"))
                    .font(metrics.unit_style.font)
                    .size(metrics.unit_style.size)
                    .line_height(metrics.unit_style.line_height)
                    .color(theme.text(TextRole::Muted).color),
            );
        }

        let label = text(self.label)
            .font(metrics.label_style.font)
            .size(metrics.label_style.size)
            .line_height(metrics.label_style.line_height)
            .style(label_style());
        let value = rich_text(value_spans);
        let mut metric = column![label, value]
            .spacing(metrics.content_gap)
            .align_x(Alignment::Start)
            .width(Length::Fill);

        if self.status.is_some() || self.trend.is_some() {
            let mut support = row![]
                .spacing(metrics.support_gap)
                .align_y(Alignment::Center);
            if let Some(status) = self.status {
                support = support.push(status);
            }
            if let Some(trend) = self.trend {
                support = support.push(trend);
            }
            metric = metric.push(support);
        }

        metric.into()
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
    content_gap: f32,
    support_gap: f32,
    value_style: TextStyle,
    label_style: TextStyle,
    unit_style: TextStyle,
}

fn metrics() -> Metrics {
    Metrics {
        content_gap: theme::gap(GapRole::Tight),
        support_gap: theme::gap(GapRole::Related),
        value_style: theme::typography(TypographyRole::Title),
        label_style: theme::typography(TypographyRole::BodySmall),
        unit_style: theme::typography(TypographyRole::BodySmall),
    }
}

#[cfg(test)]
fn value_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    crate::widgets::primitives::text::text_primary()
}

fn label_style() -> impl Fn(&crate::theme::Theme) -> text::Style {
    text_secondary()
}

#[cfg(test)]
mod metric_card_tests {
    use super::*;
    use crate::theme::Theme;
    use crate::widgets::containers::card_test_support::{CardHarness, Message};
    use iced::Size;

    #[test]
    fn formats_numeric_decimal_and_preformatted_values_once() {
        assert_eq!(MetricCard::<()>::new("Zero", 0).value, "0");
        assert_eq!(MetricCard::<()>::new("Negative", -42).value, "-42");
        assert_eq!(MetricCard::<()>::new("Decimal", 99.5).value, "99.5");
        assert_eq!(MetricCard::<()>::new("Percent", "99.9%").value, "99.9%");
    }

    #[test]
    fn owned_and_borrowed_labels_and_optional_slots_remain_separate() {
        let metric = MetricCard::<()>::new(String::from("Latency"), "18.4")
            .unit("ms")
            .status(text("healthy"))
            .trend(text("-2.1%"));

        assert_eq!(metric.label, "Latency");
        assert_eq!(metric.unit.as_deref(), Some("ms"));
        assert!(metric.status.is_some());
        assert!(metric.trend.is_some());
    }

    #[test]
    fn label_value_and_unit_use_complete_semantic_styles() {
        let theme = Theme::Dark;
        let metrics = metrics();

        assert_eq!(
            metrics.label_style,
            theme.typography(TypographyRole::BodySmall)
        );
        assert_eq!(metrics.label_style.size, 12.0);
        assert_eq!(metrics.value_style, theme.typography(TypographyRole::Title));
        assert_eq!(metrics.value_style.size, 20.0);
        assert_eq!(
            metrics.unit_style,
            theme.typography(TypographyRole::BodySmall)
        );
        assert_eq!(
            value_style()(&theme).color,
            Some(theme.text(TextRole::Primary).color)
        );
        assert_eq!(
            label_style()(&theme).color,
            Some(theme.text(TextRole::Secondary).color)
        );
    }

    #[test]
    fn rendered_layout_is_label_first_leading_fill_and_support_is_optional() {
        let basic = CardHarness::new(
            MetricCard::<Message>::new("Latency", "18.4")
                .unit("ms")
                .into(),
            Size::new(240.0, 160.0),
        );
        let supported = CardHarness::new(
            MetricCard::<Message>::new("Latency", "18.4")
                .unit("ms")
                .status(text("healthy"))
                .trend(text("-2.1%"))
                .into(),
            Size::new(240.0, 160.0),
        );
        let basic_children = basic.child_bounds();
        let supported_children = supported.child_bounds();

        assert_eq!(basic.size().width, 240.0);
        assert_eq!(basic_children.len(), 2);
        assert_eq!(supported_children.len(), 3);
        assert_eq!(basic_children[0].x, 0.0);
        assert_eq!(basic_children[1].x, 0.0);
        assert!(basic_children[0].y < basic_children[1].y);
        assert!(supported_children[1].y < supported_children[2].y);
    }
}
