use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use super::{BranchEvent, MenuBranch, MenuBranchHandle};
use crate::{
    widgets::overlays::anchored_overlay::{
        scroll::EnsureVisibleHandle, translated_bounds, AnchoredOverlay, PopoverCollision,
        PopoverPlacement, PopoverWidth,
    },
    Element,
};

impl<'a, Message> MenuBranch<'a, Message>
where
    Message: 'a,
{
    pub(in crate::widgets::navigation::menu) fn new(
        anchor: impl Into<Element<'a, Message>>,
        content: Element<'a, Message>,
        handle: MenuBranchHandle,
        ensure_visible: EnsureVisibleHandle,
    ) -> Self {
        Self {
            anchor: anchor.into(),
            content: content.map(BranchEvent::Content),
            handle,
            ensure_visible,
        }
    }
}

impl<'menu, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for MenuBranch<'menu, Message>
where
    Message: Clone + 'menu,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.anchor), Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.len() != 2 {
            tree.children = self.children();
            return;
        }
        tree.children[0].diff(self.anchor.as_widget());
        tree.children[1].diff(self.content.as_widget());
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
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.anchor.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, crate::theme::Theme, iced::Renderer>> {
        let (anchor_tree, content_tree) = tree.children.split_at_mut(1);
        let anchor = self.anchor.as_widget_mut().overlay(
            &mut anchor_tree[0],
            layout,
            renderer,
            viewport,
            translation,
        );
        let branch = self.handle.open.get().then(|| {
            let open = self.handle.open();
            let pointer_inside = self.handle.pointer_inside();
            overlay::Element::new(Box::new(
                AnchoredOverlay::new(
                    translated_bounds(layout.bounds(), translation),
                    &mut self.content,
                    &mut content_tree[0],
                    PopoverPlacement::RightStart,
                    PopoverWidth::Content,
                    PopoverCollision::FlipAndShift,
                    0.0,
                    Some(BranchEvent::Close),
                    move |event, shell: &mut Shell<'_, Message>| match event {
                        BranchEvent::Content(message) => shell.publish(message),
                        BranchEvent::Close => {
                            open.set(false);
                            pointer_inside.set(false);
                            shell.invalidate_layout();
                            shell.request_redraw();
                        }
                    },
                )
                .identity(self.handle.identity.borrow().clone())
                .report_bounds(self.handle.child_bounds.as_ref())
                .ensure_visible(self.ensure_visible.clone())
                .with_nested_overlay_map(unwrap_branch_event::<Message>),
            ))
        });

        match (anchor, branch) {
            (Some(anchor), Some(branch)) => {
                Some(overlay::Group::with_children(vec![anchor, branch]).overlay())
            }
            (Some(anchor), None) => Some(anchor),
            (None, Some(branch)) => Some(branch),
            (None, None) => None,
        }
    }
}

fn unwrap_branch_event<Message>(event: BranchEvent<Message>) -> Message {
    match event {
        BranchEvent::Content(message) => message,
        BranchEvent::Close => unreachable!("nested branch close is handled by its owner"),
    }
}
