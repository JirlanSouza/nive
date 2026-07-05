use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{Operation, Tree},
        Clipboard, Layout, Shell,
    },
    keyboard, touch, Event, Rectangle, Size, Vector,
};

use crate::{
    advanced::shell_relay,
    focus_trap,
    widgets::overlays::popover::placement::{
        content_limits, resolve_position, PopoverCollision, PopoverPlacement, PopoverWidth,
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
    trap_focus: bool,
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
            trap_focus: false,
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
        self.trap_focus = trap_focus;
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
        let content_node =
            self.content
                .as_widget_mut()
                .layout(self.content_state, renderer, &limits);
        let position = resolve_position(
            self.anchor_bounds,
            content_node.size(),
            bounds,
            self.placement,
            self.collision,
            self.gap,
        );

        content_node.move_to(position)
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

        if !self.handle_focus_trap(event, layout, renderer, &mut local_shell)
            && !self.handle_key_press(event, &mut local_shell)
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
    fn handle_focus_trap(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        shell: &mut Shell<'_, LocalMessage>,
    ) -> bool {
        if !self.trap_focus {
            return false;
        }

        let Some(direction) = focus_trap::direction_from_event(event) else {
            return false;
        };

        direction.operate(|operation| {
            self.content
                .as_widget_mut()
                .operate(self.content_state, layout, renderer, operation);
        });
        shell.capture_event();
        shell.invalidate_layout();
        shell.request_redraw();

        true
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
                shell.publish(message);
                shell.capture_event();
                shell.invalidate_layout();
                shell.request_redraw();
                return true;
            }
        }

        false
    }
}
