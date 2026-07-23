use std::time::Instant;

use iced::{
    advanced::{mouse, widget::Tree, Clipboard, Layout, Shell},
    keyboard, Event, Point, Rectangle,
};

use crate::interaction::{Orientation, PointerButton, PointerGesture, PointerGestureKind};

use super::super::helpers::{
    apply_snap, clamp_ratio, hit_bounds, maximum_ratio, metrics, minimum_ratio, SplitPaneMetrics,
};
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
                state.focus.focus_from_pointer();
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

pub(super) fn has_primary_gesture(gestures: &[PointerGesture<SplitPaneRegion>]) -> bool {
    gestures
        .iter()
        .any(|gesture| gesture.button == PointerButton::Primary)
}

pub(super) fn primary_press_outside_hit(
    event: &Event,
    cursor: mouse::Cursor,
    hit_bounds: Rectangle,
) -> bool {
    match event {
        Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
            !cursor.is_over(hit_bounds)
        }
        Event::Touch(iced::touch::Event::FingerPressed { position, .. }) => {
            !hit_bounds.contains(*position)
        }
        _ => false,
    }
}

pub(super) fn current_divider_bounds(layout: Layout<'_>) -> Option<Rectangle> {
    layout.children().nth(1).map(|layout| layout.bounds())
}

pub(super) fn current_hit_bounds(
    layout: Layout<'_>,
    orientation: Orientation,
    metrics: SplitPaneMetrics,
) -> Option<Rectangle> {
    current_divider_bounds(layout)
        .map(|divider_bounds| hit_bounds(divider_bounds, layout.bounds(), orientation, metrics))
}

pub(super) fn resize_interaction(orientation: Orientation) -> mouse::Interaction {
    match orientation {
        Orientation::Horizontal => mouse::Interaction::ResizingColumn,
        Orientation::Vertical => mouse::Interaction::ResizingRow,
    }
}

pub(super) fn axis_position(orientation: Orientation, position: Point) -> f32 {
    orientation.main_position(position)
}
impl<'a, Message> super::SplitPane<'a, Message>
where
    Message: 'a,
{
    #[allow(clippy::too_many_arguments)]
    pub(super) fn update_impl(
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
        let hit_bounds = current_hit_bounds(layout, self.orientation, metrics(self.size));

        {
            let state = tree.state.downcast_mut::<SplitPaneState>();

            if matches!(event, Event::Window(iced::window::Event::Unfocused)) {
                state.focus.deactivate();
            }

            if let Some(hit_bounds) = hit_bounds {
                if primary_press_outside_hit(event, cursor, hit_bounds) {
                    state.drag = None;
                    if state.focus.is_active() {
                        state.focus.deactivate();
                        shell.request_redraw();
                    }
                }
            }

            if !self.interactive() && (state.focus.is_active() || state.drag.is_some()) {
                state.focus.clear();
                state.drag = None;
                shell.request_redraw();
            }

            if self.interactive() && self.forward_keyboard(state, event, shell) {
                return;
            }

            if self.interactive() {
                if let Some(hit_bounds) = hit_bounds {
                    let gestures = state
                        .gestures
                        .handle_event(event, Instant::now(), |position| {
                            hit_bounds
                                .contains(position)
                                .then_some(SplitPaneRegion::Grip)
                        });

                    if has_primary_gesture(&gestures) {
                        let ratios = handle_pointer_gestures(
                            state,
                            &gestures,
                            self.orientation,
                            self.ratio,
                            self.constraints,
                            self.snap.as_ref(),
                            false,
                        );

                        for ratio in ratios {
                            publish_ratio(self.on_change.as_deref(), ratio, shell);
                        }

                        shell.capture_event();
                        shell.request_redraw();
                        return;
                    }
                }
            }
        }

        if shell.is_event_captured() {
            return;
        }

        let mut layouts = layout.children();
        let Some(leading_layout) = layouts.next() else {
            return;
        };
        let _ = layouts.next();
        let Some(trailing_layout) = layouts.next() else {
            return;
        };

        self.leading.as_widget_mut().update(
            &mut tree.children[0],
            event,
            leading_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        self.trailing.as_widget_mut().update(
            &mut tree.children[1],
            event,
            trailing_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    pub(super) fn forward_keyboard(
        &self,
        state: &mut SplitPaneState,
        event: &Event,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        if !state.focus.is_active() {
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
        state.focus.focus_from_keyboard();

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
