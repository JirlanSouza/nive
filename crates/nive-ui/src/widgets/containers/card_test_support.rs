use iced::{
    advanced::{
        layout::{Layout, Limits, Node},
        mouse,
        widget::Tree,
        Shell,
    },
    keyboard::{self, key},
    Event, Font, Pixels, Point, Rectangle, Size,
};

use crate::{focus_trap::FocusDirection, Element};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Message {
    Activated,
}

pub(crate) struct CardHarness {
    element: Element<'static, Message>,
    tree: Tree,
    node: Node,
    renderer: iced::Renderer,
    maximum: Size,
    cursor: mouse::Cursor,
}

impl CardHarness {
    pub(crate) fn new(element: Element<'static, Message>, maximum: Size) -> Self {
        let tree = Tree::new(&element);
        let mut harness = Self {
            element,
            tree,
            node: Node::new(Size::ZERO),
            renderer: test_renderer(),
            maximum,
            cursor: mouse::Cursor::Unavailable,
        };
        harness.layout();
        harness
    }

    pub(crate) fn size(&self) -> Size {
        self.node.size()
    }

    pub(crate) fn child_bounds(&self) -> Vec<Rectangle> {
        self.node.children().iter().map(Node::bounds).collect()
    }

    pub(crate) fn center(&self) -> Point {
        let bounds = self.node.bounds();
        Point::new(bounds.center_x(), bounds.center_y())
    }

    pub(crate) fn click_center(&mut self) -> Vec<Message> {
        let center = self.center();
        let mut messages = Vec::new();

        for event in [
            Event::Mouse(mouse::Event::CursorMoved { position: center }),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ] {
            messages.extend(self.update(event));
        }

        messages
    }

    pub(crate) fn focus_next(&mut self) {
        let layout = Layout::new(&self.node);
        FocusDirection::Next.operate(|operation| {
            self.element
                .as_widget_mut()
                .operate(&mut self.tree, layout, &self.renderer, operation);
        });
    }

    pub(crate) fn activate_key(&mut self, named: key::Named, repeat: bool) -> Vec<Message> {
        self.update(Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(named),
            modified_key: keyboard::Key::Named(named),
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Enter),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::NONE,
            text: None,
            repeat,
        }))
    }

    pub(crate) fn mouse_interaction(&self) -> mouse::Interaction {
        self.element.as_widget().mouse_interaction(
            &self.tree,
            Layout::new(&self.node),
            mouse::Cursor::Available(self.center()),
            &Rectangle::new(Point::ORIGIN, self.maximum),
            &self.renderer,
        )
    }

    pub(crate) fn has_overlay(&mut self) -> bool {
        self.element
            .as_widget_mut()
            .overlay(
                &mut self.tree,
                Layout::new(&self.node),
                &self.renderer,
                &Rectangle::new(Point::ORIGIN, self.maximum),
                iced::Vector::ZERO,
            )
            .is_some()
    }

    fn layout(&mut self) {
        self.element.as_widget_mut().diff(&mut self.tree);
        self.node = self.element.as_widget_mut().layout(
            &mut self.tree,
            &self.renderer,
            &Limits::new(Size::ZERO, self.maximum),
        );
    }

    fn update(&mut self, event: Event) -> Vec<Message> {
        if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
            self.cursor = mouse::Cursor::Available(position);
        }

        let mut messages = Vec::new();
        let mut clipboard = iced::advanced::clipboard::Null;
        let mut shell = Shell::new(&mut messages);

        self.element.as_widget_mut().update(
            &mut self.tree,
            &event,
            Layout::new(&self.node),
            self.cursor,
            &self.renderer,
            &mut clipboard,
            &mut shell,
            &Rectangle::new(Point::ORIGIN, self.maximum),
        );

        messages
    }
}

fn test_renderer() -> iced::Renderer {
    iced_renderer::fallback::Renderer::Secondary(iced_tiny_skia::Renderer::new(
        Font::default(),
        Pixels(14.0),
    ))
}
