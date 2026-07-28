//! A wrapper that carries the truncated label's logical-focus flag, so the
//! tooltip anchor participates in focus without owning keyboard focus itself.

use super::*;

#[derive(Debug, Default)]
struct LogicalFocusState {
    focused: bool,
}

impl operation::Focusable for LogicalFocusState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {}

    fn unfocus(&mut self) {}
}

pub(super) struct LogicalFocusCandidate<'a, Message> {
    content: Element<'a, Message>,
    focused: Rc<Cell<bool>>,
}

impl<'a, Message> LogicalFocusCandidate<'a, Message> {
    pub(super) fn new(content: impl Into<Element<'a, Message>>, focused: Rc<Cell<bool>>) -> Self {
        Self {
            content: content.into(),
            focused,
        }
    }
}

impl<Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for LogicalFocusCandidate<'_, Message>
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<LogicalFocusState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(LogicalFocusState::default())
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
        let state = tree.state.downcast_mut::<LogicalFocusState>();
        state.focused = self.focused.get();
        operation.focusable(None, layout.bounds(), state);
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        });
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
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
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
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, crate::theme::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<LogicalFocusCandidate<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(candidate: LogicalFocusCandidate<'a, Message>) -> Self {
        Element::new(candidate)
    }
}
