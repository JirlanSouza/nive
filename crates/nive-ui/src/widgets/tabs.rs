use iced::{
    widget::{button, container, row, text, Row, Space},
    Alignment, Length, Padding,
};

use crate::theme::{ControlSize, SurfaceRole};
use crate::Element;

use self::style::{self as theme_tabs, TabPart};
use super::button::ButtonFocusRing;

mod style;
use super::{icon as icon_widget, pressable::Pressable, tooltip as tooltip_widget, IconName};

pub struct TabBar<'a, Message> {
    tabs: Vec<TabItem<'a, Message>>,
    size: ControlSize,
    role: SurfaceRole,
    width: Option<Length>,
}

pub struct TabItem<'a, Message> {
    label: &'a str,
    icon: Option<IconName>,
    selected: bool,
    dirty: bool,
    disabled: bool,
    on_press: Option<Message>,
    on_close: Option<Message>,
    tooltip: Option<&'a str>,
}

impl<'a, Message> TabBar<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            size: ControlSize::Sm,
            role: SurfaceRole::Chrome,
            width: None,
        }
    }

    pub fn push(mut self, tab: TabItem<'a, Message>) -> Self {
        self.tabs.push(tab);
        self
    }

    pub fn tab(self, tab: TabItem<'a, Message>) -> Self {
        self.push(tab)
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
        let metrics = theme_tabs::metrics(self.size);
        let mut tabs = Row::new()
            .spacing(metrics.tab_gap)
            .align_y(Alignment::Center)
            .height(Length::Fixed(metrics.tab_height));

        for tab in self.tabs {
            tabs = tabs.push(tab.into_element(metrics));
        }

        let mut bar = container(tabs)
            .style(theme_tabs::bar_style(self.role))
            .padding(
                Padding::ZERO
                    .vertical(metrics.bar_padding_v)
                    .horizontal(metrics.bar_padding_h),
            )
            .height(Length::Fixed(metrics.height));

        if let Some(width) = self.width {
            bar = bar.width(width);
        }

        bar.into()
    }
}

impl<'a, Message> Default for TabBar<'a, Message>
where
    Message: Clone + 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> From<TabBar<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(tab_bar: TabBar<'a, Message>) -> Self {
        tab_bar.into_element()
    }
}

impl<'a, Message: Clone + 'a> TabItem<'a, Message> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            selected: false,
            dirty: false,
            disabled: false,
            on_press: None,
            on_close: None,
            tooltip: None,
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

    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
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

    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    pub fn on_close_maybe(mut self, message: Option<Message>) -> Self {
        self.on_close = message;
        self
    }

    fn into_element(self, metrics: theme_tabs::TabBarMetrics) -> Element<'a, Message> {
        let has_close = self.on_close.is_some();
        let main_part = if has_close {
            TabPart::Leading
        } else {
            TabPart::Full
        };
        let main = self.main_button(metrics, main_part);
        let content: Element<'a, Message> = if let Some(close_message) = self.on_close {
            let close_activation = (!self.disabled).then_some(close_message.clone());
            let close =
                button::Button::new(icon_widget::new(IconName::Close).size(metrics.close_icon_size))
                    .style(theme_tabs::tab_style(
                        self.selected,
                        TabPart::Trailing,
                        metrics.radius,
                    ))
                    .padding(Padding::ZERO)
                    .width(Length::Fixed(metrics.close_side))
                    .height(Length::Fixed(metrics.tab_height))
                    .on_press_maybe(close_activation.clone());
            row![
                main,
                Pressable::maybe(
                    close,
                    close_activation,
                    metrics.radius.into(),
                    ButtonFocusRing::Default,
                )
            ]
            .spacing(0)
            .align_y(Alignment::Center)
            .into()
        } else {
            main
        };

        match self.tooltip {
            Some(label) => tooltip_widget::bottom(content, label),
            None => content,
        }
    }

    fn main_button(
        &self,
        metrics: theme_tabs::TabBarMetrics,
        part: TabPart,
    ) -> Element<'a, Message> {
        let label = text(self.label)
            .size(metrics.font_size)
            .shaping(text::Shaping::Auto);
        let mut content = Row::new()
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .height(Length::Shrink);

        if let Some(icon) = self.icon {
            content = content.push(icon_widget::new(icon).size(metrics.icon_size));
        }

        content = content.push(label);

        if self.dirty {
            let dot = container(Space::new().width(Length::Fixed(metrics.dirty_size)))
                .style(theme_tabs::dirty_indicator_style(metrics.dirty_size))
                .width(Length::Fixed(metrics.dirty_size))
                .height(Length::Fixed(metrics.dirty_size));

            content = content.push(dot);
        }

        let activation = if self.disabled {
            None
        } else {
            self.on_press.clone()
        };
        let button = button::Button::new(content)
            .style(theme_tabs::tab_style(self.selected, part, metrics.radius))
            .padding(Padding::ZERO.horizontal(metrics.padding_h))
            .height(Length::Fixed(metrics.tab_height))
            .on_press_maybe(activation.clone());

        Pressable::maybe(
            button,
            activation,
            metrics.radius.into(),
            ButtonFocusRing::Default,
        )
    }
}
