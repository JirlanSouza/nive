use std::{cell::Cell, rc::Rc};

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use super::{HighlightVisibility, HighlightVisibilityState};
use crate::{
    widgets::{
        navigation::menu::{MENU_LIST_INSET, MENU_ROW_HEIGHT},
        overlays::anchored_overlay::scroll::EnsureVisibleHandle,
    },
    Element,
};

impl<'a, T, Message> HighlightVisibility<'a, T, Message>
where
    T: Clone,
{
    pub(in crate::widgets::controls::autocomplete) fn new(
        content: Element<'a, Message>,
        highlighted_index: Rc<Cell<Option<usize>>>,
        ensure_pending: Rc<Cell<bool>>,
        ensure_visible: EnsureVisibleHandle,
        suggestions: Vec<(T, bool)>,
        local_closed: Rc<Cell<bool>>,
        on_select: Option<Rc<dyn Fn(T) -> Message + 'a>>,
    ) -> Self {
        Self {
            content,
            highlighted_index,
            ensure_pending,
            ensure_visible,
            suggestions,
            local_closed,
            on_select,
        }
    }

    pub(in crate::widgets::controls::autocomplete) fn request_highlight_visible(
        &self,
        layout: Layout<'_>,
    ) {
        if !self.ensure_pending.replace(false) {
            return;
        }
        let Some(index) = self
            .highlighted_index
            .get()
            .filter(|index| *index < self.suggestions.len())
        else {
            return;
        };
        let bounds = layout.bounds();
        self.ensure_visible.request(Rectangle {
            x: bounds.x + MENU_LIST_INSET,
            y: bounds.y + MENU_LIST_INSET + index as f32 * MENU_ROW_HEIGHT,
            width: (bounds.width - MENU_LIST_INSET * 2.0).max(0.0),
            height: MENU_ROW_HEIGHT,
        });
    }

    pub(in crate::widgets::controls::autocomplete) fn pressed_suggestion(
        &self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) -> Option<T> {
        if self.local_closed.get() || self.on_select.is_none() {
            return None;
        }
        let point = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.position(),
            Event::Touch(iced::touch::Event::FingerPressed { position, .. }) => Some(*position),
            _ => None,
        }?;
        let bounds = layout.bounds();
        if point.x < bounds.x + MENU_LIST_INSET
            || point.x > bounds.x + bounds.width - MENU_LIST_INSET
            || point.y < bounds.y + MENU_LIST_INSET
        {
            return None;
        }
        let index = ((point.y - bounds.y - MENU_LIST_INSET) / MENU_ROW_HEIGHT).floor() as usize;
        let (value, disabled) = self.suggestions.get(index)?;
        let row_bounds = Rectangle {
            x: bounds.x + MENU_LIST_INSET,
            y: bounds.y + MENU_LIST_INSET + index as f32 * MENU_ROW_HEIGHT,
            width: (bounds.width - MENU_LIST_INSET * 2.0).max(0.0),
            height: MENU_ROW_HEIGHT,
        };
        (!disabled && row_bounds.contains(point)).then(|| value.clone())
    }
}

impl<'a, T, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for HighlightVisibility<'a, T, Message>
where
    T: Clone + 'a,
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<HighlightVisibilityState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(HighlightVisibilityState::default())
    }

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
        self.request_highlight_visible(layout);
        if let (Some(value), Some(on_select)) = (
            self.pressed_suggestion(event, layout, cursor),
            &self.on_select,
        ) {
            tree.state
                .downcast_mut::<HighlightVisibilityState>()
                .selection_requested = true;
            self.local_closed.set(true);
            shell.publish(on_select(value));
            shell.capture_event();
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let child = self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        );
        if self.on_select.is_some()
            && !self.local_closed.get()
            && cursor.position().is_some_and(|point| {
                let bounds = layout.bounds();
                let index = ((point.y - bounds.y - MENU_LIST_INSET) / MENU_ROW_HEIGHT).floor();
                index >= 0.0
                    && self
                        .suggestions
                        .get(index as usize)
                        .is_some_and(|(_, disabled)| !disabled)
                    && point.x >= bounds.x + MENU_LIST_INSET
                    && point.x <= bounds.x + bounds.width - MENU_LIST_INSET
            })
        {
            mouse::Interaction::Pointer
        } else {
            child
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
