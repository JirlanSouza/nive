use iced::{
    advanced::{
        layout, mouse, overlay,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use super::{
    overlay::PopoverOverlay,
    placement::{translated_bounds, PopoverCollision, PopoverPlacement, PopoverWidth},
};
use crate::Element;

pub(super) struct PopoverWidget<'a, Message> {
    pub(super) anchor: Element<'a, Message>,
    pub(super) content: Element<'a, Message>,
    pub(super) open: bool,
    pub(super) placement: PopoverPlacement,
    pub(super) width: PopoverWidth,
    pub(super) collision: PopoverCollision,
    pub(super) gap: f32,
    pub(super) on_dismiss: Option<Message>,
    pub(super) trap_focus: bool,
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for PopoverWidget<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.anchor), Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.len() > 2 {
            tree.children.truncate(2);
        }

        if tree.children.is_empty() {
            tree.children.push(Tree::new(&self.anchor));
        } else {
            tree.children[0].diff(self.anchor.as_widget());
        }

        if tree.children.len() < 2 {
            tree.children.push(Tree::new(&self.content));
        } else {
            tree.children[1].diff(self.content.as_widget());
        }
    }

    fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.anchor.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.anchor
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        self.anchor
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.anchor.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.anchor.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        inherited_style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.anchor.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        let (anchor_tree, content_tree) = tree.children.split_at_mut(1);
        let anchor_state = &mut anchor_tree[0];
        let content_state = &mut content_tree[0];

        let anchor_overlay = self.anchor.as_widget_mut().overlay(
            anchor_state,
            layout,
            renderer,
            viewport,
            translation,
        );

        let popover_overlay = if self.open {
            Some(overlay::Element::new(Box::new(
                PopoverOverlay::new(
                    translated_bounds(layout.bounds(), translation),
                    &mut self.content,
                    content_state,
                    self.placement,
                    self.width,
                    self.collision,
                    self.gap,
                    self.on_dismiss.clone(),
                    |message, shell: &mut Shell<'_, Message>| shell.publish(message),
                )
                .trap_focus(self.trap_focus)
                .with_nested_overlay_map(identity_message::<Message>),
            )))
        } else {
            None
        };

        match (anchor_overlay, popover_overlay) {
            (Some(anchor), Some(popover)) => {
                Some(overlay::Group::with_children(vec![anchor, popover]).overlay())
            }
            (Some(anchor), None) => Some(anchor),
            (None, Some(popover)) => Some(popover),
            (None, None) => None,
        }
    }
}

fn identity_message<Message>(message: Message) -> Message {
    message
}
