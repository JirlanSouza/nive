use std::time::Duration;

use iced::{
    advanced::{mouse, widget::Tree, Clipboard, Layout, Shell},
    keyboard, window, Event, Rectangle,
};

use crate::widgets::navigation::overflow::{wheel_delta, OverflowAxis};
use crate::widgets::navigation::tabs::geometry::{
    autoscroll_step, event_position, hit_geometry, owns_wheel_event,
};
use crate::widgets::navigation::tabs::style as theme_tabs;
use crate::widgets::navigation::tabs::{FocusMovement, TabBar, TabBarState};

impl<'a, Id, Message> TabBar<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    #[allow(clippy::too_many_arguments)]
    pub(super) fn update_impl(
        &self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        {
            let state = tree.state.downcast_ref::<TabBarState<Id>>();
            let mut content = self.content_element(state);
            content.as_widget_mut().update(
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

        if shell.is_event_captured() && !matches!(event, Event::Mouse(_) | Event::Touch(_)) {
            return;
        }

        let displayed = self.displayed_tabs();
        let metrics = theme_tabs::metrics(self.size);
        let hit_geometry = hit_geometry(
            layout,
            &displayed,
            self.on_close_request.is_some(),
            metrics.close_side,
        );
        let state = tree.state.downcast_mut::<TabBarState<Id>>();
        state.tab_bounds = hit_geometry.tab_bounds;
        state.close_bounds = hit_geometry.close_bounds;
        state.left_chevron = hit_geometry.left_chevron;
        state.right_chevron = hit_geometry.right_chevron;
        state.all_tabs_button = hit_geometry.all_tabs_button;
        state.strip_bounds = hit_geometry.strip_bounds;
        state.hovered_id = cursor.position().and_then(|position| {
            state
                .tab_bounds
                .iter()
                .find(|(_, bounds, _)| bounds.contains(position))
                .map(|(id, _, _)| id.clone())
        });
        self.reconcile_focus(state);

        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(iced::touch::Event::FingerPressed { .. })
        ) && event_position(event, cursor).is_some_and(|position| bounds.contains(position))
        {
            state.focus.focus_from_pointer();
        }

        if self.handle_window_event(state, event, shell) {
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.pressed_id = state.hovered_id.clone();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Mouse(mouse::Event::CursorLeft) => state.pressed_id = None,
            _ => {}
        }

        if self.handle_keyboard(state, event, shell) {
            return;
        }

        if self.handle_wheel(state, event, bounds, cursor, shell) {
            return;
        }

        self.handle_gestures(state, event, bounds, shell);
    }

    /// Returns `true` when the event was fully handled and `update_impl`
    /// should stop.
    fn handle_window_event(
        &self,
        state: &mut TabBarState<Id>,
        event: &Event,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            if state.dragged_id.is_some() {
                if let Some(direction) = state.edge_scroll {
                    let elapsed = state
                        .last_redraw
                        .map_or(Duration::ZERO, |last| now.saturating_duration_since(last));
                    state.last_redraw = Some(*now);
                    state.overflow.offset = state.scroll_offset;
                    let step = autoscroll_step(direction, elapsed);
                    state.overflow.offset =
                        (state.overflow.offset + step).clamp(0.0, state.max_scroll);
                    state.scroll_offset = state.overflow.offset;
                    shell.invalidate_layout();
                    shell.request_redraw();
                    return true;
                }
            }
        }

        if matches!(event, Event::Window(window::Event::Unfocused)) {
            state.focus.deactivate();
        }

        if matches!(event, Event::Window(window::Event::Unfocused)) && state.dragged_id.is_some() {
            state.dragged_id = None;
            state.insertion_target = None;
            state.invalid_target = false;
            state.edge_scroll = None;
            state.last_redraw = None;
            state.drag_session.cancel();
            shell.request_redraw();
            return true;
        }

        false
    }

    /// Returns `true` when the event was fully handled and `update_impl`
    /// should stop.
    fn handle_keyboard(
        &self,
        state: &mut TabBarState<Id>,
        event: &Event,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        if state.focus.is_active() {
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named),
                repeat: false,
                ..
            }) = event
            {
                let movement = match named {
                    keyboard::key::Named::ArrowLeft => Some(FocusMovement::Previous),
                    keyboard::key::Named::ArrowRight => Some(FocusMovement::Next),
                    keyboard::key::Named::Home => Some(FocusMovement::First),
                    keyboard::key::Named::End => Some(FocusMovement::Last),
                    _ => None,
                };

                if let Some(movement) = movement {
                    state.focus.focus_from_keyboard();
                    self.move_focus(state, movement);
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                    return true;
                }

                if matches!(
                    named,
                    keyboard::key::Named::Enter | keyboard::key::Named::Space
                ) {
                    state.focus.focus_from_keyboard();
                    if let (Some(on_select), Some(focused)) = (&self.on_select, &state.focused_id) {
                        shell.publish(on_select(focused.clone()));
                        shell.capture_event();
                        shell.request_redraw();
                    }
                    return true;
                }
            }
        }

        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            ..
        }) = event
        {
            if state.dragged_id.is_some() {
                state.dragged_id = None;
                state.insertion_target = None;
                state.pressed_id = None;
                state.invalid_target = false;
                state.edge_scroll = None;
                state.last_redraw = None;
                state.drag_session.cancel();
                shell.request_redraw();
                shell.capture_event();
                return true;
            }
        }

        false
    }

    /// Returns `true` when the event was fully handled and `update_impl`
    /// should stop.
    fn handle_wheel(
        &self,
        state: &mut TabBarState<Id>,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        if let Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) = event {
            if !owns_wheel_event(state.has_overflow, bounds, cursor) {
                return true;
            }
            let delta_x = wheel_delta(OverflowAxis::Horizontal, *delta);
            state.overflow.offset = state.scroll_offset;
            state.overflow.scroll_by(delta_x);
            state.scroll_offset = state.overflow.offset;
            if delta_x != 0.0 {
                shell.invalidate_layout();
                shell.request_redraw();
                shell.capture_event();
            }
            return true;
        }

        false
    }
}
