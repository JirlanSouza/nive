use std::time::Duration;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use super::widget::{request_redraw_after_visibility_change, TooltipState};
use crate::Element;

const WARM_DELAY: Duration = Duration::from_millis(100);
const COLD_DELAY: Duration = Duration::from_millis(500);
const WARM_WINDOW: Duration = Duration::from_millis(600);

/// Layout-neutral timing boundary for neighboring [`super::Tooltip`] widgets.
///
/// Each scope owns an independent persistent session in its Widget tree. A
/// different neighbor may reveal after 100ms for 600ms after a Tooltip was
/// actually shown; the same neighbor always waits 500ms. A candidate that
/// never became visible does not warm the session. Pointer intent wins over a
/// retained focus candidate, and at most one descendant Tooltip is shown.
/// Nested scopes are independent timing boundaries, as are separate widget
/// trees and windows.
///
/// Timing state and the operations used to arbitrate descendants remain
/// private implementation details:
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::tooltip::scope::{
///     ApplyWinner, CollectCandidates, ScopeState, WARM_DELAY, WARM_WINDOW,
/// };
/// ```
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::tooltip::widget::{TooltipState, TooltipWidget};
/// ```
pub struct TooltipScope<'a, Message> {
    content: Element<'a, Message>,
    now_override: Option<iced::time::Instant>,
}

impl<'a, Message> TooltipScope<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            now_override: None,
        }
    }

    #[cfg(test)]
    pub(super) fn at(mut self, now: iced::time::Instant) -> Self {
        self.now_override = Some(now);
        self
    }
}

impl<'a, Message> From<TooltipScope<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(scope: TooltipScope<'a, Message>) -> Self {
        Element::new(scope)
    }
}

#[derive(Debug, Default)]
struct ScopeState {
    next_owner_key: u64,
    active: Option<u64>,
    pending: Option<(u64, iced::time::Instant)>,
    last_shown: Option<(u64, iced::time::Instant)>,
    block_private_traversal: bool,
}

impl<Message> Widget<Message, crate::theme::Theme, iced::Renderer> for TooltipScope<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ScopeState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ScopeState::default())
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
        renderer: &iced::Renderer,
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
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<ScopeState>();
        operation.custom(None, layout.bounds(), state);
        let blocked = std::mem::take(&mut state.block_private_traversal);
        if !blocked {
            operation.traverse(&mut |operation| {
                self.content.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    operation,
                );
            });
        }
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
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = tree.state.downcast_mut::<ScopeState>();
        let mut next_owner_key = state.next_owner_key;
        let mut collect = CollectCandidates {
            candidates: Vec::new(),
            next_owner_key: &mut next_owner_key,
        };
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut collect);
        let candidates = std::mem::take(&mut collect.candidates);
        drop(collect);
        state.next_owner_key = next_owner_key;

        let now = self.now_override.unwrap_or_else(|| event_now(event));
        let deadline = resolve(state, &candidates, now);
        if let Some(deadline) = deadline {
            shell.request_redraw_at(deadline);
        }

        let mut apply = ApplyWinner {
            winner: state.active,
            changed: false,
        };
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut apply);
        if apply.changed {
            shell.invalidate_layout();
            request_redraw_after_visibility_change(event, shell);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
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
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    key: u64,
    hovered: bool,
    focused: bool,
}

struct CollectCandidates<'a> {
    candidates: Vec<Candidate>,
    next_owner_key: &'a mut u64,
}

impl operation::Operation for CollectCandidates<'_> {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
        operate(self);
    }

    fn custom(
        &mut self,
        _id: Option<&iced::advanced::widget::Id>,
        _bounds: Rectangle,
        state: &mut dyn std::any::Any,
    ) {
        if let Some(scope) = state.downcast_mut::<ScopeState>() {
            scope.block_private_traversal = true;
        } else if let Some(tooltip) = state.downcast_mut::<TooltipState>() {
            let key = *tooltip.owner_key.get_or_insert_with(|| {
                let key = *self.next_owner_key;
                *self.next_owner_key = self.next_owner_key.saturating_add(1);
                key
            });
            self.candidates.push(Candidate {
                key,
                hovered: tooltip.hovered && !tooltip.escape_suppressed,
                focused: tooltip.focused && !tooltip.escape_suppressed,
            });
        }
    }
}

struct ApplyWinner {
    winner: Option<u64>,
    changed: bool,
}

impl operation::Operation for ApplyWinner {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
        operate(self);
    }

    fn custom(
        &mut self,
        _id: Option<&iced::advanced::widget::Id>,
        _bounds: Rectangle,
        state: &mut dyn std::any::Any,
    ) {
        if let Some(scope) = state.downcast_mut::<ScopeState>() {
            scope.block_private_traversal = true;
        } else if let Some(tooltip) = state.downcast_mut::<TooltipState>() {
            let visible = tooltip.owner_key == self.winner;
            self.changed |= tooltip.visible != visible;
            tooltip.visible = visible;
        }
    }
}

fn resolve(
    state: &mut ScopeState,
    candidates: &[Candidate],
    now: iced::time::Instant,
) -> Option<iced::time::Instant> {
    let winner = candidates
        .iter()
        .find(|candidate| candidate.hovered)
        .or_else(|| candidates.iter().find(|candidate| candidate.focused))
        .map(|candidate| candidate.key);

    let Some(winner) = winner else {
        if let Some(active) = state.active.take() {
            state.last_shown = Some((active, now));
        }
        state.pending = None;
        return None;
    };

    if state.active == Some(winner) {
        return None;
    }

    let previous_visible = state.active.take();
    if let Some(previous) = previous_visible {
        state.last_shown = Some((previous, now));
    }
    if state.pending.is_none_or(|(key, _)| key != winner) {
        state.pending = Some((winner, now));
    }

    let warm_neighbor = previous_visible.is_some_and(|key| key != winner)
        || state.last_shown.is_some_and(|(key, closed_at)| {
            key != winner && now.saturating_duration_since(closed_at) <= WARM_WINDOW
        });
    let delay = if warm_neighbor {
        WARM_DELAY
    } else {
        COLD_DELAY
    };
    let deadline = state
        .pending
        .filter(|(key, _)| *key == winner)
        .map(|(_, entered_at)| entered_at + delay)?;

    if now >= deadline {
        state.active = Some(winner);
        state.pending = None;
        None
    } else {
        Some(deadline)
    }
}

fn event_now(event: &Event) -> iced::time::Instant {
    match event {
        Event::Window(iced::window::Event::RedrawRequested(now)) => *now,
        _ => iced::time::Instant::now(),
    }
}
