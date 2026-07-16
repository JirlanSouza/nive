use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    touch, Event, Length, Point, Rectangle, Size, Vector,
};

use crate::Element;

pub(super) struct LabelFocus<'a, Message> {
    content: Element<'a, Message>,
    target: Option<iced::widget::Id>,
    disabled: bool,
}

impl<'a, Message> LabelFocus<'a, Message> {
    pub(super) fn new(
        content: Element<'a, Message>,
        target: Option<iced::widget::Id>,
        disabled: bool,
    ) -> Self {
        Self {
            content,
            target,
            disabled,
        }
    }
}

impl<Message> Widget<Message, crate::theme::Theme, iced::Renderer> for LabelFocus<'_, Message>
where
    Message: Clone,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
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
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
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
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let activation = activation_position(event, cursor)
            .is_some_and(|position| label_bounds(layout).contains(position));

        if activation && !self.disabled {
            if let Some(target) = self.target.clone() {
                let mut focus = operation::focusable::focus(target);
                self.content.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    &mut focus,
                );
                shell.request_redraw();
            }
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
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor
            .position()
            .is_some_and(|position| label_bounds(layout).contains(position))
        {
            mouse::Interaction::Idle
        } else {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }
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
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

fn label_bounds(layout: Layout<'_>) -> Rectangle {
    layout
        .children()
        .next()
        .and_then(|column| column.children().next())
        .map_or_else(|| layout.bounds(), |label| label.bounds())
}

fn activation_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.position(),
        Event::Touch(touch::Event::FingerPressed { position, .. }) => Some(*position),
        _ => None,
    }
}

impl<'a, Message> From<LabelFocus<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(field: LabelFocus<'a, Message>) -> Self {
        Element::new(field)
    }
}
