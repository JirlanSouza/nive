use iced::{advanced::widget::operation, Point, Rectangle};

use super::{
    FocusMovement, TabBar, TabBarFocus, TabBarState, TabCloseRequest, TabCloseTrigger,
    TabDropTarget, TabRegion,
};
use crate::interaction::{
    ContextInvocation, ContextPosition, ContextRequest, ContextTarget, DropDecision,
    LinearInsertion, Orientation, SelectionSnapshot, TransferOperation,
};

impl<'a, Id, Message> TabBar<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    pub(super) fn context_request(
        &self,
        region: TabRegion,
        position: Point,
    ) -> Option<ContextRequest<Id>> {
        match region {
            TabRegion::Tab(display_index) => {
                let displayed = self.displayed_tabs();
                let displayed = displayed.get(display_index)?;
                let tab = displayed.item;
                Some(ContextRequest {
                    target: ContextTarget::Item(tab.id.clone()),
                    selection: SelectionSnapshot {
                        selected: vec![tab.id.clone()],
                        focused: Some(tab.id.clone()),
                        anchor: Some(tab.id.clone()),
                    },
                    position: ContextPosition::Pointer(position),
                    invocation: ContextInvocation::SecondaryClick,
                })
            }
            TabRegion::Empty => Some(ContextRequest {
                target: ContextTarget::Empty,
                selection: SelectionSnapshot::default(),
                position: ContextPosition::Pointer(position),
                invocation: ContextInvocation::SecondaryClick,
            }),
            _ => None,
        }
    }

    pub(super) fn close_request(&self, region: TabRegion) -> Option<TabCloseRequest<Id>> {
        let TabRegion::Tab(display_index) = region else {
            return None;
        };
        let displayed = self.displayed_tabs();
        let displayed = displayed.get(display_index)?;
        let tab = displayed.item;

        (tab.closable && self.on_close_request.is_some()).then(|| TabCloseRequest {
            id: tab.id.clone(),
            trigger: TabCloseTrigger::MiddleClick,
        })
    }

    pub(super) fn close_button_request(&self, close_index: usize) -> Option<TabCloseRequest<Id>> {
        let tab = self
            .displayed_tabs()
            .into_iter()
            .filter(|displayed| {
                displayed.item.closable
                    && !displayed.item.disabled
                    && self.on_close_request.is_some()
            })
            .nth(close_index)?
            .item;

        Some(TabCloseRequest {
            id: tab.id.clone(),
            trigger: TabCloseTrigger::CloseButton,
        })
    }

    pub(super) fn enabled_focus_order(&self) -> Vec<Id> {
        self.displayed_tabs()
            .into_iter()
            .filter(|displayed| !displayed.item.disabled)
            .map(|displayed| displayed.item.id.clone())
            .collect()
    }

    pub(super) fn reconcile_focus(&self, state: &mut TabBarState<Id>) {
        let enabled = self.enabled_focus_order();
        let focused_is_valid = state
            .focused_id
            .as_ref()
            .is_some_and(|focused| enabled.contains(focused));

        if !focused_is_valid {
            state.focused_id = self
                .active
                .as_ref()
                .filter(|active| enabled.contains(active))
                .cloned()
                .or_else(|| {
                    state.focused_id.as_ref().and_then(|removed| {
                        let old_index = state
                            .previous_focus_order
                            .iter()
                            .position(|id| id == removed)?;
                        enabled
                            .get(old_index.min(enabled.len().saturating_sub(1)))
                            .cloned()
                    })
                })
                .or_else(|| enabled.first().cloned());
        }

        state.previous_focus_order = enabled;
    }

    pub(super) fn move_focus(&self, state: &mut TabBarState<Id>, movement: FocusMovement) {
        let enabled = self.enabled_focus_order();
        if enabled.is_empty() {
            state.focused_id = None;
            return;
        }
        let current = state
            .focused_id
            .as_ref()
            .and_then(|focused| enabled.iter().position(|id| id == focused))
            .unwrap_or(0);
        let target = match movement {
            FocusMovement::Previous => current.saturating_sub(1),
            FocusMovement::Next => (current + 1).min(enabled.len() - 1),
            FocusMovement::First => 0,
            FocusMovement::Last => enabled.len() - 1,
        };
        state.focused_id = Some(enabled[target].clone());
    }

    /// Probe a per-segment reorder decision for the dragged tab id at `pointer`.
    pub(super) fn reorder_decision(
        &self,
        dragged_id: Id,
        pointer: Point,
        tab_bounds: &[(Id, Rectangle, bool)],
    ) -> DropDecision<TabDropTarget<Id>> {
        let pinned = self
            .tabs
            .iter()
            .find(|tab| tab.id == dragged_id)
            .map(|tab| tab.pinned)
            .unwrap_or(false);

        let segment: Vec<(Id, Rectangle)> = tab_bounds
            .iter()
            .filter(|(_, _, item_pinned)| *item_pinned == pinned)
            .map(|(id, bounds, _)| (id.clone(), *bounds))
            .collect();

        let pointer_main = Orientation::Horizontal.main_position(pointer);
        let other_segment_exists = tab_bounds
            .iter()
            .any(|(_, _, item_pinned)| *item_pinned != pinned);

        if other_segment_exists {
            if pinned {
                let segment_end = segment
                    .iter()
                    .map(|(_, bounds)| {
                        Orientation::Horizontal.main_position(bounds.position())
                            + Orientation::Horizontal.main_length(bounds.size())
                    })
                    .fold(f32::MIN, f32::max);
                if pointer_main > segment_end {
                    return DropDecision::Reject;
                }
            } else {
                let segment_start = segment
                    .iter()
                    .map(|(_, bounds)| Orientation::Horizontal.main_position(bounds.position()))
                    .fold(f32::MAX, f32::min);
                if pointer_main < segment_start {
                    return DropDecision::Reject;
                }
            }
        }

        let Some(insertion) =
            crate::interaction::linear_insertion(Orientation::Horizontal, pointer, segment.clone())
        else {
            return DropDecision::Reject;
        };

        let target = match insertion {
            LinearInsertion::Before(id) => TabDropTarget::Before(id),
            LinearInsertion::After(id) => TabDropTarget::After(id),
        };

        DropDecision::accept(target, TransferOperation::Move)
    }
}

impl<Id> operation::Focusable for TabBarFocus<'_, Id> {
    fn is_focused(&self) -> bool {
        operation::Focusable::is_focused(self.focus)
    }

    fn focus(&mut self) {
        operation::Focusable::focus(self.focus);
    }

    fn unfocus(&mut self) {
        operation::Focusable::unfocus(self.focus);
        *self.pressed_id = None;
    }
}
