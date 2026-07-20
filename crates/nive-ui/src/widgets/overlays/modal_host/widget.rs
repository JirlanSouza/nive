use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Operation as _, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use super::initial_focus::InitialFocusFn;
use super::overlay::ModalOverlay;
use crate::{
    focus::{contains_focus_target, FocusTarget, FocusTargetContext},
    Element, Renderer, Theme,
};

/// Viewport-alignment input the kernel resolves a frame position from: a
/// centered frame (Dialog) or a top-centered frame at a fixed offset
/// (CommandPalette), from the same overlay geometry.
#[derive(Clone, Copy)]
pub(crate) enum ModalAlignment {
    Centered,
    TopCentered(f32),
}

/// Private shared modal-hosting kernel. See the module docs in
/// `super::modal_host` for the full contract; this mirrors
/// `DialogHost`'s previous single-purpose implementation, generalized with
/// an [`InitialFocusFn`] hook and a [`ModalAlignment`] input so both Dialog
/// and CommandPalette consume one implementation of scrim resolution, paint
/// order, inert/event-captured base content, focus trap, initial focus,
/// single-modal-session-per-window, and inactive anchor-only return.
pub(crate) struct ModalHost<'a, Message> {
    content: Element<'a, Message>,
    modal: Option<ModalContent<'a, Message>>,
}

pub(crate) struct ModalContent<'a, Message> {
    content: Element<'a, Message>,
    on_backdrop: Option<Message>,
    on_escape: Option<Message>,
    initial_focus: InitialFocusFn<'a, Message>,
    alignment: ModalAlignment,
    id: Option<iced::widget::Id>,
}

/// Tracks which declarative modal session the kernel currently considers
/// itself in, so a rebuild with the same (or absent) session id continues
/// the session without recapturing the invoker or repeating initial focus,
/// while a changed explicit id replaces the workflow step (re-running
/// initial focus but preserving the original invoker).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum ModalSession {
    #[default]
    Closed,
    Open(Option<iced::widget::Id>),
}

#[derive(Debug, Default)]
struct ModalHostState {
    session: ModalSession,
    focus_context: FocusTargetContext,
    captured_target: Option<FocusTarget>,
    captured_target_available: bool,
    expected_target: Option<FocusTarget>,
    needs_initial_focus: bool,
}

impl<'a, Message> ModalHost<'a, Message>
where
    Message: Clone + 'a,
{
    pub(crate) fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            modal: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn modal(
        mut self,
        content: impl Into<Element<'a, Message>>,
        on_backdrop: Option<Message>,
        on_escape: Option<Message>,
        initial_focus: InitialFocusFn<'a, Message>,
        alignment: ModalAlignment,
    ) -> Self {
        self.modal = Some(ModalContent {
            content: content.into(),
            on_backdrop,
            on_escape,
            initial_focus,
            alignment,
            id: None,
        });
        self
    }

    /// Sets the current modal's stable declarative session identity.
    /// Rebuilding with the same (or no call at all, leaving it absent)
    /// identity continues the current modal session. A changed identity
    /// replaces the workflow step without recapturing the invoker.
    pub(crate) fn modal_id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        if let Some(modal) = &mut self.modal {
            modal.id = Some(id.into());
        }
        self
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for ModalHost<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ModalHostState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ModalHostState::default())
    }

    fn children(&self) -> Vec<Tree> {
        match &self.modal {
            Some(modal) => vec![Tree::new(&self.content), Tree::new(&modal.content)],
            None => vec![Tree::new(&self.content)],
        }
    }

    fn diff(&self, tree: &mut Tree) {
        match &self.modal {
            Some(modal) => {
                tree.diff_children(&[self.content.as_widget(), modal.content.as_widget()])
            }
            None => tree.diff_children(&[self.content.as_widget()]),
        }
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
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<ModalHostState>();
        state.focus_context.expose(operation, layout.bounds());
        state.captured_target_available = if let Some(captured) = state.captured_target.clone() {
            let mut contains = contains_focus_target(captured.clone());
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut operation::black_box(&mut contains),
            );
            let available = matches!(contains.finish(), operation::Outcome::Some(true));
            if available {
                // Base content is about to become externally inert below
                // (while `self.modal.is_some()`), so this is the only
                // remaining path that ever touches the invoker's liveness
                // bookkeeping for as long as the modal stays open.
                state.focus_context.keep_alive(&captured);
            }
            available
        } else {
            false
        };

        // While a modal is open, base content is externally inert: only the
        // private validity probe above may still traverse it. An ordinary
        // caller-supplied operation (focus/widget queries, `Task::widget`,
        // ...) must reach the modal subtree instead, through the overlay's
        // own `operate()`, not the base tree.
        if self.modal.is_some() {
            return;
        }

        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        if self.modal.is_some() {
            shell.capture_event();
            return;
        }

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
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.modal.is_some() {
            return mouse::Interaction::Idle;
        }

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
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<ModalHostState>();
        let next_session = match &self.modal {
            Some(modal) => ModalSession::Open(modal.id.clone()),
            None => ModalSession::Closed,
        };

        match (&state.session, &next_session) {
            (ModalSession::Closed, ModalSession::Open(_)) => {
                // Fresh open: capture the invoker and establish the modal
                // focus scope exactly once for this session.
                state.captured_target = state.focus_context.capture();
                state.captured_target_available = state.captured_target.is_some();
                state.expected_target = state.captured_target.clone();
                state.needs_initial_focus = true;
            }
            (ModalSession::Open(previous_id), ModalSession::Open(next_id))
                if previous_id != next_id =>
            {
                // Keyed workflow step: re-run initial focus for the new
                // step without recapturing or replacing the original
                // invoker, and without publishing dismissal.
                state.needs_initial_focus = true;
            }
            (ModalSession::Open(_), ModalSession::Closed) => {
                if state.captured_target_available {
                    if let Some(captured) = state.captured_target.as_ref() {
                        let _restored = state
                            .focus_context
                            .restore_anchor(captured, state.expected_target.as_ref());
                    }
                }
                state.captured_target = None;
                state.captured_target_available = false;
                state.expected_target = None;
            }
            // Same session (still closed, or reopened with the same/absent
            // identity): an ordinary declarative rerender, not a transition.
            _ => {}
        }
        state.session = next_session;

        if let Some(modal) = &mut self.modal {
            return Some(overlay::Element::new(Box::new(ModalOverlay::new(
                &mut modal.content,
                &mut tree.children[1],
                modal.on_backdrop.clone(),
                modal.on_escape.clone(),
                &modal.initial_focus,
                modal.alignment,
                &state.focus_context,
                &mut state.expected_target,
                &mut state.needs_initial_focus,
            ))));
        }

        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<ModalHost<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(host: ModalHost<'a, Message>) -> Self {
        Element::new(host)
    }
}
