use iced::{
    advanced::{
        layout::{Layout, Limits, Node},
        mouse, renderer,
        widget::Tree,
        Clipboard, Shell, Widget,
    },
    Event, Font, Length, Pixels, Point, Rectangle, Size,
};

use crate::Element;

pub(crate) fn renderer() -> iced::Renderer {
    iced_renderer::fallback::Renderer::Secondary(iced_tiny_skia::Renderer::new(
        Font::default(),
        Pixels(14.0),
    ))
}

pub(crate) fn layout<Message>(mut element: Element<'_, Message>, maximum: Size) -> Node {
    let mut tree = Tree::new(&element);
    let renderer = renderer();
    element
        .as_widget_mut()
        .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, maximum))
}

pub(crate) fn event_messages<Message>(
    mut element: Element<'_, Message>,
    maximum: Size,
    event: Event,
) -> Vec<Message> {
    let mut tree = Tree::new(&element);
    let renderer = renderer();
    let node =
        element
            .as_widget_mut()
            .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, maximum));
    let mut messages = Vec::new();
    let mut clipboard = iced::advanced::clipboard::Null;
    let mut shell = Shell::new(&mut messages);
    let viewport = Rectangle::new(Point::ORIGIN, maximum);
    element.as_widget_mut().update(
        &mut tree,
        &event,
        Layout::new(&node),
        mouse::Cursor::Available(Point::new(1.0, 1.0)),
        &renderer,
        &mut clipboard,
        &mut shell,
        &viewport,
    );
    drop(shell);
    messages
}

pub(crate) fn event_probe<Message: Clone + 'static>(message: Message) -> Element<'static, Message> {
    Element::new(EventProbe { message })
}

struct EventProbe<Message> {
    message: Message,
}

impl<Message: Clone> Widget<Message, crate::theme::Theme, iced::Renderer> for EventProbe<Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(24.0), Length::Fixed(20.0))
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &iced::Renderer, limits: &Limits) -> Node {
        Node::new(limits.resolve(
            Length::Fixed(24.0),
            Length::Fixed(20.0),
            Size::new(24.0, 20.0),
        ))
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        _event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        shell.publish(self.message.clone());
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut iced::Renderer,
        _theme: &crate::theme::Theme,
        _inherited_style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }
}
