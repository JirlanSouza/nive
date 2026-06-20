use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{Operation, Tree},
        Clipboard, Layout, Shell,
    },
    widget::container,
    Background, Border, Color, Event, Point, Shadow, Size, Vector,
};

use crate::{focus_trap, Element, Renderer, Theme};

pub(super) struct DialogOverlay<'a, 'b, Message> {
    dialog: &'b mut Element<'a, Message>,
    state: &'b mut Tree,
    backdrop_alpha: f32,
    on_backdrop: Option<Message>,
    on_escape: Option<Message>,
}

impl<'a, 'b, Message> DialogOverlay<'a, 'b, Message> {
    pub(super) fn new(
        dialog: &'b mut Element<'a, Message>,
        state: &'b mut Tree,
        backdrop_alpha: f32,
        on_backdrop: Option<Message>,
        on_escape: Option<Message>,
    ) -> Self {
        Self {
            dialog,
            state,
            backdrop_alpha,
            on_backdrop,
            on_escape,
        }
    }
}

impl<'a, 'b, Message> overlay::Overlay<Message, Theme, Renderer> for DialogOverlay<'a, 'b, Message>
where
    Message: Clone + 'a,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);
        let dialog_node = self
            .dialog
            .as_widget_mut()
            .layout(self.state, renderer, &limits);
        let position = centered_position(dialog_node.size(), bounds);

        layout::Node::with_children(bounds, vec![dialog_node.move_to(position)])
    }

    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        let Some(dialog_layout) = layout.children().next() else {
            return;
        };

        self.dialog
            .as_widget_mut()
            .operate(self.state, dialog_layout, renderer, operation);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let Some(dialog_layout) = layout.children().next() else {
            shell.capture_event();
            return;
        };

        if let Some(direction) = focus_trap::direction_from_event(event) {
            direction.operate(|operation| {
                self.dialog
                    .as_widget_mut()
                    .operate(self.state, dialog_layout, renderer, operation);
            });
            shell.capture_event();
            shell.invalidate_layout();
            shell.request_redraw();
            return;
        }

        let viewport = layout.bounds();
        self.dialog.as_widget_mut().update(
            self.state,
            event,
            dialog_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        if is_pointer_press(event) && !cursor.is_over(dialog_layout.bounds()) {
            if let Some(message) = self.on_backdrop.clone() {
                shell.publish(message);
            }
            shell.capture_event();
            return;
        }

        if is_escape_key_press(event) {
            if let Some(message) = self.on_escape.clone() {
                shell.publish(message);
            }
            shell.capture_event();
            return;
        }

        shell.capture_event();
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let Some(dialog_layout) = layout.children().next() else {
            return mouse::Interaction::Idle;
        };

        if !cursor.is_over(dialog_layout.bounds()) {
            return mouse::Interaction::Idle;
        }

        let viewport = layout.bounds();
        self.dialog.as_widget().mouse_interaction(
            self.state,
            dialog_layout,
            cursor,
            &viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        container::draw_background(
            renderer,
            &backdrop_style(self.backdrop_alpha),
            layout.bounds(),
        );

        let Some(dialog_layout) = layout.children().next() else {
            return;
        };

        let viewport = layout.bounds();
        self.dialog.as_widget().draw(
            self.state,
            renderer,
            theme,
            inherited_style,
            dialog_layout,
            cursor,
            &viewport,
        );
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        let dialog_layout = layout.children().next()?;
        let viewport = layout.bounds();

        self.dialog.as_widget_mut().overlay(
            self.state,
            dialog_layout,
            renderer,
            &viewport,
            Vector::ZERO,
        )
    }
}

fn backdrop_style(alpha: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::BLACK.scale_alpha(alpha))),
        border: Border::default(),
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

fn is_escape_key_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            ..
        })
    )
}

fn is_pointer_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Touch(iced::touch::Event::FingerPressed { .. })
    )
}

fn centered_position(content: Size, bounds: Size) -> Point {
    Point::new(
        center_axis(content.width, bounds.width),
        center_axis(content.height, bounds.height),
    )
}

fn center_axis(length: f32, limit: f32) -> f32 {
    let max = (limit - length).max(0.0);
    ((limit - length) / 2.0).clamp(0.0, max)
}

#[cfg(test)]
mod dialog_host_tests {
    use super::*;

    #[test]
    fn centers_dialog_inside_available_bounds() {
        let position = centered_position(Size::new(40.0, 20.0), Size::new(100.0, 80.0));

        assert_eq!(position, Point::new(30.0, 30.0));
    }

    #[test]
    fn clamps_oversized_dialog_to_origin() {
        let position = centered_position(Size::new(120.0, 90.0), Size::new(100.0, 80.0));

        assert_eq!(position, Point::ORIGIN);
    }

    #[test]
    fn backdrop_style_applies_requested_alpha() {
        let style = backdrop_style(0.5);

        if let Some(Background::Color(color)) = style.background {
            assert!((color.a - 0.5).abs() < 0.01);
        } else {
            panic!("expected color background");
        }
    }
}
