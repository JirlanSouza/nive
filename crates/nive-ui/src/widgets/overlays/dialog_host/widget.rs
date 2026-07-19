use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Operation as _, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use super::initial_focus::DialogInitialFocus;
use super::overlay::DialogOverlay;
use crate::{
    focus::{contains_focus_target, FocusTarget, FocusTargetContext},
    Element, Renderer, Theme,
};

/// Modal dialog composition that preserves the prior logical navigation anchor.
///
/// While a dialog is open, base content keeps drawing but is externally
/// inert: it receives no pointer, touch, keyboard, wheel, mouse-interaction,
/// or external widget/focus operations. Only a private invoker-validity
/// probe may still inspect it. Every event the Dialog and its nested
/// overlays leave unhandled is captured, so nothing clicks or types through
/// to base content.
///
/// The backdrop resolves from the theme's `SurfaceRole::Scrim`; there is no
/// raw alpha override. Backdrop dismissal recognizes only a primary
/// (left) mouse press or touch press with a concrete position outside the
/// Dialog frame — never a secondary/middle button, and never
/// `mouse::Cursor::Unavailable`/`Levitating` (including when a nested
/// overlay masks the position). Escape recognizes only a non-repeated
/// keypress, after Dialog-owned nested overlays and descendants have had
/// first priority. An absent route is still captured (never clicks/types
/// through) but publishes no message; programmatic removal of the dialog
/// never publishes a dismissal message either.
///
/// When a dialog closes, a still-valid target captured before opening is
/// restored as an inactive anchor: not actively or visibly focused, and
/// never overwriting a newer programmatic target set outside the dialog.
/// [`dialog_id`](Self::dialog_id) names the current declarative session so a
/// rebuild with the same (or absent) id continues it without recapturing or
/// re-resolving initial focus, while a changed id replaces the workflow step
/// and re-runs initial focus without recapturing the invoker or publishing
/// dismissal.
///
/// Canonical composition draws base content, then the Scrim, then the
/// Dialog frame, then any Dialog-owned nested overlay (Select/Popover/Menu)
/// on top — shadow prominence is independent from this event/paint order. A
/// window hosts at most one canonical modal session; a later
/// [`dialog`](Self::dialog) call replaces rather than stacks. Manually
/// nesting more than one `DialogHost` is unsupported and not detected.
///
/// Low-level backdrop-alpha customization and hosting internals are
/// intentionally unavailable to application code:
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::DialogHost;
/// let host: DialogHost<'_, ()> = DialogHost::new(iced::widget::text("Base"));
/// let _ = host.backdrop_alpha(0.5);
/// ```
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::dialog_host::overlay::DialogOverlay;
/// ```
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::dialog_host::initial_focus::FocusFirstSafeTarget;
/// ```
pub struct DialogHost<'a, Message> {
    content: Element<'a, Message>,
    dialog: Option<DialogContent<'a, Message>>,
}

struct DialogContent<'a, Message> {
    content: Element<'a, Message>,
    on_backdrop: Option<Message>,
    on_escape: Option<Message>,
    initial_focus: DialogInitialFocus,
    id: Option<iced::widget::Id>,
}

/// Tracks which declarative modal session `DialogHost` currently considers
/// itself in, so a rebuild with the same (or absent) `DialogRequest` id
/// continues the session without recapturing the invoker or repeating
/// initial focus, while a changed explicit id replaces the workflow step
/// (re-running initial focus but preserving the original invoker).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum DialogSession {
    #[default]
    Closed,
    Open(Option<iced::widget::Id>),
}

#[derive(Debug, Default)]
struct DialogHostState {
    session: DialogSession,
    focus_context: FocusTargetContext,
    captured_target: Option<FocusTarget>,
    captured_target_available: bool,
    expected_target: Option<FocusTarget>,
    needs_initial_focus: bool,
}

impl<'a, Message> DialogHost<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            dialog: None,
        }
    }

    pub fn dialog(
        mut self,
        content: impl Into<Element<'a, Message>>,
        on_backdrop: Option<Message>,
        on_escape: Option<Message>,
        initial_focus: DialogInitialFocus,
    ) -> Self {
        self.dialog = Some(DialogContent {
            content: content.into(),
            on_backdrop,
            on_escape,
            initial_focus,
            id: None,
        });
        self
    }

    /// Sets the current dialog's stable declarative session identity.
    /// Rebuilding with the same (or no call at all, leaving it absent)
    /// identity continues the current modal session. A changed identity
    /// replaces the workflow step without recapturing the invoker.
    pub fn dialog_id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        if let Some(dialog) = &mut self.dialog {
            dialog.id = Some(id.into());
        }
        self
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for DialogHost<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<DialogHostState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(DialogHostState::default())
    }

    fn children(&self) -> Vec<Tree> {
        match &self.dialog {
            Some(dialog) => vec![Tree::new(&self.content), Tree::new(&dialog.content)],
            None => vec![Tree::new(&self.content)],
        }
    }

    fn diff(&self, tree: &mut Tree) {
        match &self.dialog {
            Some(dialog) => {
                tree.diff_children(&[self.content.as_widget(), dialog.content.as_widget()])
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
        let state = tree.state.downcast_mut::<DialogHostState>();
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
                // (while `self.dialog.is_some()`), so this is the only
                // remaining path that ever touches the invoker's liveness
                // bookkeeping for as long as the modal stays open.
                state.focus_context.keep_alive(&captured);
            }
            available
        } else {
            false
        };

        // While a dialog is open, base content is externally inert: only the
        // private validity probe above may still traverse it. An ordinary
        // caller-supplied operation (focus/widget queries, `Task::widget`,
        // ...) must reach the Dialog subtree instead, through the overlay's
        // own `operate()`, not the base tree.
        if self.dialog.is_some() {
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
        if self.dialog.is_some() {
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
        if self.dialog.is_some() {
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
        let state = tree.state.downcast_mut::<DialogHostState>();
        let next_session = match &self.dialog {
            Some(dialog) => DialogSession::Open(dialog.id.clone()),
            None => DialogSession::Closed,
        };

        match (&state.session, &next_session) {
            (DialogSession::Closed, DialogSession::Open(_)) => {
                // Fresh open: capture the invoker and establish the modal
                // focus scope exactly once for this session.
                state.captured_target = state.focus_context.capture();
                state.captured_target_available = state.captured_target.is_some();
                state.expected_target = state.captured_target.clone();
                state.needs_initial_focus = true;
            }
            (DialogSession::Open(previous_id), DialogSession::Open(next_id))
                if previous_id != next_id =>
            {
                // Keyed workflow step: re-run initial focus for the new
                // step without recapturing or replacing the original
                // invoker, and without publishing dismissal.
                state.needs_initial_focus = true;
            }
            (DialogSession::Open(_), DialogSession::Closed) => {
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

        if let Some(dialog) = &mut self.dialog {
            return Some(overlay::Element::new(Box::new(DialogOverlay::new(
                &mut dialog.content,
                &mut tree.children[1],
                dialog.on_backdrop.clone(),
                dialog.on_escape.clone(),
                dialog.initial_focus.clone(),
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

impl<'a, Message> From<DialogHost<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(host: DialogHost<'a, Message>) -> Self {
        Element::new(host)
    }
}
