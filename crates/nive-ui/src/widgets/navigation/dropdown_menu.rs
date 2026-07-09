use std::borrow::Cow;

use iced::{
    widget::{button, container, row, text, Column, Row, Space},
    Alignment, Length, Padding,
};

use crate::theme::ControlSize;
use crate::Element;

use self::style as theme_menu;
use crate::widgets::controls::button::ButtonFocusRing;

mod style;
use crate::advanced::pressable::Pressable;
use crate::widgets::overlays::tooltip as tooltip_widget;
use crate::widgets::primitives::{icon as icon_widget, IconRole};

pub struct DropdownMenu<'a, Message> {
    items: Vec<MenuEntry<'a, Message>>,
    size: ControlSize,
    width: Option<Length>,
}

pub struct DropdownMenuItem<'a, Message> {
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    trailing: Option<&'a str>,
    selected: bool,
    destructive: bool,
    disabled: bool,
    on_press: Option<Message>,
    tooltip: Option<&'a str>,
}

enum MenuEntry<'a, Message> {
    Item(DropdownMenuItem<'a, Message>),
    Separator,
}

impl<'a, Message> DropdownMenu<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            size: ControlSize::Sm,
            width: None,
        }
    }

    pub fn push(mut self, item: DropdownMenuItem<'a, Message>) -> Self {
        self.items.push(MenuEntry::Item(item));
        self
    }

    pub fn item(self, item: DropdownMenuItem<'a, Message>) -> Self {
        self.push(item)
    }

    pub fn separator(mut self) -> Self {
        self.items.push(MenuEntry::Separator);
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

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn fill(self) -> Self {
        self.width(Length::Fill)
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_menu::metrics(self.size);
        let mut content = Column::new()
            .spacing(metrics.padding)
            .padding(metrics.padding)
            .width(Length::Fill);

        for item in self.items {
            content = content.push(match item {
                MenuEntry::Item(item) => item.into_element(metrics),
                MenuEntry::Separator => separator(metrics),
            });
        }

        let mut menu = container(content).style(theme_menu::menu_style(metrics.radius));

        if let Some(width) = self.width {
            menu = menu.width(width);
        }

        menu.into()
    }
}

impl<'a, Message> Default for DropdownMenu<'a, Message>
where
    Message: Clone + 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> From<DropdownMenu<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(menu: DropdownMenu<'a, Message>) -> Self {
        menu.into_element()
    }
}

impl<'a, Message: Clone + 'a> DropdownMenuItem<'a, Message> {
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            trailing: None,
            selected: false,
            destructive: false,
            disabled: false,
            on_press: None,
            tooltip: None,
        }
    }

    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn trailing(mut self, trailing: &'a str) -> Self {
        self.trailing = Some(trailing);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn tooltip(mut self, tooltip: &'a str) -> Self {
        self.tooltip = Some(tooltip);
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

    fn into_element(self, metrics: theme_menu::MenuMetrics) -> Element<'a, Message> {
        let activation = if self.disabled {
            None
        } else {
            self.on_press.clone()
        };
        let button = button::Button::new(self.content(metrics))
            .style(theme_menu::item_style(
                self.selected,
                self.destructive,
                metrics.radius,
            ))
            .padding(Padding::ZERO.horizontal(metrics.item_padding_h))
            .height(Length::Fixed(metrics.item_height))
            .width(Length::Fill)
            .on_press_maybe(activation.clone());
        let button = Pressable::maybe(
            button,
            activation,
            metrics.radius.into(),
            ButtonFocusRing::Default,
        );

        match self.tooltip {
            Some(label) => tooltip_widget::bottom(button, label),
            None => button,
        }
    }

    fn content(&self, metrics: theme_menu::MenuMetrics) -> Element<'a, Message> {
        let mut left = Row::new()
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .height(Length::Shrink);

        if let Some(icon) = self.icon {
            left = left.push(icon_widget::role(icon).size(metrics.icon_size));
        }

        left = left.push(
            text(self.label.clone())
                .size(metrics.font_size)
                .shaping(text::Shaping::Auto),
        );

        let mut content = row![left]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        if let Some(trailing) = self.trailing {
            content = content
                .push(Space::new().width(Length::Fill))
                .push(text(trailing).size(metrics.font_size));
        }

        content.into()
    }
}

fn separator<'a, Message>(metrics: theme_menu::MenuMetrics) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(Space::new().height(Length::Fixed(metrics.separator_height)))
        .style(theme_menu::separator_style())
        .height(Length::Fixed(metrics.separator_height))
        .width(Length::Fill)
        .into()
}

#[cfg(test)]
mod dropdown_menu_tests {
    use iced::Length;

    use super::*;

    #[derive(Clone)]
    enum TestMessage {}

    #[test]
    fn item_content_fills_fixed_button_height() {
        let metrics = theme_menu::metrics(ControlSize::Sm);
        let content = DropdownMenuItem::<TestMessage>::new("Rename").content(metrics);

        assert_eq!(content.as_widget().size().height, Length::Fill);
    }
}
