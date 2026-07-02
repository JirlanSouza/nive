mod tone_dot;

use iced::{
    widget::{button, container, text, Row, Space},
    Alignment, Length, Padding,
};

use crate::theme::{ControlSize, ToneRole};
use crate::Element;

use self::style::{self as theme_tree_item, TreeItemVariant};
use super::button::ButtonFocusRing;

mod style;
use super::{icon as icon_widget, pressable::Pressable, IconName};
use tone_dot::tone_dot;

pub struct TreeItem<'a, Message> {
    label: &'a str,
    depth: usize,
    expanded: Option<bool>,
    selected: bool,
    disabled: bool,
    leading_icon: Option<IconName>,
    tone: Option<ToneRole>,
    trailing_text: Option<&'a str>,
    trailing: Option<Element<'a, Message>>,
    size: ControlSize,
    on_press: Option<Message>,
    on_toggle: Option<Message>,
}

impl<'a, Message> TreeItem<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            depth: 0,
            expanded: None,
            selected: false,
            disabled: false,
            leading_icon: None,
            tone: None,
            trailing_text: None,
            trailing: None,
            size: ControlSize::Sm,
            on_press: None,
            on_toggle: None,
        }
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    pub fn leaf(mut self) -> Self {
        self.expanded = None;
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

    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn tone(mut self, tone: ToneRole) -> Self {
        self.tone = Some(tone);
        self
    }

    pub fn trailing_text(mut self, trailing: &'a str) -> Self {
        self.trailing_text = Some(trailing);
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

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    pub fn on_toggle(mut self, message: Message) -> Self {
        self.on_toggle = Some(message);
        self
    }

    pub fn on_toggle_maybe(mut self, message: Option<Message>) -> Self {
        self.on_toggle = message;
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_tree_item::metrics(self.size);
        let mut row = Row::new()
            .spacing(0)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fixed(metrics.height));

        row = row.push(Space::new().width(Length::Fixed(metrics.indent * self.depth as f32)));
        row = row.push(self.expander(metrics));
        row = row.push(self.main_button(metrics));

        row.into()
    }

    fn expander(&self, metrics: theme_tree_item::TreeItemMetrics) -> Element<'a, Message> {
        match self.expanded {
            Some(expanded) => {
                let icon = if expanded {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                };
                let activation = if self.disabled {
                    None
                } else {
                    self.on_toggle.clone().or_else(|| self.on_press.clone())
                };
                let button = button::Button::new(expander_content(icon, metrics.icon_size))
                    .style(theme_tree_item::expander_style(metrics.radius))
                    .padding(Padding::ZERO)
                    .width(Length::Fixed(metrics.expander_side))
                    .height(Length::Fixed(metrics.height))
                    .on_press_maybe(activation.clone());

                Pressable::maybe(
                    button,
                    activation,
                    metrics.radius.into(),
                    ButtonFocusRing::Default,
                )
            }
            None => Space::new()
                .width(Length::Fixed(metrics.expander_side))
                .into(),
        }
    }

    fn main_button(self, metrics: theme_tree_item::TreeItemMetrics) -> Element<'a, Message> {
        let variant = if self.selected {
            TreeItemVariant::Selected
        } else {
            TreeItemVariant::Default
        };
        let activation = if self.disabled {
            None
        } else {
            self.on_press.clone()
        };
        let content = self.main_content(metrics);

        let button = button::Button::new(content)
            .style(theme_tree_item::item_style(variant, metrics.radius))
            .padding(Padding::ZERO.horizontal(metrics.padding_h))
            .width(Length::Fill)
            .height(Length::Fixed(metrics.height))
            .on_press_maybe(activation.clone());

        Pressable::maybe(
            button,
            activation,
            metrics.radius.into(),
            ButtonFocusRing::Default,
        )
    }

    fn main_content(self, metrics: theme_tree_item::TreeItemMetrics) -> Element<'a, Message> {
        let mut content = Row::new()
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        if let Some(tone) = self.tone {
            content = content.push(tone_dot(tone, metrics.tone_size));
        }

        if let Some(icon) = self.leading_icon {
            content = content.push(icon_widget::new(icon).size(metrics.icon_size));
        }

        content = content.push(
            text(self.label)
                .size(metrics.font_size)
                .shaping(text::Shaping::Auto)
                .width(Length::Fill),
        );

        if let Some(trailing) = self.trailing_text {
            content = content.push(
                text(trailing)
                    .size(metrics.font_size)
                    .shaping(text::Shaping::Auto),
            );
        }

        if let Some(trailing) = self.trailing {
            content = content.push(trailing);
        }

        content.into()
    }
}

fn expander_content<'a, Message>(icon: IconName, icon_size: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    container(icon_widget::new(icon).size(icon_size))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

impl<'a, Message> From<TreeItem<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(item: TreeItem<'a, Message>) -> Self {
        item.into_element()
    }
}

#[cfg(test)]
mod tree_item_tests {
    use super::*;

    #[derive(Clone)]
    enum TestMessage {}

    #[test]
    fn main_content_fills_fixed_button_height() {
        let metrics = theme_tree_item::metrics(ControlSize::Sm);
        let content = TreeItem::<TestMessage>::new("Node").main_content(metrics);

        assert_eq!(content.as_widget().size().height, Length::Fill);
    }

    #[test]
    fn expander_content_fills_fixed_button_height() {
        let metrics = theme_tree_item::metrics(ControlSize::Sm);
        let content = expander_content::<TestMessage>(IconName::ChevronRight, metrics.icon_size);

        assert_eq!(content.as_widget().size().height, Length::Fill);
    }
}
