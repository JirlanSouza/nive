use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use crate::Element;

pub(super) struct FieldGrid<'a, Message> {
    children: Vec<Element<'a, Message>>,
    gap: f32,
    minimum: Option<f32>,
}

impl<'a, Message> FieldGrid<'a, Message> {
    pub(super) fn new(children: Vec<Element<'a, Message>>, gap: f32, minimum: Option<f32>) -> Self {
        Self {
            children,
            gap,
            minimum,
        }
    }
}

impl<Message> Widget<Message, crate::theme::Theme, iced::Renderer> for FieldGrid<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let available = limits.max().width;
        let finite = available.is_finite();
        let columns = match (self.minimum, finite) {
            (Some(minimum), true) => {
                (((available + self.gap) / (minimum + self.gap)).floor() as usize).max(1)
            }
            _ => 1,
        };
        let track = finite.then(|| {
            ((available - self.gap * columns.saturating_sub(1) as f32) / columns as f32).max(0.0)
        });
        let mut nodes = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .map(|(child, state)| {
                let child_limits = match track {
                    Some(track) => limits.loose().width(Length::Fixed(track)),
                    None => limits.loose().width(Length::Shrink),
                };
                child.as_widget_mut().layout(state, renderer, &child_limits)
            })
            .collect::<Vec<_>>();
        let mut y = 0.0;
        let mut intrinsic_width: f32 = 0.0;

        for row in nodes.chunks_mut(columns) {
            let row_height = row
                .iter()
                .map(|node| node.size().height)
                .fold(0.0_f32, f32::max);
            for (column, node) in row.iter_mut().enumerate() {
                let x = track.map_or(0.0, |track| column as f32 * (track + self.gap));
                intrinsic_width = intrinsic_width.max(x + node.size().width);
                node.move_to_mut((x, y));
            }
            y += row_height + self.gap;
        }
        if !nodes.is_empty() {
            y -= self.gap;
        }

        let width = if finite && !limits.compression().width {
            Length::Fill
        } else {
            Length::Shrink
        };
        let size = limits.resolve(width, Length::Shrink, Size::new(intrinsic_width, y));
        layout::Node::with_children(size, nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((child, state), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                state, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
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
        for ((child, state), layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(_, layout)| layout.bounds().intersects(viewport))
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
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
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

impl<'a, Message> From<FieldGrid<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(grid: FieldGrid<'a, Message>) -> Self {
        Element::new(grid)
    }
}
