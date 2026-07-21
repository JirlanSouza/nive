use iced::{widget::Row, Alignment, Length};

use crate::Element;

use super::action::ToolbarAction;
use super::style as theme_toolbar;

pub struct ToolbarGroup<'a, Message> {
    items: Vec<ToolbarAction<'a, Message>>,
}

impl<'a, Message: Clone + 'a> ToolbarGroup<'a, Message> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(mut self, action: ToolbarAction<'a, Message>) -> Self {
        self.items.push(action);
        self
    }

    pub fn action(self, action: ToolbarAction<'a, Message>) -> Self {
        self.push(action)
    }

    pub(super) fn into_element(
        self,
        metrics: theme_toolbar::ToolbarMetrics,
    ) -> Element<'a, Message> {
        let mut items = Row::new()
            .spacing(metrics.item_gap)
            .align_y(Alignment::Center)
            .height(Length::Fixed(metrics.action_height));

        for action in self.items {
            items = items.push(action.into_element(metrics));
        }

        items.into()
    }
}

impl<'a, Message: Clone + 'a> Default for ToolbarGroup<'a, Message> {
    fn default() -> Self {
        Self::new()
    }
}
