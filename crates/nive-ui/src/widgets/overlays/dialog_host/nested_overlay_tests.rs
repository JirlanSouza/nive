use super::{DialogHost, DialogInitialFocus};
use crate::{
    accessibility::FocusRoot,
    test_support::{ManagedFocusEntry, WidgetHarness},
    widgets::{
        button,
        overlays::dialog::{DialogAction, DialogActionFooter, DialogTerminalAction},
        overlays::popover::PopoverFocusPolicy,
        Dialog, Popover,
    },
    Element,
};
use iced::{keyboard, mouse, Size};

const VIEWPORT: Size = Size::new(500.0, 500.0);
const CONFIRM_MESSAGE: &str = "confirm";
const DIALOG_ESCAPE: &str = "escape";
const DIALOG_BACKDROP: &str = "backdrop";
const POPOVER_DISMISS: &str = "popover-dismiss";
const INNER_ACTION: &str = "inner-action-pressed";

fn ids() -> (
    iced::widget::Id,
    iced::widget::Id,
    iced::widget::Id,
    iced::widget::Id,
    iced::widget::Id,
) {
    (
        iced::widget::Id::new("invoker"),
        iced::widget::Id::new("popover-anchor"),
        iced::widget::Id::new("inner-action"),
        iced::widget::Id::new("confirm"),
        iced::widget::Id::new("cancel"),
    )
}

fn rooted_host(
    dialog_open: bool,
    popover_open: bool,
    escape_route: Option<&'static str>,
) -> Element<'static, &'static str> {
    let (invoker_id, anchor_id, inner_id, confirm_id, cancel_id) = ids();

    let base = button::primary("Invoker")
        .id(invoker_id)
        .on_press("invoker-pressed");

    let popover = Popover::new(
        button::primary("Assign reviewer")
            .id(anchor_id)
            .on_press("anchor-pressed"),
    )
    .content(
        button::primary("Inner action")
            .id(inner_id)
            .on_press(INNER_ACTION),
    )
    .open(popover_open)
    .focus_policy(PopoverFocusPolicy::Trap)
    .on_dismiss(POPOVER_DISMISS);

    let dialog = Dialog::new(popover).footer(DialogActionFooter::with_one(
        DialogAction::cancel("Cancel", "cancel").id(cancel_id),
        DialogTerminalAction::primary(CONFIRM_MESSAGE, CONFIRM_MESSAGE).id(confirm_id),
    ));

    let host = if dialog_open {
        DialogHost::new(base).dialog(
            dialog,
            Some(DIALOG_BACKDROP),
            escape_route,
            DialogInitialFocus::default(),
        )
    } else {
        DialogHost::new(base)
    };

    FocusRoot::new(host).into()
}

fn harness(dialog_open: bool, popover_open: bool) -> WidgetHarness<'static, &'static str> {
    WidgetHarness::new(
        rooted_host(dialog_open, popover_open, Some(DIALOG_ESCAPE)),
        VIEWPORT,
    )
}

fn escape() -> iced::Event {
    let key = keyboard::Key::Named(keyboard::key::Named::Escape);
    iced::Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Escape),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat: false,
    })
}

fn repeated_escape() -> iced::Event {
    let key = keyboard::Key::Named(keyboard::key::Named::Escape);
    iced::Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Escape),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat: true,
    })
}

fn tab() -> iced::Event {
    let key = keyboard::Key::Named(keyboard::key::Named::Tab);
    iced::Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Tab),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat: false,
    })
}

// --- 4.13: Escape routing through a real nested overlay ---

#[test]
fn escape_with_the_popover_closed_reaches_the_dialog_escape_route() {
    let mut harness = harness(true, false);

    let result = harness
        .update_nested_overlay(escape())
        .expect("dialog overlay");

    assert_eq!(result.messages, vec![DIALOG_ESCAPE]);
    assert!(result.captured);
}

#[test]
fn escape_with_the_popover_open_dismisses_only_the_popover() {
    let mut harness = harness(true, true);

    let result = harness
        .update_nested_overlay(escape())
        .expect("nested overlay chain");

    assert_eq!(result.messages, vec![POPOVER_DISMISS]);
    assert!(result.captured);
}

#[test]
fn a_later_distinct_escape_reaches_the_dialog_after_the_popover_closes() {
    let mut harness = harness(true, true);
    let inner = harness
        .update_nested_overlay(escape())
        .expect("nested overlay chain");
    assert_eq!(inner.messages, vec![POPOVER_DISMISS]);

    // The app reacts to `popover-dismiss` by closing the popover.
    harness.replace(rooted_host(true, false, Some(DIALOG_ESCAPE)));

    let outer = harness
        .update_nested_overlay(escape())
        .expect("dialog overlay");
    assert_eq!(outer.messages, vec![DIALOG_ESCAPE]);
}

#[test]
fn repeated_escape_is_inert_at_every_level() {
    let mut closed_popover = harness(true, false);
    let result = closed_popover
        .update_nested_overlay(repeated_escape())
        .expect("dialog overlay");
    assert!(result.messages.is_empty());

    // The Popover itself does not special-case OS key-repeat (any
    // Escape `KeyPressed` requests dismissal); it only ignores a
    // *second* dismissal request once one has already fired. Trigger
    // the popover's real dismissal first, then confirm a repeated
    // Escape after that is inert too.
    let mut open_popover = harness(true, true);
    let first = open_popover
        .update_nested_overlay(escape())
        .expect("nested overlay chain");
    assert_eq!(first.messages, vec![POPOVER_DISMISS]);

    let result = open_popover
        .update_nested_overlay(repeated_escape())
        .expect("nested overlay chain");
    assert!(result.messages.is_empty());
}

#[test]
fn absent_dialog_escape_route_is_still_captured_but_publishes_nothing() {
    let mut harness = WidgetHarness::new(rooted_host(true, false, None), VIEWPORT);

    let result = harness
        .update_nested_overlay(escape())
        .expect("dialog overlay");

    assert!(result.messages.is_empty());
    assert!(result.captured);
}

#[test]
fn confirming_the_dialog_action_publishes_exactly_one_message_not_a_dismissal_too() {
    let mut harness = harness(true, false);
    // The body's Popover trigger is the initial-focus target and would
    // itself consume Enter as a focused button; move focus off it and
    // onto the footer's terminal Confirm action first (body trigger ->
    // Cancel -> Confirm) so this exercises the footer's own Enter
    // default specifically.
    let _ = harness.focused_overlay_ids();
    assert!(harness.focus_overlay_next());
    assert!(harness.focus_overlay_next());
    let _ = harness.focused_overlay_ids();

    let key = keyboard::Key::Named(keyboard::key::Named::Enter);
    let enter = iced::Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Enter),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: None,
        repeat: false,
    });

    let result = harness.update_overlay(enter).expect("open dialog overlay");

    assert_eq!(result.messages, vec![CONFIRM_MESSAGE]);
}

// --- 4.15: persistent-tree bounds/composition with a real nested overlay ---

#[test]
fn nested_popover_is_tracked_as_a_second_overlay_level() {
    let mut harness = harness(true, true);

    let bounds = harness.nested_overlay_bounds();

    assert_eq!(bounds.len(), 2, "dialog level, then the popover level");
}

#[test]
fn closed_popover_contributes_no_second_overlay_level() {
    let mut harness = harness(true, false);

    let bounds = harness.nested_overlay_bounds();

    assert_eq!(bounds.len(), 1, "dialog level only");
}

#[test]
fn pointer_press_inside_the_nested_popover_is_not_treated_as_dialog_backdrop() {
    let mut harness = harness(true, true);
    let bounds = harness.nested_overlay_bounds();
    let popover_bounds = bounds[1];
    harness.set_cursor(popover_bounds.center());

    let result = harness
        .update_nested_overlay(iced::Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .expect("nested overlay chain");

    assert!(result.messages.is_empty());
    assert!(result.captured, "must not click through to base content");
}

// --- 5.9: forward traversal across body, footer, and the nested overlay ---

#[test]
fn tab_cycles_forward_between_the_body_trigger_and_the_footer_action() {
    let (_, anchor_id, _, confirm_id, cancel_id) = ids();
    let mut harness = harness(true, false);

    // First call settles: it triggers the coordinator binding a
    // `FocusRoot` ancestor performs on this same pass, so the initial
    // target selected before binding is re-confirmed through the
    // now-bound coordinator. The second call observes the result.
    let _ = harness.focused_overlay_ids();
    assert_eq!(harness.focused_overlay_ids(), vec![anchor_id]);

    // Body trigger -> footer Cancel -> footer terminal Confirm.
    assert!(harness.focus_overlay_next());
    let _ = harness.focused_overlay_ids();
    assert_eq!(harness.focused_overlay_ids(), vec![cancel_id]);

    assert!(harness.focus_overlay_next());
    let _ = harness.focused_overlay_ids();
    assert_eq!(harness.focused_overlay_ids(), vec![confirm_id]);
}

#[test]
fn shift_tab_cycles_backward_between_the_footer_action_and_the_body_trigger() {
    let (_, anchor_id, _, confirm_id, cancel_id) = ids();
    let mut harness = harness(true, false);

    // Reach the footer's terminal Confirm action the same way the
    // forward test does, then reverse from there: Confirm -> Cancel ->
    // body trigger, proving Shift+Tab retraces the same modal scope
    // rather than a distinct or wrapped-around traversal order.
    let _ = harness.focused_overlay_ids();
    assert!(harness.focus_overlay_next());
    assert!(harness.focus_overlay_next());
    let _ = harness.focused_overlay_ids();
    assert_eq!(harness.focused_overlay_ids(), vec![confirm_id]);

    assert!(harness.focus_overlay_previous());
    let _ = harness.focused_overlay_ids();
    assert_eq!(harness.focused_overlay_ids(), vec![cancel_id]);

    assert!(harness.focus_overlay_previous());
    let _ = harness.focused_overlay_ids();
    assert_eq!(harness.focused_overlay_ids(), vec![anchor_id]);
}

#[test]
fn tab_inside_the_open_popover_stays_trapped_and_never_reaches_the_footer() {
    let (_, _, inner_id, _, _) = ids();
    let mut harness = harness(true, true);

    harness
        .update_nested_overlay(tab())
        .expect("nested overlay chain");

    // `PopoverFocusPolicy::Trap` keeps the only focusable target (the
    // inner action) focused; a base-layer `focused_overlay_ids()` check
    // would incorrectly still report the dialog's own last state, so
    // this asserts against the nested chain specifically. The first
    // read settles the `FocusRoot` coordinator binding for this fresh
    // `overlay::Nested` chain; the second observes the settled result.
    let _ = harness.focused_nested_overlay_ids();
    assert_eq!(harness.focused_nested_overlay_ids(), vec![inner_id]);
}

// --- 5.12: cross-change focus lifecycle with a real nested overlay ---

#[test]
fn full_lifecycle_composes_initial_entry_trap_inner_escape_outer_close_and_anchor_return() {
    let (invoker_id, anchor_id, inner_id, _, _) = ids();
    let mut harness = WidgetHarness::new(rooted_host(false, false, Some(DIALOG_ESCAPE)), VIEWPORT);

    // Focus the invoker before the dialog opens.
    harness.focus(invoker_id.clone());

    // Closed -> open: capture the invoker, resolve initial focus to the
    // first safe body target (the Popover's own anchor button). The
    // first call settles the `FocusRoot` coordinator binding; see
    // `tab_cycles_forward_between_the_body_trigger_and_the_footer_action`.
    harness.replace(rooted_host(true, false, Some(DIALOG_ESCAPE)));
    let _ = harness.focused_overlay_ids();
    assert_eq!(harness.focused_overlay_ids(), vec![anchor_id]);

    // Open the nested Popover; its own trap policy takes over focus.
    harness.replace(rooted_host(true, true, Some(DIALOG_ESCAPE)));
    harness
        .update_nested_overlay(tab())
        .expect("nested overlay chain");
    let _ = harness.focused_nested_overlay_ids();
    assert_eq!(harness.focused_nested_overlay_ids(), vec![inner_id]);

    // Inner Escape closes only the Popover.
    let inner_escape = harness
        .update_nested_overlay(escape())
        .expect("nested overlay chain");
    assert_eq!(inner_escape.messages, vec![POPOVER_DISMISS]);
    harness.replace(rooted_host(true, false, Some(DIALOG_ESCAPE)));
    assert!(harness.has_overlay(), "dialog remains open");

    // Outer Escape now reaches the Dialog and closes it.
    let outer_escape = harness
        .update_nested_overlay(escape())
        .expect("dialog overlay");
    assert_eq!(outer_escape.messages, vec![DIALOG_ESCAPE]);
    harness.replace(rooted_host(false, false, Some(DIALOG_ESCAPE)));
    assert!(!harness.has_overlay());

    // Anchor-only return: the invoker is restored as an inactive
    // logical anchor, never actively or visibly focused.
    let snapshot = harness.managed_focus();
    let restored = snapshot
        .entries
        .into_iter()
        .find(|entry: &ManagedFocusEntry| entry.id.as_ref() == Some(&invoker_id))
        .expect("invoker remains a managed focus target");
    assert!(restored.anchor_only);
    assert!(!restored.active);
    assert!(!restored.visible);
}
