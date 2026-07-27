use std::{cell::Cell, rc::Rc};

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    mouse::Button,
    touch, Event, Length, Rectangle, Size, Vector,
};

use crate::Element;

/// Pointer state for one tree row, published by [`PointerProbe`] and read by the
/// row's style.
///
/// A tree row is wider than the button inside it — indentation and the branch
/// expander are siblings, not children — so the button cannot paint a fill that
/// spans the row. The row container can, but a style closure never sees the
/// cursor. This carries what it needs across.
#[derive(Debug, Clone, Default)]
pub(super) struct RowPointer {
    hovered: Rc<Cell<bool>>,
    pressed: Rc<Cell<bool>>,
}

impl RowPointer {
    pub(super) fn hovered(&self) -> bool {
        self.hovered.get()
    }

    pub(super) fn pressed(&self) -> bool {
        self.pressed.get()
    }
}

/// Wraps a row and records whether the pointer is over it, and whether it is
/// being pressed.
///
/// Observes only: every event is forwarded to the row untouched, so the button
/// and the expander inside keep receiving exactly what they did before.
pub(super) struct PointerProbe<'a, Message> {
    row: Element<'a, Message>,
    pointer: RowPointer,
}

impl<'a, Message> PointerProbe<'a, Message> {
    pub(super) fn new(row: impl Into<Element<'a, Message>>, pointer: RowPointer) -> Self {
        Self {
            row: row.into(),
            pointer,
        }
    }
}

impl<Message> Widget<Message, crate::theme::Theme, iced::Renderer> for PointerProbe<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.row)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.row]);
    }

    fn size(&self) -> Size<Length> {
        self.row.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.row.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.row
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
        self.row
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
        let hovered = cursor.is_over(layout.bounds());
        self.pointer.hovered.set(hovered);
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                self.pointer.pressed.set(hovered);
            }
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. }) => {
                self.pointer.pressed.set(false);
            }
            // A pointer that leaves mid-press is no longer pressing this row.
            _ if !hovered => self.pointer.pressed.set(false),
            _ => {}
        }

        self.row.as_widget_mut().update(
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

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        // Published here as well as in `update`, because a row can be drawn
        // after the cursor moved without an event reaching this subtree.
        self.pointer.hovered.set(cursor.is_over(layout.bounds()));

        self.row.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
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
        self.row.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        self.row.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<PointerProbe<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(probe: PointerProbe<'a, Message>) -> Self {
        Element::new(probe)
    }
}
