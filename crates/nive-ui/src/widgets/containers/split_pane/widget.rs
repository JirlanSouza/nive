use std::time::Instant;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    border::Radius,
    Event, Length, Point, Rectangle, Size, Vector,
};

use crate::advanced::pressable::draw_focus_ring;
use crate::interaction::{Orientation, StepAdjustment};
use crate::widgets::controls::button::ButtonFocusRing;

use super::helpers::{clamp_ratio, cross_length, main_length, metrics, pane_sizes};
use super::state::{SplitPaneRegion, SplitPaneState};
use super::SplitPane;

use self::draw::draw_grip;
use self::event::{
    current_grip_bounds, handle_pointer_gestures, primary_press_outside_grip, publish_ratio,
    resize_interaction,
};

mod draw;
mod event;

#[cfg(test)]
mod tests;

pub(super) const KEYBOARD_STEP: StepAdjustment = StepAdjustment::new(0.01, 0.1);

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer> for SplitPane<'a, Message>
where
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SplitPaneState>()
    }

    fn state(&self) -> tree::State {
        SplitPaneState::new_state()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.leading), Tree::new(&self.trailing)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.leading.as_widget(), self.trailing.as_widget()]);
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
        let metrics = metrics();
        let size = limits.resolve(self.width, self.height, Size::ZERO);
        let main_length = main_length(self.orientation, size);
        let cross_length = cross_length(self.orientation, size);
        let handle_length = metrics.handle_size.min(main_length);
        let available_length = (main_length - handle_length).max(0.0);
        let ratio = clamp_ratio(self.ratio, self.constraints, available_length);
        let leading_length = available_length * ratio;
        let trailing_length = (available_length - leading_length).max(0.0);
        let (leading_size, trailing_size) = pane_sizes(
            self.orientation,
            cross_length,
            leading_length,
            trailing_length,
        );
        let grip_size = match self.orientation {
            Orientation::Horizontal => Size::new(handle_length, size.height),
            Orientation::Vertical => Size::new(size.width, handle_length),
        };

        let leading = self
            .leading
            .as_widget_mut()
            .layout(
                &mut tree.children[0],
                renderer,
                &layout::Limits::new(leading_size, leading_size),
            )
            .move_to(Point::ORIGIN);

        let grip_origin = match self.orientation {
            Orientation::Horizontal => Point::new(leading_length, 0.0),
            Orientation::Vertical => Point::new(0.0, leading_length),
        };
        let grip = layout::Node::new(grip_size).move_to(grip_origin);

        let trailing_origin = match self.orientation {
            Orientation::Horizontal => Point::new(leading_length + handle_length, 0.0),
            Orientation::Vertical => Point::new(0.0, leading_length + handle_length),
        };
        let trailing = self
            .trailing
            .as_widget_mut()
            .layout(
                &mut tree.children[1],
                renderer,
                &layout::Limits::new(trailing_size, trailing_size),
            )
            .move_to(trailing_origin);

        let state = tree.state.downcast_mut::<SplitPaneState>();
        state.grip_bounds = Rectangle::new(grip_origin, grip_size);
        state.available_length = available_length;

        layout::Node::with_children(size, vec![leading, grip, trailing])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<SplitPaneState>();

        operation.focusable(self.id.as_ref(), layout.bounds(), state);

        let mut layouts = layout.children();
        let Some(leading_layout) = layouts.next() else {
            return;
        };
        let _ = layouts.next();
        let Some(trailing_layout) = layouts.next() else {
            return;
        };

        self.leading.as_widget_mut().operate(
            &mut tree.children[0],
            leading_layout,
            renderer,
            operation,
        );
        self.trailing.as_widget_mut().operate(
            &mut tree.children[1],
            trailing_layout,
            renderer,
            operation,
        );
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
        {
            let state = tree.state.downcast_mut::<SplitPaneState>();
            let grip_bounds = current_grip_bounds(layout).unwrap_or(state.grip_bounds);

            state.grip_bounds = grip_bounds;

            if primary_press_outside_grip(event, cursor, grip_bounds) {
                state.drag = None;
                if state.focused {
                    state.focused = false;
                    shell.request_redraw();
                }
            }

            if !self.locked && self.forward_keyboard(state, event, shell) {
                return;
            }

            if !self.locked {
                let gestures = state
                    .gestures
                    .handle_event(event, Instant::now(), |position| {
                        grip_bounds
                            .contains(position)
                            .then_some(SplitPaneRegion::Grip)
                    });

                if !gestures.is_empty() {
                    let ratios = handle_pointer_gestures(
                        state,
                        &gestures,
                        self.orientation,
                        self.ratio,
                        self.constraints,
                        self.snap.as_ref(),
                        false,
                    );

                    for ratio in ratios {
                        publish_ratio(self.on_change.as_deref(), ratio, shell);
                    }

                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
            }
        }

        if shell.is_event_captured() {
            return;
        }

        let mut layouts = layout.children();
        let Some(leading_layout) = layouts.next() else {
            return;
        };
        let _ = layouts.next();
        let Some(trailing_layout) = layouts.next() else {
            return;
        };

        self.leading.as_widget_mut().update(
            &mut tree.children[0],
            event,
            leading_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        self.trailing.as_widget_mut().update(
            &mut tree.children[1],
            event,
            trailing_layout,
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
        let state = tree.state.downcast_ref::<SplitPaneState>();
        let grip_bounds = current_grip_bounds(layout).unwrap_or(state.grip_bounds);

        if !self.locked && (state.drag.is_some() || cursor.is_over(grip_bounds)) {
            return resize_interaction(self.orientation);
        }

        let mut interaction = mouse::Interaction::None;
        let mut layouts = layout.children();

        if let Some(leading_layout) = layouts.next() {
            interaction = interaction.max(self.leading.as_widget().mouse_interaction(
                &tree.children[0],
                leading_layout,
                cursor,
                viewport,
                renderer,
            ));
        }

        let _ = layouts.next();

        if let Some(trailing_layout) = layouts.next() {
            interaction = interaction.max(self.trailing.as_widget().mouse_interaction(
                &tree.children[1],
                trailing_layout,
                cursor,
                viewport,
                renderer,
            ));
        }

        interaction
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
        let mut layouts = layout.children();
        let Some(leading_layout) = layouts.next() else {
            return;
        };
        let Some(grip_layout) = layouts.next() else {
            return;
        };
        let Some(trailing_layout) = layouts.next() else {
            return;
        };

        self.leading.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            leading_layout,
            cursor,
            viewport,
        );

        draw_grip(
            renderer,
            theme,
            grip_layout.bounds(),
            self.orientation,
            self.handle_role,
        );

        self.trailing.as_widget().draw(
            &tree.children[1],
            renderer,
            theme,
            inherited_style,
            trailing_layout,
            cursor,
            viewport,
        );

        let state = tree.state.downcast_ref::<SplitPaneState>();

        if state.focused {
            draw_focus_ring(
                renderer,
                theme,
                grip_layout.bounds(),
                Radius::from(2.0),
                ButtonFocusRing::Default,
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
        let mut layouts = layout.children();
        let leading_layout = layouts.next()?;
        let _ = layouts.next();
        let trailing_layout = layouts.next()?;

        let mut overlays = Vec::new();
        let (leading_tree, trailing_tree) = tree.children.split_at_mut(1);

        if let Some(overlay) = self.leading.as_widget_mut().overlay(
            &mut leading_tree[0],
            leading_layout,
            renderer,
            viewport,
            translation,
        ) {
            overlays.push(overlay);
        }

        if let Some(overlay) = self.trailing.as_widget_mut().overlay(
            &mut trailing_tree[0],
            trailing_layout,
            renderer,
            viewport,
            translation,
        ) {
            overlays.push(overlay);
        }

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}
