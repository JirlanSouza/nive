use iced::{
    advanced::{
        layout, mouse, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    keyboard, Event, Length, Point, Rectangle, Size,
};

use super::{FocusRoot, FocusRootState};
use crate::{
    advanced::focus::FocusState,
    focus::{capture_focus_target, restore_focus_target},
    test_support::WidgetHarness,
    Element, Renderer, Theme,
};

struct FocusProbe;

impl Widget<(), Theme, Renderer> for FocusProbe {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<FocusState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(FocusState::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(40.0), Length::Fixed(20.0))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(40.0, 20.0))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        tree.state
            .downcast_mut::<FocusState>()
            .register(operation, None, layout.bounds());
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
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(iced::touch::Event::FingerPressed { .. })
        ) && cursor.is_over(layout.bounds())
        {
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

impl From<FocusProbe> for Element<'_, ()> {
    fn from(probe: FocusProbe) -> Self {
        Element::new(probe)
    }
}

fn harness() -> WidgetHarness<'static, ()> {
    WidgetHarness::new(FocusRoot::new(FocusProbe).into(), Size::new(200.0, 120.0))
}

#[test]
fn root_is_layout_neutral_and_binds_focus_before_delegated_operations() {
    let mut harness = harness();

    assert_eq!(harness.bounds().size(), Size::new(40.0, 20.0));
    let initial = harness.managed_focus();
    assert_eq!(initial.entries.len(), 1);
    assert_eq!(initial.entries[0].bounds, harness.bounds());
    assert!(initial.entries[0].root_identity.is_some());

    harness.focus_next();
    let focused = harness.managed_focus();
    assert!(focused.entries[0].active);
    assert!(focused.entries[0].visible);
}

#[test]
fn root_observes_pointer_keyboard_and_window_events_without_replacing_child_status() {
    let mut harness = harness();
    harness.set_cursor(Point::new(10.0, 10.0));
    let pointer = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert!(pointer.captured);
    let pointer_focus = harness.managed_focus();
    assert!(pointer_focus.entries[0].active);
    assert!(!pointer_focus.entries[0].visible);

    harness.set_cursor(Point::new(100.0, 100.0));
    let empty = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert!(!empty.captured);
    let anchor = harness.managed_focus();
    assert!(anchor.entries[0].anchor_only);

    let keyboard = harness.update(Event::Keyboard(keyboard::Event::ModifiersChanged(
        keyboard::Modifiers::SHIFT,
    )));
    assert!(!keyboard.captured);
    struct FocusCurrent;
    impl operation::Operation for FocusCurrent {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
            operate(self);
        }

        fn focusable(
            &mut self,
            _id: Option<&iced::advanced::widget::Id>,
            _bounds: Rectangle,
            state: &mut dyn operation::Focusable,
        ) {
            state.focus();
        }
    }
    harness.operate(&mut FocusCurrent);
    assert!(harness.managed_focus().entries[0].visible);

    harness.update(Event::Window(iced::window::Event::Unfocused));
    assert!(harness.managed_focus().entries[0].anchor_only);
    harness.update(Event::Window(iced::window::Event::Focused));
    assert!(harness.managed_focus().entries[0].anchor_only);
}

/// A message published by *this* event (e.g. "open the dialog") only takes
/// effect on the next `view()` rebuild — the modal-active pass in this same
/// `update()` call still sees the old, closed tree, so it can't detect the
/// modal that's about to appear. Nothing re-checks modal activity outside of
/// an `Event`-driven `update()` pass, so without a follow-up redraw, a modal
/// opened by a plain click with no further input afterward would never get
/// detected — and its toasts would never pause. Regression for that gap.
#[test]
fn a_real_event_requests_a_follow_up_redraw_so_a_just_opened_modal_gets_detected() {
    let mut harness = harness();

    let result = harness.update(Event::Mouse(mouse::Event::CursorMoved {
        position: Point::new(1.0, 1.0),
    }));

    assert_ne!(result.redraw_request, iced::window::RedrawRequest::Wait);
}

/// The follow-up redraw itself arrives as a `RedrawRequested` event; it must
/// not request yet another one, or every real event would chain into a
/// self-sustaining redraw loop instead of settling after one extra frame.
#[test]
fn a_redraw_requested_event_does_not_request_another_redraw() {
    let mut harness = harness();

    let result = harness.update(Event::Window(iced::window::Event::RedrawRequested(
        iced::time::Instant::now(),
    )));

    assert_eq!(result.redraw_request, iced::window::RedrawRequest::Wait);
}

#[test]
fn nested_root_is_diagnosed_and_remains_finite() {
    let nested: Element<'static, ()> = FocusRoot::new(FocusRoot::new(FocusProbe)).into();
    let mut harness = WidgetHarness::new(nested, Size::new(200.0, 120.0));

    let snapshot = harness.managed_focus();

    assert_eq!(snapshot.entries.len(), 1);
    assert!(
        harness
            .state_at::<FocusRootState>(&[0])
            .nested_root_detected
    );
}

#[test]
fn opaque_capture_and_restore_operations_reject_a_newer_activation() {
    let mut harness = harness();
    harness.focus_next();
    let mut capture = capture_focus_target();
    let captured = match harness.operation_outcome(&mut capture) {
        operation::Outcome::Some(Some(target)) => target,
        _ => panic!("focused root did not expose an opaque target"),
    };
    assert!(captured.is_valid());

    struct FocusCurrent;
    impl operation::Operation for FocusCurrent {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
            operate(self);
        }

        fn focusable(
            &mut self,
            _id: Option<&iced::advanced::widget::Id>,
            _bounds: Rectangle,
            state: &mut dyn operation::Focusable,
        ) {
            state.focus();
        }
    }

    harness.operate(&mut FocusCurrent);
    let mut capture_expected = capture_focus_target();
    let expected = match harness.operation_outcome(&mut capture_expected) {
        operation::Outcome::Some(Some(target)) => target,
        _ => panic!("expected current target"),
    };
    let mut restore = restore_focus_target(captured.clone(), Some(expected.clone()));
    assert!(matches!(
        harness.operation_outcome(&mut restore),
        operation::Outcome::Some(true)
    ));

    harness.operate(&mut FocusCurrent);
    let mut stale_restore = restore_focus_target(captured, Some(expected));
    assert!(matches!(
        harness.operation_outcome(&mut stale_restore),
        operation::Outcome::Some(false)
    ));
}
