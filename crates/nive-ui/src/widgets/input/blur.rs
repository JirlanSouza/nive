use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Operation as _, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Rectangle, Size as IcedSize, Vector,
};

use crate::Element;

pub(super) struct InputBlur<'a, Message> {
    pub(super) content: Element<'a, Message>,
    pub(super) on_blur: Message,
}

#[derive(Debug, Default)]
struct InputBlurState;

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer> for InputBlur<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<InputBlurState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(InputBlurState)
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> IcedSize<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> IcedSize<Length> {
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
        let was_focused =
            child_has_focus(&mut self.content, &mut tree.children[0], layout, renderer);

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

        let is_focused =
            child_has_focus(&mut self.content, &mut tree.children[0], layout, renderer);

        if was_focused && !is_focused {
            shell.publish(self.on_blur.clone());
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

pub(super) fn child_has_focus<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    layout: Layout<'_>,
    renderer: &iced::Renderer,
) -> bool {
    let mut count_focused = operation::focusable::count();

    content.as_widget_mut().operate(
        tree,
        layout,
        renderer,
        &mut operation::black_box(&mut count_focused),
    );

    match count_focused.finish() {
        operation::Outcome::Some(count) => count.focused.is_some(),
        _ => false,
    }
}
