use std::time::{Duration, Instant};

use iced::{keyboard, mouse, touch, Event, Point};

use self::touch_input::Touch;
use super::{PointerButton, PointerGesture, PointerGestureKind};

mod mouse_input;
mod touch_input;

#[cfg(test)]
mod tests;

/// Widget-agnostic pointer gesture state normalizing mouse and touch input
/// into the shared [`PointerGesture`] vocabulary.
///
/// Mouse and touch interaction are mutually exclusive: once a touch is
/// active, mouse events are ignored until the touch ends, and vice versa.
#[derive(Debug, Clone)]
pub struct PointerGestureState<Region> {
    pub(super) modifiers: keyboard::Modifiers,
    pub(super) cursor_position: Option<Point>,
    pub(super) pressed: Option<Pressed<Region>>,
    pub(super) active_touch: Option<Touch<Region>>,
    pub(super) last_click: Option<ClickMemory<Region>>,
    pub(super) drag_threshold: f32,
    pub(super) click_distance: f32,
    pub(super) click_interval: Duration,
}

impl<Region> Default for PointerGestureState<Region> {
    fn default() -> Self {
        Self {
            modifiers: keyboard::Modifiers::NONE,
            cursor_position: None,
            pressed: None,
            active_touch: None,
            last_click: None,
            drag_threshold: 4.0,
            click_distance: 4.0,
            click_interval: Duration::from_millis(500),
        }
    }
}

impl<Region> PointerGestureState<Region> {
    /// Creates a pointer gesture state with the default drag and click
    /// thresholds.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the drag threshold in pixels.
    pub fn with_drag_threshold(mut self, threshold: f32) -> Self {
        self.drag_threshold = threshold.max(0.0);
        self
    }
}

impl<Region> PointerGestureState<Region>
where
    Region: Clone + PartialEq,
{
    /// Handles an iced event and emits normalized pointer gestures.
    ///
    /// Mouse and touch events share the same gesture vocabulary. During an
    /// active mouse interaction, touch events are ignored; during an active
    /// touch interaction, mouse events are ignored.
    pub fn handle_event(
        &mut self,
        event: &Event,
        now: Instant,
        region_at: impl Fn(Point) -> Option<Region>,
    ) -> Vec<PointerGesture<Region>> {
        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                self.modifiers = *modifiers;
                Vec::new()
            }
            Event::Keyboard(keyboard::Event::KeyPressed { modifiers, .. })
            | Event::Keyboard(keyboard::Event::KeyReleased { modifiers, .. }) => {
                self.modifiers = *modifiers;
                Vec::new()
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                self.cursor_position = Some(*position);
                if self.active_touch.is_some() {
                    return Vec::new();
                }
                self.handle_cursor_moved(*position)
            }
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                if self.active_touch.is_some() {
                    return Vec::new();
                }
                self.handle_button_pressed((*button).into(), now, region_at)
            }
            Event::Mouse(mouse::Event::ButtonReleased(button)) => {
                if self.active_touch.is_some() {
                    return Vec::new();
                }
                self.handle_button_released((*button).into(), now)
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                if self.active_touch.is_some() {
                    return Vec::new();
                }
                self.cancel_drag()
            }
            Event::Touch(touch::Event::FingerPressed { id, position }) => {
                if self.pressed.is_some() {
                    return Vec::new();
                }
                self.handle_touch_pressed(*id, *position, now, region_at)
            }
            Event::Touch(touch::Event::FingerMoved { id, position }) => {
                self.handle_touch_moved(*id, *position)
            }
            Event::Touch(touch::Event::FingerLifted { id, position }) => {
                self.handle_touch_lifted(*id, *position, now)
            }
            Event::Touch(touch::Event::FingerLost { id, position }) => {
                self.handle_touch_lost(*id, *position)
            }
            _ => Vec::new(),
        }
    }

    /// Cancels an active drag and emits `DragCancelled` when a drag had
    /// started.
    pub fn cancel_drag(&mut self) -> Vec<PointerGesture<Region>> {
        if let Some(pressed) = self.pressed.take() {
            return if pressed.drag_started {
                vec![gesture(
                    PointerGestureKind::DragCancelled,
                    pressed.button,
                    pressed.region,
                    self.cursor_position.unwrap_or(pressed.origin),
                    self.modifiers,
                )]
            } else {
                Vec::new()
            };
        }

        let Some(touch) = self.active_touch.take() else {
            return Vec::new();
        };

        if touch.drag_started {
            vec![gesture(
                PointerGestureKind::DragCancelled,
                PointerButton::Primary,
                touch.region,
                touch.origin,
                self.modifiers,
            )]
        } else {
            Vec::new()
        }
    }

    pub(super) fn next_click_count(
        &self,
        button: PointerButton,
        region: &Region,
        pressed_at: Instant,
        position: Point,
        now: Instant,
    ) -> u8 {
        self.last_click
            .as_ref()
            .filter(|click| click.button == button)
            .filter(|click| &click.region == region)
            .filter(|click| distance(click.position, position) <= self.click_distance)
            .filter(|click| {
                now.duration_since(click.clicked_at) <= self.click_interval
                    && now.duration_since(pressed_at) <= self.click_interval
            })
            .map_or(1, |click| click.count.saturating_add(1))
    }
}

#[derive(Debug, Clone)]
pub(super) struct Pressed<Region> {
    pub(super) button: PointerButton,
    pub(super) region: Region,
    pub(super) origin: Point,
    pub(super) drag_started: bool,
    pub(super) pressed_at: Instant,
}

#[derive(Debug, Clone)]
pub(super) struct ClickMemory<Region> {
    pub(super) button: PointerButton,
    pub(super) region: Region,
    pub(super) position: Point,
    pub(super) clicked_at: Instant,
    pub(super) count: u8,
}

pub(super) fn gesture<Region>(
    kind: PointerGestureKind,
    button: PointerButton,
    region: Region,
    position: Point,
    modifiers: keyboard::Modifiers,
) -> PointerGesture<Region> {
    PointerGesture {
        kind,
        button,
        region,
        position,
        modifiers,
    }
}

pub(super) fn distance(a: Point, b: Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;

    (dx * dx + dy * dy).sqrt()
}
