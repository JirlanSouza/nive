use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use super::marker::is_unconsumed_confirm_enter;
use super::{DialogAction, DialogActionFooterWidget, DialogActionRole, ReflowLayout};
use crate::theme::{self, ControlSize, GapRole};
use crate::widgets::controls::button;
use crate::{Element, Renderer, Theme};

pub(super) fn action_button<'a, Message: Clone + 'a>(
    action: DialogAction<'a, Message>,
) -> Element<'a, Message> {
    let mut button = match action.role {
        DialogActionRole::Cancel | DialogActionRole::Secondary => button::secondary(action.label),
        DialogActionRole::Primary => button::primary(action.label),
        DialogActionRole::Destructive => button::destructive(action.label),
    }
    .size(ControlSize::Md)
    .disabled(action.disabled)
    .on_press(action.message);

    if let Some(id) = action.id {
        button = button.id(id);
    }

    button.into()
}

impl<'a, Message> DialogActionFooterWidget<'a, Message>
where
    Message: Clone + 'a,
{
    pub(super) fn slots(&self) -> Vec<&Element<'a, Message>> {
        let mut slots = Vec::with_capacity(self.actions.len() + 1);
        if let Some(status) = &self.status {
            slots.push(status);
        }
        slots.extend(self.actions.iter());
        slots
    }

    pub(super) fn slots_mut(&mut self) -> Vec<&mut Element<'a, Message>> {
        let mut slots = Vec::with_capacity(self.actions.len() + 1);
        if let Some(status) = &mut self.status {
            slots.push(status);
        }
        slots.extend(self.actions.iter_mut());
        slots
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for DialogActionFooterWidget<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn state(&self) -> tree::State {
        tree::State::None
    }

    fn children(&self) -> Vec<Tree> {
        self.slots().iter().map(|slot| Tree::new(*slot)).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let widgets: Vec<_> = self.slots().iter().map(|slot| slot.as_widget()).collect();
        tree.diff_children(&widgets);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn size_hint(&self) -> Size<Length> {
        self.size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let gap = theme::gap(GapRole::Related);
        let max_width = limits.max().width;
        let unbounded = layout::Limits::new(Size::ZERO, Size::new(f32::INFINITY, f32::INFINITY));

        let action_start = if self.status.is_some() { 1 } else { 0 };

        let status_node = self.status.as_mut().map(|status| {
            status
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &unbounded)
        });

        let action_nodes: Vec<_> = self
            .actions
            .iter_mut()
            .enumerate()
            .map(|(index, action)| {
                action.as_widget_mut().layout(
                    &mut tree.children[action_start + index],
                    renderer,
                    &unbounded,
                )
            })
            .collect();

        let actions_width: f32 = action_nodes
            .iter()
            .map(|node| node.size().width)
            .sum::<f32>()
            + gap * action_nodes.len().saturating_sub(1) as f32;
        let actions_height = action_nodes
            .iter()
            .map(|node| node.size().height)
            .fold(0.0_f32, f32::max);
        let status_width = status_node.as_ref().map_or(0.0, |node| node.size().width);
        let status_height = status_node.as_ref().map_or(0.0, |node| node.size().height);

        let single_row_width = if status_node.is_some() {
            status_width + gap + actions_width
        } else {
            actions_width
        };

        let reflow = if single_row_width <= max_width {
            ReflowLayout::SingleRow
        } else if actions_width <= max_width {
            ReflowLayout::StackedStatus
        } else {
            ReflowLayout::StackedActions
        };

        match reflow {
            ReflowLayout::SingleRow => {
                let mut children = Vec::with_capacity(1 + action_nodes.len());

                if let Some(status_node) = status_node {
                    children.push(status_node.move_to(iced::Point::new(0.0, 0.0)));
                }

                let row_height = actions_height.max(status_height);
                let actions_x_start = max_width - actions_width;
                let mut x = actions_x_start.max(0.0);
                for node in action_nodes {
                    let y = (row_height - node.size().height) / 2.0;
                    children.push(node.move_to(iced::Point::new(x, y.max(0.0))));
                    x += children.last().unwrap().size().width + gap;
                }

                layout::Node::with_children(Size::new(max_width, row_height), children)
            }
            ReflowLayout::StackedStatus => {
                let mut children = Vec::with_capacity(1 + action_nodes.len());
                let mut y = 0.0;

                if let Some(status_node) = status_node {
                    let height = status_node.size().height;
                    children.push(status_node.move_to(iced::Point::new(0.0, 0.0)));
                    y = height + gap;
                }

                let actions_x_start = (max_width - actions_width).max(0.0);
                let mut x = actions_x_start;
                for node in action_nodes {
                    children.push(node.move_to(iced::Point::new(x, y)));
                    x += children.last().unwrap().size().width + gap;
                }

                let total_height = y + actions_height;
                layout::Node::with_children(Size::new(max_width, total_height), children)
            }
            ReflowLayout::StackedActions => {
                let mut children = Vec::with_capacity(1 + action_nodes.len());
                let mut y = 0.0;

                if let Some(status_node) = status_node {
                    let height = status_node.size().height;
                    children.push(status_node.move_to(iced::Point::new(0.0, 0.0)));
                    y = height + gap;
                }

                // Full-width stacking changes each action's own layout (not
                // just its outer bounds), so every action is re-laid-out
                // against its real, persistent tree slot with a fixed-width
                // Limits rather than reusing the natural-width measurement.
                let stretched = layout::Limits::new(
                    Size::new(max_width, 0.0),
                    Size::new(max_width, f32::INFINITY),
                );
                for (index, action) in self.actions.iter_mut().enumerate() {
                    let node = action.as_widget_mut().layout(
                        &mut tree.children[action_start + index],
                        renderer,
                        &stretched,
                    );
                    let height = node.size().height;
                    children.push(node.move_to(iced::Point::new(0.0, y)));
                    y += height + gap;
                }

                let total_height = if children.is_empty() { 0.0 } else { y - gap };
                layout::Node::with_children(Size::new(max_width, total_height.max(0.0)), children)
            }
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        for ((slot, state), child_layout) in self
            .slots_mut()
            .into_iter()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            slot.as_widget_mut()
                .operate(state, child_layout, renderer, operation);
        }
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
        for ((slot, state), child_layout) in self
            .slots_mut()
            .into_iter()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            slot.as_widget_mut().update(
                state,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

            if shell.is_event_captured() {
                return;
            }
        }

        if let Some(message) = self.enter_default.clone() {
            if is_unconsumed_confirm_enter(event) {
                shell.publish(message);
                shell.capture_event();
            }
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
        self.slots()
            .into_iter()
            .zip(tree.children.iter())
            .zip(layout.children())
            .map(|((slot, state), child_layout)| {
                slot.as_widget()
                    .mouse_interaction(state, child_layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((slot, state), child_layout) in self
            .slots()
            .into_iter()
            .zip(tree.children.iter())
            .zip(layout.children())
        {
            slot.as_widget().draw(
                state,
                renderer,
                theme,
                inherited_style,
                child_layout,
                cursor,
                viewport,
            );
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
        let _ = (tree, layout, renderer, viewport, translation);
        None
    }
}
