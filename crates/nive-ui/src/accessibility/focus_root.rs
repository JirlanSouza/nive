mod operation;
mod overlay;

use iced::{
    advanced::{
        layout, mouse, renderer,
        widget::{tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    window, Event, Length, Rectangle, Size, Vector,
};

use crate::{Element, Renderer, Theme};

use self::{
    operation::{BindingPass, FocusOperation},
    overlay::FocusOverlay,
};
use crate::focus::{lock_coordinator, FocusCoordinator, InputOrigin, SharedFocusCoordinator};

/// Layout- and paint-neutral logical-focus boundary for one widget tree.
///
/// Standalone `nive-ui` applications opt into managed logical navigation by
/// wrapping their final window content explicitly:
///
/// ```
/// use nive_ui::{accessibility::FocusRoot, Element};
///
/// let content: Element<'_, ()> = iced::widget::text("Content").into();
/// let rooted: Element<'_, ()> = FocusRoot::new(content).into();
/// ```
///
/// The advanced boundary is intentionally absent from the ordinary prelude:
///
/// ```compile_fail
/// use nive_ui::prelude::FocusRoot;
/// ```
pub struct FocusRoot<'a, Message> {
    content: Element<'a, Message>,
    on_modal_change: Option<Box<dyn Fn(bool) -> Message + 'a>>,
}

impl<'a, Message> FocusRoot<'a, Message> {
    /// Wraps `content` in one independent managed-focus domain.
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            on_modal_change: None,
        }
    }

    /// Publishes a message whenever this root's modal activity changes.
    ///
    /// The root aggregates every open modal session below it — `Dialog`,
    /// `CommandPalette`, or any other consumer of the shared modal kernel —
    /// so hosts opt in to nothing and cannot forget to report. Use it to
    /// suspend ambient timed behavior (notification expiry, polling) while
    /// the user is held in a modal step.
    pub fn on_modal_change(mut self, on_modal_change: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_modal_change = Some(Box::new(on_modal_change));
        self
    }
}

impl<'a, Message> From<FocusRoot<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(root: FocusRoot<'a, Message>) -> Self {
        Element::new(root)
    }
}

#[derive(Debug)]
pub(crate) struct FocusRootState {
    pub(crate) coordinator: SharedFocusCoordinator,
    pub(super) nested_root_detected: bool,
}

impl Default for FocusRootState {
    fn default() -> Self {
        Self {
            coordinator: FocusCoordinator::shared(),
            nested_root_detected: false,
        }
    }
}

impl<Message> Widget<Message, Theme, Renderer> for FocusRoot<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<FocusRootState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(FocusRootState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
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
        renderer: &Renderer,
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
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<FocusRootState>();
        operation.custom(None, layout.bounds(), state);
        let generation = lock_coordinator(&state.coordinator).begin_liveness();
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut FocusOperation::new(operation, &state.coordinator, generation),
            );
        });
        let viewport = layout.bounds();
        let has_overlay = self
            .content
            .as_widget_mut()
            .overlay(
                &mut tree.children[0],
                layout,
                renderer,
                &viewport,
                Vector::ZERO,
            )
            .is_some();
        if !has_overlay {
            lock_coordinator(&state.coordinator).finish_liveness(generation);
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<FocusRootState>();
        let generation = lock_coordinator(&state.coordinator).ensure_liveness();
        lock_coordinator(&state.coordinator).begin_modal_pass();
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout,
            renderer,
            &mut FocusOperation::new(&mut BindingPass, &state.coordinator, generation),
        );
        let changed = lock_coordinator(&state.coordinator).finish_modal_pass();
        if changed {
            if let Some(on_modal_change) = &self.on_modal_change {
                let active = lock_coordinator(&state.coordinator).modal_active();
                shell.publish(on_modal_change(active));
            }
        }
        observe_event_before(&state.coordinator, event);
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
        observe_event_after(&state.coordinator, event);
        lock_coordinator(&state.coordinator).finish_liveness(generation);

        // The modal-active pass above only sees whatever `self.content`
        // already is *this* event — it can't detect a modal that a message
        // published *by this same event* is about to open, since that only
        // takes effect on the next `view()` rebuild. Nothing else re-checks
        // modal activity outside of an `Event`-driven `update()` pass, so
        // without this, a modal opened via a plain click with no further
        // input afterward would never get detected and its toasts would
        // never pause. One extra redraw per real input event catches the
        // rebuilt tree up within a frame; excluding `RedrawRequested` itself
        // keeps this from chaining into a self-sustaining redraw loop.
        //
        // The other half of that contract lives in the modal kernel: this
        // whole method only runs for a follow-up frame because
        // `ModalHost`/`ModalOverlay` deliberately leave `RedrawRequested`
        // uncaptured while a session is open. An open modal makes base content
        // inert to input, and `UserInterface` skips the base tree entirely
        // once an overlay captures — so a kernel that captured the frame
        // signal too would starve the pass above of the very event it needs,
        // silently pinning modal activity to whatever it was before the modal
        // opened.
        if !matches!(event, Event::Window(window::Event::RedrawRequested(_))) {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
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
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'a, Message, Theme, Renderer>> {
        let coordinator = tree
            .state
            .downcast_ref::<FocusRootState>()
            .coordinator
            .clone();
        self.content
            .as_widget_mut()
            .overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
            .map(|overlay| FocusOverlay::wrap(overlay, coordinator, 0))
    }
}

pub(super) fn observe_event_before(coordinator: &SharedFocusCoordinator, event: &Event) {
    let mut coordinator = lock_coordinator(coordinator);
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(iced::touch::Event::FingerPressed { .. }) => {
            coordinator.begin_pointer_transaction();
        }
        Event::Keyboard(_) => {
            coordinator.record_origin(InputOrigin::Keyboard);
        }
        Event::Window(iced::window::Event::Unfocused) => {
            coordinator.deactivate_window();
        }
        Event::Window(iced::window::Event::Focused) => {
            coordinator.activate_window();
        }
        _ => {}
    }
}

pub(super) fn observe_event_after(coordinator: &SharedFocusCoordinator, event: &Event) {
    if matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(iced::touch::Event::FingerPressed { .. })
    ) {
        lock_coordinator(coordinator).finish_pointer_transaction();
    }
}

#[cfg(test)]
mod focus_root_tests;

#[cfg(test)]
mod navigation_tests;
