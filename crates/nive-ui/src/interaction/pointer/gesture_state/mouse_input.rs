use std::time::Instant;

use iced::Point;

use super::super::{PointerButton, PointerGesture, PointerGestureKind};
use super::{distance, gesture, ClickMemory, PointerGestureState, Pressed};

impl<Region> PointerGestureState<Region>
where
    Region: Clone + PartialEq,
{
    pub(super) fn handle_button_pressed(
        &mut self,
        button: PointerButton,
        now: Instant,
        region_at: impl Fn(Point) -> Option<Region>,
    ) -> Vec<PointerGesture<Region>> {
        let Some(position) = self.cursor_position else {
            return Vec::new();
        };
        let Some(region) = region_at(position) else {
            return Vec::new();
        };

        self.pressed = Some(Pressed {
            button,
            region: region.clone(),
            origin: position,
            drag_started: false,
            pressed_at: now,
        });

        vec![gesture(
            PointerGestureKind::Pressed,
            button,
            region,
            position,
            self.modifiers,
        )]
    }

    pub(super) fn handle_cursor_moved(&mut self, position: Point) -> Vec<PointerGesture<Region>> {
        let Some(pressed) = self.pressed.as_mut() else {
            return Vec::new();
        };

        if !pressed.drag_started && distance(pressed.origin, position) < self.drag_threshold {
            return Vec::new();
        }

        if pressed.drag_started {
            vec![gesture(
                PointerGestureKind::DragMoved,
                pressed.button,
                pressed.region.clone(),
                position,
                self.modifiers,
            )]
        } else {
            pressed.drag_started = true;
            vec![
                gesture(
                    PointerGestureKind::DragStarted,
                    pressed.button,
                    pressed.region.clone(),
                    position,
                    self.modifiers,
                ),
                gesture(
                    PointerGestureKind::DragMoved,
                    pressed.button,
                    pressed.region.clone(),
                    position,
                    self.modifiers,
                ),
            ]
        }
    }

    pub(super) fn handle_button_released(
        &mut self,
        button: PointerButton,
        now: Instant,
    ) -> Vec<PointerGesture<Region>> {
        let Some(pressed) = self.pressed.take() else {
            return Vec::new();
        };

        if pressed.button != button {
            return Vec::new();
        }

        let position = self.cursor_position.unwrap_or(pressed.origin);
        let mut gestures = vec![gesture(
            PointerGestureKind::Released,
            pressed.button,
            pressed.region.clone(),
            position,
            self.modifiers,
        )];

        if pressed.drag_started {
            gestures.push(gesture(
                PointerGestureKind::DragReleased,
                pressed.button,
                pressed.region,
                position,
                self.modifiers,
            ));
            self.last_click = None;
        } else {
            let count = self.next_click_count(
                pressed.button,
                &pressed.region,
                pressed.pressed_at,
                position,
                now,
            );
            gestures.push(gesture(
                PointerGestureKind::Clicked { count },
                pressed.button,
                pressed.region.clone(),
                position,
                self.modifiers,
            ));
            self.last_click = Some(ClickMemory {
                button: pressed.button,
                region: pressed.region,
                position,
                clicked_at: now,
                count,
            });
        }

        gestures
    }
}
