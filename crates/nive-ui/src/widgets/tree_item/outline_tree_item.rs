use crate::theme::{ControlSize, ToneRole};
use crate::Element;

use super::super::AppIcon;
use super::TreeItem;

pub struct OutlineTreeItem<'a, Message> {
    item: TreeItem<'a, Message>,
}

impl<'a, Message> OutlineTreeItem<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(label: &'a str) -> Self {
        Self {
            item: TreeItem::new(label).xs(),
        }
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.item = self.item.depth(depth);
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.item = self.item.expanded(expanded);
        self
    }

    pub fn leaf(mut self) -> Self {
        self.item = self.item.leaf();
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.item = self.item.selected(selected);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.item = self.item.disabled(disabled);
        self
    }

    pub fn leading_icon(mut self, icon: AppIcon) -> Self {
        self.item = self.item.leading_icon(icon);
        self
    }

    pub fn tone(mut self, tone: ToneRole) -> Self {
        self.item = self.item.tone(tone);
        self
    }

    pub fn trailing_text(mut self, trailing: &'a str) -> Self {
        self.item = self.item.trailing_text(trailing);
        self
    }

    pub fn trailing(mut self, trailing: impl Into<Element<'a, Message>>) -> Self {
        self.item = self.item.trailing(trailing);
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.item = self.item.on_press(message);
        self
    }

    pub fn on_toggle(mut self, message: Message) -> Self {
        self.item = self.item.on_toggle(message);
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.item = self.item.size(size);
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
}

impl<'a, Message> From<OutlineTreeItem<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(item: OutlineTreeItem<'a, Message>) -> Self {
        item.item.into()
    }
}
