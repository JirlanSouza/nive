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

use super::super::super::super::split_divider;
use super::super::super::state::SplitStackState;
use super::super::super::{SplitCollapse, SplitResize, SplitSizing, SplitStack, SplitStackPane};
use super::super::{divider_layouts, pane_layouts};

pub(super) const ORIGIN: Vector = Vector::new(50.0, 30.0);

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Message {
    Resize(SplitResize),
    Collapse(SplitCollapse),
    Pane(usize),
}

struct EventProbe {
    index: usize,
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
        shell.publish(Message::Pane(self.index));
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

/// Drives a `SplitStack` the way a controlled app does: render, collect the
/// proposed sizes, feed them back, render again.
pub(super) struct Harness {
    element: Element<'static, Message>,
    tree: Tree,
    node: Node,
    renderer: iced::Renderer,
    size: Size,
    cursor: mouse::Cursor,
    orientation: Orientation,
    control_size: ControlSize,
    sizing: Vec<SplitSizing>,
    minimums: Vec<f32>,
    collapsible: Vec<bool>,
    threshold: Option<f32>,
    locked: bool,
    with_callback: bool,
}

impl Harness {
    pub(super) fn new(sizing: Vec<SplitSizing>, minimums: Vec<f32>, size: Size) -> Self {
        Self::build(
            Orientation::Horizontal,
            ControlSize::Sm,
            sizing,
            minimums,
            size,
            false,
            true,
        )
    }

    pub(super) fn configured(
        orientation: Orientation,
        sizing: Vec<SplitSizing>,
        minimums: Vec<f32>,
        size: Size,
        locked: bool,
        with_callback: bool,
    ) -> Self {
        Self::build(
            orientation,
            ControlSize::Sm,
            sizing,
            minimums,
            size,
            locked,
            with_callback,
        )
    }

    fn build(
        orientation: Orientation,
        control_size: ControlSize,
        sizing: Vec<SplitSizing>,
        minimums: Vec<f32>,
        size: Size,
        locked: bool,
        with_callback: bool,
    ) -> Self {
        let mut harness = Self {
            element: Element::new(EventProbe { index: 0 }),
            tree: Tree::empty(),
            node: Node::new(Size::ZERO),
            renderer: test_renderer(),
            size,
            cursor: mouse::Cursor::Unavailable,
            orientation,
            control_size,
            collapsible: vec![false; sizing.len()],
            threshold: None,
            sizing,
            minimums,
            locked,
            with_callback,
        };
        harness.rebuild();
        harness
    }

    /// Marks which panes may collapse and attaches the collapse callback.
    pub(super) fn collapsible(mut self, flags: Vec<bool>, threshold: Option<f32>) -> Self {
        self.collapsible = flags;
        self.threshold = threshold;
        self.rebuild();
        self
    }

    fn rebuild(&mut self) {
        let mut stack = SplitStack::new(self.orientation)
            .size(self.control_size)
            .locked(self.locked)
            .id("split-stack");

        for (index, sizing) in self.sizing.iter().enumerate() {
            let content = EventProbe { index };
            let pane = match sizing {
                SplitSizing::Fixed(size) => SplitStackPane::fixed(content, *size),
                SplitSizing::Fill => SplitStackPane::fill(content),
            };
            stack = stack.pane(
                pane.minimum(self.minimums[index])
                    .collapsible(self.collapsible[index]),
            );
        }

        if self.with_callback {
            stack = stack.on_resize(Message::Resize);
        }

        if self.collapsible.iter().any(|flag| *flag) {
            stack = stack.on_collapse(Message::Collapse);
            if let Some(threshold) = self.threshold {
                stack = stack.collapse_threshold(threshold);
            }
        }

        self.element = stack.into();
        self.tree = Tree::new(&self.element);
        self.layout();
    }

    fn layout(&mut self) {
        self.element.as_widget_mut().diff(&mut self.tree);
        self.node = self.element.as_widget_mut().layout(
            &mut self.tree,
            &self.renderer,
            &Limits::new(Size::ZERO, self.size),
        );
    }

    /// Feeds a proposed resize back into the app-owned sizes, as a real app does.
    pub(super) fn apply(&mut self, resize: SplitResize) {
        if let Some(SplitSizing::Fixed(size)) = self.sizing.get_mut(resize.divider) {
            *size = resize.leading;
        }
        if let Some(SplitSizing::Fixed(size)) = self.sizing.get_mut(resize.divider + 1) {
            *size = resize.trailing;
        }
        self.rebuild();
    }

    pub(super) fn resize_container(&mut self, size: Size) {
        self.size = size;
        self.layout();
    }

    pub(super) fn update(&mut self, event: Event) -> Vec<Message> {
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
        drop(shell);

        messages
    }

    /// Presses a divider and drags it by `delta` along the main axis.
    ///
    /// The first move only crosses the gesture drag threshold and anchors the
    /// drag, so `delta` is measured from that anchor.
    pub(super) fn drag(&mut self, divider: usize, delta: f32) -> Vec<Message> {
        let anchor = self.begin_drag(divider);

        self.move_by(anchor, delta)
    }

    /// Drops the cursor position, as iced does once the pointer is outside.
    pub(super) fn clear_cursor(&mut self) {
        self.cursor = mouse::Cursor::Unavailable;
    }

    pub(super) fn cursor_left(&mut self) -> Vec<Message> {
        self.update(Event::Mouse(mouse::Event::CursorLeft))
    }

    pub(super) fn cursor_entered(&mut self) -> Vec<Message> {
        self.update(Event::Mouse(mouse::Event::CursorEntered))
    }

    /// Presses a divider and crosses the gesture drag threshold, leaving the
    /// button down so the caller can drive the rest of the drag by hand.
    pub(super) fn begin_drag(&mut self, divider: usize) -> Point {
        let start = self.divider_hit_point(divider);
        let _ = self.update(Event::Mouse(mouse::Event::CursorMoved { position: start }));
        let _ = self.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));

        let anchor = self.offset_point(start, 8.0);
        let _ = self.update(Event::Mouse(mouse::Event::CursorMoved { position: anchor }));

        anchor
    }

    pub(super) fn move_by(&mut self, from: Point, delta: f32) -> Vec<Message> {
        let target = self.offset_point(from, delta);
        self.update(Event::Mouse(mouse::Event::CursorMoved { position: target }))
    }

    pub(super) fn press_key(&mut self, key: iced::keyboard::key::Named) -> Vec<Message> {
        let key = iced::keyboard::Key::Named(key);

        self.update(Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: iced::keyboard::key::Physical::Code(iced::keyboard::key::Code::KeyA),
            location: iced::keyboard::Location::Standard,
            modifiers: iced::keyboard::Modifiers::NONE,
            text: None,
            repeat: false,
        }))
    }

    fn offset_point(&self, from: Point, delta: f32) -> Point {
        match self.orientation {
            Orientation::Horizontal => Point::new(from.x + delta, from.y),
            Orientation::Vertical => Point::new(from.x, from.y + delta),
        }
    }

    pub(super) fn divider_hit_point(&self, divider: usize) -> Point {
        let bounds = self.divider_bounds(divider);

        Point::new(bounds.center_x(), bounds.center_y())
    }

    pub(super) fn pane_bounds(&self, index: usize) -> Rectangle {
        pane_layouts(Layout::with_offset(ORIGIN, &self.node))
            .nth(index)
            .expect("pane layout")
            .bounds()
    }

    pub(super) fn divider_bounds(&self, index: usize) -> Rectangle {
        divider_layouts(Layout::with_offset(ORIGIN, &self.node))
            .nth(index)
            .expect("divider layout")
            .bounds()
    }

    /// Main-axis length of every pane, in order.
    pub(super) fn lengths(&self) -> Vec<f32> {
        (0..self.sizing.len())
            .map(|index| self.orientation.main_length(self.pane_bounds(index).size()))
            .collect()
    }

    pub(super) fn hit_bounds(&self, divider: usize) -> Rectangle {
        split_divider::hit_bounds(
            self.divider_bounds(divider),
            Layout::with_offset(ORIGIN, &self.node).bounds(),
            self.orientation,
            split_divider::metrics(self.control_size),
        )
    }

    pub(super) fn state(&self) -> &SplitStackState {
        self.tree.state.downcast_ref::<SplitStackState>()
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
