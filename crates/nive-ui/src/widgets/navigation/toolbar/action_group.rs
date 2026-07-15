use iced::{widget::Row, Alignment, Length};

use crate::theme::ControlSize;
use crate::Element;

use super::action::ToolbarAction;
use super::separator::separator;
use super::style as theme_toolbar;

/// Inline group of toolbar-style actions that preserves control height.
///
/// Use [`Toolbar`](super::Toolbar) when the actions live in a surface bar.
pub struct ActionGroup<'a, Message> {
    items: Vec<ActionGroupItem<'a, Message>>,
    size: ControlSize,
    width: Length,
}

enum ActionGroupItem<'a, Message> {
    Action(ToolbarAction<'a, Message>),
    Separator,
}

impl<'a, Message> ActionGroup<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            size: ControlSize::Sm,
            width: Length::Shrink,
        }
    }

    pub fn push(mut self, action: ToolbarAction<'a, Message>) -> Self {
        self.items.push(ActionGroupItem::Action(action));
        self
    }

    pub fn action(self, action: ToolbarAction<'a, Message>) -> Self {
        self.push(action)
    }

    pub fn separator(mut self) -> Self {
        self.items.push(ActionGroupItem::Separator);
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

    crate::impl_layout_builders!(fill_width_direct, shrink_width_direct);

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_toolbar::metrics(self.size);
        let mut items = Row::new()
            .spacing(metrics.item_gap)
            .align_y(Alignment::Center)
            .height(Length::Fixed(metrics.action_height));

        for item in self.items {
            items = items.push(match item {
                ActionGroupItem::Action(action) => action.into_element(metrics),
                ActionGroupItem::Separator => separator(metrics),
            });
        }

        items.width(self.width).into()
    }
}

impl<'a, Message> Default for ActionGroup<'a, Message>
where
    Message: Clone + 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> From<ActionGroup<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(group: ActionGroup<'a, Message>) -> Self {
        group.into_element()
    }
}

#[cfg(test)]
mod action_group_tests {
    use super::*;

    #[test]
    fn defaults_to_small_control_size() {
        assert_eq!(ActionGroup::<()>::new().size, ControlSize::Sm);
    }

    #[test]
    fn size_builder_sets_control_size() {
        assert_eq!(
            ActionGroup::<()>::new().size(ControlSize::Lg).size,
            ControlSize::Lg
        );
    }
}
