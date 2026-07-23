use iced::{advanced::Shell, Event, Rectangle};

use crate::interaction::{PointerButton, PointerGestureKind};
use crate::widgets::navigation::overflow::OverflowDirection;
use crate::widgets::navigation::tabs::geometry::snapshot_tab_region;
use crate::widgets::navigation::tabs::{
    TabBar, TabBarState, TabRegion, CHEVRON_SCROLL_STEP_FACTOR,
};

impl<'a, Id, Message> TabBar<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    pub(super) fn handle_gestures(
        &self,
        state: &mut TabBarState<Id>,
        event: &Event,
        bounds: Rectangle,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds_for_gestures = bounds;
        // Snapshot the data needed by `tab_region_at` so the closure does not
        // borrow `state` (which `handle_event` mutably borrows through the
        // gesture state).
        let tab_bounds = state.tab_bounds.clone();
        let close_bounds = state.close_bounds.clone();
        let left_chevron = state.left_chevron;
        let right_chevron = state.right_chevron;
        let all_tabs_button = state.all_tabs_button;
        let gestures = state
            .gestures
            .handle_event(event, std::time::Instant::now(), |point| {
                if bounds_for_gestures.contains(point) {
                    Some(snapshot_tab_region(
                        &tab_bounds,
                        &close_bounds,
                        left_chevron,
                        right_chevron,
                        all_tabs_button,
                        point,
                    ))
                } else {
                    None
                }
            });

        // We cannot borrow state mutably while gestures still borrow it; clone
        // the gestures out so we can mutate state underneath.
        for gesture in gestures {
            let region = gesture.region;
            match (gesture.button, gesture.kind, region) {
                (PointerButton::Secondary, PointerGestureKind::Clicked { .. }, region) => {
                    if let Some(on_context) = &self.on_context {
                        if let Some(request) = self.context_request(region, gesture.position) {
                            shell.publish(on_context(request));
                            shell.capture_event();
                        }
                    }
                }
                (PointerButton::Middle, PointerGestureKind::Clicked { .. }, region) => {
                    if let Some(on_close) = &self.on_close_request {
                        if let Some(request) = self.close_request(region) {
                            shell.publish(on_close(request));
                            shell.capture_event();
                        }
                    }
                }
                (PointerButton::Primary, PointerGestureKind::Clicked { .. }, region) => {
                    self.handle_primary_click(state, region, shell);
                }
                (
                    PointerButton::Primary,
                    PointerGestureKind::DragStarted,
                    TabRegion::Tab(display_index),
                ) if self.on_reorder.is_some() => {
                    self.drag_started(state, &gesture, display_index);
                }
                (PointerButton::Primary, PointerGestureKind::DragMoved, _) => {
                    self.drag_moved(state, &gesture, shell);
                }
                (PointerButton::Primary, PointerGestureKind::DragReleased, _) => {
                    self.drag_released(state, &gesture, bounds, shell);
                }
                (PointerButton::Primary, PointerGestureKind::DragCancelled, _) => {
                    self.drag_cancelled(state);
                }
                _ => {}
            }
        }
    }

    fn handle_primary_click(
        &self,
        state: &mut TabBarState<Id>,
        region: TabRegion,
        shell: &mut Shell<'_, Message>,
    ) {
        match region {
            TabRegion::Close(close_index) => {
                if let (Some(on_close), Some(request)) = (
                    &self.on_close_request,
                    self.close_button_request(close_index),
                ) {
                    shell.publish(on_close(request));
                    shell.capture_event();
                }
            }
            TabRegion::Tab(display_index) => {
                if let Some(tab) = self
                    .displayed_tabs()
                    .get(display_index)
                    .map(|item| item.item)
                {
                    if !tab.disabled {
                        state.focused_id = Some(tab.id.clone());
                        if let Some(on_select) = &self.on_select {
                            shell.publish(on_select(tab.id.clone()));
                            shell.capture_event();
                        }
                    }
                }
            }
            TabRegion::ChevronLeft if state.has_overflow => {
                state.overflow.offset = state.scroll_offset;
                state
                    .overflow
                    .page_step(OverflowDirection::Backward, CHEVRON_SCROLL_STEP_FACTOR);
                state.scroll_offset = state.overflow.offset;
                shell.invalidate_layout();
                shell.request_redraw();
                shell.capture_event();
            }
            TabRegion::ChevronRight if state.has_overflow => {
                state.overflow.offset = state.scroll_offset;
                state
                    .overflow
                    .page_step(OverflowDirection::Forward, CHEVRON_SCROLL_STEP_FACTOR);
                state.scroll_offset = state.overflow.offset;
                shell.invalidate_layout();
                shell.request_redraw();
                shell.capture_event();
            }
            _ => {}
        }
    }
}
