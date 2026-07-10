use iced::{
    widget::{container, row, text, Space},
    Alignment, Length, Padding,
};

use crate::theme::{ControlSize, ToneRole};
use crate::Element;

use super::style as theme_metadata;
use crate::widgets::primitives::tone_dot::tone_dot;

pub struct DataRow<'a, Message> {
    label: &'a str,
    value: Option<&'a str>,
    tone: Option<ToneRole>,
    leading: Option<Element<'a, Message>>,
    trailing: Option<Element<'a, Message>>,
    size: ControlSize,
    width: Option<Length>,
}

impl<'a, Message> DataRow<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            value: None,
            tone: None,
            leading: None,
            trailing: None,
            size: ControlSize::Sm,
            width: None,
        }
    }

    pub fn value(mut self, value: &'a str) -> Self {
        self.value = Some(value);
        self
    }

    pub fn tone(mut self, tone: ToneRole) -> Self {
        self.tone = Some(tone);
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

    pub fn danger(self) -> Self {
        self.tone(ToneRole::Danger)
    }

    pub fn leading(mut self, leading: impl Into<Element<'a, Message>>) -> Self {
        self.leading = Some(leading.into());
        self
    }

    pub fn trailing(mut self, trailing: impl Into<Element<'a, Message>>) -> Self {
        self.trailing = Some(trailing.into());
        self
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

    crate::impl_layout_builders!(width_opt, fill_width_opt, shrink_width_opt);

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_metadata::metrics(self.size);
        let mut content = row![]
            .spacing(metrics.row_gap)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        if let Some(tone) = self.tone {
            content = content.push(tone_dot(tone, metrics.tone_size));
        }

        if let Some(leading) = self.leading {
            content = content.push(leading);
        }

        content = content.push(
            text(self.label)
                .size(metrics.value_size)
                .style(theme_metadata::value_style())
                .shaping(text::Shaping::Auto),
        );

        if let Some(value) = self.value {
            content = content.push(Space::new().width(Length::Fill)).push(
                text(value)
                    .size(metrics.label_size)
                    .style(theme_metadata::secondary_value_style())
                    .shaping(text::Shaping::Auto),
            );
        }

        if let Some(trailing) = self.trailing {
            content = content
                .push(Space::new().width(Length::Fill))
                .push(trailing);
        }

        let mut row = container(content)
            .style(theme_metadata::row_style(metrics.radius))
            .padding(
                Padding::ZERO
                    .vertical(metrics.padding_v)
                    .horizontal(metrics.padding_h),
            )
            .height(Length::Fixed(metrics.row_height));

        if let Some(width) = self.width {
            row = row.width(width);
        }

        row.into()
    }
}

impl<'a, Message> From<DataRow<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(row: DataRow<'a, Message>) -> Self {
        row.into_element()
    }
}
