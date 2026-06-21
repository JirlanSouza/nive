use iced::{
    widget::{container, text, Row, Space},
    Alignment, Length, Padding,
};

use crate::theme::ControlSize;
use crate::Element;

use super::super::{feedback::LoadingIndicator, icon as icon_widget, IconName};
use super::style as theme_button;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TextAlign {
    Center,
    Start,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Content<'a> {
    Label(&'a str),
    Icon(IconName),
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ContentSpec<'a> {
    pub(super) content: Content<'a>,
    pub(super) leading_icon: Option<IconName>,
    pub(super) trailing_icon: Option<IconName>,
    pub(super) size: ControlSize,
    pub(super) text_align: TextAlign,
    pub(super) loading: bool,
    pub(super) reserve_loading_indicator: bool,
}

impl Content<'_> {
    pub(super) fn default_width(self, size: ControlSize) -> Length {
        match self {
            Content::Icon(_) => Length::Fixed(theme_button::icon_side(size)),
            Content::Label(_) => Length::Fill,
        }
    }

    pub(super) fn default_height(self, size: ControlSize) -> Length {
        match self {
            Content::Icon(_) => Length::Fixed(theme_button::icon_side(size)),
            Content::Label(_) => Length::Fixed(theme_button::metrics(size).height),
        }
    }

    pub(super) fn default_padding(self, size: ControlSize) -> Padding {
        match self {
            Content::Icon(_) => Padding::ZERO,
            Content::Label(_) => Padding::ZERO.horizontal(theme_button::metrics(size).padding_h),
        }
    }
}

pub(super) fn element<'a, Message>(spec: ContentSpec<'a>, width: Length) -> Element<'a, Message>
where
    Message: 'a,
{
    let metrics = theme_button::metrics(spec.size);
    let icon_size = metrics.icon_size;

    let content: Element<'a, Message> = match spec.content {
        Content::Label(label) => {
            let label = text(label)
                .size(metrics.font_size)
                .wrapping(text::Wrapping::None);
            let label = match spec.text_align {
                TextAlign::Center => label.width(Length::Shrink).center(),
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
                    row = row.push(icon_widget::new(icon).size(icon_size));
                } else if spec.reserve_loading_indicator {
                    row = row.push(Space::new().width(Length::Fixed(icon_size)));
                }

                row = row.push(label);

                if let Some(icon) = spec.trailing_icon {
                    row = row.push(icon_widget::new(icon).size(icon_size));
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
                icon_widget::new(app_icon).size(icon_size).into()
            }
        }
    };

    match spec.text_align {
        TextAlign::Center => container(content)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(width)
            .height(Length::Fill)
            .into(),
        TextAlign::Start => container(content)
            .align_x(Alignment::Start)
            .align_y(Alignment::Center)
            .width(width)
            .height(Length::Fill)
            .into(),
    }
}

fn loading_indicator_slot<'a, Message>(size: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    container(LoadingIndicator::new().neutral().xs())
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}
