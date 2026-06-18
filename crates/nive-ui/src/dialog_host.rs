use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{Operation, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    widget::container,
    Background, Border, Color, Event, Length, Point, Rectangle, Shadow, Size, Vector,
};

use crate::{focus_trap, Element, Renderer, Theme};

const DEFAULT_BACKDROP_ALPHA: f32 = 0.62;

pub struct DialogHost<'a, Message> {
    content: Element<'a, Message>,
    dialog: Option<DialogContent<'a, Message>>,
    backdrop_alpha: f32,
}

struct DialogContent<'a, Message> {
    content: Element<'a, Message>,
    on_backdrop: Option<Message>,
    on_escape: Option<Message>,
}

impl<'a, Message> DialogHost<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            dialog: None,
            backdrop_alpha: DEFAULT_BACKDROP_ALPHA,
        }
    }

    pub fn dialog(
        mut self,
        content: impl Into<Element<'a, Message>>,
        on_backdrop: Option<Message>,
        on_escape: Option<Message>,
    ) -> Self {
        self.dialog = Some(DialogContent {
            content: content.into(),
            on_backdrop,
            on_escape,
        });
        self
    }

    pub fn backdrop_alpha(mut self, alpha: f32) -> Self {
        self.backdrop_alpha = alpha;
        self
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for DialogHost<'a, Message>
where
    Message: Clone + 'a,
{
    fn children(&self) -> Vec<Tree> {
        match &self.dialog {
            Some(dialog) => vec![Tree::new(&self.content), Tree::new(&dialog.content)],
            None => vec![Tree::new(&self.content)],
        }
    }

    fn diff(&self, tree: &mut Tree) {
        match &self.dialog {
            Some(dialog) => {
                tree.diff_children(&[self.content.as_widget(), dialog.content.as_widget()])
            }
            None => tree.diff_children(&[self.content.as_widget()]),
        }
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        if self.dialog.is_some() {
            shell.capture_event();
            return;
        }

        self.content.as_widget_mut().update(
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

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.dialog.is_some() {
            return mouse::Interaction::Idle;
        }

        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if let Some(dialog) = &mut self.dialog {
            return Some(overlay::Element::new(Box::new(DialogOverlay::new(
                &mut dialog.content,
                &mut tree.children[1],
                self.backdrop_alpha,
                dialog.on_backdrop.clone(),
                dialog.on_escape.clone(),
            ))));
        }

        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<DialogHost<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(host: DialogHost<'a, Message>) -> Self {
        Element::new(host)
    }
}

struct DialogOverlay<'a, 'b, Message> {
    dialog: &'b mut Element<'a, Message>,
    state: &'b mut Tree,
    backdrop_alpha: f32,
    on_backdrop: Option<Message>,
    on_escape: Option<Message>,
}

impl<'a, 'b, Message> DialogOverlay<'a, 'b, Message> {
    fn new(
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
