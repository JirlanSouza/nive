use iced::{
    advanced::{
        clipboard,
        layout::{Layout, Limits, Node},
        mouse, overlay, renderer,
        widget::{operation, tree, Id, Operation as _, Tree},
        Clipboard, Shell, Widget,
    },
    event, Event, Font, Length, Pixels, Point, Rectangle, Size, Vector,
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

pub(crate) struct WidgetHarness<'a, Message> {
    element: Element<'a, Message>,
    tree: Tree,
    node: Node,
    renderer: iced::Renderer,
    maximum: Size,
    cursor: mouse::Cursor,
    clipboard: MemoryClipboard,
}

impl<'a, Message> WidgetHarness<'a, Message> {
    pub(crate) fn new(mut element: Element<'a, Message>, maximum: Size) -> Self {
        let mut tree = Tree::new(&element);
        let renderer = renderer();
        let node =
            element
                .as_widget_mut()
                .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, maximum));

        Self {
            element,
            tree,
            node,
            renderer,
            maximum,
            cursor: mouse::Cursor::Unavailable,
            clipboard: MemoryClipboard::default(),
        }
    }

    pub(crate) fn bounds(&self) -> Rectangle {
        Layout::new(&self.node).bounds()
    }

    pub(crate) fn state<T: 'static>(&self) -> &T {
        self.tree.state.downcast_ref::<T>()
    }

    pub(crate) fn state_at<T: 'static>(&self, path: &[usize]) -> &T {
        let tree = path
            .iter()
            .fold(&self.tree, |tree, index| &tree.children[*index]);
        tree.state.downcast_ref::<T>()
    }

    pub(crate) fn relayout(&mut self, maximum: Size) {
        self.maximum = maximum;
        self.node = self.element.as_widget_mut().layout(
            &mut self.tree,
            &self.renderer,
            &Limits::new(Size::ZERO, maximum),
        );
    }

    pub(crate) fn replace(&mut self, element: Element<'a, Message>) {
        self.element = element;
        self.tree.diff(self.element.as_widget());
        self.relayout(self.maximum);
    }

    pub(crate) fn operate(&mut self, operation: &mut dyn operation::Operation) {
        self.element.as_widget_mut().operate(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            operation,
        );
    }

    pub(crate) fn operation_outcome<T: 'static>(
        &mut self,
        operation: &mut dyn operation::Operation<T>,
    ) -> operation::Outcome<T> {
        self.operate(&mut operation::black_box(operation));
        operation.finish()
    }

    pub(crate) fn focus(&mut self, id: Id) {
        self.operate(&mut operation::focusable::focus(id));
    }

    pub(crate) fn focus_next(&mut self) {
        crate::focus_trap::FocusDirection::Next.operate(|operation| self.operate(operation));
    }

    pub(crate) fn focus_previous(&mut self) {
        crate::focus_trap::FocusDirection::Previous.operate(|operation| self.operate(operation));
    }

    pub(crate) fn focused_count(&mut self) -> operation::focusable::Count {
        let mut count = operation::focusable::count();
        self.element.as_widget_mut().operate(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            &mut operation::black_box(&mut count),
        );

        match count.finish() {
            operation::Outcome::Some(count) => count,
            _ => operation::focusable::Count::default(),
        }
    }

    pub(crate) fn focused_widgets(&mut self) -> usize {
        struct FocusedWidgets(usize);

        impl operation::Operation for FocusedWidgets {
            fn focusable(
                &mut self,
                _id: Option<&Id>,
                _bounds: Rectangle,
                state: &mut dyn operation::Focusable,
            ) {
                self.0 += usize::from(state.is_focused());
            }

            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
                operate(self);
            }
        }

        let mut focused = FocusedWidgets(0);
        self.operate(&mut focused);
        focused.0
    }

    pub(crate) fn focused_ids(&mut self) -> Vec<Id> {
        struct FocusedIds(Vec<Id>);

        impl operation::Operation for FocusedIds {
            fn focusable(
                &mut self,
                id: Option<&Id>,
                _bounds: Rectangle,
                state: &mut dyn operation::Focusable,
            ) {
                if state.is_focused() {
                    if let Some(id) = id {
                        self.0.push(id.clone());
                    }
                }
            }

            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
                operate(self);
            }
        }

        let mut focused = FocusedIds(Vec::new());
        self.operate(&mut focused);
        focused.0
    }

    pub(crate) fn named_bounds(&mut self, name: &'static str) -> Option<Rectangle> {
        let mut probe = BoundsProbe::new(name);
        self.element.as_widget_mut().operate(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            &mut probe,
        );
        probe.bounds
    }

    pub(crate) fn focusable_bounds(&mut self, target: &Id) -> Option<Rectangle> {
        struct FocusableBounds<'a> {
            target: &'a Id,
            bounds: Option<Rectangle>,
        }

        impl operation::Operation for FocusableBounds<'_> {
            fn focusable(
                &mut self,
                id: Option<&Id>,
                bounds: Rectangle,
                _state: &mut dyn operation::Focusable,
            ) {
                if id == Some(self.target) {
                    self.bounds = Some(bounds);
                }
            }

            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
                operate(self);
            }
        }

        let mut probe = FocusableBounds {
            target,
            bounds: None,
        };
        self.operate(&mut probe);
        probe.bounds
    }

    pub(crate) fn focusable_ids(&mut self) -> Vec<Id> {
        struct FocusableIds(Vec<Id>);

        impl operation::Operation for FocusableIds {
            fn focusable(
                &mut self,
                id: Option<&Id>,
                _bounds: Rectangle,
                _state: &mut dyn operation::Focusable,
            ) {
                if let Some(id) = id {
                    self.0.push(id.clone());
                }
            }

            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
                operate(self);
            }
        }

        let mut ids = FocusableIds(Vec::new());
        self.operate(&mut ids);
        ids.0
    }

    pub(crate) fn managed_focus(&mut self) -> ManagedFocusSnapshot {
        let mut probe = ManagedFocusProbe::default();
        self.operate(&mut probe);
        probe.finish()
    }

    pub(crate) fn mouse_interaction(&self) -> mouse::Interaction {
        self.element.as_widget().mouse_interaction(
            &self.tree,
            Layout::new(&self.node),
            self.cursor,
            &Rectangle::new(Point::ORIGIN, self.maximum),
            &self.renderer,
        )
    }

    pub(crate) fn draw(&mut self) {
        let theme = crate::theme::active();
        let style = renderer::Style {
            text_color: theme.text(crate::theme::TextRole::Primary).color,
        };
        let viewport = Rectangle::new(Point::ORIGIN, self.maximum);

        self.element.as_widget().draw(
            &self.tree,
            &mut self.renderer,
            &theme,
            &style,
            Layout::new(&self.node),
            self.cursor,
            &viewport,
        );
    }

    pub(crate) fn has_overlay(&mut self) -> bool {
        let viewport = Rectangle::new(Point::ORIGIN, self.maximum);
        self.element
            .as_widget_mut()
            .overlay(
                &mut self.tree,
                Layout::new(&self.node),
                &self.renderer,
                &viewport,
                Vector::ZERO,
            )
            .is_some()
    }

    pub(crate) fn draw_overlay(&mut self) -> bool {
        let viewport = Rectangle::new(Point::ORIGIN, self.maximum);
        let theme = crate::theme::active();
        let style = renderer::Style {
            text_color: theme.text(crate::theme::TextRole::Primary).color,
        };
        let Some(mut overlay) = self.element.as_widget_mut().overlay(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            &viewport,
            Vector::ZERO,
        ) else {
            return false;
        };
        let node = overlay
            .as_overlay_mut()
            .layout(&self.renderer, self.maximum);
        overlay.as_overlay().draw(
            &mut self.renderer,
            &theme,
            &style,
            Layout::new(&node),
            self.cursor,
        );
        true
    }

    pub(crate) fn overlay_bounds(&mut self) -> Option<Rectangle> {
        let viewport = Rectangle::new(Point::ORIGIN, self.maximum);
        let mut overlay = self.element.as_widget_mut().overlay(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            &viewport,
            Vector::ZERO,
        )?;
        let node = overlay
            .as_overlay_mut()
            .layout(&self.renderer, self.maximum);
        Some(overlay_content_bounds(&node, self.maximum))
    }

    pub(crate) fn update_overlay(&mut self, event: Event) -> Option<UpdateResult<Message>> {
        let viewport = Rectangle::new(Point::ORIGIN, self.maximum);
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        let mut overlay = self.element.as_widget_mut().overlay(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            &viewport,
            Vector::ZERO,
        )?;
        let node = overlay
            .as_overlay_mut()
            .layout(&self.renderer, self.maximum);
        overlay.as_overlay_mut().update(
            &event,
            Layout::new(&node),
            self.cursor,
            &self.renderer,
            &mut self.clipboard,
            &mut shell,
        );
        let captured = shell.event_status() == event::Status::Captured;
        let layout_invalid = shell.is_layout_invalid();
        let redraw_request = shell.redraw_request();
        let input_method_enabled = shell.input_method().is_enabled();
        drop(shell);
        drop(overlay);

        if layout_invalid {
            self.relayout(self.maximum);
        }

        Some(UpdateResult {
            messages,
            captured,
            layout_invalid,
            redraw_request,
            input_method_enabled,
        })
    }

    pub(crate) fn update_nested_overlay(&mut self, event: Event) -> Option<UpdateResult<Message>> {
        let viewport = Rectangle::new(Point::ORIGIN, self.maximum);
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        let overlay = self.element.as_widget_mut().overlay(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            &viewport,
            Vector::ZERO,
        )?;
        let mut nested = overlay::Nested::new(overlay);
        let node = nested.layout(&self.renderer, self.maximum);
        nested.update(
            &event,
            Layout::new(&node),
            self.cursor,
            &self.renderer,
            &mut self.clipboard,
            &mut shell,
        );
        let captured = shell.event_status() == event::Status::Captured;
        let layout_invalid = shell.is_layout_invalid();
        let redraw_request = shell.redraw_request();
        let input_method_enabled = shell.input_method().is_enabled();
        drop(shell);
        drop(nested);

        if layout_invalid {
            self.relayout(self.maximum);
        }

        Some(UpdateResult {
            messages,
            captured,
            layout_invalid,
            redraw_request,
            input_method_enabled,
        })
    }

    pub(crate) fn nested_overlay_bounds(&mut self) -> Vec<Rectangle> {
        let viewport = Rectangle::new(Point::ORIGIN, self.maximum);
        let Some(overlay) = self.element.as_widget_mut().overlay(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            &viewport,
            Vector::ZERO,
        ) else {
            return Vec::new();
        };
        let mut nested = overlay::Nested::new(overlay);
        let node = nested.layout(&self.renderer, self.maximum);
        let mut bounds = Vec::new();
        collect_nested_overlay_bounds(Layout::new(&node), &mut bounds);
        bounds
    }

    pub(crate) fn focused_overlay_count(&mut self) -> Option<operation::focusable::Count> {
        let viewport = Rectangle::new(Point::ORIGIN, self.maximum);
        let mut overlay = self.element.as_widget_mut().overlay(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            &viewport,
            Vector::ZERO,
        )?;
        let node = overlay
            .as_overlay_mut()
            .layout(&self.renderer, self.maximum);
        let mut count = operation::focusable::count();
        overlay.as_overlay_mut().operate(
            Layout::new(&node),
            &self.renderer,
            &mut operation::black_box(&mut count),
        );

        match count.finish() {
            operation::Outcome::Some(count) => Some(count),
            _ => Some(operation::focusable::Count::default()),
        }
    }

    pub(crate) fn overlay_scroll_offsets(&mut self) -> Vec<Vector> {
        struct ScrollOffsets(Vec<Vector>);

        impl operation::Operation for ScrollOffsets {
            fn scrollable(
                &mut self,
                _id: Option<&Id>,
                _bounds: Rectangle,
                _content_bounds: Rectangle,
                translation: Vector,
                _state: &mut dyn operation::Scrollable,
            ) {
                self.0.push(translation);
            }

            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
                operate(self);
            }
        }

        let viewport = Rectangle::new(Point::ORIGIN, self.maximum);
        let Some(mut overlay) = self.element.as_widget_mut().overlay(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            &viewport,
            Vector::ZERO,
        ) else {
            return Vec::new();
        };
        let node = overlay
            .as_overlay_mut()
            .layout(&self.renderer, self.maximum);
        let mut offsets = ScrollOffsets(Vec::new());
        overlay
            .as_overlay_mut()
            .operate(Layout::new(&node), &self.renderer, &mut offsets);
        offsets.0
    }

    pub(crate) fn focus_overlay_next(&mut self) -> bool {
        let mut operated = false;
        crate::focus_trap::FocusDirection::Next.operate(|operation| {
            let viewport = Rectangle::new(Point::ORIGIN, self.maximum);
            let Some(mut overlay) = self.element.as_widget_mut().overlay(
                &mut self.tree,
                Layout::new(&self.node),
                &self.renderer,
                &viewport,
                Vector::ZERO,
            ) else {
                return;
            };
            let node = overlay
                .as_overlay_mut()
                .layout(&self.renderer, self.maximum);
            overlay
                .as_overlay_mut()
                .operate(Layout::new(&node), &self.renderer, operation);
            operated = true;
        });
        operated
    }

    pub(crate) fn set_cursor(&mut self, position: Point) {
        self.cursor = mouse::Cursor::Available(position);
    }

    pub(crate) fn clear_cursor(&mut self) {
        self.cursor = mouse::Cursor::Unavailable;
    }

    pub(crate) fn clipboard(&self, kind: clipboard::Kind) -> Option<&str> {
        self.clipboard.read_ref(kind)
    }

    pub(crate) fn set_clipboard(&mut self, kind: clipboard::Kind, contents: impl Into<String>) {
        self.clipboard.write(kind, contents.into());
    }

    pub(crate) fn update(&mut self, event: Event) -> UpdateResult<Message> {
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        let viewport = Rectangle::new(Point::ORIGIN, self.maximum);

        self.element.as_widget_mut().update(
            &mut self.tree,
            &event,
            Layout::new(&self.node),
            self.cursor,
            &self.renderer,
            &mut self.clipboard,
            &mut shell,
            &viewport,
        );

        let captured = shell.event_status() == event::Status::Captured;
        let layout_invalid = shell.is_layout_invalid();
        let redraw_request = shell.redraw_request();
        let input_method_enabled = shell.input_method().is_enabled();
        drop(shell);

        if layout_invalid {
            self.node = self.element.as_widget_mut().layout(
                &mut self.tree,
                &self.renderer,
                &Limits::new(Size::ZERO, self.maximum),
            );
        }

        UpdateResult {
            messages,
            captured,
            layout_invalid,
            redraw_request,
            input_method_enabled,
        }
    }
}

fn collect_nested_overlay_bounds(layout: Layout<'_>, bounds: &mut Vec<Rectangle>) {
    let mut children = layout.children();
    let Some(current) = children.next() else {
        return;
    };
    bounds.push(current.bounds());
    if let Some(nested) = children.next() {
        collect_nested_overlay_bounds(nested, bounds);
    }
}

mod focus;
pub(crate) use focus::*;

fn overlay_content_bounds(node: &Node, maximum: Size) -> Rectangle {
    let bounds = node.bounds();
    if bounds.position() == Point::ORIGIN && bounds.size() == maximum {
        node.children()
            .last()
            .map_or(bounds, |child| overlay_content_bounds(child, maximum))
    } else {
        bounds
    }
}

pub(crate) struct UpdateResult<Message> {
    pub(crate) messages: Vec<Message>,
    pub(crate) captured: bool,
    pub(crate) layout_invalid: bool,
    pub(crate) redraw_request: iced::window::RedrawRequest,
    pub(crate) input_method_enabled: bool,
}

#[derive(Debug, Default)]
pub(crate) struct MemoryClipboard {
    standard: Option<String>,
    primary: Option<String>,
}

impl MemoryClipboard {
    fn read_ref(&self, kind: clipboard::Kind) -> Option<&str> {
        match kind {
            clipboard::Kind::Standard => self.standard.as_deref(),
            clipboard::Kind::Primary => self.primary.as_deref(),
        }
    }
}

impl Clipboard for MemoryClipboard {
    fn read(&self, kind: clipboard::Kind) -> Option<String> {
        self.read_ref(kind).map(str::to_owned)
    }

    fn write(&mut self, kind: clipboard::Kind, contents: String) {
        match kind {
            clipboard::Kind::Standard => self.standard = Some(contents),
            clipboard::Kind::Primary => self.primary = Some(contents),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FormStateFixture {
    pub(crate) hovered: bool,
    pub(crate) focused: bool,
    pub(crate) pressed: bool,
    pub(crate) read_only: bool,
    pub(crate) disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct FakeClock {
    now_ms: u64,
}

impl FakeClock {
    pub(crate) const fn at(now_ms: u64) -> Self {
        Self { now_ms }
    }

    pub(crate) const fn now_ms(self) -> u64 {
        self.now_ms
    }

    pub(crate) fn advance(&mut self, elapsed_ms: u64) {
        self.now_ms = self.now_ms.saturating_add(elapsed_ms);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AnchoredGeometryFixture {
    pub(crate) anchor: Rectangle,
    pub(crate) viewport: Rectangle,
    pub(crate) intrinsic_content: Size,
}

impl AnchoredGeometryFixture {
    pub(crate) const fn new(
        anchor: Rectangle,
        viewport: Rectangle,
        intrinsic_content: Size,
    ) -> Self {
        Self {
            anchor,
            viewport,
            intrinsic_content,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct EnsureVisibleFixture {
    pub(crate) viewport: Rectangle,
    pub(crate) target: Rectangle,
    pub(crate) current_offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PopupStateFixture {
    pub(crate) capable: bool,
    pub(crate) disabled: bool,
    pub(crate) open: bool,
    pub(crate) selected: bool,
    pub(crate) highlighted: bool,
    pub(crate) focused: bool,
    pub(crate) pressed: bool,
}

impl PopupStateFixture {
    pub(crate) const fn enabled() -> Self {
        Self {
            capable: true,
            disabled: false,
            open: false,
            selected: false,
            highlighted: false,
            focused: false,
            pressed: false,
        }
    }
}

impl FormStateFixture {
    pub(crate) const INTERACTIVE: [Self; 4] = [
        Self::enabled(),
        Self {
            hovered: true,
            ..Self::enabled()
        },
        Self {
            focused: true,
            ..Self::enabled()
        },
        Self {
            pressed: true,
            ..Self::enabled()
        },
    ];

    pub(crate) const fn enabled() -> Self {
        Self {
            hovered: false,
            focused: false,
            pressed: false,
            read_only: false,
            disabled: false,
        }
    }

    pub(crate) const fn read_only() -> Self {
        Self {
            read_only: true,
            ..Self::enabled()
        }
    }

    pub(crate) const fn disabled() -> Self {
        Self {
            disabled: true,
            ..Self::enabled()
        }
    }
}

pub(crate) fn named_probe<'a, Message>(
    name: &'static str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    Element::new(NamedProbe {
        id: Id::new(name),
        content: content.into(),
    })
}

struct NamedProbe<'a, Message> {
    id: Id,
    content: Element<'a, Message>,
}

impl<Message> Widget<Message, crate::theme::Theme, iced::Renderer> for NamedProbe<'_, Message> {
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

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &Limits) -> Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        operation.container(Some(&self.id), layout.bounds());
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
        self.content.as_widget_mut().update(
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
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
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
            tree,
            renderer,
            theme,
            inherited_style,
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
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

struct BoundsProbe {
    id: Id,
    bounds: Option<Rectangle>,
}

impl BoundsProbe {
    fn new(name: &'static str) -> Self {
        Self {
            id: Id::new(name),
            bounds: None,
        }
    }
}

impl operation::Operation for BoundsProbe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&self.id) {
            self.bounds = Some(bounds);
        }
    }
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

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use iced::{advanced::clipboard, mouse, widget::Space};

    use super::{
        event_probe, named_probe, AnchoredGeometryFixture, EnsureVisibleFixture, FakeClock,
        FormStateFixture, PopupStateFixture, WidgetHarness,
    };

    #[test]
    fn named_probe_reports_bounds_without_child_indices() {
        let content = named_probe("form-control", Space::new().width(80).height(32));
        let mut harness = WidgetHarness::<()>::new(content, iced::Size::new(320.0, 200.0));

        assert_eq!(
            harness.named_bounds("form-control"),
            Some(iced::Rectangle::new(
                iced::Point::ORIGIN,
                iced::Size::new(80.0, 32.0),
            ))
        );
        assert_eq!(harness.named_bounds("missing"), None);
    }

    #[test]
    fn harness_preserves_event_and_clipboard_state() {
        let mut harness = WidgetHarness::new(event_probe("updated"), iced::Size::new(100.0, 80.0));
        harness.set_cursor(iced::Point::new(4.0, 4.0));
        harness.set_clipboard(clipboard::Kind::Standard, "copied");

        let result = harness.update(iced::Event::Mouse(mouse::Event::CursorEntered));

        assert_eq!(result.messages, vec!["updated"]);
        assert!(!result.captured);
        assert!(!result.layout_invalid);
        assert!(!result.input_method_enabled);
        assert_eq!(harness.clipboard(clipboard::Kind::Standard), Some("copied"));
        assert_eq!(harness.bounds().size(), iced::Size::new(24.0, 20.0));

        harness.clear_cursor();
    }

    #[test]
    fn form_state_fixture_covers_required_interaction_modes() {
        assert!(FormStateFixture::INTERACTIVE
            .iter()
            .any(|state| state.hovered));
        assert!(FormStateFixture::INTERACTIVE
            .iter()
            .any(|state| state.focused));
        assert!(FormStateFixture::INTERACTIVE
            .iter()
            .any(|state| state.pressed));
        assert!(FormStateFixture::read_only().read_only);
        assert!(FormStateFixture::disabled().disabled);
    }

    #[test]
    fn overlay_fixtures_are_deterministic_and_finite() {
        let mut clock = FakeClock::at(100);
        clock.advance(500);
        assert_eq!(clock.now_ms(), 600);

        let geometry = AnchoredGeometryFixture::new(
            iced::Rectangle::new(iced::Point::new(20.0, 20.0), iced::Size::new(80.0, 24.0)),
            iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(320.0, 200.0)),
            iced::Size::new(180.0, 120.0),
        );
        assert!(geometry.anchor.width.is_finite());
        assert!(geometry.viewport.height.is_finite());
        assert!(geometry.intrinsic_content.width.is_finite());

        let ensure_visible = EnsureVisibleFixture {
            viewport: geometry.viewport,
            target: geometry.anchor,
            current_offset: 0.0,
        };
        assert_eq!(ensure_visible.current_offset, 0.0);
        assert_eq!(ensure_visible.target, geometry.anchor);

        let state = PopupStateFixture::enabled();
        assert!(state.capable);
        assert!(!state.disabled);
        assert!(!state.open);
        assert!(!state.selected);
        assert!(!state.highlighted);
        assert!(!state.focused);
        assert!(!state.pressed);
    }
}
