use iced::{
    advanced::{mouse, Layout, Shell},
    keyboard, Event, Point, Rectangle,
};

use crate::interaction::{Orientation, PointerButton, PointerGesture, PointerGestureKind};

use super::super::helpers::{apply_snap, clamp_ratio, maximum_ratio, minimum_ratio};
use super::super::state::{DragSession, SnapConfig, SplitPaneRegion, SplitPaneState};
use super::super::SplitPaneConstraints;

pub(super) fn handle_pointer_gestures(
    state: &mut SplitPaneState,
    gestures: &[PointerGesture<SplitPaneRegion>],
    orientation: Orientation,
    ratio: f32,
    constraints: SplitPaneConstraints,
    snap: Option<&SnapConfig>,
    locked: bool,
) -> Vec<f32> {
    if locked {
        return Vec::new();
    }

    let mut ratios = Vec::new();

    for gesture in gestures {
        if gesture.button != PointerButton::Primary {
            continue;
        }

        match gesture.kind {
            PointerGestureKind::Pressed => {
                state.focused = true;
            }
            PointerGestureKind::DragStarted => {
                state.drag = Some(DragSession {
                    origin_ratio: ratio,
                    origin_position: gesture.position,
                });
            }
            PointerGestureKind::DragMoved => {
                let Some(drag) = state.drag else {
                    continue;
                };

                if state.available_length <= 0.0 {
                    continue;
                }

                let delta = axis_position(orientation, gesture.position)
                    - axis_position(orientation, drag.origin_position);
                let next_ratio = drag.origin_ratio + delta / state.available_length;

                ratios.push(constrained_ratio(
                    next_ratio,
                    constraints,
                    state.available_length,
                    snap,
                ));
            }
            PointerGestureKind::Released | PointerGestureKind::DragReleased => {
                state.drag = None;
            }
            PointerGestureKind::DragCancelled => {
                state.drag = None;
            }
            PointerGestureKind::Clicked { count: 2 } => {
                ratios.push(constrained_ratio(
                    0.5,
                    constraints,
                    state.available_length,
                    snap,
                ));
            }
            PointerGestureKind::Clicked { .. } => {}
        }
    }

    ratios
}

pub(super) fn constrained_ratio(
    ratio: f32,
    constraints: SplitPaneConstraints,
    available_length: f32,
    snap: Option<&SnapConfig>,
) -> f32 {
    let minimum = minimum_ratio(constraints, available_length);
    let maximum = maximum_ratio(constraints, available_length);
    let ratio = apply_snap(ratio, snap, minimum, maximum);

    clamp_ratio(ratio, constraints, available_length)
}

pub(super) fn publish_ratio<Message>(
    on_change: Option<&dyn Fn(f32) -> Message>,
    ratio: f32,
    shell: &mut Shell<'_, Message>,
) {
    if let Some(on_change) = on_change {
        shell.publish(on_change(ratio));
    }
}

pub(super) fn primary_press_outside_grip(
    event: &Event,
    cursor: mouse::Cursor,
    grip_bounds: Rectangle,
) -> bool {
    matches!(
        event,
        Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
    ) && !cursor.is_over(grip_bounds)
}

pub(super) fn current_grip_bounds(layout: Layout<'_>) -> Option<Rectangle> {
    layout.children().nth(1).map(|layout| layout.bounds())
}

pub(super) fn resize_interaction(orientation: Orientation) -> mouse::Interaction {
    match orientation {
        Orientation::Horizontal => mouse::Interaction::ResizingColumn,
        Orientation::Vertical => mouse::Interaction::ResizingRow,
    }
}

pub(super) fn axis_position(orientation: Orientation, position: Point) -> f32 {
    match orientation {
        Orientation::Horizontal => position.x,
        Orientation::Vertical => position.y,
    }
}
impl<'a, Message> super::SplitPane<'a, Message>
where
    Message: 'a,
{
    pub(super) fn forward_keyboard(
        &self,
        state: &mut SplitPaneState,
        event: &Event,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        if !state.focused {
            return false;
        }

        let Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            repeat: false,
            ..
        }) = event
        else {
            return false;
        };

        let Some(delta) = super::KEYBOARD_STEP.delta(key, *modifiers, self.orientation) else {
            return false;
        };

        let next_ratio = constrained_ratio(
            self.ratio + delta,
            self.constraints,
            state.available_length,
            self.snap.as_ref(),
        );

        publish_ratio(self.on_change.as_deref(), next_ratio, shell);
        shell.capture_event();
        shell.request_redraw();

        true
    }
}
