use std::borrow::Cow;

use iced::{
    advanced::{
        layout::Limits,
        widget::{operation, Id, Tree},
        Layout,
    },
    Font, Pixels,
};
use nive_ui::{
    accessibility::{FocusDirection, FocusRoot},
    widgets::Input,
};

use super::*;

struct FocusHarness<'a, Message> {
    element: Element<'a, Message>,
    tree: Tree,
    node: Node,
    renderer: nive_ui::Renderer,
    cursor: mouse::Cursor,
}

impl<'a, Message> FocusHarness<'a, Message> {
    fn new(mut element: Element<'a, Message>) -> Self {
        let mut tree = Tree::new(&element);
        let renderer = iced_renderer::fallback::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::default(),
            Pixels(14.0),
        ));
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &Limits::new(Size::ZERO, Size::new(480.0, 180.0)),
        );
        Self {
            element,
            tree,
            node,
            renderer,
            cursor: mouse::Cursor::Unavailable,
        }
    }

    fn set_cursor(&mut self, position: iced::Point) {
        self.cursor = mouse::Cursor::Available(position);
    }

    fn update(&mut self, event: Event) -> Vec<Message> {
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        let mut clipboard = iced::advanced::clipboard::Null;
        let viewport = Layout::new(&self.node).bounds();
        self.element.as_widget_mut().update(
            &mut self.tree,
            &event,
            Layout::new(&self.node),
            self.cursor,
            &self.renderer,
            &mut clipboard,
            &mut shell,
            &viewport,
        );
        drop(shell);
        messages
    }

    fn operate(&mut self, operation: &mut dyn operation::Operation) {
        self.element.as_widget_mut().operate(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            operation,
        );
    }

    fn focus(&mut self, id: Id) {
        self.operate(&mut operation::focusable::focus(id));
    }

    fn navigate(&mut self, direction: FocusDirection) {
        direction.operate(|operation| self.operate(operation));
    }

    fn focused_targets(&mut self) -> Vec<Option<Id>> {
        #[derive(Default)]
        struct FocusedTargets(Vec<Option<Id>>);

        impl operation::Operation for FocusedTargets {
            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
                operate(self);
            }

            fn focusable(
                &mut self,
                id: Option<&Id>,
                _bounds: Rectangle,
                state: &mut dyn operation::Focusable,
            ) {
                if state.is_focused() {
                    self.0.push(id.cloned());
                }
            }
        }

        let mut focused = FocusedTargets::default();
        self.operate(&mut focused);
        focused.0
    }

    fn track_state(&self) -> &TrackState {
        self.tree.children[0].children[1]
            .state
            .downcast_ref::<TrackState>()
    }

    fn track_item_bounds(&self, index: usize) -> Rectangle {
        let track = Layout::new(&self.node)
            .children()
            .nth(1)
            .expect("bottom track layout");
        current_item_bounds(track)[index]
    }
}

fn tab(label: &'static str, disabled: bool) -> BottomHeaderTab<'static, &'static str> {
    BottomHeaderTab {
        panel_id: label,
        label: Cow::Borrowed(label),
        icon: None,
        badge: None,
        status: None,
        disabled,
        tooltip: None,
    }
}

#[test]
fn composite_focus_starts_active_and_skips_disabled_tabs() {
    let track = BottomPanelTabTrack::new(ControlSize::Sm, 1)
        .push(tab("Output", false), Some(1_u8))
        .push(tab("Problems", false), Some(2))
        .push(tab("Disabled", true), None)
        .push(tab("Terminal", false), Some(4));
    let mut state = TrackState::default();

    track.reconcile_focus(&mut state);
    assert_eq!(state.focused_index, Some(1));
    assert_eq!(track.enabled_indices(), vec![0, 1, 3]);
}

#[test]
fn traversal_crosses_standard_controls_without_changing_controlled_tab() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Message {
        Changed,
        Selected(usize),
    }

    let before = Id::new("before-bottom-track");
    let after = Id::new("after-bottom-track");
    let track = BottomPanelTabTrack::new(ControlSize::Sm, 1)
        .push(tab("Output", false), Some(Message::Selected(0)))
        .push(tab("Problems", false), Some(Message::Selected(1)))
        .push(tab("Terminal", false), Some(Message::Selected(2)));
    let content = iced::widget::Column::with_children(vec![
        Input::new("Before", "")
            .id(before.clone())
            .on_change(|_| Message::Changed)
            .into(),
        track.into(),
        Input::new("After", "")
            .id(after.clone())
            .on_change(|_| Message::Changed)
            .into(),
    ]);
    let mut harness = FocusHarness::new(FocusRoot::new(content).into());

    harness.focus(before.clone());
    assert_eq!(harness.focused_targets(), vec![Some(before)]);

    harness.navigate(FocusDirection::Next);
    assert_eq!(harness.focused_targets(), vec![None]);
    assert!(harness.track_state().focus.is_active());
    assert_eq!(harness.track_state().focused_index, Some(1));
    assert_eq!(harness.track_state().last_active_index, Some(1));

    harness.navigate(FocusDirection::Next);
    assert_eq!(harness.focused_targets(), vec![Some(after)]);
    assert!(!harness.track_state().focus.is_active());
    assert_eq!(harness.track_state().focused_index, Some(1));
    assert_eq!(harness.track_state().last_active_index, Some(1));

    harness.navigate(FocusDirection::Previous);
    assert_eq!(harness.focused_targets(), vec![None]);
    assert!(harness.track_state().focus.is_active());
    assert_eq!(harness.track_state().focused_index, Some(1));
}

#[test]
fn pointer_hit_test_uses_the_track_position_inside_its_parent() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Message {
        Selected(usize),
    }

    let track = BottomPanelTabTrack::new(ControlSize::Sm, 0)
        .push(tab("Problems", false), Some(Message::Selected(0)))
        .push(tab("Logs", false), Some(Message::Selected(1)))
        .push(tab("Events", false), Some(Message::Selected(2)));
    let content = iced::widget::Column::with_children(vec![
        iced::widget::Space::new().height(64).into(),
        track.into(),
        iced::widget::Space::new().height(24).into(),
    ]);
    let mut harness = FocusHarness::new(FocusRoot::new(content).into());
    let logs = harness.track_item_bounds(1);

    assert!(logs.y >= 64.0);
    harness.set_cursor(logs.center());
    assert!(harness
        .update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .is_empty());
    assert!(harness.track_state().focus.is_active());
    assert_eq!(harness.track_state().focused_index, Some(1));

    assert_eq!(
        harness.update(Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        ))),
        vec![Message::Selected(1)]
    );

    assert!(harness
        .update(Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::ArrowRight),
            modified_key: keyboard::Key::Named(keyboard::key::Named::ArrowRight),
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::ArrowRight),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::NONE,
            text: None,
            repeat: false,
        }))
        .is_empty());
    assert_eq!(harness.track_state().focused_index, Some(2));
    assert!(harness.track_state().focus.is_focus_visible());
}

#[test]
fn vertical_wheel_maps_to_horizontal_motion() {
    assert_eq!(
        horizontal_wheel(mouse::ScrollDelta::Lines { x: 0.0, y: 2.0 }),
        48.0
    );
    assert_eq!(
        horizontal_wheel(mouse::ScrollDelta::Pixels { x: 3.0, y: 20.0 }),
        3.0
    );
}

#[test]
fn reveal_uses_minimum_displacement_and_clamps_offset() {
    let mut state = TrackState {
        item_bounds: vec![Rectangle::new(
            iced::Point::new(140.0, 0.0),
            Size::new(80.0, 28.0),
        )],
        viewport_width: 180.0,
        max_offset: 100.0,
        ..TrackState::default()
    };

    reveal_index(&mut state, 0);
    assert_eq!(state.offset, 40.0);
}
