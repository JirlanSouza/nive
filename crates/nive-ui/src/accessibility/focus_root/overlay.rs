use iced::advanced::{
    layout, mouse, overlay, renderer, widget::operation, Clipboard, Layout, Shell,
};
use iced::{Event, Size};

use super::{observe_event_after, observe_event_before};
use crate::{
    accessibility::focus_root::operation::{BindingPass, FocusOperation},
    focus::{lock_coordinator, SharedFocusCoordinator},
    Renderer, Theme,
};

pub(super) struct FocusOverlay<'a, Message> {
    inner: overlay::Element<'a, Message, Theme, Renderer>,
    coordinator: SharedFocusCoordinator,
    depth: usize,
}

impl<'a, Message> FocusOverlay<'a, Message>
where
    Message: 'a,
{
    pub(super) fn wrap(
        inner: overlay::Element<'a, Message, Theme, Renderer>,
        coordinator: SharedFocusCoordinator,
        depth: usize,
    ) -> overlay::Element<'a, Message, Theme, Renderer> {
        overlay::Element::new(Box::new(Self {
            inner,
            coordinator,
            depth,
        }))
    }
}

impl<Message> overlay::Overlay<Message, Theme, Renderer> for FocusOverlay<'_, Message> {
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
        let generation = lock_coordinator(&self.coordinator).ensure_liveness();
        self.inner.as_overlay_mut().operate(
            layout,
            renderer,
            &mut FocusOperation::new(operation, &self.coordinator, generation),
        );
        let has_nested = self
            .inner
            .as_overlay_mut()
            .overlay(layout, renderer)
            .is_some();
        if !has_nested {
            lock_coordinator(&self.coordinator).finish_liveness(generation);
        }
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let generation = lock_coordinator(&self.coordinator).ensure_liveness();
        self.inner.as_overlay_mut().operate(
            layout,
            renderer,
            &mut FocusOperation::new(&mut BindingPass, &self.coordinator, generation),
        );
        observe_event_before(&self.coordinator, event);
        self.inner
            .as_overlay_mut()
            .update(event, layout, cursor, renderer, clipboard, shell);
        if self.depth == 0 || shell.is_event_captured() {
            observe_event_after(&self.coordinator, event);
        }
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
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        let coordinator = self.coordinator.clone();
        let depth = self.depth + 1;
        self.inner
            .as_overlay_mut()
            .overlay(layout, renderer)
            .map(|overlay| FocusOverlay::wrap(overlay, coordinator, depth))
    }

    fn index(&self) -> f32 {
        self.inner.as_overlay().index()
    }
}

#[cfg(test)]
mod overlay_tests;
