use std::time::Instant;

use iced::{
    advanced::{mouse, widget::Tree, Clipboard, Layout, Shell},
    keyboard::{self, key::Named},
    Event, Rectangle,
};

use crate::interaction::{Orientation, PointerButton, PointerGesture, PointerGestureKind};

use super::super::super::split_divider::{hit_bounds, metrics};
use super::super::sizing;
use super::super::state::{DragSession, SplitStackRegion, SplitStackState};
use super::super::{SplitCollapse, SplitResize, SplitStack};
use super::{divider_layouts, pane_layouts, KEYBOARD_STEP};

/// What one batch of pointer gestures proposed.
#[derive(Default)]
struct DragOutcome {
    resizes: Vec<SplitResize>,
    collapse: Option<SplitCollapse>,
}

pub(super) fn divider_hit_bounds<Message>(
    stack: &SplitStack<'_, Message>,
    divider: Layout<'_>,
    layout: Layout<'_>,
) -> Rectangle {
    hit_bounds(
        divider.bounds(),
        layout.bounds(),
        stack.orientation,
        metrics(stack.size),
    )
}

pub(super) fn focused_hit_bounds<Message>(
    stack: &SplitStack<'_, Message>,
    tree: &Tree,
    layout: Layout<'_>,
) -> Option<Rectangle> {
    let focused = tree.state.downcast_ref::<SplitStackState>().focused_divider;

    divider_layouts(layout)
        .nth(focused)
        .map(|divider| divider_hit_bounds(stack, divider, layout))
}

/// A press that landed away from every divider.
///
/// An unresolvable cursor position is never treated as outside, so a press
/// while the pointer sits beyond the window cannot end an ongoing drag.
fn primary_press_outside(event: &Event, cursor: mouse::Cursor, hits: &[Rectangle]) -> bool {
    let position = match event {
        Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
            cursor.position()
        }
        Event::Touch(iced::touch::Event::FingerPressed { position, .. }) => Some(*position),
        _ => return false,
    };

    position.is_some_and(|position| !hits.iter().any(|hit| hit.contains(position)))
}

/// Cross-axis arrow step, which never collides with the resize keys.
fn roving_step(key: &keyboard::Key, orientation: Orientation) -> Option<isize> {
    let named = match key {
        keyboard::Key::Named(named) => named,
        _ => return None,
    };

    match (orientation, named) {
        (Orientation::Horizontal, Named::ArrowUp) | (Orientation::Vertical, Named::ArrowLeft) => {
            Some(-1)
        }
        (Orientation::Horizontal, Named::ArrowDown)
        | (Orientation::Vertical, Named::ArrowRight) => Some(1),
        _ => None,
    }
}

impl<'a, Message> SplitStack<'a, Message>
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
        // Snapshot the hit rectangles before the gesture state is borrowed, so
        // the region closure does not need to reach back into it.
        let hits = divider_layouts(layout)
            .map(|divider| divider_hit_bounds(self, divider, layout))
            .collect::<Vec<_>>();

        {
            let state = tree.state.downcast_mut::<SplitStackState>();

            if matches!(event, Event::Window(iced::window::Event::Unfocused)) {
                state.focus.deactivate();
            }

            if primary_press_outside(event, cursor, &hits) {
                state.drag = None;
                if state.focus.is_active() {
                    state.focus.deactivate();
                    shell.request_redraw();
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
                let gestures = state
                    .gestures
                    .handle_event(event, Instant::now(), |position| {
                        hits.iter()
                            .position(|hit| hit.contains(position))
                            .map(SplitStackRegion::Divider)
                    });

                if gestures
                    .iter()
                    .any(|gesture| gesture.button == PointerButton::Primary)
                {
                    let outcome = self.handle_pointer_gestures(state, &gestures);

                    for resize in outcome.resizes {
                        if let Some(on_resize) = self.on_resize.as_deref() {
                            shell.publish(on_resize(resize));
                        }
                    }

                    // Published last so its restore length wins over the
                    // clamped size the same drag just proposed.
                    if let Some(collapse) = outcome.collapse {
                        if let Some(on_collapse) = self.on_collapse.as_deref() {
                            shell.publish(on_collapse(collapse));
                        }
                    }

                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
            }
        }

        if shell.is_event_captured() {
            return;
        }

        for ((content, child), pane) in self
            .contents
            .iter_mut()
            .zip(&mut tree.children)
            .zip(pane_layouts(layout))
        {
            content.as_widget_mut().update(
                child, event, pane, cursor, renderer, clipboard, shell, viewport,
            );

            if shell.is_event_captured() {
                return;
            }
        }
    }

    fn handle_pointer_gestures(
        &self,
        state: &mut SplitStackState,
        gestures: &[PointerGesture<SplitStackRegion>],
    ) -> DragOutcome {
        let mut outcome = DragOutcome::default();

        for gesture in gestures {
            if gesture.button != PointerButton::Primary {
                continue;
            }

            let SplitStackRegion::Divider(region) = gesture.region;

            match gesture.kind {
                PointerGestureKind::Pressed => {
                    state.focused_divider = region;
                    state.focus.focus_from_pointer();
                }
                PointerGestureKind::DragStarted => {
                    let (Some(leading), Some(trailing)) = (
                        state.resolved.get(region).copied(),
                        state.resolved.get(region + 1).copied(),
                    ) else {
                        continue;
                    };

                    state.drag = Some(DragSession {
                        divider: region,
                        origin_leading: leading,
                        origin_trailing: trailing,
                        origin_position: gesture.position,
                    });
                }
                PointerGestureKind::DragMoved => {
                    let Some(drag) = state.drag else {
                        continue;
                    };

                    let delta = self.orientation.main_position(gesture.position)
                        - self.orientation.main_position(drag.origin_position);

                    if let Some(resize) = self.resize_divider(drag.divider, drag.pair(), delta) {
                        outcome.resizes.push(resize);
                    }

                    // Ending the session here is what keeps a collapse one-shot:
                    // later moves in the same drag find no session and bail.
                    if let Some(collapse) = self.collapse_for(drag, delta) {
                        outcome.collapse = Some(collapse);
                        state.drag = None;
                    }
                }
                PointerGestureKind::Released
                | PointerGestureKind::DragReleased
                | PointerGestureKind::DragCancelled => {
                    state.drag = None;
                }
                PointerGestureKind::Clicked { .. } => {}
            }
        }

        outcome
    }

    /// Resolves an over-travelled drag into the pane it should collapse.
    fn collapse_for(&self, drag: DragSession, delta: f32) -> Option<SplitCollapse> {
        self.on_collapse.as_ref()?;

        let sizes = [drag.origin_leading, drag.origin_trailing];
        let minimums = [
            sizing::pane_minimum(&self.minimums, drag.divider),
            sizing::pane_minimum(&self.minimums, drag.divider + 1),
        ];
        let overtravel = sizing::overtravel(&sizes, &minimums, 0, delta);

        if overtravel == 0.0 || overtravel.abs() < self.collapse_threshold {
            return None;
        }

        let (pane, restore) = if overtravel < 0.0 {
            (drag.divider, drag.origin_leading)
        } else {
            (drag.divider + 1, drag.origin_trailing)
        };

        self.collapsible
            .get(pane)
            .copied()
            .unwrap_or(false)
            .then_some(SplitCollapse {
                divider: drag.divider,
                pane,
                restore,
            })
    }

    pub(super) fn forward_keyboard(
        &self,
        state: &mut SplitStackState,
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

        let dividers = self.contents.len().saturating_sub(1);
        if dividers == 0 {
            return false;
        }

        if let Some(step) = KEYBOARD_STEP.delta(key, *modifiers, self.orientation) {
            let divider = state.focused_divider;
            let (Some(leading), Some(trailing)) = (
                state.resolved.get(divider).copied(),
                state.resolved.get(divider + 1).copied(),
            ) else {
                return false;
            };

            state.focus.focus_from_keyboard();

            if let Some(resize) =
                self.resize_divider(divider, (leading, trailing), step * state.available)
            {
                if let Some(on_resize) = self.on_resize.as_deref() {
                    shell.publish(on_resize(resize));
                }
            }

            shell.capture_event();
            shell.request_redraw();

            return true;
        }

        let next = match key {
            keyboard::Key::Named(Named::Home) => Some(0),
            keyboard::Key::Named(Named::End) => Some(dividers - 1),
            _ => roving_step(key, self.orientation).map(|step| {
                (state.focused_divider as isize + step).clamp(0, dividers as isize - 1) as usize
            }),
        };

        let Some(next) = next else {
            return false;
        };

        state.focus.focus_from_keyboard();
        state.focused_divider = next;
        shell.capture_event();
        shell.request_redraw();

        true
    }

    /// Clamps a divider move against its own two panes only.
    fn resize_divider(&self, divider: usize, pair: (f32, f32), delta: f32) -> Option<SplitResize> {
        let sizes = [pair.0, pair.1];
        let minimums = [
            sizing::pane_minimum(&self.minimums, divider),
            sizing::pane_minimum(&self.minimums, divider + 1),
        ];
        let (leading, trailing) = sizing::resize(&sizes, &minimums, 0, delta)?;

        Some(SplitResize {
            divider,
            leading,
            trailing,
        })
    }
}

impl DragSession {
    fn pair(self) -> (f32, f32) {
        (self.origin_leading, self.origin_trailing)
    }
}
