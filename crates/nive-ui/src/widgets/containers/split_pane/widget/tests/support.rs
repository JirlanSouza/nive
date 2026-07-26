use iced::{
    advanced::{
        layout::{Layout, Limits, Node},
        mouse, renderer,
        widget::{operation, Tree},
        Clipboard, Shell, Widget,
    },
    Event, Font, Length, Pixels, Point, Rectangle, Size, Vector,
};

use crate::interaction::Orientation;
use crate::theme::ControlSize;
use crate::{Element, Theme};

use super::super::super::helpers::{metrics, SplitDividerMetrics};
use super::super::super::state::SplitPaneState;
use super::super::super::SplitPane;
use super::super::event::{current_divider_bounds, current_hit_bounds};

pub(super) const ORIGIN: Vector = Vector::new(50.0, 30.0);

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Message {
    Ratio(f32),
    Leading,
    Trailing,
}

#[derive(Debug)]
pub(super) struct UpdateResult {
    pub(super) messages: Vec<Message>,
    pub(super) captured: bool,
}

struct EventProbe {
    message: Message,
}

impl EventProbe {
    fn new(message: Message) -> Self {
        Self { message }
    }
}

impl Widget<Message, Theme, iced::Renderer> for EventProbe {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &iced::Renderer, limits: &Limits) -> Node {
        Node::new(limits.resolve(Length::Fill, Length::Fill, Size::ZERO))
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
        _theme: &Theme,
        _inherited_style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }
}

impl<'a> From<EventProbe> for Element<'a, Message> {
    fn from(probe: EventProbe) -> Self {
        Element::new(probe)
    }
}

pub(super) struct Harness {
    element: Element<'static, Message>,
    tree: Tree,
    node: Node,
    renderer: iced::Renderer,
    size: Size,
    cursor: mouse::Cursor,
    orientation: Orientation,
    metrics: SplitDividerMetrics,
}

impl Harness {
    pub(super) fn new(
        orientation: Orientation,
        control_size: ControlSize,
        size: Size,
        ratio: f32,
        locked: bool,
    ) -> Self {
        Self::new_with_callback(orientation, control_size, size, ratio, locked, true)
    }

    pub(super) fn new_with_callback(
        orientation: Orientation,
        control_size: ControlSize,
        size: Size,
        ratio: f32,
        locked: bool,
        with_callback: bool,
    ) -> Self {
        let pane = SplitPane::new(
            EventProbe::new(Message::Leading),
            EventProbe::new(Message::Trailing),
        )
        .orientation(orientation)
        .size(control_size)
        .ratio(ratio)
        .locked(locked)
        .id("split-pane");
        let pane = if with_callback {
            pane.on_change(Message::Ratio)
        } else {
            pane
        };
        let element = pane.into();
        let tree = Tree::new(&element);
        let mut harness = Self {
            element,
            tree,
            node: Node::new(Size::ZERO),
            renderer: test_renderer(),
            size,
            cursor: mouse::Cursor::Unavailable,
            orientation,
            metrics: metrics(control_size),
        };
        harness.layout();
        harness
    }

    pub(super) fn update(&mut self, event: Event) -> UpdateResult {
        if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
            self.cursor = mouse::Cursor::Available(position);
        }

        let mut messages = Vec::new();
        let mut clipboard = iced::advanced::clipboard::Null;
        let viewport = Rectangle::new(Point::new(ORIGIN.x, ORIGIN.y), Size::new(4096.0, 4096.0));
        let mut shell = Shell::new(&mut messages);

        self.element.as_widget_mut().update(
            &mut self.tree,
            &event,
            Layout::with_offset(ORIGIN, &self.node),
            self.cursor,
            &self.renderer,
            &mut clipboard,
            &mut shell,
            &viewport,
        );
        let captured = shell.is_event_captured();
        drop(shell);

        UpdateResult { messages, captured }
    }

    pub(super) fn move_to(&mut self, position: Point) -> UpdateResult {
        self.update(Event::Mouse(mouse::Event::CursorMoved { position }))
    }

    pub(super) fn press(&mut self, button: mouse::Button, position: Point) -> UpdateResult {
        let _ = self.move_to(position);
        self.update(Event::Mouse(mouse::Event::ButtonPressed(button)))
    }

    pub(super) fn divider_bounds(&self) -> Rectangle {
        current_divider_bounds(Layout::with_offset(ORIGIN, &self.node))
            .expect("split pane divider layout")
    }

    pub(super) fn hit_bounds(&self) -> Rectangle {
        current_hit_bounds(
            Layout::with_offset(ORIGIN, &self.node),
            self.orientation,
            self.metrics,
        )
        .expect("split pane hit layout")
    }

    pub(super) fn local_divider_bounds(&self) -> Rectangle {
        self.node
            .children()
            .get(1)
            .expect("split pane divider node")
            .bounds()
    }

    pub(super) fn bounds(&self) -> Rectangle {
        Layout::with_offset(ORIGIN, &self.node).bounds()
    }

    pub(super) fn child_bounds(&self, index: usize) -> Rectangle {
        Layout::with_offset(ORIGIN, &self.node)
            .children()
            .nth(index)
            .expect("split pane child layout")
            .bounds()
    }

    pub(super) fn metrics(&self) -> SplitDividerMetrics {
        self.metrics
    }

    pub(super) fn state(&self) -> &SplitPaneState {
        self.tree.state.downcast_ref::<SplitPaneState>()
    }

    pub(super) fn focusable_bounds(&mut self) -> Vec<Rectangle> {
        let mut operation = FocusBounds::default();
        self.element.as_widget_mut().operate(
            &mut self.tree,
            Layout::with_offset(ORIGIN, &self.node),
            &self.renderer,
            &mut operation,
        );
        operation.bounds
    }

    pub(super) fn mouse_interaction(&self) -> mouse::Interaction {
        let viewport = Rectangle::new(Point::new(ORIGIN.x, ORIGIN.y), Size::new(4096.0, 4096.0));

        self.element.as_widget().mouse_interaction(
            &self.tree,
            Layout::with_offset(ORIGIN, &self.node),
            self.cursor,
            &viewport,
            &self.renderer,
        )
    }

    pub(super) fn hit_only_point(&self) -> Point {
        let divider = self.divider_bounds();
        let hit = self.hit_bounds();
        let point = match self.orientation {
            Orientation::Horizontal => Point::new(hit.x + 1.0, divider.center_y()),
            Orientation::Vertical => Point::new(divider.center_x(), hit.y + 1.0),
        };

        assert!(hit.contains(point));
        assert!(!divider.contains(point));

        point
    }

    fn layout(&mut self) {
        self.element.as_widget_mut().diff(&mut self.tree);
        self.node = self.element.as_widget_mut().layout(
            &mut self.tree,
            &self.renderer,
            &Limits::new(Size::ZERO, self.size),
        );
    }
}

#[derive(Default)]
struct FocusBounds {
    bounds: Vec<Rectangle>,
}

impl operation::Operation<()> for FocusBounds {
    fn focusable(
        &mut self,
        _id: Option<&iced::widget::Id>,
        bounds: Rectangle,
        _state: &mut dyn operation::Focusable,
    ) {
        self.bounds.push(bounds);
    }

    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation<()>)) {
        operate(self);
    }

    fn finish(&self) -> operation::Outcome<()> {
        operation::Outcome::None
    }
}

pub(super) fn test_renderer() -> iced::Renderer {
    iced_renderer::fallback::Renderer::Secondary(iced_tiny_skia::Renderer::new(
        Font::default(),
        Pixels(14.0),
    ))
}
