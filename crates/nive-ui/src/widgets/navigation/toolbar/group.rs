use iced::{
    widget::{container, Row},
    Alignment, Length,
};

use crate::Element;

use super::action::ToolbarAction;
use super::separator::separator;
use super::style as theme_toolbar;

pub struct ToolbarGroup<'a, Message> {
    items: Vec<ToolbarItem<'a, Message>>,
}

enum ToolbarItem<'a, Message> {
    Action(ToolbarAction<'a, Message>),
    Separator,
}

impl<'a, Message: Clone + 'a> ToolbarGroup<'a, Message> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(mut self, action: ToolbarAction<'a, Message>) -> Self {
        self.items.push(ToolbarItem::Action(action));
        self
    }

    pub fn action(self, action: ToolbarAction<'a, Message>) -> Self {
        self.push(action)
    }

    pub fn separator(mut self) -> Self {
        self.items.push(ToolbarItem::Separator);
        self
    }

    pub(super) fn into_element(
        self,
        metrics: theme_toolbar::ToolbarMetrics,
    ) -> Element<'a, Message> {
        let mut items = Row::new()
            .spacing(metrics.item_gap)
            .align_y(Alignment::Center)
            .height(Length::Fixed(metrics.action_height));

        for item in self.items {
            items = items.push(match item {
                ToolbarItem::Action(action) => action.into_element(metrics),
                ToolbarItem::Separator => separator(metrics),
            });
        }

        container(items)
            .style(theme_toolbar::group_style(metrics.radius))
            .padding(metrics.group_padding)
            .height(Length::Fixed(
                metrics.action_height + metrics.group_padding * 2.0,
            ))
            .into()
    }
}

impl<'a, Message: Clone + 'a> Default for ToolbarGroup<'a, Message> {
    fn default() -> Self {
        Self::new()
    }
}
