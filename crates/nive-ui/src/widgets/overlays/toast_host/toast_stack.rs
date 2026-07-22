use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Alignment, Event, Length, Padding, Rectangle, Size, Vector,
};

use super::{ToastStack, ToastStackState};
use crate::{Element, Renderer, Theme};

impl<'a, Message> ToastStack<'a, Message> {
    pub(super) fn new(items: Vec<(u64, Element<'a, Message>)>) -> Self {
        let (keys, children) = items.into_iter().unzip();
        Self {
            keys,
            children,
            spacing: 0.0,
            width: Length::Shrink,
            max_width: f32::INFINITY,
        }
    }

    pub(super) fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub(super) fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub(super) fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = max_width;
        self
    }
}

impl<Message> Widget<Message, Theme, Renderer> for ToastStack<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ToastStackState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ToastStackState {
            keys: self.keys.clone(),
        })
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<ToastStackState>();

        if tree.children.len() != self.children.len() {
            tree.children = self.children.iter().map(Tree::new).collect();
        } else {
            for (index, (child, child_tree)) in self
                .children
                .iter()
                .zip(tree.children.iter_mut())
                .enumerate()
            {
                if state.keys.get(index) == Some(&self.keys[index]) {
                    child.as_widget().diff(child_tree);
                } else {
                    *child_tree = Tree::new(child.as_widget());
                }
            }
        }

        state.keys.clone_from(&self.keys);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.max_width(self.max_width).width(self.width);

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &limits,
            self.width,
            Length::Shrink,
            Padding::ZERO,
            self.spacing,
            Alignment::Start,
            &mut self.children,
            &mut tree.children,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, state), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, viewport);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
