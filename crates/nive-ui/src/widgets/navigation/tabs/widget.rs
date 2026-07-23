use iced::{
    advanced::{
        layout::{self, Layout},
        mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use super::{TabBar, TabBarState};

mod compose;
mod dnd;
mod draw;
mod pointer;
mod update;

impl<'a, Id, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for TabBar<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TabBarState<Id>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TabBarState::<Id>::default())
    }

    fn children(&self) -> Vec<Tree> {
        let state = TabBarState::<Id>::default();
        vec![Tree::new(self.content_element(&state))]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_ref::<TabBarState<Id>>();
        let content = self.content_element(state);

        if tree.children.is_empty() {
            tree.children.push(Tree::new(&content));
        } else {
            tree.children[0].diff(content.as_widget());
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width.unwrap_or(Length::Shrink), Length::Shrink)
    }

    fn size_hint(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.layout_impl(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        self.operate_impl(tree, layout, renderer, operation);
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
        self.update_impl(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
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
        self.mouse_interaction_impl(tree, layout, cursor, viewport, renderer)
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
        self.draw_impl(
            tree,
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
        self.overlay_impl(tree, layout, renderer, viewport, translation)
    }
}
