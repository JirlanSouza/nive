use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    Event, Length, Point, Rectangle, Size, Vector,
};

use crate::Element;

pub(super) struct FlowItem<'a, Message> {
    element: Element<'a, Message>,
    separator: bool,
}

impl<'a, Message> FlowItem<'a, Message> {
    pub(super) fn action(element: Element<'a, Message>) -> Self {
        Self {
            element,
            separator: false,
        }
    }

    pub(super) fn separator(element: Element<'a, Message>) -> Self {
        Self {
            element,
            separator: true,
        }
    }
}

pub(super) struct Flow<'a, Message> {
    items: Vec<FlowItem<'a, Message>>,
    hidden: Vec<bool>,
    width: Length,
    spacing: f32,
    row_gap: f32,
    wrap: bool,
}

impl<'a, Message> Flow<'a, Message> {
    pub(super) fn new(
        items: Vec<FlowItem<'a, Message>>,
        width: Length,
        spacing: f32,
        row_gap: f32,
        wrap: bool,
    ) -> Self {
        let hidden = vec![false; items.len()];

        Self {
            items,
            hidden,
            width,
            spacing,
            row_gap,
            wrap,
        }
    }
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer> for Flow<'a, Message>
where
    Message: Clone + 'a,
{
    fn children(&self) -> Vec<Tree> {
        self.items
            .iter()
            .map(|item| Tree::new(&item.element))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &self
                .items
                .iter()
                .map(|item| item.element.as_widget())
                .collect::<Vec<_>>(),
        );
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.hidden.fill(false);

        let limits = limits.width(self.width);
        let child_limits = layout::Limits::NONE;
        let maximum = limits.max().width;
        let mut measured = self
            .items
            .iter_mut()
            .zip(&mut tree.children)
            .map(|(item, tree)| {
                item.element
                    .as_widget_mut()
                    .layout(tree, renderer, &child_limits)
            })
            .collect::<Vec<_>>();
        let mut positioned = Vec::with_capacity(measured.len());
        let mut row_start = 0;
        let mut row_height: f32 = 0.0;
        let mut intrinsic_width: f32 = 0.0;
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        let mut index = 0;

        while index < measured.len() {
            let is_separator = self.items[index].separator;
            let size = measured[index].size();

            if is_separator {
                let next_width = measured
                    .get(index + 1)
                    .map_or(0.0, |node| node.size().width);
                let boundary_width = size.width + self.spacing + next_width;

                if self.wrap && x > 0.0 && x + boundary_width > maximum {
                    finish_row(row_start, positioned.len(), row_height, &mut positioned);
                    intrinsic_width = intrinsic_width.max((x - self.spacing).max(0.0));
                    y += row_height + self.row_gap;
                    x = 0.0;
                    row_height = 0.0;
                    self.hidden[index] = true;
                    positioned.push(layout::Node::new(Size::ZERO).move_to(Point::new(0.0, y)));
                    index += 1;
                    row_start = positioned.len();
                    continue;
                }
            } else if self.wrap && x > 0.0 && x + size.width > maximum {
                finish_row(row_start, positioned.len(), row_height, &mut positioned);
                intrinsic_width = intrinsic_width.max((x - self.spacing).max(0.0));
                y += row_height + self.row_gap;
                x = 0.0;
                row_height = 0.0;
                row_start = positioned.len();
            }

            let node = std::mem::replace(&mut measured[index], layout::Node::new(Size::ZERO));
            row_height = row_height.max(size.height);
            positioned.push(node.move_to(Point::new(x, y)));
            x += size.width + self.spacing;
            index += 1;
        }

        finish_row(row_start, positioned.len(), row_height, &mut positioned);
        intrinsic_width = intrinsic_width.max((x - self.spacing).max(0.0));
        let intrinsic = Size::new(intrinsic_width, y + row_height);
        let size = limits.resolve(self.width, Length::Shrink, intrinsic);

        layout::Node::with_children(size, positioned)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        for (index, ((item, tree), child_layout)) in self
            .items
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .enumerate()
        {
            if self.hidden[index] {
                continue;
            }

            item.element
                .as_widget_mut()
                .operate(tree, child_layout, renderer, operation);
        }
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
        for (index, ((item, tree), child_layout)) in self
            .items
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .enumerate()
        {
            if self.hidden[index] {
                continue;
            }

            item.element.as_widget_mut().update(
                tree,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
            if shell.is_event_captured() {
                break;
            }
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
        self.items
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .enumerate()
            .filter_map(|(index, ((item, tree), child_layout))| {
                if self.hidden[index] {
                    return None;
                }

                Some(item.element.as_widget().mouse_interaction(
                    tree,
                    child_layout,
                    cursor,
                    viewport,
                    renderer,
                ))
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let clip = layout.bounds().intersection(viewport).unwrap_or_default();
        renderer.with_layer(clip, |renderer| {
            for (index, ((item, tree), child_layout)) in self
                .items
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .enumerate()
            {
                if self.hidden[index] {
                    continue;
                }

                item.element.as_widget().draw(
                    tree,
                    renderer,
                    theme,
                    inherited_style,
                    child_layout,
                    cursor,
                    &clip,
                );
            }
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        let children = self
            .items
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .enumerate()
            .filter_map(|(index, ((item, tree), child_layout))| {
                if self.hidden[index] {
                    return None;
                }

                item.element.as_widget_mut().overlay(
                    tree,
                    child_layout,
                    renderer,
                    viewport,
                    translation,
                )
            })
            .collect::<Vec<_>>();

        (!children.is_empty()).then(|| overlay::Group::with_children(children).overlay())
    }
}

fn finish_row(start: usize, end: usize, height: f32, nodes: &mut [layout::Node]) {
    if start >= end {
        return;
    }
    for node in &mut nodes[start..end] {
        node.translate_mut(Vector::new(0.0, (height - node.size().height) / 2.0));
    }
}

impl<'a, Message> From<Flow<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(flow: Flow<'a, Message>) -> Self {
        Element::new(flow)
    }
}
