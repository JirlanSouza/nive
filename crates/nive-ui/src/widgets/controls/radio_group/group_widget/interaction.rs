use iced::{
    advanced::{layout, mouse, widget::Tree, Clipboard, Layout, Shell},
    keyboard::{self, key},
    touch, Event, Length, Point, Rectangle, Size,
};

use super::event_position;
use crate::theme::choice::ChoiceMetrics;
use crate::widgets::controls::radio_group::{RadioGroupLayout, RadioGroupState, RadioGroupWidget};

impl<T, Message> RadioGroupWidget<'_, T, Message>
where
    T: Clone + Eq,
    Message: Clone,
{
    pub(super) fn layout_impl(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width);
        let state = tree.state.downcast_ref::<RadioGroupState>();
        let metrics = ChoiceMetrics::for_theme(crate::theme::active(), self.size);
        let maximum = limits.max().width;
        let finite = maximum.is_finite();
        let mut nodes = Vec::with_capacity(self.options.len());

        match self.layout {
            RadioGroupLayout::Vertical => {
                let mut y = 0.0;
                let mut intrinsic_width: f32 = 0.0;
                for index in 0..self.options.len() {
                    let mut option = self.option_element(index, state, self.option_width());
                    let node = option.as_widget_mut().layout(
                        &mut tree.children[index],
                        renderer,
                        &limits.width(self.width),
                    );
                    intrinsic_width = intrinsic_width.max(node.size().width);
                    let height = node.size().height;
                    nodes.push(node.move_to(Point::new(0.0, y)));
                    y += height + metrics.option_gap;
                }
                let height = (y - metrics.option_gap).max(0.0);
                let size = limits.resolve(
                    self.width,
                    Length::Shrink,
                    Size::new(intrinsic_width, height),
                );
                layout::Node::with_children(size, nodes)
            }
            RadioGroupLayout::HorizontalWrap => {
                let mut x = 0.0;
                let mut y = 0.0;
                let mut row_height: f32 = 0.0;
                let mut intrinsic_width: f32 = 0.0;
                for index in 0..self.options.len() {
                    let mut option = self.option_element(index, state, Length::Shrink);
                    let mut node = option.as_widget_mut().layout(
                        &mut tree.children[index],
                        renderer,
                        &layout::Limits::NONE,
                    );
                    if finite && node.size().width > maximum {
                        let mut option = self.option_element(index, state, Length::Fill);
                        node = option.as_widget_mut().layout(
                            &mut tree.children[index],
                            renderer,
                            &layout::Limits::new(Size::ZERO, Size::new(maximum, f32::INFINITY)),
                        );
                    }
                    if finite && x > 0.0 && x + node.size().width > maximum {
                        intrinsic_width = intrinsic_width.max((x - metrics.option_gap).max(0.0));
                        y += row_height + metrics.option_gap;
                        x = 0.0;
                        row_height = 0.0;
                    }
                    row_height = row_height.max(node.size().height);
                    let width = node.size().width;
                    nodes.push(node.move_to(Point::new(x, y)));
                    x += width + metrics.option_gap;
                }
                intrinsic_width = intrinsic_width.max((x - metrics.option_gap).max(0.0));
                let intrinsic = Size::new(intrinsic_width, y + row_height);
                let size = limits.resolve(self.width, Length::Shrink, intrinsic);
                layout::Node::with_children(size, nodes)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn update_impl(
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
        {
            let Tree {
                state, children, ..
            } = tree;
            let state = state.downcast_ref::<RadioGroupState>();
            for (index, (child_tree, child_layout)) in
                children.iter_mut().zip(layout.children()).enumerate()
            {
                self.option_element(index, state, self.option_width())
                    .as_widget_mut()
                    .update(
                        child_tree,
                        event,
                        child_layout,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
            }
        }

        let interactive = self.interactive();
        let hit = event_position(event, cursor).and_then(|position| {
            layout
                .children()
                .enumerate()
                .find_map(|(index, child)| child.bounds().contains(position).then_some(index))
        });
        let state = tree.state.downcast_mut::<RadioGroupState>();
        if !interactive {
            state.focus.clear();
            state.focused_index = None;
            return;
        }

        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. })
        ) {
            if let Some(index) = hit.filter(|index| !self.options[*index].disabled) {
                state.focus.focus_from_pointer();
                state.focused_index = Some(index);
                shell.request_redraw();
            } else {
                state.focus.deactivate();
            }
        }

        if !state.focus.is_active() {
            return;
        }

        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(named),
            repeat: false,
            ..
        }) = event
        {
            let focus_key = matches!(
                named,
                key::Named::ArrowUp
                    | key::Named::ArrowLeft
                    | key::Named::ArrowDown
                    | key::Named::ArrowRight
                    | key::Named::Home
                    | key::Named::End
                    | key::Named::Space
            );
            if focus_key {
                state.focus.focus_from_keyboard();
            }
            let target = match named {
                key::Named::ArrowUp | key::Named::ArrowLeft => self.move_focus(state, -1),
                key::Named::ArrowDown | key::Named::ArrowRight => self.move_focus(state, 1),
                key::Named::Home => self.options.iter().position(|option| !option.disabled),
                key::Named::End => self.options.iter().rposition(|option| !option.disabled),
                key::Named::Space => self.focus_target(state),
                _ => None,
            };
            if let Some(index) = target {
                state.focused_index = Some(index);
                self.publish_if_changed(index, shell);
                shell.capture_event();
                shell.request_redraw();
            } else if focus_key {
                shell.request_redraw();
            }
        }

        if matches!(event, Event::Window(iced::window::Event::Unfocused)) {
            state.focus.deactivate();
            state.focused_index = None;
            shell.request_redraw();
        }
    }
}
