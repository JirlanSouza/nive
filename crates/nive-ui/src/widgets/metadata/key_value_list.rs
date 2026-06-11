use iced::{
    widget::{container, text, Column, Row},
    Alignment, Length, Padding,
};

use crate::theme::{ControlSize, SurfaceRole, ToneRole};
use crate::Element;

use super::style as theme_metadata;
use super::tone_dot::tone_dot;

enum MetadataValue<'a, Message> {
    Text(&'a str),
    Element(Element<'a, Message>),
}

pub struct KeyValueList<'a, Message> {
    items: Vec<MetadataItem<'a, Message>>,
    size: ControlSize,
    role: SurfaceRole,
    width: Option<Length>,
}

pub struct MetadataItem<'a, Message> {
    label: &'a str,
    value: MetadataValue<'a, Message>,
    tone: Option<ToneRole>,
    label_width: Option<Length>,
}

impl<'a, Message> KeyValueList<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            size: ControlSize::Sm,
            role: SurfaceRole::Panel,
            width: None,
        }
    }

    pub fn push(mut self, item: MetadataItem<'a, Message>) -> Self {
        self.items.push(item);
        self
    }

    pub fn item(self, item: MetadataItem<'a, Message>) -> Self {
        self.push(item)
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

    pub fn role(mut self, role: SurfaceRole) -> Self {
        self.role = role;
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn fill(self) -> Self {
        self.width(Length::Fill)
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_metadata::metrics(self.size);
        let mut content = Column::new().spacing(metrics.gap).width(Length::Fill);

        for item in self.items {
            content = content.push(item.into_element(metrics));
        }

        let mut list = container(content)
            .style(theme_metadata::item_style(self.role, metrics.radius))
            .padding(
                Padding::ZERO
                    .vertical(metrics.padding_v)
                    .horizontal(metrics.padding_h),
            );

        if let Some(width) = self.width {
            list = list.width(width);
        }

        list.into()
    }
}

impl<'a, Message> Default for KeyValueList<'a, Message>
where
    Message: Clone + 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> From<KeyValueList<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(list: KeyValueList<'a, Message>) -> Self {
        list.into_element()
    }
}

impl<'a, Message> MetadataItem<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(label: &'a str, value: &'a str) -> Self {
        Self {
            label,
            value: MetadataValue::Text(value),
            tone: None,
            label_width: None,
        }
    }

    pub fn value(mut self, value: impl Into<Element<'a, Message>>) -> Self {
        self.value = MetadataValue::Element(value.into());
        self
    }

    pub fn tone(mut self, tone: ToneRole) -> Self {
        self.tone = Some(tone);
        self
    }

    pub fn label_width(mut self, width: impl Into<Length>) -> Self {
        self.label_width = Some(width.into());
        self
    }

    fn into_element(self, metrics: theme_metadata::MetadataMetrics) -> Element<'a, Message> {
        let label_width = self.label_width.unwrap_or(Length::FillPortion(1));
        let mut row = Row::new()
            .spacing(metrics.row_gap)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        if let Some(tone) = self.tone {
            row = row.push(tone_dot(tone, metrics.tone_size));
        }

        row = row.push(
            text(self.label)
                .size(metrics.label_size)
                .style(theme_metadata::label_style())
                .shaping(text::Shaping::Auto)
                .width(label_width),
        );

        let value: Element<'a, Message> = match self.value {
            MetadataValue::Text(value) => text(value)
                .size(metrics.value_size)
                .style(theme_metadata::value_style())
                .shaping(text::Shaping::Auto)
                .width(Length::FillPortion(2))
                .into(),
            MetadataValue::Element(value) => value,
        };

        row.push(value).into()
    }
}
