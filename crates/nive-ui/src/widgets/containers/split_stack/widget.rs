use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Point, Rectangle, Size, Vector,
};

use crate::interaction::{Orientation, StepAdjustment};

use super::super::split_divider::{draw_grip, metrics, resize_interaction, resolve_visual_state};
use super::sizing;
use super::state::SplitStackState;
use super::SplitStack;

use self::event::{divider_hit_bounds, focused_hit_bounds};

mod event;

#[cfg(test)]
mod tests;

pub(super) const KEYBOARD_STEP: StepAdjustment = StepAdjustment::new(0.01, 0.1);

/// Layout nodes interleave panes and dividers, so panes sit on even indices.
pub(super) fn pane_layouts<'a>(layout: Layout<'a>) -> impl Iterator<Item = Layout<'a>> {
    layout.children().step_by(2)
}

pub(super) fn divider_layouts<'a>(layout: Layout<'a>) -> impl Iterator<Item = Layout<'a>> {
    layout.children().skip(1).step_by(2)
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer> for SplitStack<'a, Message>
where
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SplitStackState>()
    }

    fn state(&self) -> tree::State {
        SplitStackState::new_state()
    }

    fn children(&self) -> Vec<Tree> {
        self.contents.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.contents);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn size_hint(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(self.width, self.height, Size::ZERO);
        if self.contents.is_empty() {
            return layout::Node::with_children(size, Vec::new());
        }

        let orientation = self.orientation;
        let divider_length = metrics(self.size).layout_thickness;
        let main = orientation.main_length(size);
        let cross = orientation.cross_length(size);
        let dividers = self.contents.len() - 1;
        let available = (main - divider_length * dividers as f32).max(0.0);

        let state = tree.state.downcast_mut::<SplitStackState>();
        sizing::resolve_into(&self.sizing, &self.minimums, available, &mut state.resolved);
        state.available = available;
        state.focused_divider = state.focused_divider.min(dividers.saturating_sub(1));

        let mut nodes = Vec::with_capacity(self.contents.len() + dividers);
        let mut offset = 0.0;

        for (index, content) in self.contents.iter_mut().enumerate() {
            let length = state.resolved.get(index).copied().unwrap_or(0.0);
            let pane_size = orientation.size(length, cross);
            nodes.push(
                content
                    .as_widget_mut()
                    .layout(
                        &mut tree.children[index],
                        renderer,
                        &layout::Limits::new(pane_size, pane_size),
                    )
                    .move_to(origin(orientation, offset)),
            );
            offset += length;

            if index < dividers {
                let divider_size = orientation.size(divider_length, cross);
                nodes.push(layout::Node::new(divider_size).move_to(origin(orientation, offset)));
                offset += divider_length;
            }
        }

        layout::Node::with_children(size, nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let hit = focused_hit_bounds(self, tree, layout);
        let state = tree.state.downcast_mut::<SplitStackState>();

        match hit {
            Some(hit) if self.interactive() => {
                state.focus.register(operation, self.id.as_ref(), hit);
            }
            _ => {
                state.focus.clear();
                state.drag = None;
            }
        }

        for ((content, child), layout) in self
            .contents
            .iter_mut()
            .zip(&mut tree.children)
            .zip(pane_layouts(layout))
        {
            content
                .as_widget_mut()
                .operate(child, layout, renderer, operation);
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
        self.update_impl(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
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
        let state = tree.state.downcast_ref::<SplitStackState>();
        let over_divider = divider_layouts(layout)
            .any(|divider| cursor.is_over(divider_hit_bounds(self, divider, layout)));

        if self.interactive() && (state.drag.is_some() || over_divider) {
            return resize_interaction(self.orientation);
        }

        self.contents
            .iter()
            .zip(&tree.children)
            .zip(pane_layouts(layout))
            .map(|((content, child), layout)| {
                content
                    .as_widget()
                    .mouse_interaction(child, layout, cursor, viewport, renderer)
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
        for ((content, child), pane) in self
            .contents
            .iter()
            .zip(&tree.children)
            .zip(pane_layouts(layout))
        {
            content.as_widget().draw(
                child,
                renderer,
                theme,
                inherited_style,
                pane,
                cursor,
                viewport,
            );
        }

        let state = tree.state.downcast_ref::<SplitStackState>();
        let metrics = metrics(self.size);

        for (index, divider) in divider_layouts(layout).enumerate() {
            let engaged = state.drag.is_some_and(|drag| drag.divider == index)
                || (state.focus.is_focus_visible() && state.focused_divider == index);
            let hovered = cursor.is_over(divider_hit_bounds(self, divider, layout));
            let visual = resolve_visual_state(self.interactive(), engaged, hovered);

            draw_grip(
                renderer,
                theme,
                divider.bounds(),
                self.orientation,
                metrics,
                visual,
            );
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
        let overlays = self
            .contents
            .iter_mut()
            .zip(&mut tree.children)
            .zip(pane_layouts(layout))
            .filter_map(|((content, child), pane)| {
                content
                    .as_widget_mut()
                    .overlay(child, pane, renderer, viewport, translation)
            })
            .collect::<Vec<_>>();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

impl<Message> SplitStack<'_, Message> {
    pub(super) fn interactive(&self) -> bool {
        !self.locked && self.on_resize.is_some() && self.contents.len() > 1
    }
}

/// Position of a node that starts `offset` along the main axis.
fn origin(orientation: Orientation, offset: f32) -> Point {
    match orientation {
        Orientation::Horizontal => Point::new(offset, 0.0),
        Orientation::Vertical => Point::new(0.0, offset),
    }
}
