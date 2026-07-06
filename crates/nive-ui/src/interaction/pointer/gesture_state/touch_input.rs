use std::time::Instant;

use iced::{touch, Point};

use super::super::{PointerButton, PointerGesture, PointerGestureKind};
use super::{distance, gesture, ClickMemory, PointerGestureState};

/// An active touch interaction tracked by [`PointerGestureState`].
#[derive(Debug, Clone)]
pub struct Touch<Region> {
    pub finger_id: touch::Finger,
    pub region: Region,
    pub origin: Point,
    pub drag_started: bool,
    pub pressed_at: Instant,
}

impl<Region> PointerGestureState<Region>
where
    Region: Clone + PartialEq,
{
    pub(super) fn handle_touch_pressed(
        &mut self,
        finger_id: touch::Finger,
        position: Point,
        now: Instant,
        region_at: impl Fn(Point) -> Option<Region>,
    ) -> Vec<PointerGesture<Region>> {
        if self.active_touch.is_some() {
            return Vec::new();
        }
        let Some(region) = region_at(position) else {
            return Vec::new();
        };

        self.active_touch = Some(Touch {
            finger_id,
            region: region.clone(),
            origin: position,
            drag_started: false,
            pressed_at: now,
        });

        vec![gesture(
            PointerGestureKind::Pressed,
            PointerButton::Primary,
            region,
            position,
            self.modifiers,
        )]
    }

    pub(super) fn handle_touch_moved(
        &mut self,
        finger_id: touch::Finger,
        position: Point,
    ) -> Vec<PointerGesture<Region>> {
        let Some(touch) = self.active_touch.as_mut() else {
            return Vec::new();
        };
        if touch.finger_id != finger_id {
            return Vec::new();
        }

        if !touch.drag_started && distance(touch.origin, position) < self.drag_threshold {
            return Vec::new();
        }

        if touch.drag_started {
            vec![gesture(
                PointerGestureKind::DragMoved,
                PointerButton::Primary,
                touch.region.clone(),
                position,
                self.modifiers,
            )]
        } else {
            touch.drag_started = true;
            vec![
                gesture(
                    PointerGestureKind::DragStarted,
                    PointerButton::Primary,
                    touch.region.clone(),
                    position,
                    self.modifiers,
                ),
                gesture(
                    PointerGestureKind::DragMoved,
                    PointerButton::Primary,
                    touch.region.clone(),
                    position,
                    self.modifiers,
                ),
            ]
        }
    }

    pub(super) fn handle_touch_lifted(
        &mut self,
        finger_id: touch::Finger,
        position: Point,
        now: Instant,
    ) -> Vec<PointerGesture<Region>> {
        let Some(touch) = self.active_touch.as_ref() else {
            return Vec::new();
        };
        if touch.finger_id != finger_id {
            return Vec::new();
        }
        let touch = self.active_touch.take().expect("checked above");

        let mut gestures = vec![gesture(
            PointerGestureKind::Released,
            PointerButton::Primary,
            touch.region.clone(),
            position,
            self.modifiers,
        )];

        if touch.drag_started {
            gestures.push(gesture(
                PointerGestureKind::DragReleased,
                PointerButton::Primary,
                touch.region,
                position,
                self.modifiers,
            ));
            self.last_click = None;
        } else {
            let count = self.next_click_count(
                PointerButton::Primary,
                &touch.region,
                touch.pressed_at,
                position,
                now,
            );
            gestures.push(gesture(
                PointerGestureKind::Clicked { count },
                PointerButton::Primary,
                touch.region.clone(),
                position,
                self.modifiers,
            ));
            self.last_click = Some(ClickMemory {
                button: PointerButton::Primary,
                region: touch.region,
                position,
                clicked_at: now,
                count,
            });
        }

        gestures
    }

    pub(super) fn handle_touch_lost(
        &mut self,
        finger_id: touch::Finger,
        position: Point,
    ) -> Vec<PointerGesture<Region>> {
        let Some(touch) = self.active_touch.as_ref() else {
            return Vec::new();
        };
        if touch.finger_id != finger_id {
            return Vec::new();
        }
        let touch = self.active_touch.take().expect("checked above");

        if touch.drag_started {
            vec![gesture(
                PointerGestureKind::DragCancelled,
                PointerButton::Primary,
                touch.region,
                position,
                self.modifiers,
            )]
        } else {
            Vec::new()
        }
    }
}
