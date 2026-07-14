use std::borrow::Cow;

use iced::{
    widget::{button, container, text, Row, Space},
    Alignment, Length, Padding,
};

use crate::theme::{ControlSize, ToneRole};
use crate::Element;

use self::style::{self as theme_tree_item, TreeItemVariant};
use crate::widgets::controls::button::ButtonFocusRing;

mod style;
use crate::advanced::pressable::Pressable;
use crate::widgets::primitives::tone_dot::tone_dot;
use crate::widgets::primitives::{icon as icon_widget, IconRole};

/// Primitive row widget for rendering one tree-like item.
///
/// `TreeItem` is intentionally stateless. It renders indentation, an optional
/// branch expander, selection/disabled styling, leading icon, tone indicator,
/// trailing content, and row/toggle messages. It does not own hierarchy,
/// selection, focus, keyboard navigation, type-ahead, clipboard, or drag/drop
/// behavior; use [`super::Tree`] for the high-level hierarchy widget.
pub struct TreeItem<'a, Message> {
    label: Cow<'a, str>,
    depth: usize,
    expanded: Option<bool>,
    selected: bool,
    disabled: bool,
    focused: bool,
    leading_icon: Option<IconRole>,
    tone: Option<ToneRole>,
    trailing_text: Option<Cow<'a, str>>,
    trailing: Option<Element<'a, Message>>,
    size: ControlSize,
    on_press: Option<Message>,
    on_toggle: Option<Message>,
    on_context_request: Option<Message>,
    dragging: bool,
}

impl<'a, Message> TreeItem<'a, Message>
where
    Message: Clone + 'a,
{
    /// Creates a row with a text label.
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            depth: 0,
            expanded: None,
            selected: false,
            disabled: false,
            focused: false,
            leading_icon: None,
            tone: None,
            trailing_text: None,
            trailing: None,
            size: ControlSize::Sm,
            on_press: None,
            on_toggle: None,
            on_context_request: None,
            dragging: false,
        }
    }

    /// Sets indentation depth, where `0` is a root-level row.
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Renders the row as a branch with the given expansion state.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    /// Renders the row as a leaf with no expander affordance.
    pub fn leaf(mut self) -> Self {
        self.expanded = None;
        self
    }

    /// Sets selected styling for the full row.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets disabled styling and suppresses row and toggle messages.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the row as holding tree model focus. Renders a layout-neutral
    /// focus-visible affordance independent of `selected`.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Adds a leading icon before the label.
    pub fn leading_icon(mut self, icon: IconRole) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Adds a tone indicator to the row.
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

    /// Adds trailing secondary text.
    pub fn trailing_text(mut self, trailing: impl Into<Cow<'a, str>>) -> Self {
        self.trailing_text = Some(trailing.into());
        self
    }

    /// Adds custom trailing content.
    pub fn trailing(mut self, trailing: impl Into<Element<'a, Message>>) -> Self {
        self.trailing = Some(trailing.into());
        self
    }

    /// Sets row size.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Uses extra-small row size.
    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    /// Uses small row size.
    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    /// Uses medium row size.
    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    /// Uses large row size.
    pub fn lg(self) -> Self {
        self.size(ControlSize::Lg)
    }

    /// Emits `message` when the main row body is pressed.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets an optional row press message.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    /// Emits `message` when the branch expander is pressed.
    pub fn on_toggle(mut self, message: Message) -> Self {
        self.on_toggle = Some(message);
        self
    }

    /// Sets an optional branch expander message.
    pub fn on_toggle_maybe(mut self, message: Option<Message>) -> Self {
        self.on_toggle = message;
        self
    }

    /// Stores a context-request message for composed tree implementations.
    pub fn on_context_request(mut self, message: Message) -> Self {
        self.on_context_request = Some(message);
        self
    }

    /// Sets an optional context-request message.
    pub fn on_context_request_maybe(mut self, message: Option<Message>) -> Self {
        self.on_context_request = message;
        self
    }

    /// Sets dragging feedback styling.
    pub fn dragging(mut self, dragging: bool) -> Self {
        self.dragging = dragging;
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_tree_item::metrics(self.size);
        let mut row = Row::new()
            .spacing(0)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fixed(metrics.height));

        row = row.push(self.indent_button(metrics));
        let selected = self.selected;
        let disabled = self.disabled;
        let focused = self.focused;

        row = row.push(self.expander(metrics));
        row = row.push(self.main_button(metrics));

        container(row)
            .style(theme_tree_item::row_style(selected, disabled, focused))
            .width(Length::Fill)
            .height(Length::Fixed(metrics.height))
            .into()
    }

    fn indent_button(&self, metrics: theme_tree_item::TreeItemMetrics) -> Element<'a, Message> {
        let indent_width = metrics.indent * self.depth as f32;
        if indent_width == 0.0 {
            return Space::new().width(Length::Fixed(0.0)).into();
        }

        button::Button::new(Space::new().width(Length::Fill).height(Length::Fill))
            .style(theme_tree_item::indent_style(self.selected))
            .padding(Padding::ZERO)
            .width(Length::Fixed(indent_width))
            .height(Length::Fixed(metrics.height))
            .into()
    }

    fn expander(&self, metrics: theme_tree_item::TreeItemMetrics) -> Element<'a, Message> {
        match self.expanded {
            Some(expanded) => {
                let icon = if expanded {
                    IconRole::NiveDisclosureDown
                } else {
                    IconRole::NiveDisclosureRight
                };
                let activation = if self.disabled {
                    None
                } else {
                    self.on_toggle.clone().or_else(|| self.on_press.clone())
                };
                let button = button::Button::new(expander_content(icon, metrics.icon_size))
                    .style(theme_tree_item::expander_style(
                        self.selected,
                        metrics.radius,
                    ))
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
        let disabled = self.disabled;
        let mut content = Row::new()
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill);

        if let Some(tone) = self.tone {
            content = content.push(tone_dot(tone, metrics.tone_size));
        }

        if let Some(icon) = self.leading_icon {
            content = content.push(icon_widget::role(icon).custom_size(metrics.icon_size));
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
                    .shaping(text::Shaping::Auto)
                    .style(theme_tree_item::trailing_text_style(disabled)),
            );
        }

        if let Some(trailing) = self.trailing {
            content = content.push(trailing);
        }

        content.into()
    }
}

fn expander_content<'a, Message>(icon: IconRole, icon_size: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    container(icon_widget::role(icon).custom_size(icon_size))
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
    fn accepts_owned_label_and_trailing_text() {
        let metrics = theme_tree_item::metrics(ControlSize::Sm);
        let item =
            TreeItem::<TestMessage>::new(String::from("Node")).trailing_text(String::from("4 KB"));

        assert_eq!(item.label.as_ref(), "Node");
        assert_eq!(item.trailing_text.as_deref(), Some("4 KB"));

        let content = item.main_content(metrics);

        assert_eq!(content.as_widget().size().height, Length::Fill);
    }

    #[test]
    fn expander_content_fills_fixed_button_height() {
        let metrics = theme_tree_item::metrics(ControlSize::Sm);
        let content =
            expander_content::<TestMessage>(IconRole::NiveDisclosureRight, metrics.icon_size);

        assert_eq!(content.as_widget().size().height, Length::Fill);
    }

    #[test]
    fn default_context_request_and_dragging_are_unset() {
        let item = TreeItem::<TestMessage>::new("Node");

        assert!(item.on_context_request.is_none());
        assert!(!item.dragging);
    }

    #[test]
    fn indent_button_is_space_at_depth_zero() {
        let metrics = theme_tree_item::metrics(ControlSize::Sm);
        let item = TreeItem::<TestMessage>::new("Node").depth(0);
        let element = item.indent_button(metrics);

        assert_eq!(element.as_widget().size().width, Length::Fixed(0.0));
    }
}
