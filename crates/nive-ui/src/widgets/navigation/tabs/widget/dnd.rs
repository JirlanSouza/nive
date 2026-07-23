use iced::{advanced::Shell, Rectangle};

use crate::interaction::dnd::{DragSessionFeedback, DragSessionOutcome};
use crate::interaction::{
    CollectionTransferPayload, Drag, DropDecision, PointerGesture, TransferData, TransferOperation,
    TransferOperations,
};
use crate::widgets::navigation::tabs::geometry::{
    edge_scroll_direction, gesture_to_pointer, singleton_payload,
};
use crate::widgets::navigation::tabs::style as theme_tabs;
use crate::widgets::navigation::tabs::{
    TabBar, TabBarState, TabDrop, TabDropTarget, TabRegion, TabTearOff, TEAR_OFF_HYSTERESIS,
};

impl<'a, Id, Message> TabBar<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    pub(super) fn drag_started(
        &self,
        state: &mut TabBarState<Id>,
        gesture: &PointerGesture<TabRegion>,
        display_index: usize,
    ) {
        let displayed_tabs = self.displayed_tabs();
        let Some(displayed) = displayed_tabs.get(display_index) else {
            return;
        };
        if displayed.item.disabled {
            return;
        }
        let dragged_id = displayed.item.id.clone();
        state.dragged_id = Some(dragged_id.clone());

        let outcome = state.drag_session.handle_gesture(
            &gesture_to_pointer(gesture, TabRegion::Tab(display_index)),
            || {
                Some(Drag::<CollectionTransferPayload<Id>, ()> {
                    payload: TransferData::local(singleton_payload(dragged_id.clone())),
                    origin: (),
                    operations: TransferOperations::MOVE,
                    preferred: TransferOperation::Move,
                })
            },
            |_context| DropDecision::<TabDropTarget<Id>>::Reject,
        );

        if let DragSessionOutcome::Feedback(_) = outcome {
            // Drag has started; first move event will probe targets.
        }
    }

    pub(super) fn drag_moved(
        &self,
        state: &mut TabBarState<Id>,
        gesture: &PointerGesture<TabRegion>,
        shell: &mut Shell<'_, Message>,
    ) {
        let Some(dragged_id) = state.dragged_id.clone() else {
            return;
        };
        let tab_bounds = state.tab_bounds.clone();
        let mut probed_target: Option<TabDropTarget<Id>> = None;
        let outcome = state.drag_session.handle_gesture(
            &gesture_to_pointer(gesture, TabRegion::Empty),
            || None,
            |context| {
                let decision = if context.preferred_operation() == Some(TransferOperation::Move) {
                    self.reorder_decision(dragged_id.clone(), context.position, &tab_bounds)
                } else {
                    DropDecision::<TabDropTarget<Id>>::Reject
                };
                probed_target = match &decision {
                    DropDecision::Accept { target, .. } => Some(target.clone()),
                    _ => None,
                };
                decision
            },
        );

        if let DragSessionOutcome::Feedback(feedback) = outcome {
            let accepted = matches!(feedback, DragSessionFeedback::Accepted(_));
            state.insertion_target = accepted.then_some(probed_target).flatten();
            state.invalid_target = !accepted;
            let direction = state.strip_bounds.and_then(|strip| {
                edge_scroll_direction(
                    gesture.position,
                    strip,
                    theme_tabs::metrics(self.size).height,
                    state.scroll_offset,
                    state.max_scroll,
                )
            });
            if direction != state.edge_scroll {
                state.last_redraw = None;
            }
            state.edge_scroll = direction;
            if direction.is_some() {
                shell.request_redraw();
            }
            shell.request_redraw();
        }
    }

    pub(super) fn drag_released(
        &self,
        state: &mut TabBarState<Id>,
        gesture: &PointerGesture<TabRegion>,
        bounds: Rectangle,
        shell: &mut Shell<'_, Message>,
    ) {
        let Some(dragged_id) = state.dragged_id.clone() else {
            state.drag_session.cancel();
            return;
        };
        let strip_outer = Rectangle {
            x: bounds.x - TEAR_OFF_HYSTERESIS,
            y: bounds.y - TEAR_OFF_HYSTERESIS,
            width: bounds.width + TEAR_OFF_HYSTERESIS * 2.0,
            height: bounds.height + TEAR_OFF_HYSTERESIS * 2.0,
        };

        if self.on_tear_off.is_some() && !strip_outer.contains(gesture.position) {
            let payload = singleton_payload(dragged_id.clone());
            if let Some(on_tear_off) = &self.on_tear_off {
                shell.publish(on_tear_off(TabTearOff {
                    payload,
                    position: gesture.position,
                }));
                shell.capture_event();
            }
            state.dragged_id = None;
            state.insertion_target = None;
            state.invalid_target = false;
            state.edge_scroll = None;
            state.last_redraw = None;
            state.drag_session.cancel();
            return;
        }

        if !strip_outer.contains(gesture.position) {
            state.dragged_id = None;
            state.insertion_target = None;
            state.invalid_target = false;
            state.edge_scroll = None;
            state.last_redraw = None;
            state.drag_session.cancel();
            return;
        }

        // If dragged id is no longer present in tabs, silently end.
        if !self.tabs.iter().any(|tab| tab.id == dragged_id) {
            state.dragged_id = None;
            state.insertion_target = None;
            state.invalid_target = false;
            state.edge_scroll = None;
            state.last_redraw = None;
            state.drag_session.cancel();
            return;
        }

        let tab_bounds = state.tab_bounds.clone();
        let outcome = state.drag_session.handle_gesture(
            &gesture_to_pointer(gesture, TabRegion::Empty),
            || None,
            |context| self.reorder_decision(dragged_id.clone(), context.position, &tab_bounds),
        );

        if let DragSessionOutcome::Commit(Some(commit)) = outcome {
            if let Some(on_reorder) = &self.on_reorder {
                let payload = match &commit.payload {
                    TransferData::Local(payload) => payload.clone(),
                    _ => singleton_payload(dragged_id.clone()),
                };
                shell.publish(on_reorder(TabDrop {
                    payload,
                    target: commit.target,
                    operation: commit.operation,
                }));
                shell.capture_event();
            }
        }
        state.dragged_id = None;
        state.insertion_target = None;
        state.invalid_target = false;
        state.edge_scroll = None;
        state.last_redraw = None;
    }

    pub(super) fn drag_cancelled(&self, state: &mut TabBarState<Id>) {
        state.dragged_id = None;
        state.insertion_target = None;
        state.invalid_target = false;
        state.edge_scroll = None;
        state.last_redraw = None;
        state.drag_session.cancel();
    }
}
