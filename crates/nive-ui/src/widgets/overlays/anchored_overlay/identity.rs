use std::any::Any;

use iced::advanced::widget::Operation;
use iced::{advanced::Layout, Rectangle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OverlayIdentity {
    path: Vec<u32>,
}

impl OverlayIdentity {
    pub(crate) fn root() -> Self {
        Self { path: vec![0] }
    }

    fn child(&self, ordinal: u32) -> Self {
        let mut path = self.path.clone();
        path.push(ordinal);
        Self { path }
    }

    pub(crate) fn depth(&self) -> usize {
        self.path.len().saturating_sub(1)
    }

    #[cfg(test)]
    pub(crate) fn is_direct_child_of(&self, parent: &Self) -> bool {
        self.path.len() == parent.path.len() + 1 && self.path.starts_with(&parent.path)
    }
}

#[derive(Debug)]
pub(crate) struct OverlayNodeState {
    identity: OverlayIdentity,
}

impl Default for OverlayNodeState {
    fn default() -> Self {
        Self {
            identity: OverlayIdentity::root(),
        }
    }
}

impl OverlayNodeState {
    pub(crate) fn identity(&self) -> &OverlayIdentity {
        &self.identity
    }
}

pub(crate) fn expose_node(
    operation: &mut dyn Operation,
    bounds: Rectangle,
    state: &mut OverlayNodeState,
) {
    operation.custom(None, bounds, state);
}

pub(crate) fn bind_descendants<Message>(
    parent: &OverlayIdentity,
    content: &mut crate::Element<'_, Message>,
    tree: &mut iced::advanced::widget::Tree,
    layout: Layout<'_>,
    renderer: &iced::Renderer,
) {
    let mut operation = BindOverlayChildren {
        parent,
        next_ordinal: 0,
    };
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut operation);
}

struct BindOverlayChildren<'a> {
    parent: &'a OverlayIdentity,
    next_ordinal: u32,
}

impl Operation for BindOverlayChildren<'_> {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&iced::widget::Id>, _bounds: Rectangle, state: &mut dyn Any) {
        let Some(node) = state.downcast_mut::<OverlayNodeState>() else {
            return;
        };

        node.identity = self.parent.child(self.next_ordinal);
        self.next_ordinal = self.next_ordinal.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_identity_is_chain_local_and_ordered() {
        let root = OverlayIdentity::root();
        let first = root.child(0);
        let second = root.child(1);

        assert!(first.is_direct_child_of(&root));
        assert!(second.is_direct_child_of(&root));
        assert_ne!(first, second);
        assert_eq!(first.depth(), 1);
    }
}
