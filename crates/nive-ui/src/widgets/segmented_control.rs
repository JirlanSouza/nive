use iced::{
    widget::{button, container, row, text, Row},
    Alignment, Length, Padding,
};

use crate::theme::ControlSize;
use crate::Element;

use self::style::{self as theme_segmented_control, SegmentPosition};
use super::button::ButtonFocusRing;

mod style;
use super::{icon as icon_widget, pressable::Pressable, IconName};

pub struct SegmentedControl<'a, Message> {
    items: Vec<SegmentedItem<'a, Message>>,
    size: ControlSize,
    fill: bool,
}

pub struct SegmentedItem<'a, Message> {
    label: &'a str,
    icon: Option<IconName>,
    selected: bool,
    disabled: bool,
    on_press: Option<Message>,
}

impl<'a, Message> SegmentedControl<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            size: ControlSize::Sm,
            fill: false,
        }
    }

    pub fn push(mut self, item: SegmentedItem<'a, Message>) -> Self {
        self.items.push(item);
        self
    }

    pub fn item(self, item: SegmentedItem<'a, Message>) -> Self {
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

    pub fn fill(mut self) -> Self {
        self.fill = true;
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_segmented_control::metrics(self.size);
        let item_count = self.items.len();
        let items = self.items.into_iter().enumerate().map(|(index, item)| {
            item.into_element(
                metrics,
                theme_segmented_control::position_for_index(index, item_count),
                self.fill,
            )
        });
        let mut content = Row::new()
            .spacing(0)
            .align_y(Alignment::Center)
            .height(Length::Fixed(metrics.height + metrics.outer_padding * 2.0));

        for item in items {
            content = content.push(item);
        }

        let mut segmented = container(content)
            .style(theme_segmented_control::container_style(metrics.radius))
            .padding(metrics.outer_padding);

        if self.fill {
            segmented = segmented.width(Length::Fill);
        }

        segmented.into()
    }
}

impl<'a, Message> Default for SegmentedControl<'a, Message>
where
    Message: Clone + 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> From<SegmentedControl<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(control: SegmentedControl<'a, Message>) -> Self {
        control.into_element()
    }
}

impl<'a, Message: Clone + 'a> SegmentedItem<'a, Message> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            selected: false,
            disabled: false,
            on_press: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    fn into_element(
        self,
        metrics: theme_segmented_control::SegmentedControlMetrics,
        position: SegmentPosition,
        fill: bool,
    ) -> Element<'a, Message> {
        let label = text(self.label).size(metrics.font_size);
        let content: Element<'a, Message> = if let Some(icon) = self.icon {
            row![
                icon_widget::new(icon).size(metrics.icon_size),
                label.shaping(text::Shaping::Auto),
            ]
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .into()
        } else {
            label.shaping(text::Shaping::Auto).into()
        };
        let mut item = button::Button::new(content)
            .style(theme_segmented_control::item_style(
                self.selected,
                position,
                metrics.radius,
            ))
            .padding(Padding::ZERO.horizontal(metrics.padding_h))
            .height(Length::Fixed(metrics.height));

        if fill {
            item = item.width(Length::Fill);
        }

        let activation = if self.disabled {
            None
        } else {
            self.on_press.clone()
        };
        let item = item.on_press_maybe(activation.clone());

        Pressable::maybe(
            item,
            activation,
            metrics.radius.into(),
            ButtonFocusRing::Default,
        )
    }
}
