use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, Operation, Tree},
        Clipboard, Layout, Shell,
    },
    keyboard::{self, key::Named},
    touch, Event, Rectangle, Size, Vector,
};

use crate::{
    advanced::shell_relay,
    focus::{contains_focus_target, FocusTarget, FocusTargetContext},
    focus_trap,
    widgets::overlays::anchored_overlay::{
        content_limits, resolve_geometry, GeometryInput, PopoverCollision, PopoverFocusPolicy,
        PopoverPlacement, PopoverWidth,
    },
    Element,
};

use super::PopoverDismissalCause;
use super::{identity::bind_descendants, OverlayIdentity};

pub(crate) struct AnchoredOverlay<'a, 'b, LocalMessage, Message, OnMessage> {
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
    dismissal_cause: Option<&'b mut Option<PopoverDismissalCause>>,
    focus_context: Option<&'b FocusTargetContext>,
    expected_target: Option<&'b mut Option<FocusTarget>>,
    identity: OverlayIdentity,
}

impl<'a, 'b, LocalMessage, Message, OnMessage>
    AnchoredOverlay<'a, 'b, LocalMessage, Message, OnMessage>
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
            dismissal_cause: None,
            focus_context: None,
            expected_target: None,
            identity: OverlayIdentity::root(),
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

    pub(crate) fn identity(mut self, identity: OverlayIdentity) -> Self {
        self.identity = identity;
        self
    }

    pub(crate) fn focus_policy(
        mut self,
        focus_policy: PopoverFocusPolicy,
        focus_entered: &'b mut bool,
        dismissal_requested: &'b mut bool,
        dismissal_cause: &'b mut Option<PopoverDismissalCause>,
        focus_context: &'b FocusTargetContext,
        expected_target: &'b mut Option<FocusTarget>,
    ) -> Self {
        self.focus_policy = focus_policy;
        self.focus_entered = Some(focus_entered);
        self.dismissal_requested = Some(dismissal_requested);
        self.dismissal_cause = Some(dismissal_cause);
        self.focus_context = Some(focus_context);
        self.expected_target = Some(expected_target);
        self
    }
}

impl<'a, 'b, LocalMessage, Message, OnMessage>
    overlay::Overlay<Message, crate::theme::Theme, iced::Renderer>
    for AnchoredOverlay<'a, 'b, LocalMessage, Message, OnMessage>
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
        let node = self
            .content
            .as_widget_mut()
            .layout(self.content_state, renderer, &final_limits)
            .move_to(geometry.frame.position());
        bind_descendants(
            &self.identity,
            self.content,
            self.content_state,
            Layout::new(&node),
            renderer,
        );
        node
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

        if local_shell.event_status() == iced::event::Status::Ignored
            && is_primary_press_inside(event, layout, cursor)
        {
            local_shell.capture_event();
        }

        self.refresh_owned_target(layout, renderer);
        if !local_shell.is_empty() && is_owned_activation(event) {
            self.mark_restoration_cause(PopoverDismissalCause::RestoreAnchor);
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

        let interaction = self.content.as_widget().mouse_interaction(
            self.content_state,
            layout,
            cursor,
            &viewport,
            renderer,
        );

        if interaction == mouse::Interaction::None && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Idle
        } else {
            interaction
        }
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

    fn index(&self) -> f32 {
        1.0 + self.identity.depth() as f32
    }
}

impl<LocalMessage: Clone, Message, OnMessage>
    AnchoredOverlay<'_, '_, LocalMessage, Message, OnMessage>
{
    fn enter_focus_once(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        shell: &mut Shell<'_, LocalMessage>,
    ) {
        if self.focus_policy == PopoverFocusPolicy::RetainAnchor {
            return;
        }
        let Some(focus_entered) = self.focus_entered.as_deref_mut() else {
            return;
        };
        if *focus_entered {
            return;
        }

        focus_trap::FocusDirection::Next.operate(|operation| {
            self.content
                .as_widget_mut()
                .operate(self.content_state, layout, renderer, operation);
        });
        *focus_entered = true;
        self.refresh_owned_target(layout, renderer);
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
        self.refresh_owned_target(layout, renderer);
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

        if exits && !self.dismissal_already_requested() {
            if let Some(message) = self.on_dismiss.clone() {
                self.mark_dismissal_requested(PopoverDismissalCause::TraversalExit);
                shell.publish(message);
                shell.request_redraw();
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
        if let Some(message) = self.on_dismiss.clone() {
            if !self.dismissal_already_requested() {
                self.mark_dismissal_requested(PopoverDismissalCause::RestoreAnchor);
                shell.publish(message);
            }
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
                cursor.position().is_some_and(|position| {
                    !layout.bounds().contains(position) && !self.anchor_bounds.contains(position)
                })
            }
            Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                !layout.bounds().contains(*position) && !self.anchor_bounds.contains(*position)
            }
            _ => false,
        };

        if pressed_outside {
            if let Some(message) = self.on_dismiss.clone() {
                if !self.dismissal_already_requested() {
                    self.mark_dismissal_requested(PopoverDismissalCause::RestoreAnchor);
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

    fn mark_dismissal_requested(&mut self, cause: PopoverDismissalCause) {
        if let Some(dismissal_requested) = self.dismissal_requested.as_deref_mut() {
            *dismissal_requested = true;
        }
        self.mark_restoration_cause(cause);
    }

    fn mark_restoration_cause(&mut self, cause: PopoverDismissalCause) {
        if let Some(dismissal_cause) = self.dismissal_cause.as_deref_mut() {
            *dismissal_cause = Some(cause);
        }
    }

    fn refresh_owned_target(&mut self, layout: Layout<'_>, renderer: &iced::Renderer) {
        let Some(context) = self.focus_context else {
            return;
        };
        let Some(current) = context.capture() else {
            return;
        };
        let mut contains = contains_focus_target(current.clone());
        self.content.as_widget_mut().operate(
            self.content_state,
            layout,
            renderer,
            &mut operation::black_box(&mut contains),
        );
        if matches!(contains.finish(), operation::Outcome::Some(true)) {
            if let Some(expected_target) = self.expected_target.as_deref_mut() {
                *expected_target = Some(current);
            }
        }
    }
}

fn is_owned_activation(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Enter | Named::Space),
                repeat: false,
                ..
            })
    )
}

fn is_primary_press_inside(event: &Event, layout: Layout<'_>, cursor: mouse::Cursor) -> bool {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
            cursor.is_over(layout.bounds())
        }
        Event::Touch(touch::Event::FingerPressed { position, .. }) => {
            layout.bounds().contains(*position)
        }
        _ => false,
    }
}
