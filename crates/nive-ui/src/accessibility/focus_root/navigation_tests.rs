use iced::{
    advanced::{
        layout, mouse, renderer,
        widget::{operation, tree, Id, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    keyboard, touch, Event, Length, Point, Rectangle, Size,
};

use super::FocusRoot;
use crate::{
    advanced::focus::{FocusState, FocusVisibility},
    test_support::WidgetHarness,
    widgets::{button, Input, RadioGroup, RadioOption},
    Element, Renderer, Theme,
};

#[derive(Debug, Clone)]
struct ManagedControl {
    id: Option<Id>,
    visibility: FocusVisibility,
    enabled: bool,
}

impl ManagedControl {
    fn button(id: Id) -> Self {
        Self {
            id: Some(id),
            visibility: FocusVisibility::Auto,
            enabled: true,
        }
    }

    fn input(id: Id) -> Self {
        Self {
            id: Some(id),
            visibility: FocusVisibility::AlwaysWhileActive,
            enabled: true,
        }
    }

    fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    fn unkeyed() -> Self {
        Self {
            id: None,
            visibility: FocusVisibility::Auto,
            enabled: true,
        }
    }
}

impl Widget<(), Theme, Renderer> for ManagedControl {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<FocusState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(FocusState::new(self.visibility))
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(80.0), Length::Fixed(24.0))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(80.0, 24.0))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        if self.enabled {
            tree.state.downcast_mut::<FocusState>().register(
                operation,
                self.id.as_ref(),
                layout.bounds(),
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, ()>,
        _viewport: &Rectangle,
    ) {
        let pressed_over = match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                cursor.is_over(layout.bounds())
            }
            Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                layout.bounds().contains(*position)
            }
            _ => false,
        };
        if self.enabled && pressed_over {
            tree.state.downcast_mut::<FocusState>().focus_from_pointer();
            shell.capture_event();
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }
}

impl From<ManagedControl> for Element<'static, ()> {
    fn from(control: ManagedControl) -> Self {
        Element::new(control)
    }
}

#[derive(Debug, Clone, Copy)]
enum PressOrigin {
    Pointer,
    Touch,
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Next,
    Previous,
}

fn harness(ids: &[Id; 3]) -> WidgetHarness<'static, ()> {
    harness_with(ids, [true; 3])
}

fn harness_with(ids: &[Id; 3], enabled: [bool; 3]) -> WidgetHarness<'static, ()> {
    let content = iced::widget::Column::with_children(vec![
        ManagedControl::button(ids[0].clone())
            .enabled(enabled[0])
            .into(),
        ManagedControl::input(ids[1].clone())
            .enabled(enabled[1])
            .into(),
        ManagedControl::button(ids[2].clone())
            .enabled(enabled[2])
            .into(),
    ])
    .spacing(4);
    WidgetHarness::new(FocusRoot::new(content).into(), Size::new(200.0, 160.0))
}

fn element_with(ids: &[Id; 3], enabled: [bool; 3]) -> Element<'static, ()> {
    let content = iced::widget::Column::with_children(vec![
        ManagedControl::button(ids[0].clone())
            .enabled(enabled[0])
            .into(),
        ManagedControl::input(ids[1].clone())
            .enabled(enabled[1])
            .into(),
        ManagedControl::button(ids[2].clone())
            .enabled(enabled[2])
            .into(),
    ])
    .spacing(4);
    FocusRoot::new(content).into()
}

#[test]
fn traversal_continues_from_anchor_after_empty_pointer_and_touch_presses() {
    let cases = [
        (PressOrigin::Pointer, Direction::Next, 2),
        (PressOrigin::Pointer, Direction::Previous, 0),
        (PressOrigin::Touch, Direction::Next, 2),
        (PressOrigin::Touch, Direction::Previous, 0),
    ];

    for (origin, direction, expected) in cases {
        let ids = [Id::unique(), Id::unique(), Id::unique()];
        let mut harness = harness(&ids);
        let input_bounds = harness.focusable_bounds(&ids[1]).expect("input bounds");
        let input_position = input_bounds.center();
        let empty_position = Point::new(160.0, 140.0);

        match origin {
            PressOrigin::Pointer => {
                harness.set_cursor(input_position);
                harness.update(Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                )));
                harness.set_cursor(empty_position);
                harness.update(Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                )));
            }
            PressOrigin::Touch => {
                harness.update(Event::Touch(touch::Event::FingerPressed {
                    id: touch::Finger(1),
                    position: input_position,
                }));
                harness.update(Event::Touch(touch::Event::FingerPressed {
                    id: touch::Finger(1),
                    position: empty_position,
                }));
            }
        }

        let anchor = harness.managed_focus();
        assert!(anchor
            .entries
            .iter()
            .any(|entry| { entry.id.as_ref() == Some(&ids[1]) && entry.anchor_only }));

        harness.update(Event::Keyboard(keyboard::Event::ModifiersChanged(
            keyboard::Modifiers::default(),
        )));
        match direction {
            Direction::Next => harness.focus_next(),
            Direction::Previous => harness.focus_previous(),
        }

        let focused = harness.managed_focus();
        assert_eq!(
            focused.entries.iter().filter(|entry| entry.active).count(),
            1
        );
        assert!(focused.entries.iter().any(|entry| {
            entry.id.as_ref() == Some(&ids[expected]) && entry.active && entry.visible
        }));
    }
}

#[test]
fn targeted_focus_replaces_shared_ownership() {
    let ids = [Id::unique(), Id::unique(), Id::unique()];
    let mut harness = harness(&ids);

    harness.focus(ids[1].clone());

    let focused = harness.managed_focus();
    assert_eq!(
        focused.entries.iter().filter(|entry| entry.active).count(),
        1
    );
    assert!(focused
        .entries
        .iter()
        .any(|entry| { entry.id.as_ref() == Some(&ids[1]) && entry.active && entry.visible }));
}

#[test]
fn disabled_anchor_is_invalidated_before_native_fallback() {
    let ids = [Id::unique(), Id::unique(), Id::unique()];
    let mut forward = harness(&ids);
    forward.focus(ids[1].clone());

    forward.replace(element_with(&ids, [true, false, true]));
    assert!(!forward
        .managed_focus()
        .entries
        .iter()
        .any(|entry| entry.active));
    forward.focus_next();
    assert!(forward
        .managed_focus()
        .entries
        .iter()
        .any(|entry| entry.id.as_ref() == Some(&ids[0]) && entry.active));

    let mut reverse = harness(&ids);
    reverse.focus(ids[1].clone());
    reverse.replace(element_with(&ids, [true, false, true]));
    reverse.managed_focus();
    reverse.focus_previous();
    assert!(reverse
        .managed_focus()
        .entries
        .iter()
        .any(|entry| entry.id.as_ref() == Some(&ids[2]) && entry.active));
}

#[test]
fn removal_and_id_shift_do_not_transfer_the_removed_anchor() {
    let ids = [Id::unique(), Id::unique(), Id::unique()];
    let mut harness = harness(&ids);
    harness.focus(ids[1].clone());
    let shortened = iced::widget::Column::with_children(vec![
        ManagedControl::button(ids[0].clone()).into(),
        ManagedControl::button(ids[2].clone()).into(),
    ])
    .spacing(4);

    harness.replace(FocusRoot::new(shortened).into());
    let after_removal = harness.managed_focus();
    assert!(!after_removal
        .entries
        .iter()
        .any(|entry| entry.active || entry.anchor_only));
    harness.focus_next();
    assert!(harness
        .managed_focus()
        .entries
        .iter()
        .any(|entry| entry.id.as_ref() == Some(&ids[0]) && entry.active));
}

#[test]
fn keyed_and_unkeyed_persistent_rebuilds_keep_the_same_target_state() {
    let ids = [Id::unique(), Id::unique(), Id::unique()];
    let mut keyed = harness(&ids);
    let keyed_token = keyed.managed_focus().entries[1].token;
    keyed.replace(element_with(&ids, [true; 3]));
    assert_eq!(keyed.managed_focus().entries[1].token, keyed_token);

    let mut unkeyed = WidgetHarness::new(
        FocusRoot::new(ManagedControl::unkeyed()).into(),
        Size::new(200.0, 120.0),
    );
    let unkeyed_token = unkeyed.managed_focus().entries[0].token;
    unkeyed.replace(FocusRoot::new(ManagedControl::unkeyed()).into());
    assert_eq!(unkeyed.managed_focus().entries[0].token, unkeyed_token);
}

mod event_and_mixed_tests;
