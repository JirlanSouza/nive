use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use iced::{
    advanced::{
        clipboard, layout, mouse, overlay, renderer,
        widget::{operation, Id, Operation as _},
        Clipboard, Layout, Shell,
    },
    Event, Point, Rectangle, Size,
};

use crate::{Renderer, Theme};

#[derive(Debug)]
struct ManagedState {
    level: u8,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Calls {
    bound: Vec<u8>,
    layouts: Vec<u8>,
    draws: Vec<u8>,
    updates: Vec<u8>,
}

struct BindingOperation<'a> {
    inner: &'a mut dyn operation::Operation,
    calls: Arc<Mutex<Calls>>,
}

impl operation::Operation for BindingOperation<'_> {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
        self.inner.traverse(&mut |inner| {
            operate(&mut BindingOperation {
                inner,
                calls: Arc::clone(&self.calls),
            });
        });
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        self.inner.container(id, bounds);
    }

    fn custom(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn Any) {
        if let Some(managed) = state.downcast_ref::<ManagedState>() {
            self.calls
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .bound
                .push(managed.level);
        } else {
            self.inner.custom(id, bounds, state);
        }
    }
}

struct RootOverlay<'a> {
    inner: overlay::Element<'a, u8, Theme, Renderer>,
    calls: Arc<Mutex<Calls>>,
}

impl<'a> RootOverlay<'a> {
    fn wrap(
        inner: overlay::Element<'a, u8, Theme, Renderer>,
        calls: Arc<Mutex<Calls>>,
    ) -> overlay::Element<'a, u8, Theme, Renderer> {
        overlay::Element::new(Box::new(Self { inner, calls }))
    }
}

impl overlay::Overlay<u8, Theme, Renderer> for RootOverlay<'_> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        self.inner.as_overlay_mut().layout(renderer, bounds)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.inner
            .as_overlay()
            .draw(renderer, theme, style, layout, cursor);
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        self.inner.as_overlay_mut().operate(
            layout,
            renderer,
            &mut BindingOperation {
                inner: operation,
                calls: Arc::clone(&self.calls),
            },
        );
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, u8>,
    ) {
        self.inner
            .as_overlay_mut()
            .update(event, layout, cursor, renderer, clipboard, shell);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.inner
            .as_overlay()
            .mouse_interaction(layout, cursor, renderer)
    }

    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'a>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'a, u8, Theme, Renderer>> {
        let calls = Arc::clone(&self.calls);
        self.inner
            .as_overlay_mut()
            .overlay(layout, renderer)
            .map(move |inner| overlay::Element::new(Box::new(RootOverlay { inner, calls })))
    }

    fn index(&self) -> f32 {
        self.inner.as_overlay().index()
    }
}

struct ProbeOverlay {
    level: u8,
    size: Size,
    index: f32,
    calls: Arc<Mutex<Calls>>,
    state: ManagedState,
    nested: Option<ProbeNested>,
}

impl overlay::Overlay<u8, Theme, Renderer> for ProbeOverlay {
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .layouts
            .push(self.level);
        layout::Node::new(self.size)
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .draws
            .push(self.level);
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        operation.custom(None, layout.bounds(), &mut self.state);
        operation.container(None, layout.bounds());
    }

    fn update(
        &mut self,
        _event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, u8>,
    ) {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .updates
            .push(self.level);
        shell.publish(self.level);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }

    fn overlay<'a>(
        &'a mut self,
        _layout: Layout<'a>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'a, u8, Theme, Renderer>> {
        self.nested
            .as_mut()
            .map(|nested| overlay::Element::new(Box::new(ProbeNestedRef { inner: nested })))
    }

    fn index(&self) -> f32 {
        self.index
    }
}

struct ProbeNested {
    level: u8,
    size: Size,
    index: f32,
    calls: Arc<Mutex<Calls>>,
    state: ManagedState,
}

struct ProbeNestedRef<'a> {
    inner: &'a mut ProbeNested,
}

impl overlay::Overlay<u8, Theme, Renderer> for ProbeNestedRef<'_> {
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        self.inner
            .calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .layouts
            .push(self.inner.level);
        layout::Node::new(self.inner.size)
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        self.inner
            .calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .draws
            .push(self.inner.level);
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        operation.custom(None, layout.bounds(), &mut self.inner.state);
        operation.container(None, layout.bounds());
    }

    fn update(
        &mut self,
        _event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, u8>,
    ) {
        self.inner
            .calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .updates
            .push(self.inner.level);
        shell.publish(self.inner.level);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }

    fn index(&self) -> f32 {
        self.inner.index
    }
}

mod tests;
