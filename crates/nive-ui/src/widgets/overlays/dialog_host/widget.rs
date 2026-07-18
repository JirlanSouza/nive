use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Operation as _, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use super::overlay::DialogOverlay;
use crate::{
    focus::{contains_focus_target, FocusTarget, FocusTargetContext},
    Element, Renderer, Theme,
};

const DEFAULT_BACKDROP_ALPHA: f32 = 0.62;

/// Modal dialog composition that preserves the prior logical navigation anchor.
///
/// When a dialog closes, a still-valid target captured before opening is
/// restored as an inactive anchor. The target does not receive active focus or
/// a visible focus indication, and a newer programmatic target is never
/// overwritten.
pub struct DialogHost<'a, Message> {
    content: Element<'a, Message>,
    dialog: Option<DialogContent<'a, Message>>,
    backdrop_alpha: f32,
}

struct DialogContent<'a, Message> {
    content: Element<'a, Message>,
    on_backdrop: Option<Message>,
    on_escape: Option<Message>,
}

#[derive(Debug, Default)]
struct DialogHostState {
    was_open: bool,
    focus_context: FocusTargetContext,
    captured_target: Option<FocusTarget>,
    captured_target_available: bool,
    expected_target: Option<FocusTarget>,
}

impl<'a, Message> DialogHost<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            dialog: None,
            backdrop_alpha: DEFAULT_BACKDROP_ALPHA,
        }
    }

    pub fn dialog(
        mut self,
        content: impl Into<Element<'a, Message>>,
        on_backdrop: Option<Message>,
        on_escape: Option<Message>,
    ) -> Self {
        self.dialog = Some(DialogContent {
            content: content.into(),
            on_backdrop,
            on_escape,
        });
        self
    }

    pub fn backdrop_alpha(mut self, alpha: f32) -> Self {
        self.backdrop_alpha = alpha;
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
            let mut contains = contains_focus_target(captured);
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut operation::black_box(&mut contains),
            );
            matches!(contains.finish(), operation::Outcome::Some(true))
        } else {
            false
        };
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
        let is_open = self.dialog.is_some();
        if is_open && !state.was_open {
            state.captured_target = state.focus_context.capture();
            state.captured_target_available = state.captured_target.is_some();
            state.expected_target = state.captured_target.clone();
        } else if !is_open && state.was_open {
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
        state.was_open = is_open;

        if let Some(dialog) = &mut self.dialog {
            return Some(overlay::Element::new(Box::new(DialogOverlay::new(
                &mut dialog.content,
                &mut tree.children[1],
                self.backdrop_alpha,
                dialog.on_backdrop.clone(),
                dialog.on_escape.clone(),
                &state.focus_context,
                &mut state.expected_target,
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
