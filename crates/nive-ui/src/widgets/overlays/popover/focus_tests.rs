use super::*;
use crate::{accessibility::FocusRoot, test_support::WidgetHarness, widgets::button};
use iced::{
    keyboard::{self, key},
    mouse, touch, Event, Point, Size,
};

#[test]
fn focus_first_escape_restores_the_opaque_anchor_once() {
    let anchor = iced::widget::Id::new("popover-anchor");
    let mut harness = rooted_popover_harness(true, PopoverFocusPolicy::FocusFirst);

    harness.focus(anchor.clone());
    enter_overlay_focus(&mut harness);
    assert!(!managed_target(&mut harness, &anchor).active);
    assert_eq!(
        harness
            .focused_overlay_count()
            .and_then(|count| count.focused),
        Some(0)
    );

    let dismissed = harness
        .update_overlay(key_pressed(key::Named::Escape, key::Code::Escape))
        .expect("open Popover overlay");
    assert_eq!(dismissed.messages, vec!["dismiss"]);

    harness.replace(rooted_popover(false, PopoverFocusPolicy::FocusFirst));
    assert!(!harness.has_overlay());
    let restored = managed_target(&mut harness, &anchor);
    assert!(restored.active);
    assert!(restored.visible);

    assert!(!harness.has_overlay());
    assert!(managed_target(&mut harness, &anchor).active);
}

#[test]
fn newer_programmatic_target_prevents_escape_restoration() {
    let anchor = iced::widget::Id::new("popover-anchor");
    let newer = iced::widget::Id::new("popover-newer");
    let mut harness = rooted_popover_harness(true, PopoverFocusPolicy::FocusFirst);

    harness.focus(anchor.clone());
    enter_overlay_focus(&mut harness);
    harness
        .update_overlay(key_pressed(key::Named::Escape, key::Code::Escape))
        .expect("open Popover overlay");
    harness.focus(newer.clone());

    harness.replace(rooted_popover(false, PopoverFocusPolicy::FocusFirst));
    assert!(!harness.has_overlay());
    assert!(!managed_target(&mut harness, &anchor).active);
    assert!(managed_target(&mut harness, &newer).active);
}

#[test]
fn ordinary_focus_first_tab_leave_never_restores_over_destination() {
    let anchor = iced::widget::Id::new("popover-anchor");
    let destination = iced::widget::Id::new("popover-newer");
    let mut harness = rooted_popover_harness(true, PopoverFocusPolicy::FocusFirst);

    harness.focus(anchor.clone());
    enter_overlay_focus(&mut harness);
    assert!(harness.focus_overlay_next());
    let dismissed = harness
        .update_overlay(key_pressed(key::Named::Tab, key::Code::Tab))
        .expect("open Popover overlay");
    assert_eq!(dismissed.messages, vec!["dismiss"]);
    assert!(!dismissed.captured);
    harness.focus(destination.clone());

    harness.replace(rooted_popover(false, PopoverFocusPolicy::FocusFirst));
    assert!(!harness.has_overlay());
    assert!(!managed_target(&mut harness, &anchor).active);
    assert!(managed_target(&mut harness, &destination).active);
}

#[test]
fn trap_cycles_inside_the_shared_root_domain() {
    let mut harness = rooted_popover_harness(true, PopoverFocusPolicy::Trap);
    harness.focus(iced::widget::Id::new("popover-anchor"));
    enter_overlay_focus(&mut harness);
    assert_eq!(
        harness.focused_overlay_count(),
        Some(iced::advanced::widget::operation::focusable::Count {
            total: 2,
            focused: Some(0),
        })
    );

    let first_tab = harness
        .update_overlay(key_pressed(key::Named::Tab, key::Code::Tab))
        .expect("open Popover overlay");
    assert!(first_tab.captured);
    assert_eq!(
        harness
            .focused_overlay_count()
            .and_then(|count| count.focused),
        Some(1)
    );

    let wrapped = harness
        .update_overlay(key_pressed(key::Named::Tab, key::Code::Tab))
        .expect("open Popover overlay");
    assert!(wrapped.captured);
    assert_eq!(
        harness
            .focused_overlay_count()
            .and_then(|count| count.focused),
        Some(0)
    );
}

#[test]
fn retain_anchor_never_moves_focus_into_overlay() {
    let anchor = iced::widget::Id::new("popover-anchor");
    let mut harness = rooted_popover_harness(true, PopoverFocusPolicy::RetainAnchor);

    harness.focus(anchor.clone());
    enter_overlay_focus(&mut harness);

    assert!(managed_target(&mut harness, &anchor).active);
    assert_eq!(
        harness
            .focused_overlay_count()
            .and_then(|count| count.focused),
        None
    );
}

#[test]
fn outside_mouse_and_touch_restore_the_opaque_anchor() {
    let anchor = iced::widget::Id::new("popover-anchor");
    let events = [
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Touch(touch::Event::FingerPressed {
            id: touch::Finger(7),
            position: Point::new(340.0, 220.0),
        }),
    ];

    for event in events {
        let mut harness = rooted_popover_harness(true, PopoverFocusPolicy::FocusFirst);
        harness.focus(anchor.clone());
        enter_overlay_focus(&mut harness);
        harness.set_cursor(Point::new(340.0, 220.0));

        let dismissed = harness.update_overlay(event).expect("open Popover overlay");
        assert_eq!(dismissed.messages, vec!["dismiss"]);
        assert!(dismissed.captured);

        harness.replace(rooted_popover(false, PopoverFocusPolicy::FocusFirst));
        assert!(!harness.has_overlay());
        assert!(managed_target(&mut harness, &anchor).active);
    }
}

#[test]
fn owned_keyboard_activation_restores_after_controlled_close() {
    let anchor = iced::widget::Id::new("popover-anchor");
    let mut harness = rooted_popover_harness(true, PopoverFocusPolicy::FocusFirst);
    harness.focus(anchor.clone());
    enter_overlay_focus(&mut harness);

    let activated = harness
        .update_overlay(key_pressed(key::Named::Enter, key::Code::Enter))
        .expect("open Popover overlay");
    assert_eq!(activated.messages, vec!["first"]);

    harness.replace(rooted_popover(false, PopoverFocusPolicy::FocusFirst));
    assert!(!harness.has_overlay());
    assert!(managed_target(&mut harness, &anchor).active);
}

#[test]
fn programmatic_close_is_silent_and_does_not_restore() {
    let anchor = iced::widget::Id::new("popover-anchor");
    let mut harness = rooted_popover_harness(true, PopoverFocusPolicy::FocusFirst);
    harness.focus(anchor.clone());
    enter_overlay_focus(&mut harness);

    harness.replace(rooted_popover(false, PopoverFocusPolicy::FocusFirst));
    assert!(!harness.has_overlay());
    let anchor = managed_target(&mut harness, &anchor);
    assert!(!anchor.active);
    assert!(!anchor.anchor_only);
}

#[test]
fn trigger_reactivation_target_is_preserved_on_close() {
    let anchor = iced::widget::Id::new("popover-anchor");
    let mut harness = rooted_popover_harness(true, PopoverFocusPolicy::FocusFirst);
    harness.focus(anchor.clone());
    enter_overlay_focus(&mut harness);

    harness.focus(anchor.clone());
    harness.replace(rooted_popover(false, PopoverFocusPolicy::FocusFirst));
    assert!(!harness.has_overlay());
    assert!(managed_target(&mut harness, &anchor).active);
}

#[test]
fn removed_anchor_is_diagnosed_and_never_restored() {
    let anchor = iced::widget::Id::new("popover-anchor");
    let mut harness = rooted_popover_harness(true, PopoverFocusPolicy::FocusFirst);
    harness.focus(anchor);
    enter_overlay_focus(&mut harness);
    assert!(harness.state_at::<widget::PopoverState>(&[0, 0]).was_open);
    assert!(harness
        .state_at::<widget::PopoverState>(&[0, 0])
        .captured_target
        .is_some());
    harness
        .update_overlay(key_pressed(key::Named::Escape, key::Code::Escape))
        .expect("open Popover overlay");

    harness.replace(rooted_popover_with_anchor(
        false,
        PopoverFocusPolicy::FocusFirst,
        false,
    ));
    let snapshot = harness.managed_focus();

    assert!(!snapshot.entries.iter().any(|entry| entry.active));
    assert!(
        !harness
            .state_at::<widget::PopoverState>(&[0, 0])
            .captured_target_available
    );
    assert!(!harness.state_at::<widget::PopoverState>(&[0, 0]).was_open);
    assert!(
        harness
            .state_at::<widget::PopoverState>(&[0, 0])
            .invalid_anchor
    );
}

fn rooted_popover_harness(
    open: bool,
    focus_policy: PopoverFocusPolicy,
) -> WidgetHarness<'static, &'static str> {
    WidgetHarness::new(rooted_popover(open, focus_policy), Size::new(360.0, 240.0))
}

fn rooted_popover(open: bool, focus_policy: PopoverFocusPolicy) -> Element<'static, &'static str> {
    rooted_popover_with_anchor(open, focus_policy, true)
}

fn rooted_popover_with_anchor(
    open: bool,
    focus_policy: PopoverFocusPolicy,
    anchor_enabled: bool,
) -> Element<'static, &'static str> {
    let anchor = button::primary("Anchor")
        .id(iced::widget::Id::new("popover-anchor"))
        .disabled(!anchor_enabled)
        .on_press("toggle");
    let content = iced::widget::Column::with_children(vec![
        button::primary("First")
            .id(iced::widget::Id::new("popover-first"))
            .on_press("first")
            .into(),
        button::primary("Second")
            .id(iced::widget::Id::new("popover-second"))
            .on_press("second")
            .into(),
    ]);
    let popover = Popover::new(anchor)
        .content(content)
        .open(open)
        .focus_policy(focus_policy)
        .on_dismiss("dismiss");
    let content = iced::widget::Column::with_children(vec![
        popover.into(),
        button::primary("Newer")
            .id(iced::widget::Id::new("popover-newer"))
            .on_press("newer")
            .into(),
    ]);
    FocusRoot::new(content).into()
}

fn enter_overlay_focus(harness: &mut WidgetHarness<'static, &'static str>) {
    harness
        .update_overlay(Event::Window(iced::window::Event::RedrawRequested(
            iced::time::Instant::now(),
        )))
        .expect("open Popover overlay");
}

fn managed_target(
    harness: &mut WidgetHarness<'static, &'static str>,
    id: &iced::widget::Id,
) -> crate::test_support::ManagedFocusEntry {
    harness
        .managed_focus()
        .entries
        .into_iter()
        .find(|entry| entry.id.as_ref() == Some(id))
        .unwrap_or_else(|| panic!("missing managed target {id:?}"))
}

fn key_pressed(named: key::Named, code: key::Code) -> Event {
    let key = keyboard::Key::Named(named);
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: key::Physical::Code(code),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat: false,
    })
}
