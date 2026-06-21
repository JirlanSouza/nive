mod outline_tree_item;
mod tone_dot;

use iced::{
    widget::{button, text, Row, Space},
    Alignment, Length, Padding,
};

use crate::theme::{ControlSize, ToneRole};
use crate::Element;

use self::style::{self as theme_tree_item, TreeItemVariant};
use super::button::ButtonFocusRing;

mod style;
use super::{icon as icon_widget, pressable::Pressable, IconName};
use tone_dot::tone_dot;

pub use outline_tree_item::OutlineTreeItem;

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
                let marker = if expanded { "v" } else { ">" };
                let activation = if self.disabled {
                    None
                } else {
                    self.on_toggle.clone().or_else(|| self.on_press.clone())
                };
                let button = button::Button::new(text(marker).size(metrics.font_size))
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
        let mut content = Row::new()
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .width(Length::Fill);

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

        let activation = if self.disabled {
            None
        } else {
            self.on_press.clone()
        };
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
}

impl<'a, Message> From<TreeItem<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(item: TreeItem<'a, Message>) -> Self {
        item.into_element()
    }
}
