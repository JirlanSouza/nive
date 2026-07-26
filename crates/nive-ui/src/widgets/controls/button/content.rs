use iced::{
    widget::{container, Row, Space},
    Alignment, Length, Padding,
};

use crate::theme::ControlSize;
use crate::Element;

use super::style as theme_button;
use crate::widgets::display::measured_text::{EllipsisStrategy, MeasuredText};
use crate::widgets::feedback::Spinner;
use crate::widgets::primitives::{icon as icon_widget, IconRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TextAlign {
    Center,
    Start,
}

pub(super) enum Content<'a, Message> {
    Label(Cow<'a, str>),
    Icon(IconRef),
    Custom(Element<'a, Message>),
}

pub(super) struct ContentSpec<'a, Message> {
    pub(super) content: Content<'a, Message>,
    pub(super) leading_icon: Option<IconRef>,
    pub(super) trailing_icon: Option<IconRef>,
    pub(super) size: ControlSize,
    pub(super) text_align: TextAlign,
    pub(super) loading: bool,
    pub(super) reserve_loading_indicator: bool,
}

impl<Message> Content<'_, Message> {
    pub(super) fn default_width(&self, size: ControlSize) -> Length {
        match self {
            Content::Icon(_) => Length::Fixed(theme_button::icon_side(size)),
            Content::Label(_) => Length::Shrink,
            Content::Custom(_) => Length::Shrink,
        }
    }

    pub(super) fn default_height(&self, size: ControlSize) -> Length {
        match self {
            Content::Icon(_) => Length::Fixed(theme_button::icon_side(size)),
            Content::Label(_) | Content::Custom(_) => {
                Length::Fixed(theme_button::metrics(size).height)
            }
        }
    }

    pub(super) fn default_padding(&self, size: ControlSize) -> Padding {
        match self {
            Content::Icon(_) => Padding::ZERO,
            Content::Label(_) | Content::Custom(_) => {
                Padding::ZERO.horizontal(theme_button::metrics(size).padding_h)
            }
        }
    }
}

pub(super) fn element<'a, Message>(
    spec: ContentSpec<'a, Message>,
    width: Length,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let metrics = theme_button::metrics(spec.size);
    let icon_size = metrics.icon_size;

    let content: Element<'a, Message> = match spec.content {
        Content::Label(label) => {
            let label: Element<'a, Message> = Element::new(MeasuredText::new_inherited(
                label,
                EllipsisStrategy::End,
                crate::theme::TypographyRole::ControlStrong,
            ));
            let label: Element<'a, Message> = match spec.text_align {
                TextAlign::Center => container(label)
                    .width(Length::Shrink)
                    .align_x(Alignment::Center)
                    .into(),
                TextAlign::Start => label,
            };

            let has_icon = spec.reserve_loading_indicator
                || spec.leading_icon.is_some()
                || spec.trailing_icon.is_some();

            if has_icon {
                let mut row = Row::new()
                    .spacing(metrics.gap)
                    .align_y(Alignment::Center)
                    .width(Length::Shrink)
                    .height(Length::Shrink);

                if spec.loading {
                    row = row.push(loading_indicator_slot(icon_size));
                } else if let Some(icon) = spec.leading_icon {
                    row = row.push(icon_widget::reference(icon).custom_size(icon_size));
                } else if spec.reserve_loading_indicator {
                    row = row.push(Space::new().width(Length::Fixed(icon_size)));
                }

                row = row.push(label);

                if let Some(icon) = spec.trailing_icon {
                    row = row.push(icon_widget::reference(icon).custom_size(icon_size));
                }

                container(row).into()
            } else {
                container(label).into()
            }
        }
        Content::Icon(app_icon) => {
            if spec.loading {
                loading_indicator_slot(icon_size)
            } else {
                icon_widget::reference(app_icon)
                    .custom_size(icon_size)
                    .into()
            }
        }
        Content::Custom(content) => content,
    };

    match spec.text_align {
        TextAlign::Center => container(content)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(width)
            .height(Length::Fill)
            .clip(true)
            .into(),
        TextAlign::Start => container(content)
            .align_x(Alignment::Start)
            .align_y(Alignment::Center)
            .width(width)
            .height(Length::Fill)
            .clip(true)
            .into(),
    }
}

fn loading_indicator_slot<'a, Message>(size: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    container(Spinner::new().inherit_color().custom_size(size))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}
use std::borrow::Cow;
