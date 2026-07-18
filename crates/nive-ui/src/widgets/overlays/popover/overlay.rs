use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, Operation, Tree},
        Clipboard, Layout, Shell,
    },
    keyboard::{self, key::Named},
    touch, Event, Rectangle, Size, Vector,
};

use super::PopoverFocusPolicy;
use crate::{
    advanced::shell_relay,
    focus_trap,
    widgets::overlays::{
        anchored_overlay::{resolve_geometry, GeometryInput},
        popover::placement::{content_limits, PopoverCollision, PopoverPlacement, PopoverWidth},
    },
    Element,
};

pub struct PopoverOverlay<'a, 'b, LocalMessage, Message, OnMessage> {
    anchor_bounds: Rectangle,
    content: &'b mut Element<'a, LocalMessage>,
    content_state: &'b mut Tree,
    placement: PopoverPlacement,
    width: PopoverWidth,
    collision: PopoverCollision,
    gap: f32,
    on_dismiss: Option<LocalMessage>,
    on_key_press: Option<fn(&keyboard::Event) -> Option<LocalMessage>>,
    on_message: OnMessage,
    map_overlay: Option<fn(LocalMessage) -> Message>,
    focus_policy: PopoverFocusPolicy,
    focus_entered: Option<&'b mut bool>,
    dismissal_requested: Option<&'b mut bool>,
}

impl<'a, 'b, LocalMessage, Message, OnMessage>
    PopoverOverlay<'a, 'b, LocalMessage, Message, OnMessage>
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        anchor_bounds: Rectangle,
        content: &'b mut Element<'a, LocalMessage>,
        content_state: &'b mut Tree,
        placement: PopoverPlacement,
        width: PopoverWidth,
        collision: PopoverCollision,
        gap: f32,
        on_dismiss: Option<LocalMessage>,
        on_message: OnMessage,
    ) -> Self {
        Self {
            anchor_bounds,
            content,
            content_state,
            placement,
            width,
            collision,
            gap,
            on_dismiss,
            on_key_press: None,
            on_message,
            map_overlay: None,
            focus_policy: PopoverFocusPolicy::RetainAnchor,
            focus_entered: None,
            dismissal_requested: None,
        }
    }

    pub fn on_key_press(mut self, map: fn(&keyboard::Event) -> Option<LocalMessage>) -> Self {
        self.on_key_press = Some(map);
        self
    }

    pub fn with_nested_overlay_map(mut self, map_overlay: fn(LocalMessage) -> Message) -> Self {
        self.map_overlay = Some(map_overlay);
        self
    }

    pub fn trap_focus(mut self, trap_focus: bool) -> Self {
        self.focus_policy = if trap_focus {
            PopoverFocusPolicy::Trap
        } else {
            PopoverFocusPolicy::RetainAnchor
        };
        self
    }

    pub fn focus_policy(
        mut self,
        focus_policy: PopoverFocusPolicy,
        focus_entered: &'b mut bool,
        dismissal_requested: &'b mut bool,
    ) -> Self {
        self.focus_policy = focus_policy;
        self.focus_entered = Some(focus_entered);
        self.dismissal_requested = Some(dismissal_requested);
        self
    }
}

impl<'a, 'b, LocalMessage, Message, OnMessage>
    overlay::Overlay<Message, crate::theme::Theme, iced::Renderer>
    for PopoverOverlay<'a, 'b, LocalMessage, Message, OnMessage>
where
    LocalMessage: Clone + 'a,
    Message: 'a,
    OnMessage: for<'shell> FnMut(LocalMessage, &mut Shell<'shell, Message>),
{
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let limits = content_limits(self.width, self.anchor_bounds, bounds);
        let intrinsic_node =
            self.content
                .as_widget_mut()
                .layout(self.content_state, renderer, &limits);
        let geometry = resolve_geometry(GeometryInput {
            anchor: self.anchor_bounds,
            viewport: Rectangle::with_size(bounds),
            intrinsic_content: intrinsic_node.size(),
            placement: self.placement,
            collision: self.collision,
            width: self.width,
            gap: self.gap,
        });
        let final_limits = layout::Limits::new(Size::ZERO, geometry.frame.size())
            .width(geometry.frame.width)
            .max_height(geometry.frame.height);
        self.content
            .as_widget_mut()
            .layout(self.content_state, renderer, &final_limits)
            .move_to(geometry.frame.position())
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        self.enter_focus_once(layout, renderer, &mut local_shell);

        if !self.handle_focus_trap(event, layout, renderer, &mut local_shell)
            && !self.handle_focus_first_exit(event, layout, renderer, &mut local_shell)
            && !self.handle_key_press(event, &mut local_shell)
            && !self.handle_escape_dismiss(event, &mut local_shell)
            && !self.handle_outside_press(event, layout, cursor, &mut local_shell)
        {
            let viewport = layout.bounds();

            self.content.as_widget_mut().update(
                self.content_state,
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                &mut local_shell,
                &viewport,
            );
        }

        shell_relay::propagate_to_parent(&mut local_shell, shell);
        drop(local_shell);

        for message in local_messages {
            (self.on_message)(message, shell);
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let viewport = layout.bounds();

        self.content.as_widget().mouse_interaction(
            self.content_state,
            layout,
            cursor,
            &viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(self.content_state, layout, renderer, operation);
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let viewport = layout.bounds();

        self.content.as_widget().draw(
            self.content_state,
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            &viewport,
        );
    }

    fn overlay<'overlay>(
        &'overlay mut self,
        layout: Layout<'overlay>,
        renderer: &iced::Renderer,
    ) -> Option<overlay::Element<'overlay, Message, crate::theme::Theme, iced::Renderer>> {
        let map_overlay = self.map_overlay.as_ref()?;
        let viewport = layout.bounds();

        self.content
            .as_widget_mut()
            .overlay(
                self.content_state,
                layout,
                renderer,
                &viewport,
                Vector::ZERO,
            )
            .map(|overlay| overlay.map(map_overlay))
    }
}

impl<LocalMessage: Clone, Message, OnMessage>
    PopoverOverlay<'_, '_, LocalMessage, Message, OnMessage>
{
    fn enter_focus_once(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        shell: &mut Shell<'_, LocalMessage>,
    ) {
        if self.focus_policy == PopoverFocusPolicy::RetainAnchor
            || self.focus_entered.as_deref() == Some(&true)
        {
            return;
        }

        focus_trap::FocusDirection::Next.operate(|operation| {
            self.content
                .as_widget_mut()
                .operate(self.content_state, layout, renderer, operation);
        });
        if let Some(focus_entered) = self.focus_entered.as_deref_mut() {
            *focus_entered = true;
        }
        shell.request_redraw();
    }

    fn handle_focus_trap(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        shell: &mut Shell<'_, LocalMessage>,
    ) -> bool {
        if self.focus_policy != PopoverFocusPolicy::Trap {
            return false;
        }

        let Some(direction) = focus_trap::direction_from_event(event) else {
            return false;
        };

        let mut count = operation::focusable::count();
        self.content.as_widget_mut().operate(
            self.content_state,
            layout,
            renderer,
            &mut operation::black_box(&mut count),
        );
        let wraps = matches!(
            count.finish(),
            operation::Outcome::Some(operation::focusable::Count {
                focused: Some(focused),
                total,
            }) if total > 0 && match direction {
                focus_trap::FocusDirection::Next => focused + 1 == total,
                focus_trap::FocusDirection::Previous => focused == 0,
            }
        );

        direction.operate(|operation| {
            self.content
                .as_widget_mut()
                .operate(self.content_state, layout, renderer, operation);
        });
        if wraps {
            direction.operate(|operation| {
                self.content.as_widget_mut().operate(
                    self.content_state,
                    layout,
                    renderer,
                    operation,
                );
            });
        }
        shell.capture_event();
        shell.invalidate_layout();
        shell.request_redraw();

        true
    }

    fn handle_focus_first_exit(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        shell: &mut Shell<'_, LocalMessage>,
    ) -> bool {
        if self.focus_policy != PopoverFocusPolicy::FocusFirst {
            return false;
        }

        let Some(direction) = focus_trap::direction_from_event(event) else {
            return false;
        };
        let mut count = operation::focusable::count();
        self.content.as_widget_mut().operate(
            self.content_state,
            layout,
            renderer,
            &mut operation::black_box(&mut count),
        );
        let operation::Outcome::Some(count) = count.finish() else {
            return false;
        };
        let exits = match direction {
            focus_trap::FocusDirection::Next => count
                .focused
                .is_some_and(|focused| focused + 1 == count.total),
            focus_trap::FocusDirection::Previous => count.focused == Some(0),
        };

        if exits {
            if !self.dismissal_already_requested() {
                if let Some(message) = self.on_dismiss.clone() {
                    self.mark_dismissal_requested();
                    shell.publish(message);
                    shell.request_redraw();
                }
            }
        }

        false
    }

    fn handle_key_press(&mut self, event: &Event, shell: &mut Shell<'_, LocalMessage>) -> bool {
        let Event::Keyboard(event) = event else {
            return false;
        };

        let Some(map) = self.on_key_press else {
            return false;
        };

        let Some(message) = map(event) else {
            return false;
        };

        shell.publish(message);
        shell.capture_event();
        shell.invalidate_layout();
        shell.request_redraw();

        true
    }

    fn handle_escape_dismiss(
        &mut self,
        event: &Event,
        shell: &mut Shell<'_, LocalMessage>,
    ) -> bool {
        let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event else {
            return false;
        };
        if !matches!(key, keyboard::Key::Named(Named::Escape)) {
            return false;
        }
        let Some(message) = self.on_dismiss.clone() else {
            return false;
        };

        if !self.dismissal_already_requested() {
            self.mark_dismissal_requested();
            shell.publish(message);
        }
        shell.capture_event();
        shell.invalidate_layout();
        shell.request_redraw();
        true
    }

    fn handle_outside_press(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, LocalMessage>,
    ) -> bool {
        let pressed_outside = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                !cursor.is_over(layout.bounds())
            }
            Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                !layout.bounds().contains(*position)
            }
            _ => false,
        };

        if pressed_outside {
            if let Some(message) = self.on_dismiss.clone() {
                if !self.dismissal_already_requested() {
                    self.mark_dismissal_requested();
                    shell.publish(message);
                }
                shell.capture_event();
                shell.invalidate_layout();
                shell.request_redraw();
                return true;
            }
        }

        false
    }

    fn dismissal_already_requested(&self) -> bool {
        self.dismissal_requested.as_deref() == Some(&true)
    }

    fn mark_dismissal_requested(&mut self) {
        if let Some(dismissal_requested) = self.dismissal_requested.as_deref_mut() {
            *dismissal_requested = true;
        }
    }
}
