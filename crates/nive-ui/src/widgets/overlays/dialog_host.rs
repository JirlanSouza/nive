mod initial_focus;
mod widget;

pub use initial_focus::DialogInitialFocus;
pub use widget::DialogHost;

#[cfg(test)]
mod dialog_focus_tests {
    use super::{DialogHost, DialogInitialFocus};
    use crate::{
        accessibility::FocusRoot,
        test_support::{ManagedFocusEntry, WidgetHarness},
        widgets::button,
        Element,
    };
    use iced::{keyboard, mouse, Event, Point, Size};

    #[test]
    fn owned_button_close_restores_previous_target_as_anchor_only() {
        let previous = iced::widget::Id::new("dialog-previous");
        let next = iced::widget::Id::new("dialog-next");
        let mut harness = dialog_harness(false, true);

        // Focus the invoker before the dialog opens: base content is only
        // externally operable while no dialog is active. Opening then
        // captures whatever is currently focused as the invoker.
        harness.focus(previous.clone());
        harness.replace(rooted_dialog(true, true));
        click_dialog_button(&mut harness);
        harness.replace(rooted_dialog(false, true));
        assert!(!harness.has_overlay());

        let restored = managed_target(&mut harness, &previous);
        assert!(restored.anchor_only);
        assert!(!restored.active);
        assert!(!restored.visible);

        harness.update(Event::Keyboard(keyboard::Event::ModifiersChanged(
            keyboard::Modifiers::NONE,
        )));
        harness.focus_next();
        let destination = managed_target(&mut harness, &next);
        assert!(destination.active);
        assert!(destination.visible);
    }

    #[test]
    fn newer_programmatic_target_is_not_overwritten_when_dialog_closes() {
        let previous = iced::widget::Id::new("dialog-previous");
        let newer = iced::widget::Id::new("dialog-next");
        let mut harness = dialog_harness(false, true);

        harness.focus(previous.clone());
        harness.replace(rooted_dialog(true, true));
        click_dialog_button(&mut harness);
        // A newer external programmatic target set while the dialog is
        // closing/closed must win over the captured invoker.
        harness.replace(rooted_dialog(false, true));
        harness.focus(newer.clone());
        assert!(!harness.has_overlay());

        let previous = managed_target(&mut harness, &previous);
        let newer = managed_target(&mut harness, &newer);
        assert!(!previous.active);
        assert!(!previous.anchor_only);
        assert!(newer.active);
    }

    #[test]
    fn removed_previous_target_uses_native_no_anchor_fallback() {
        let previous = iced::widget::Id::new("dialog-previous");
        let next = iced::widget::Id::new("dialog-next");
        let mut harness = dialog_harness(false, true);

        harness.focus(previous);
        harness.replace(rooted_dialog(true, true));
        click_dialog_button(&mut harness);
        harness.replace(rooted_dialog(false, false));
        assert!(!harness.has_overlay());

        assert!(harness
            .managed_focus()
            .entries
            .iter()
            .all(|entry| !entry.active && !entry.anchor_only && !entry.visible));
        harness.focus_next();
        assert!(managed_target(&mut harness, &next).active);
    }

    fn dialog_harness(open: bool, include_previous: bool) -> WidgetHarness<'static, &'static str> {
        WidgetHarness::new(
            rooted_dialog(open, include_previous),
            Size::new(360.0, 240.0),
        )
    }

    fn rooted_dialog(open: bool, include_previous: bool) -> Element<'static, &'static str> {
        let mut content = Vec::new();
        if include_previous {
            content.push(
                button::primary("Previous")
                    .id(iced::widget::Id::new("dialog-previous"))
                    .on_press("previous")
                    .into(),
            );
        }
        content.push(
            button::primary("Next")
                .id(iced::widget::Id::new("dialog-next"))
                .on_press("next")
                .into(),
        );
        let host = DialogHost::new(iced::widget::Column::with_children(content));
        let host = if open {
            host.dialog(
                button::primary("Close")
                    .id(iced::widget::Id::new("dialog-close"))
                    .on_press("close"),
                Some("backdrop"),
                Some("escape"),
                DialogInitialFocus::default(),
            )
        } else {
            host
        };

        FocusRoot::new(host).into()
    }

    fn click_dialog_button(harness: &mut WidgetHarness<'static, &'static str>) {
        let bounds = harness.overlay_bounds().expect("open dialog overlay");
        harness.set_cursor(Point::new(bounds.center_x(), bounds.center_y()));
        let pressed = harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )))
            .expect("open dialog overlay");
        assert!(pressed.captured);

        let released = harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonReleased(
                mouse::Button::Left,
            )))
            .expect("open dialog overlay");
        assert_eq!(released.messages, vec!["close"]);
    }

    fn managed_target(
        harness: &mut WidgetHarness<'static, &'static str>,
        id: &iced::widget::Id,
    ) -> ManagedFocusEntry {
        harness
            .managed_focus()
            .entries
            .into_iter()
            .find(|entry| entry.id.as_ref() == Some(id))
            .unwrap_or_else(|| panic!("missing managed target {id:?}"))
    }
}

#[cfg(test)]
mod dialog_boundary_tests {
    use super::{DialogHost, DialogInitialFocus};
    use crate::{test_support::WidgetHarness, widgets::button, Element};
    use iced::{keyboard, mouse, touch, Event, Point, Size};

    const VIEWPORT: Size = Size::new(400.0, 300.0);

    fn harness_with_dialog(
        on_backdrop: Option<&'static str>,
        on_escape: Option<&'static str>,
    ) -> WidgetHarness<'static, &'static str> {
        let base = button::primary("Base action")
            .id(iced::widget::Id::new("base-action"))
            .on_press("base-pressed");
        let dialog_content = button::primary("Dialog action")
            .id(iced::widget::Id::new("dialog-action"))
            .on_press("dialog-pressed");

        let host = DialogHost::new(base).dialog(
            dialog_content,
            on_backdrop,
            on_escape,
            DialogInitialFocus::default(),
        );
        WidgetHarness::new(Element::from(host), VIEWPORT)
    }

    fn escape(repeat: bool) -> Event {
        let key = keyboard::Key::Named(keyboard::key::Named::Escape);
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Escape),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::NONE,
            text: None,
            repeat,
        })
    }

    #[test]
    fn primary_press_outside_the_dialog_publishes_backdrop_exactly_once() {
        let mut harness = harness_with_dialog(Some("backdrop"), Some("escape"));
        harness.set_cursor(Point::new(5.0, 5.0));

        let result = harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )))
            .expect("open dialog overlay");

        assert_eq!(result.messages, vec!["backdrop"]);
        assert!(result.captured);
    }

    #[test]
    fn touch_press_outside_the_dialog_publishes_backdrop() {
        let mut harness = harness_with_dialog(Some("backdrop"), None);
        harness.set_cursor(Point::new(5.0, 5.0));

        let result = harness
            .update_overlay(Event::Touch(touch::Event::FingerPressed {
                id: touch::Finger(0),
                position: Point::new(5.0, 5.0),
            }))
            .expect("open dialog overlay");

        assert_eq!(result.messages, vec!["backdrop"]);
    }

    #[test]
    fn secondary_press_outside_the_dialog_does_not_dismiss() {
        let mut harness = harness_with_dialog(Some("backdrop"), None);
        harness.set_cursor(Point::new(5.0, 5.0));

        let result = harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Right,
            )))
            .expect("open dialog overlay");

        assert!(result.messages.is_empty());
        assert!(result.captured, "outside input must still be consumed");
    }

    #[test]
    fn press_inside_the_dialog_does_not_dismiss() {
        let mut harness = harness_with_dialog(Some("backdrop"), None);
        let bounds = harness.overlay_bounds().expect("open dialog overlay");
        harness.set_cursor(Point::new(bounds.center_x(), bounds.center_y()));

        let result = harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )))
            .expect("open dialog overlay");

        assert!(result.messages.is_empty());
    }

    #[test]
    fn outside_press_without_a_configured_backdrop_route_is_still_consumed_silently() {
        let mut harness = harness_with_dialog(None, None);
        harness.set_cursor(Point::new(5.0, 5.0));

        let result = harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )))
            .expect("open dialog overlay");

        assert!(result.messages.is_empty());
        assert!(result.captured);
    }

    #[test]
    fn escape_publishes_the_configured_message_exactly_once() {
        let mut harness = harness_with_dialog(None, Some("escape"));

        let result = harness
            .update_overlay(escape(false))
            .expect("open dialog overlay");

        assert_eq!(result.messages, vec!["escape"]);
    }

    #[test]
    fn repeated_escape_publishes_nothing() {
        let mut harness = harness_with_dialog(None, Some("escape"));

        let result = harness
            .update_overlay(escape(true))
            .expect("open dialog overlay");

        assert!(result.messages.is_empty());
    }

    #[test]
    fn escape_without_a_configured_route_publishes_nothing_but_is_captured() {
        let mut harness = harness_with_dialog(None, None);

        let result = harness
            .update_overlay(escape(false))
            .expect("open dialog overlay");

        assert!(result.messages.is_empty());
        assert!(result.captured);
    }

    #[test]
    fn base_content_is_inert_to_pointer_input_while_a_dialog_is_open() {
        let mut harness = harness_with_dialog(None, None);
        let base_bounds = harness.bounds();
        harness.set_cursor(Point::new(base_bounds.x + 5.0, base_bounds.y + 5.0));

        let pressed = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        let released = harness.update(Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        )));

        assert!(pressed.messages.is_empty());
        assert!(released.messages.is_empty());
    }

    #[test]
    fn external_focus_operation_does_not_reach_base_content_while_open() {
        let base_action = iced::widget::Id::new("base-action");
        let mut harness = harness_with_dialog(None, None);

        harness.focus(base_action.clone());

        assert!(!harness
            .managed_focus()
            .entries
            .iter()
            .any(|entry| entry.id.as_ref() == Some(&base_action) && entry.active));
    }

    #[test]
    fn dialog_content_still_activates_normally() {
        let mut harness = harness_with_dialog(None, None);
        let bounds = harness.overlay_bounds().expect("open dialog overlay");
        harness.set_cursor(Point::new(bounds.center_x(), bounds.center_y()));

        harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )))
            .expect("open dialog overlay");
        let released = harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonReleased(
                mouse::Button::Left,
            )))
            .expect("open dialog overlay");

        assert_eq!(released.messages, vec!["dialog-pressed"]);
    }

    // --- 2.9: safe-viewport-margin clamping through the real host pipeline
    // ---
    //
    // `DialogOverlay`'s own `safe_max_size` arithmetic and `Dialog`'s own
    // Limits-clamping are each unit-tested in isolation (`dialog_host_tests`
    // and `dialog_widget_tests`); these two prove the two compose correctly
    // through `DialogHost` with a real header/body/footer Dialog rather than
    // a single button. Fixed header/footer sizing and body-only overflow are
    // already covered by `dialog_widget_tests`
    // (`header_and_footer_are_positioned_before_and_after_the_body`,
    // `total_height_never_exceeds_the_limits_max_height`) and are not
    // duplicated here.

    fn harness_with_tall_dialog(viewport: Size) -> WidgetHarness<'static, &'static str> {
        use crate::widgets::Dialog;

        let base = button::primary("Base action")
            .id(iced::widget::Id::new("base-action"))
            .on_press("base-pressed");
        let long_body =
            iced::widget::column((0..200).map(|i| iced::widget::text(format!("Line {i}")).into()));
        let dialog = Dialog::<&'static str>::new(long_body)
            .header(iced::widget::text("Title"))
            .footer(iced::widget::text("Footer"));

        let host = DialogHost::new(base).dialog(dialog, None, None, DialogInitialFocus::default());
        WidgetHarness::new(Element::from(host), viewport)
    }

    #[test]
    fn dialog_clamps_to_the_safe_margin_and_height_cap_on_a_narrow_low_viewport() {
        let viewport = Size::new(300.0, 500.0);
        let mut harness = harness_with_tall_dialog(viewport);

        let bounds = harness.overlay_bounds().expect("open dialog overlay");

        // Width: safe margin only (720px Lg target would need it, but this
        // is the default Sm target at 420px, still wider than the
        // 300 - 32 = 268px safe cap).
        assert!(bounds.width <= viewport.width - 32.0 + f32::EPSILON);
        // Height: min(0.80 * 500, 500 - 32) = min(400, 468) = 400, not the
        // long body's natural (uncapped) height.
        assert!(bounds.height <= 400.0 + f32::EPSILON);
        // Centered within the viewport, never off-screen.
        assert!(bounds.x >= 0.0 && bounds.y >= 0.0);
        assert!(bounds.x + bounds.width <= viewport.width + f32::EPSILON);
        assert!(bounds.y + bounds.height <= viewport.height + f32::EPSILON);
    }

    #[test]
    fn dialog_layout_stays_non_negative_below_the_safe_margin() {
        // Smaller than 2 * SAFE_VIEWPORT_MARGIN (32px) on both axes: the
        // safe area collapses to zero rather than going negative.
        let viewport = Size::new(20.0, 20.0);
        let mut harness = harness_with_tall_dialog(viewport);

        let bounds = harness.overlay_bounds().expect("open dialog overlay");

        assert!(bounds.width >= 0.0);
        assert!(bounds.height >= 0.0);
        assert!(bounds.x >= 0.0 && bounds.y >= 0.0);
    }
}

#[cfg(test)]
mod dialog_initial_focus_tests {
    use super::{DialogHost, DialogInitialFocus};
    use crate::{
        test_support::WidgetHarness,
        widgets::{
            button,
            overlays::dialog::{
                Dialog, DialogAction, DialogActionFooter, DialogHeader, DialogTerminalAction,
            },
        },
        Element,
    };
    use iced::Size;

    const VIEWPORT: Size = Size::new(500.0, 500.0);

    fn open_and_focused_ids(
        dialog: Element<'static, &'static str>,
        initial_focus: DialogInitialFocus,
    ) -> Vec<iced::widget::Id> {
        let base = iced::widget::text("base");
        let host = DialogHost::new(base).dialog(dialog, None, None, initial_focus);
        let mut harness = WidgetHarness::new(Element::from(host), VIEWPORT);
        harness.focused_overlay_ids()
    }

    #[test]
    fn first_policy_prefers_the_first_body_focusable_over_the_footer() {
        let body_button = iced::widget::Id::new("body-button");

        let dialog: Element<'static, &'static str> = Dialog::new(
            button::primary("Body action")
                .id(body_button.clone())
                .on_press("body"),
        )
        .header(DialogHeader::new("Title").close("Close", "close"))
        .footer(DialogActionFooter::new(DialogTerminalAction::primary(
            "Save", "save",
        )))
        .into();

        let focused = open_and_focused_ids(dialog, DialogInitialFocus::default());

        assert_eq!(focused, vec![body_button]);
    }

    #[test]
    fn first_policy_falls_back_to_a_footer_cancel_action_when_the_body_has_nothing_focusable() {
        let cancel_id = iced::widget::Id::new("cancel");

        let dialog: Element<'static, &'static str> = Dialog::new(iced::widget::text("Static body"))
            .footer(DialogActionFooter::with_one(
                DialogAction::cancel("Cancel", "cancel").id(cancel_id.clone()),
                DialogTerminalAction::primary("Save", "save"),
            ))
            .into();

        let focused = open_and_focused_ids(dialog, DialogInitialFocus::default());

        assert_eq!(focused, vec![cancel_id]);
    }

    #[test]
    fn first_policy_never_focuses_a_destructive_only_terminal_action() {
        let dialog: Element<'static, &'static str> = Dialog::new(iced::widget::text("Static body"))
            .footer(DialogActionFooter::new(DialogTerminalAction::destructive(
                "Delete", "delete",
            )))
            .into();

        let focused = open_and_focused_ids(dialog, DialogInitialFocus::default());

        assert!(
            focused.is_empty(),
            "no safe target exists, so nothing should be auto-focused"
        );
    }

    #[test]
    fn target_policy_focuses_the_explicit_id() {
        let field = iced::widget::Id::new("second-field");

        let dialog: Element<'static, &'static str> = Dialog::new(iced::widget::column![
            button::primary("First")
                .id(iced::widget::Id::new("first-field"))
                .on_press("first"),
            button::primary("Second")
                .id(field.clone())
                .on_press("second"),
        ])
        .into();

        let focused = open_and_focused_ids(dialog, DialogInitialFocus::Target(field.clone()));

        assert_eq!(focused, vec![field]);
    }

    #[test]
    fn target_policy_falls_back_to_first_when_the_target_is_missing() {
        let body_button = iced::widget::Id::new("body-button");

        let dialog: Element<'static, &'static str> = Dialog::new(
            button::primary("Body action")
                .id(body_button.clone())
                .on_press("body"),
        )
        .into();

        let focused = open_and_focused_ids(
            dialog,
            DialogInitialFocus::Target(iced::widget::Id::new("does-not-exist")),
        );

        assert_eq!(focused, vec![body_button]);
    }
}

#[cfg(test)]
mod dialog_identity_tests {
    use super::{DialogHost, DialogInitialFocus};
    use crate::{
        test_support::WidgetHarness,
        widgets::{button, overlays::dialog::Dialog},
        Element,
    };
    use iced::Size;

    const VIEWPORT: Size = Size::new(500.0, 500.0);

    fn two_button_dialog(
        first: iced::widget::Id,
        second: iced::widget::Id,
    ) -> Element<'static, &'static str> {
        Dialog::new(iced::widget::column![
            button::primary("First").id(first).on_press("first"),
            button::primary("Second").id(second).on_press("second"),
        ])
        .into()
    }

    fn host(
        dialog: Element<'static, &'static str>,
        session_id: Option<iced::widget::Id>,
    ) -> Element<'static, &'static str> {
        let base = iced::widget::text("base");
        let mut host =
            DialogHost::new(base).dialog(dialog, None, None, DialogInitialFocus::default());
        if let Some(id) = session_id {
            host = host.dialog_id(id);
        }
        Element::from(host)
    }

    #[test]
    fn same_explicit_identity_does_not_repeat_initial_focus_on_rerender() {
        let first = iced::widget::Id::new("first");
        let second = iced::widget::Id::new("second");
        let session = iced::widget::Id::new("session");

        let mut harness = WidgetHarness::new(
            host(
                two_button_dialog(first.clone(), second.clone()),
                Some(session.clone()),
            ),
            VIEWPORT,
        );
        assert_eq!(harness.focused_overlay_ids(), vec![first.clone()]);

        assert!(harness.focus_overlay_next());
        assert_eq!(harness.focused_overlay_ids(), vec![second.clone()]);

        // Rebuild with the same session id: an ordinary declarative
        // rerender must not recapture or re-run initial focus, so the
        // user's own in-dialog tab position is preserved.
        harness.replace(host(
            two_button_dialog(first, second.clone()),
            Some(session),
        ));

        assert_eq!(harness.focused_overlay_ids(), vec![second]);
    }

    #[test]
    fn absent_identity_across_rerenders_behaves_like_the_same_session() {
        let first = iced::widget::Id::new("first");
        let second = iced::widget::Id::new("second");

        let mut harness = WidgetHarness::new(
            host(two_button_dialog(first.clone(), second.clone()), None),
            VIEWPORT,
        );
        assert!(harness.focus_overlay_next());
        assert_eq!(harness.focused_overlay_ids(), vec![second.clone()]);

        harness.replace(host(two_button_dialog(first, second.clone()), None));

        assert_eq!(harness.focused_overlay_ids(), vec![second]);
    }

    #[test]
    fn changed_identity_replaces_the_workflow_step_and_reruns_initial_focus() {
        let first = iced::widget::Id::new("first");
        let second = iced::widget::Id::new("second");
        let step_one = iced::widget::Id::new("step-one");
        let step_two = iced::widget::Id::new("step-two");

        let mut harness = WidgetHarness::new(
            host(
                two_button_dialog(first.clone(), second.clone()),
                Some(step_one),
            ),
            VIEWPORT,
        );
        assert!(harness.focus_overlay_next());
        assert_eq!(harness.focused_overlay_ids(), vec![second]);

        // A new explicit step id, still open (no intermediate closed
        // frame), replaces the subtree and re-runs initial focus for the
        // new step rather than preserving the outgoing step's tab
        // position.
        harness.replace(host(
            two_button_dialog(first.clone(), iced::widget::Id::new("third")),
            Some(step_two),
        ));

        assert_eq!(harness.focused_overlay_ids(), vec![first]);
    }
}

/// Covers tasks 4.13/4.15/5.9/5.12: a real Dialog-owned nested `Popover`
/// (not a synthetic rectangle) hosted inside a `Dialog` body inside
/// `DialogHost`, exercised through the same `overlay::Nested`
/// infrastructure the popover-in-popover and Menu-submenu tests already
/// use — proving Dialog composes with the shared nested-overlay and focus
/// foundations rather than bypassing them.
#[cfg(test)]
mod nested_overlay_tests {
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
        let mut harness =
            WidgetHarness::new(rooted_host(false, false, Some(DIALOG_ESCAPE)), VIEWPORT);

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
}
