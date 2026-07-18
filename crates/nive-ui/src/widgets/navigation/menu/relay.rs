use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use crate::{advanced::shell_relay, Element};

type OnMessage<'a, LocalMessage, Message> =
    dyn for<'shell> Fn(LocalMessage, &mut Shell<'shell, Message>) + 'a;

pub(super) struct MessageRelay<'a, LocalMessage, Message> {
    content: Element<'a, LocalMessage>,
    on_message: Box<OnMessage<'a, LocalMessage, Message>>,
}

impl<'a, LocalMessage, Message> MessageRelay<'a, LocalMessage, Message> {
    pub(super) fn new(
        content: impl Into<Element<'a, LocalMessage>>,
        on_message: impl for<'shell> Fn(LocalMessage, &mut Shell<'shell, Message>) + 'a,
    ) -> Self {
        Self {
            content: content.into(),
            on_message: Box::new(on_message),
        }
    }
}

impl<'a, LocalMessage, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for MessageRelay<'a, LocalMessage, Message>
where
    LocalMessage: 'a,
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
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
        self.content.as_widget_mut().layout(tree, renderer, limits)
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
            .operate(tree, layout, renderer, operation);
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
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);
        self.content.as_widget_mut().update(
            tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
            viewport,
        );
        shell_relay::propagate_to_parent(&mut local_shell, shell);
        drop(local_shell);

        for message in local_messages {
            (self.on_message)(message, shell);
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
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
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
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
            .map(|content| {
                overlay::Element::new(Box::new(RelayOverlay {
                    content,
                    on_message: self.on_message.as_ref(),
                }))
            })
    }
}

impl<'a, LocalMessage, Message> From<MessageRelay<'a, LocalMessage, Message>>
    for Element<'a, Message>
where
    LocalMessage: 'a,
    Message: 'a,
{
    fn from(relay: MessageRelay<'a, LocalMessage, Message>) -> Self {
        Element::new(relay)
    }
}

struct RelayOverlay<'a, LocalMessage, Message> {
    content: overlay::Element<'a, LocalMessage, crate::theme::Theme, iced::Renderer>,
    on_message: &'a OnMessage<'a, LocalMessage, Message>,
}

impl<LocalMessage, Message> overlay::Overlay<Message, crate::theme::Theme, iced::Renderer>
    for RelayOverlay<'_, LocalMessage, Message>
{
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        self.content.as_overlay_mut().layout(renderer, bounds)
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        self.content
            .as_overlay_mut()
            .operate(layout, renderer, operation);
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
        self.content.as_overlay_mut().update(
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
        );
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
        self.content
            .as_overlay()
            .mouse_interaction(layout, cursor, renderer)
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.content
            .as_overlay()
            .draw(renderer, theme, style, layout, cursor);
    }

    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
    ) -> Option<overlay::Element<'a, Message, crate::theme::Theme, iced::Renderer>> {
        self.content
            .as_overlay_mut()
            .overlay(layout, renderer)
            .map(|content| {
                overlay::Element::new(Box::new(RelayOverlay {
                    content,
                    on_message: self.on_message,
                }))
            })
    }
}
