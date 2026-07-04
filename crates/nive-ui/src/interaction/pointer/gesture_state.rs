use std::time::{Duration, Instant};

use iced::{keyboard, mouse, Event, Point};

use super::{PointerButton, PointerGesture, PointerGestureKind};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct PointerGestureState<Region> {
    modifiers: keyboard::Modifiers,
    cursor_position: Option<Point>,
    pressed: Option<Pressed<Region>>,
    last_click: Option<ClickMemory<Region>>,
    drag_threshold: f32,
    click_distance: f32,
    click_interval: Duration,
}

impl<Region> Default for PointerGestureState<Region> {
    fn default() -> Self {
        Self {
            modifiers: keyboard::Modifiers::NONE,
            cursor_position: None,
            pressed: None,
            last_click: None,
            drag_threshold: 4.0,
            click_distance: 4.0,
            click_interval: Duration::from_millis(500),
        }
    }
}

#[allow(dead_code)]
impl<Region> PointerGestureState<Region> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_drag_threshold(mut self, threshold: f32) -> Self {
        self.drag_threshold = threshold.max(0.0);
        self
    }
}

#[allow(dead_code)]
impl<Region> PointerGestureState<Region>
where
    Region: Clone + PartialEq,
{
    pub(crate) fn handle_event(
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
                self.handle_cursor_moved(*position)
            }
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                self.handle_button_pressed((*button).into(), now, region_at)
            }
            Event::Mouse(mouse::Event::ButtonReleased(button)) => {
                self.handle_button_released((*button).into(), now)
            }
            Event::Mouse(mouse::Event::CursorLeft) => self.cancel_drag(),
            _ => Vec::new(),
        }
    }

    pub(crate) fn cancel_drag(&mut self) -> Vec<PointerGesture<Region>> {
        let Some(pressed) = self.pressed.take() else {
            return Vec::new();
        };

        if pressed.drag_started {
            vec![gesture(
                PointerGestureKind::DragCancelled,
                pressed.button,
                pressed.region,
                self.cursor_position.unwrap_or(pressed.origin),
                self.modifiers,
            )]
        } else {
            Vec::new()
        }
    }

    fn handle_button_pressed(
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

    fn handle_cursor_moved(&mut self, position: Point) -> Vec<PointerGesture<Region>> {
        let Some(pressed) = &mut self.pressed else {
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

    fn handle_button_released(
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
            let count = self.next_click_count(&pressed, position, now);
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

    fn next_click_count(&self, pressed: &Pressed<Region>, position: Point, now: Instant) -> u8 {
        self.last_click
            .as_ref()
            .filter(|click| click.button == pressed.button)
            .filter(|click| click.region == pressed.region)
            .filter(|click| distance(click.position, position) <= self.click_distance)
            .filter(|click| {
                now.duration_since(click.clicked_at) <= self.click_interval
                    && now.duration_since(pressed.pressed_at) <= self.click_interval
            })
            .map_or(1, |click| click.count.saturating_add(1))
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Pressed<Region> {
    button: PointerButton,
    region: Region,
    origin: Point,
    drag_started: bool,
    pressed_at: Instant,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ClickMemory<Region> {
    button: PointerButton,
    region: Region,
    position: Point,
    clicked_at: Instant,
    count: u8,
}

#[allow(dead_code)]
fn gesture<Region>(
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

#[allow(dead_code)]
fn distance(a: Point, b: Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;

    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod pointer_tests {
    use super::*;

    fn moved(x: f32, y: f32) -> Event {
        Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(x, y),
        })
    }

    fn press(button: mouse::Button) -> Event {
        Event::Mouse(mouse::Event::ButtonPressed(button))
    }

    fn release(button: mouse::Button) -> Event {
        Event::Mouse(mouse::Event::ButtonReleased(button))
    }

    #[test]
    fn maps_iced_buttons_to_pointer_buttons() {
        assert_eq!(
            PointerButton::from(mouse::Button::Left),
            PointerButton::Primary
        );
        assert_eq!(
            PointerButton::from(mouse::Button::Right),
            PointerButton::Secondary
        );
        assert_eq!(
            PointerButton::from(mouse::Button::Middle),
            PointerButton::Middle
        );
        assert_eq!(
            PointerButton::from(mouse::Button::Back),
            PointerButton::Auxiliary(4)
        );
        assert_eq!(
            PointerButton::from(mouse::Button::Forward),
            PointerButton::Auxiliary(5)
        );
        assert_eq!(
            PointerButton::from(mouse::Button::Other(8)),
            PointerButton::Auxiliary(8)
        );
    }

    #[test]
    fn normalizes_click_count() {
        let mut state = PointerGestureState::new();
        let now = Instant::now();

        state.handle_event(&moved(1.0, 1.0), now, |_| Some("row"));
        state.handle_event(&press(mouse::Button::Left), now, |_| Some("row"));
        let first = state.handle_event(
            &release(mouse::Button::Left),
            now + Duration::from_millis(20),
            |_| Some("row"),
        );

        state.handle_event(
            &press(mouse::Button::Left),
            now + Duration::from_millis(120),
            |_| Some("row"),
        );
        let second = state.handle_event(
            &release(mouse::Button::Left),
            now + Duration::from_millis(140),
            |_| Some("row"),
        );

        assert!(matches!(
            first.last().map(|gesture| gesture.kind),
            Some(PointerGestureKind::Clicked { count: 1 })
        ));
        assert!(matches!(
            second.last().map(|gesture| gesture.kind),
            Some(PointerGestureKind::Clicked { count: 2 })
        ));
    }

    #[test]
    fn waits_for_drag_threshold_then_moves_and_releases() {
        let mut state = PointerGestureState::new().with_drag_threshold(5.0);
        let now = Instant::now();

        state.handle_event(&moved(0.0, 0.0), now, |_| Some("row"));
        state.handle_event(&press(mouse::Button::Left), now, |_| Some("row"));
        assert!(state
            .handle_event(&moved(3.0, 0.0), now, |_| Some("row"))
            .is_empty());

        let started = state.handle_event(&moved(6.0, 0.0), now, |_| Some("row"));
        let released = state.handle_event(&release(mouse::Button::Left), now, |_| Some("row"));

        assert_eq!(
            started
                .iter()
                .map(|gesture| gesture.kind)
                .collect::<Vec<_>>(),
            vec![
                PointerGestureKind::DragStarted,
                PointerGestureKind::DragMoved
            ]
        );
        assert_eq!(
            released
                .iter()
                .map(|gesture| gesture.kind)
                .collect::<Vec<_>>(),
            vec![
                PointerGestureKind::Released,
                PointerGestureKind::DragReleased
            ]
        );
    }

    #[test]
    fn cancels_active_drag() {
        let mut state = PointerGestureState::new().with_drag_threshold(1.0);
        let now = Instant::now();

        state.handle_event(&moved(0.0, 0.0), now, |_| Some("row"));
        state.handle_event(&press(mouse::Button::Left), now, |_| Some("row"));
        state.handle_event(&moved(2.0, 0.0), now, |_| Some("row"));
        let cancelled = state.handle_event(&Event::Mouse(mouse::Event::CursorLeft), now, |_| {
            Some("row")
        });

        assert_eq!(cancelled[0].kind, PointerGestureKind::DragCancelled);
    }

    #[test]
    fn captures_current_modifiers() {
        let mut state = PointerGestureState::new();
        let now = Instant::now();

        state.handle_event(
            &Event::Keyboard(keyboard::Event::ModifiersChanged(
                keyboard::Modifiers::SHIFT,
            )),
            now,
            |_| Some("row"),
        );
        state.handle_event(&moved(0.0, 0.0), now, |_| Some("row"));
        let gestures = state.handle_event(&press(mouse::Button::Left), now, |_| Some("row"));

        assert_eq!(gestures[0].modifiers, keyboard::Modifiers::SHIFT);
    }
}
