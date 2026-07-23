use iced::{
    advanced::{
        layout, overlay,
        widget::{operation, Tree},
        Layout,
    },
    Length, Point, Rectangle, Size, Vector,
};

use crate::theme::Theme;
use crate::widgets::controls::segmented_control::typed::{
    SegmentedControl, SegmentedControlVariant, SegmentedFocus, SegmentedState,
};

impl<'a, T, Message> SegmentedControl<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    pub(super) fn layout_impl(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let metrics = self.metrics(crate::theme::active());
        let limits = limits
            .width(self.width)
            .height(Length::Fixed(metrics.form.height));
        let border = metrics.perimeter_width;
        let inset = if self.variant == SegmentedControlVariant::Default {
            2.0
        } else {
            0.0
        };
        let padding = metrics.form.padding.left.max(0.0);
        let mut intrinsic_items = Vec::with_capacity(self.options.len());

        for index in 0..self.options.len() {
            let mut content = self.item_content(index, f32::INFINITY);
            tree.children[index].diff(content.as_widget());
            let node = content.as_widget_mut().layout(
                &mut tree.children[index],
                renderer,
                &layout::Limits::NONE,
            );
            intrinsic_items.push(node.size().width + padding * 2.0);
        }

        let intrinsic_width = intrinsic_items.iter().sum::<f32>() + (border + inset) * 2.0;
        let resolved = limits.resolve(
            self.width,
            Length::Fixed(metrics.form.height),
            Size::new(intrinsic_width, metrics.form.height),
        );
        let inner_width = (resolved.width - (border + inset) * 2.0).max(0.0);
        let item_width = if self.options.is_empty() {
            0.0
        } else {
            inner_width / self.options.len() as f32
        };
        let item_height = (resolved.height - (border + inset) * 2.0).max(0.0);
        let mut nodes = Vec::with_capacity(self.options.len());
        let mut bounds = Vec::with_capacity(self.options.len());
        let left = border + inset;
        let right = left + inner_width;

        for index in 0..self.options.len() {
            let x = left + item_width * index as f32;
            let next_x = if index + 1 == self.options.len() {
                right
            } else {
                left + item_width * (index + 1) as f32
            };
            let item = Rectangle {
                x,
                y: border + inset,
                width: (next_x - x).max(0.0),
                height: item_height,
            };
            let content_width = (item.width - padding * 2.0).max(0.0);
            let content = self.item_content(index, content_width);
            self.contents[index] = content;
            tree.children[index].diff(self.contents[index].as_widget());
            let node = self.contents[index].as_widget_mut().layout(
                &mut tree.children[index],
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(content_width, item.height)),
            );
            let content_y = item.y + (item.height - node.size().height).max(0.0) / 2.0;
            nodes.push(node.move_to(Point::new(item.x + padding, content_y)));
            bounds.push(item);
        }

        tree.state.downcast_mut::<SegmentedState>().item_bounds = bounds;
        layout::Node::with_children(resolved, nodes)
    }

    pub(super) fn operate_impl(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<SegmentedState>();
        if self.interactive() {
            let SegmentedState {
                focus,
                focused_index,
                pressed_index,
                touch,
                ..
            } = state;
            focus.expose(operation, self.id.as_ref(), layout.bounds());
            operation.focusable(
                self.id.as_ref(),
                layout.bounds(),
                &mut SegmentedFocus {
                    focus,
                    focused_index,
                    pressed_index,
                    touch,
                },
            );
        } else {
            state.focus.clear();
        }
        for (index, (tree, child_layout)) in
            tree.children.iter_mut().zip(layout.children()).enumerate()
        {
            self.contents[index]
                .as_widget_mut()
                .operate(tree, child_layout, renderer, operation);
        }
    }

    pub(super) fn overlay_impl<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        let overlays = self
            .contents
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
            .filter_map(|((content, tree), layout)| {
                content
                    .as_widget_mut()
                    .overlay(tree, layout, renderer, viewport, translation)
            })
            .collect::<Vec<_>>();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}
