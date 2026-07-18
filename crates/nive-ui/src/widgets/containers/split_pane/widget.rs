use std::time::Instant;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Point, Rectangle, Size, Vector,
};

use crate::interaction::{Orientation, StepAdjustment};

use super::helpers::{clamp_ratio, cross_length, main_length, metrics, pane_sizes};
use super::state::{SplitPaneRegion, SplitPaneState};
use super::SplitPane;

use self::draw::{draw_grip, resolve_visual_state};
use self::event::{
    current_divider_bounds, current_hit_bounds, handle_pointer_gestures, has_primary_gesture,
    primary_press_outside_hit, publish_ratio, resize_interaction,
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
        let metrics = metrics(self.size);
        let size = limits.resolve(self.width, self.height, Size::ZERO);
        let main_length = main_length(self.orientation, size);
        let cross_length = cross_length(self.orientation, size);
        let divider_length = metrics.layout_thickness.min(main_length);
        let available_length = (main_length - divider_length).max(0.0);
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
            Orientation::Horizontal => Size::new(divider_length, size.height),
            Orientation::Vertical => Size::new(size.width, divider_length),
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
            Orientation::Horizontal => Point::new(leading_length + divider_length, 0.0),
            Orientation::Vertical => Point::new(0.0, leading_length + divider_length),
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

        if self.interactive() {
            if let Some(hit_bounds) =
                current_hit_bounds(layout, self.orientation, metrics(self.size))
            {
                state
                    .focus
                    .register(operation, self.id.as_ref(), hit_bounds);
            }
        } else {
            state.focus.clear();
            state.drag = None;
        }

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
        let hit_bounds = current_hit_bounds(layout, self.orientation, metrics(self.size));

        {
            let state = tree.state.downcast_mut::<SplitPaneState>();

            if matches!(event, Event::Window(iced::window::Event::Unfocused)) {
                state.focus.deactivate();
            }

            if let Some(hit_bounds) = hit_bounds {
                if primary_press_outside_hit(event, cursor, hit_bounds) {
                    state.drag = None;
                    if state.focus.is_active() {
                        state.focus.deactivate();
                        shell.request_redraw();
                    }
                }
            }

            if !self.interactive() && (state.focus.is_active() || state.drag.is_some()) {
                state.focus.clear();
                state.drag = None;
                shell.request_redraw();
            }

            if self.interactive() && self.forward_keyboard(state, event, shell) {
                return;
            }

            if self.interactive() {
                if let Some(hit_bounds) = hit_bounds {
                    let gestures = state
                        .gestures
                        .handle_event(event, Instant::now(), |position| {
                            hit_bounds
                                .contains(position)
                                .then_some(SplitPaneRegion::Grip)
                        });

                    if has_primary_gesture(&gestures) {
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
        let hit_bounds = current_hit_bounds(layout, self.orientation, metrics(self.size));

        if self.interactive()
            && (state.drag.is_some()
                || hit_bounds.is_some_and(|hit_bounds| cursor.is_over(hit_bounds)))
        {
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
        let divider_bounds = current_divider_bounds(layout);
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

        let state = tree.state.downcast_ref::<SplitPaneState>();

        let hit_bounds = current_hit_bounds(layout, self.orientation, metrics(self.size));
        let visual_state = resolve_visual_state(
            self.interactive(),
            state.drag.is_some() || state.focus.is_focus_visible(),
            hit_bounds.is_some_and(|bounds| cursor.is_over(bounds)),
        );

        draw_grip(
            renderer,
            theme,
            divider_bounds.unwrap_or(grip_layout.bounds()),
            self.orientation,
            metrics(self.size),
            visual_state,
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

impl<Message> SplitPane<'_, Message> {
    fn interactive(&self) -> bool {
        !self.locked && self.on_change.is_some()
    }
}
