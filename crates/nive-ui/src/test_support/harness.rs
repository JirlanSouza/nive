use iced::{
    advanced::{
        clipboard,
        layout::{Layout, Limits, Node},
        mouse, overlay, renderer,
        widget::{operation, Id, Operation as _, Tree},
        Clipboard, Shell,
    },
    event, Event, Font, Pixels, Point, Rectangle, Size, Vector,
};

use super::focus::{ManagedFocusProbe, ManagedFocusSnapshot};
use super::{BoundsProbe, MemoryClipboard, UpdateResult, WidgetHarness};
use crate::Element;

pub(crate) fn renderer() -> iced::Renderer {
    iced_renderer::fallback::Renderer::Secondary(iced_tiny_skia::Renderer::new(
        Font::default(),
        Pixels(14.0),
    ))
}

/// Renders what the software backend has queued into a pixel buffer.
pub(crate) fn rasterize(renderer: &mut iced::Renderer, size: Size) -> tiny_skia::Pixmap {
    let width = size.width as u32;
    let height = size.height as u32;
    let mut pixmap = tiny_skia::Pixmap::new(width, height).expect("pixmap");
    let mut mask = tiny_skia::Mask::new(width, height).expect("mask");
    let viewport = iced_graphics::Viewport::with_physical_size(Size::new(width, height), 1.0);

    let iced_renderer::fallback::Renderer::Secondary(renderer) = renderer else {
        panic!("test renderer is the software backend");
    };
    renderer.draw(
        &mut pixmap.as_mut(),
        &mut mask,
        &viewport,
        &[Rectangle::new(Point::ORIGIN, size)],
        iced::Color::BLACK,
    );

    pixmap
}

pub(crate) fn pixel(pixmap: &tiny_skia::Pixmap, point: Point) -> [u8; 4] {
    let index = (point.y as u32 * pixmap.width() + point.x as u32) as usize * 4;
    let pixels = pixmap.data();

    [
        pixels[index],
        pixels[index + 1],
        pixels[index + 2],
        pixels[index + 3],
    ]
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

    /// Mirrors the base-tree visit `iced_runtime::UserInterface::operate()`
    /// always performs before touching an overlay, so a subsequent
    /// overlay-only `operate()` call shares its liveness generation with
    /// base content instead of running in isolation. Without this, base
    /// content untouched by an overlay-scoped probe goes stale and is
    /// pruned by the very next `finish_liveness()`, which would wrongly
    /// invalidate an outer invoker's managed focus target while a modal
    /// remains open.
    pub(super) fn prime_root_liveness(&mut self) {
        self.operate(&mut operation::black_box(&mut operation::focusable::count()));
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

    /// One rasterization of the whole surface, for callers sampling many pixels.
    ///
    /// Prefer this over [`pixel`] in a loop, which would redraw and rasterize
    /// the entire surface per sample.
    pub(crate) fn pixmap(&mut self) -> tiny_skia::Pixmap {
        self.draw();

        rasterize(&mut self.renderer, self.maximum)
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
        self.overlay_bounds_within(Rectangle::new(Point::ORIGIN, self.maximum))
    }

    /// Overlay bounds as seen from a host that clips its content, so an anchor
    /// scrolled out of view can be exercised.
    pub(crate) fn overlay_bounds_within(&mut self, viewport: Rectangle) -> Option<Rectangle> {
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
        collect_nested_overlay_bounds(Layout::new(&node), self.maximum, &mut bounds);
        bounds
    }

    pub(crate) fn focused_overlay_count(&mut self) -> Option<operation::focusable::Count> {
        self.prime_root_liveness();
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

    pub(crate) fn focused_overlay_ids(&mut self) -> Vec<Id> {
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

        self.prime_root_liveness();
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
        let mut focused = FocusedIds(Vec::new());
        overlay
            .as_overlay_mut()
            .operate(Layout::new(&node), &self.renderer, &mut focused);
        focused.0
    }

    /// Same as [`Self::focused_overlay_ids`], but walks the complete
    /// [`overlay::Nested`] chain instead of only the first overlay level —
    /// use this to inspect focus inside a Dialog-owned nested overlay
    /// (Select/Popover/Menu) rather than the Dialog itself.
    pub(crate) fn focused_nested_overlay_ids(&mut self) -> Vec<Id> {
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

        self.prime_root_liveness();
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
        let mut focused = FocusedIds(Vec::new());
        nested.operate(Layout::new(&node), &self.renderer, &mut focused);
        focused.0
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
        self.focus_overlay_direction(crate::focus_trap::FocusDirection::Next)
    }

    pub(crate) fn focus_overlay_previous(&mut self) -> bool {
        self.focus_overlay_direction(crate::focus_trap::FocusDirection::Previous)
    }

    pub(super) fn focus_overlay_direction(
        &mut self,
        direction: crate::focus_trap::FocusDirection,
    ) -> bool {
        let mut operated = false;
        direction.operate(|operation| {
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

fn collect_nested_overlay_bounds(layout: Layout<'_>, maximum: Size, bounds: &mut Vec<Rectangle>) {
    let mut children = layout.children();
    let Some(current) = children.next() else {
        return;
    };
    bounds.push(overlay_content_layout_bounds(current, maximum));
    if let Some(nested) = children.next() {
        collect_nested_overlay_bounds(nested, maximum, bounds);
    }
}

/// Mirrors [`overlay_content_bounds`]'s "skip the full-viewport passthrough"
/// rule for a `Layout` rather than a `Node`: a level whose own bounds exactly
/// cover the maximum viewport is a wrapper with no visual footprint of its
/// own (a scrim, a `Group`), so the real content is its last child.
fn overlay_content_layout_bounds(layout: Layout<'_>, maximum: Size) -> Rectangle {
    let bounds = layout.bounds();
    if bounds.position() == Point::ORIGIN && bounds.size() == maximum {
        layout.children().last().map_or(bounds, |child| {
            overlay_content_layout_bounds(child, maximum)
        })
    } else {
        bounds
    }
}

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
