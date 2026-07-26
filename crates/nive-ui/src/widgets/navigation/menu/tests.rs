use std::time::Duration;

use super::*;
use iced::{
    advanced::mouse,
    keyboard::{self, key},
    touch,
    widget::Space,
    Event, Point, Size,
};
use nive_core::{Action, ShortcutBinding};

use crate::test_support::{layout as widget_layout, WidgetHarness};
#[allow(unused_imports)]
use crate::IconRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Message {
    Save,
    Toggle(CheckboxState),
    Select(u8),
    Dismiss,
}

#[test]
fn action_projection_preserves_command_semantics_and_ui_decoration() {
    let action = Action::new("file.save", "Save", Message::Save)
        .shortcut(ShortcutBinding::primary_character('s'));
    let command = MenuCommand::from_action(&action)
        .icon(IconRole::ActionConfirm)
        .destructive()
        .dismiss_policy(MenuDismissPolicy::KeepOpen);

    assert_eq!(command.id(), Some(ActionId::new("file.save")));
    assert_eq!(command.label(), "Save");
    assert_eq!(
        command.shortcut_binding(),
        Some(ShortcutBinding::primary_character('s'))
    );
    assert!(!command.is_disabled());
    assert_eq!(command.on_press, Some(Message::Save));
    assert_eq!(command.icon, Some(IconRole::ActionConfirm.into()));
    assert!(command.destructive);
    assert_eq!(command.dismiss_policy, MenuDismissPolicy::KeepOpen);
}

#[test]
fn disabled_action_cannot_be_reenabled_by_menu_decoration() {
    let action = Action::new("file.save", "Save", Message::Save).disabled();
    let command = MenuCommand::from_action(&action).disabled(false);

    assert!(command.is_disabled());
    assert_eq!(command.on_press, None);
}

#[test]
fn fluent_categories_build_one_anchored_menu() {
    let child = Menu::new(Space::new()).command(MenuCommand::new("Child"));
    let menu: Menu<'_, Message> = Menu::new(Space::new())
        .open(true)
        .on_dismiss(Message::Dismiss)
        .command(MenuCommand::new("Save").on_press(Message::Save))
        .checkbox(MenuCheckbox::new("Pinned", CheckboxState::Unchecked).on_toggle(Message::Toggle))
        .radio_group(
            MenuRadioGroup::new(Some(1))
                .option(MenuRadioOption::new(1, "One"))
                .option(MenuRadioOption::new(2, "Two"))
                .on_select(Message::Select),
        )
        .separator()
        .submenu(MenuSubmenu::new("More", child));

    assert_eq!(menu.entries.len(), 6);
    let _: Element<'_, Message> = menu.into();
}

#[test]
fn separators_normalize_and_duplicate_radio_values_are_inert() {
    let duplicate_group = MenuRadioGroup::<_, Message>::new(None)
        .option(MenuRadioOption::new(1, "One"))
        .option(MenuRadioOption::new(1, "Duplicate"))
        .on_select(Message::Select);
    assert!(!duplicate_group.has_unique_values());

    let invalid = Menu::new(Space::new())
        .radio_group(duplicate_group)
        .into_content();
    let mut invalid = WidgetHarness::new(invalid, Size::new(320.0, 120.0));
    assert!(invalid.bounds().width.is_finite());
    assert!(invalid.bounds().height.is_finite());
    assert_eq!(invalid.focused_count().total, 0);
    invalid.set_cursor(Point::new(12.0, 12.0));
    invalid.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    let released = invalid.update(Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    )));
    assert!(released.messages.is_empty());

    let menu: Menu<'_, Message> = Menu::new(Space::new())
        .separator()
        .command(MenuCommand::new("Only"))
        .separator()
        .separator();
    assert_eq!(menu.entries.len(), 2);
}

#[test]
fn natural_width_is_renderer_measured_and_clamped() {
    // No artificial floor: a menu with a short label shrinks to its
    // natural content width instead of stretching to a fixed minimum.
    let short = Menu::new(Space::new())
        .command(MenuCommand::new("Save").on_press(Message::Save))
        .into_content();
    let short = WidgetHarness::new(short, Size::new(800.0, 300.0));
    assert!(short.bounds().width < 180.0);
    assert!(short.bounds().width > 0.0);

    let long = Menu::new(Space::new())
        .command(
            MenuCommand::new(
                "A command label deliberately wider than the maximum desktop menu width",
            )
            .on_press(Message::Save),
        )
        .into_content();
    let long = WidgetHarness::new(long, Size::new(800.0, 300.0));
    assert_eq!(long.bounds().width, MENU_MAX_WIDTH);
}

#[test]
fn popover_frame_matches_content_natural_width() {
    // Regression test: the Popover surface used to wrap its content in a
    // `Length::Fill` container regardless of `PopoverWidth` mode, so a
    // `Content`-sized Menu's visible frame silently stretched to the
    // safe-viewport cap instead of shrinking to the menu's own natural
    // width (see `surface_with_constraints` in overlays/popover.rs).
    fn build(open: bool) -> Menu<'static, Message> {
        Menu::new(Space::new().width(120).height(24))
            .open(open)
            .command(
                MenuCommand::new("Rename")
                    .icon(IconRole::EditModify)
                    .shortcut(ShortcutBinding::named(
                        nive_core::NamedShortcutKey::Enter,
                        nive_core::ShortcutModifiers::NONE,
                    ))
                    .on_press(Message::Save),
            )
            .checkbox(
                MenuCheckbox::new("Copy link", CheckboxState::Checked)
                    .shortcut(ShortcutBinding::primary_character('c'))
                    .on_toggle(Message::Toggle),
            )
            .separator()
            .command(MenuCommand::new("Disabled command").disabled(true))
            .command(MenuCommand::new("Callback absent"))
            .command(
                MenuCommand::new("Delete")
                    .icon(IconRole::EditDelete)
                    .destructive()
                    .on_press(Message::Save),
            )
    }

    let content_only = build(false).into_content();
    let content_harness = WidgetHarness::new(content_only, Size::new(800.0, 400.0));
    let natural_width = content_harness.bounds().width;

    let mut full_harness = WidgetHarness::new(build(true).into(), Size::new(800.0, 400.0));
    full_harness.set_cursor(Point::new(10.0, 10.0));
    full_harness.update(Event::Window(iced::window::Event::RedrawRequested(
        iced::time::Instant::now(),
    )));
    let overlay_width = full_harness.overlay_bounds().expect("open overlay").width;

    assert_eq!(
        overlay_width, natural_width,
        "popover frame width ({overlay_width}) should match the menu content's natural width ({natural_width})"
    );
}

#[test]
fn natural_width_respects_narrow_finite_viewports() {
    let content = Menu::new(Space::new())
        .command(MenuCommand::new("A very long command label").on_press(Message::Save))
        .into_content();
    let content = WidgetHarness::new(content, Size::new(124.0, 300.0));

    assert_eq!(content.bounds().width, 124.0);
    assert!(content.bounds().width.is_finite());
}

#[test]
fn canonical_list_metrics_are_fixed() {
    let content = Menu::new(Space::new())
        .command(MenuCommand::new("Save").on_press(Message::Save))
        .separator()
        .command(MenuCommand::new("Close").on_press(Message::Save))
        .into_content();
    let content = WidgetHarness::new(content, Size::new(400.0, 300.0));

    assert!(content.bounds().width < 180.0);
    assert_eq!(
        content.bounds().height,
        MENU_LIST_INSET * 2.0 + MENU_ROW_HEIGHT * 2.0 + 1.0 + MENU_SEPARATOR_MARGIN * 2.0
    );
}

#[test]
fn dismiss_all_publishes_leaf_before_dismissal() {
    let content = Menu::new(Space::new())
        .on_dismiss(Message::Dismiss)
        .command(MenuCommand::new("Save").on_press(Message::Save))
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(320.0, 120.0));
    harness.set_cursor(Point::new(12.0, 12.0));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    let released = harness.update(Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    )));

    assert_eq!(released.messages, vec![Message::Save, Message::Dismiss]);
}

#[test]
fn keep_open_and_absent_dismissal_publish_only_leaf() {
    for content in [
        Menu::new(Space::new())
            .on_dismiss(Message::Dismiss)
            .command(
                MenuCommand::new("Save")
                    .on_press(Message::Save)
                    .dismiss_policy(MenuDismissPolicy::KeepOpen),
            )
            .into_content(),
        Menu::new(Space::new())
            .command(MenuCommand::new("Save").on_press(Message::Save))
            .into_content(),
    ] {
        let mut harness = WidgetHarness::new(content, Size::new(320.0, 120.0));
        harness.set_cursor(Point::new(12.0, 12.0));
        harness.update(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        let released = harness.update(Event::Mouse(mouse::Event::ButtonReleased(
            mouse::Button::Left,
        )));

        assert_eq!(released.messages, vec![Message::Save]);
    }
}

#[test]
fn persistent_choice_and_transient_interaction_keep_geometry_stable() {
    let content = Menu::new(Space::new())
        .checkbox(
            MenuCheckbox::new("Pinned", CheckboxState::Checked)
                .on_toggle(Message::Toggle)
                .dismiss_policy(MenuDismissPolicy::KeepOpen),
        )
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(320.0, 120.0));
    let initial = harness.bounds();
    harness.set_cursor(Point::new(12.0, 12.0));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert_eq!(harness.bounds(), initial);
    harness.update(Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    )));
    harness.focus_next();
    assert_eq!(harness.bounds(), initial);
}

#[test]
fn trailing_track_uses_the_widest_peer_measurement_for_every_row() {
    let content = Menu::new(Space::new())
        .command(
            MenuCommand::new("Save")
                .shortcut(ShortcutBinding::primary_character('s'))
                .on_press(Message::Save),
        )
        .command(MenuCommand::new("Close").on_press(Message::Dismiss))
        .into_content();
    let node = widget_layout(content, Size::new(320.0, 120.0));
    let column = &node.children()[0];
    let first_row = &column.children()[0].children()[0];
    let second_row = &column.children()[1].children()[0];
    let first_trailing = first_row.children().last().expect("first trailing track");
    let second_trailing = second_row.children().last().expect("second trailing track");

    assert!(first_trailing.size().width > 0.0);
    assert_eq!(first_trailing.size().width, second_trailing.size().width);
}

#[test]
fn persistent_choice_and_leading_icon_use_separate_stable_tracks() {
    let content = Menu::new(Space::new())
        .radio_group(
            MenuRadioGroup::new(Some(1))
                .option(MenuRadioOption::new(1, "Selected").icon(IconRole::ActionConfirm))
                .option(MenuRadioOption::new(2, "Peer"))
                .on_select(Message::Select),
        )
        .into_content();
    let node = widget_layout(content, Size::new(320.0, 120.0));
    let column = &node.children()[0];

    for row in column.children() {
        let tracks = &row.children()[0].children();
        assert_eq!(tracks[0].size().width, MENU_ICON_SIZE);
        assert_eq!(tracks[1].size().width, MENU_ICON_SIZE);
    }
}

#[test]
fn truncated_highlight_forwards_private_focus_to_tooltip_only() {
    let content = Menu::new(Space::new())
        .command(
            MenuCommand::new(
                "A renderer-measured command label that must truncate inside the menu",
            )
            .on_press(Message::Save),
        )
        .command(MenuCommand::new("Close").on_press(Message::Dismiss))
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(220.0, 120.0));

    assert_eq!(harness.focused_count().total, 1);
    harness.focus_next();
    harness.update(Event::Window(iced::window::Event::RedrawRequested(
        iced::time::Instant::now(),
    )));
    assert!(harness.has_overlay());
    assert_eq!(harness.focused_count().total, 1);

    harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));
    harness.update(Event::Window(iced::window::Event::RedrawRequested(
        iced::time::Instant::now(),
    )));
    assert!(!harness.has_overlay());
}

#[test]
fn root_composite_is_one_focus_target_with_bounded_navigation() {
    let content = Menu::new(Space::new())
        .command(MenuCommand::new("Display only"))
        .command(MenuCommand::new("Save").on_press(Message::Save))
        .separator()
        .checkbox(MenuCheckbox::new("Pinned", CheckboxState::Unchecked).on_toggle(Message::Toggle))
        .command(
            MenuCommand::new("Disabled")
                .disabled(true)
                .on_press(Message::Save),
        )
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(320.0, 240.0));

    assert_eq!(harness.focused_count().total, 1);
    harness.focus_next();
    assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(1));

    harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));
    assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(3));
    harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));
    assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(3));

    harness.update(key_pressed(key::Named::Home, key::Code::Home));
    assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(1));
    harness.update(key_pressed(key::Named::End, key::Code::End));
    assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(3));
}

#[test]
fn highlighted_row_stays_visible_in_the_popover_owned_scroll_viewport() {
    let mut menu = Menu::new(Space::new().width(40).height(24)).open(true);
    for index in 0..20 {
        menu =
            menu.command(MenuCommand::new(format!("Command {index:02}")).on_press(Message::Save));
    }
    let mut harness = WidgetHarness::new(menu.into(), Size::new(260.0, 120.0));

    assert_eq!(harness.overlay_scroll_offsets(), vec![iced::Vector::ZERO]);

    let end = harness
        .update_overlay(key_pressed(key::Named::End, key::Code::End))
        .expect("open Menu overlay");
    assert!(end.captured);
    let end_offsets = harness.overlay_scroll_offsets();
    assert_eq!(end_offsets.len(), 1);
    assert!(end_offsets[0].y > 0.0);

    let home = harness
        .update_overlay(key_pressed(key::Named::Home, key::Code::Home))
        .expect("open Menu overlay");
    assert!(home.captured);
    let home_offsets = harness.overlay_scroll_offsets();
    assert_eq!(home_offsets.len(), 1);
    assert!(home_offsets[0].y <= MENU_LIST_INSET);
}

#[test]
fn highlighted_capable_row_reconciles_by_label_after_reorder() {
    let content = Menu::new(Space::new())
        .command(MenuCommand::new("Save").on_press(Message::Save))
        .command(MenuCommand::new("Close").on_press(Message::Dismiss))
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(320.0, 120.0));
    harness.focus_next();
    harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));
    assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(1));

    harness.replace(
        Menu::new(Space::new())
            .command(MenuCommand::new("Close").on_press(Message::Dismiss))
            .command(MenuCommand::new("Save").on_press(Message::Save))
            .into_content(),
    );

    assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(0));
}

#[test]
fn parked_pointer_does_not_reset_keyboard_navigation() {
    let content = Menu::new(Space::new())
        .command(MenuCommand::new("One").on_press(Message::Save))
        .command(MenuCommand::new("Two").on_press(Message::Save))
        .command(MenuCommand::new("Three").on_press(Message::Save))
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(320.0, 140.0));
    harness.focus_next();
    let parked = Point::new(12.0, 12.0);
    harness.set_cursor(parked);
    harness.update(Event::Mouse(mouse::Event::CursorMoved { position: parked }));

    harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));
    harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));

    assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(2));
}

#[test]
fn right_opens_child_without_adding_focus_and_activates_its_leaf() {
    let child =
        Menu::new(Space::new()).command(MenuCommand::new("Save child").on_press(Message::Save));
    let content = Menu::new(Space::new())
        .on_dismiss(Message::Dismiss)
        .submenu(MenuSubmenu::new("More", child))
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(640.0, 320.0));
    harness.focus_next();

    harness.update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight));

    assert!(harness.has_overlay());
    assert_eq!(harness.focused_count().total, 1);
    assert_eq!(
        harness
            .focused_overlay_count()
            .expect("child overlay")
            .total,
        0
    );
    let activated = harness
        .update_overlay(key_pressed(key::Named::Enter, key::Code::Enter))
        .expect("child overlay update");
    assert_eq!(activated.messages, vec![Message::Save, Message::Dismiss]);
}

#[test]
fn nested_escape_unwinds_only_the_innermost_level() {
    let grandchild =
        Menu::new(Space::new()).command(MenuCommand::new("Save child").on_press(Message::Save));
    let child = Menu::new(Space::new()).submenu(MenuSubmenu::new("Advanced", grandchild));
    let content = Menu::new(Space::new())
        .on_dismiss(Message::Dismiss)
        .submenu(MenuSubmenu::new("More", child))
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(800.0, 400.0));
    harness.focus_next();
    harness.update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight));
    harness
        .update_nested_overlay(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight))
        .expect("first child overlay");
    assert!(harness.nested_overlay_bounds().len() >= 2);

    let escaped = harness
        .update_nested_overlay(key_pressed(key::Named::Escape, key::Code::Escape))
        .expect("nested child overlay");

    assert!(escaped.messages.is_empty());
    assert_eq!(harness.nested_overlay_bounds().len(), 1);
    assert_eq!(harness.focused_count().total, 1);
}

#[test]
fn left_closes_child_and_right_can_reopen_the_same_branch() {
    let child =
        Menu::new(Space::new()).command(MenuCommand::new("Save child").on_press(Message::Save));
    let content = Menu::new(Space::new())
        .submenu(MenuSubmenu::new("More", child))
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(640.0, 320.0));
    harness.focus_next();
    harness.update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight));
    assert!(harness.has_overlay());

    let closed = harness
        .update_overlay(key_pressed(key::Named::ArrowLeft, key::Code::ArrowLeft))
        .expect("child overlay");
    assert!(closed.messages.is_empty());
    assert!(!harness.has_overlay());

    harness.update(key_pressed(key::Named::ArrowRight, key::Code::ArrowRight));
    assert!(harness.has_overlay());
    assert_eq!(harness.focused_count().total, 1);
}

#[test]
fn pointer_intent_uses_open_delay_and_transfer_grace() {
    let child =
        Menu::new(Space::new()).command(MenuCommand::new("Save child").on_press(Message::Save));
    let content = Menu::new(Space::new())
        .submenu(MenuSubmenu::new("More", child))
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(640.0, 320.0));
    let start = iced::time::Instant::now();
    harness.update(Event::Window(iced::window::Event::RedrawRequested(start)));
    let row = Point::new(12.0, 12.0);
    harness.set_cursor(row);
    harness.update(Event::Mouse(mouse::Event::CursorMoved { position: row }));
    harness.update(Event::Window(iced::window::Event::RedrawRequested(
        start + Duration::from_millis(199),
    )));
    assert!(!harness.has_overlay());

    harness.update(Event::Window(iced::window::Event::RedrawRequested(
        start + Duration::from_millis(200),
    )));
    assert!(harness.has_overlay());

    let child = harness.overlay_bounds().expect("child bounds");
    let child_center = child.center();
    harness.set_cursor(child_center);
    let transfer = Event::Mouse(mouse::Event::CursorMoved {
        position: child_center,
    });
    harness.update(transfer.clone());
    harness
        .update_nested_overlay(transfer)
        .expect("child receives transfer");
    assert_eq!(
        harness.state::<widget::MenuListState>().open_submenu,
        Some(0)
    );
    harness.update(Event::Window(iced::window::Event::RedrawRequested(
        start + Duration::from_millis(500),
    )));
    assert!(harness.has_overlay());

    let outside = Point::new(400.0, 200.0);
    harness.set_cursor(outside);
    let leave = Event::Mouse(mouse::Event::CursorMoved { position: outside });
    harness.update(leave.clone());
    // The real runtime dispatches every pointer event to each active
    // overlay in the chain, refreshing the branch's own pointer-inside
    // tracking; mirror that here so the transfer-grace check below sees
    // the same "pointer has left" signal a live app would.
    harness
        .update_nested_overlay(leave)
        .expect("child receives the leave event");
    harness.update(Event::Window(iced::window::Event::RedrawRequested(
        start + Duration::from_millis(799),
    )));
    assert!(harness.has_overlay());
    harness.update(Event::Window(iced::window::Event::RedrawRequested(
        start + Duration::from_millis(800),
    )));
    assert!(!harness.has_overlay());
}

#[test]
fn root_composite_activation_preserves_leaf_then_dismiss_order() {
    let content = Menu::new(Space::new())
        .on_dismiss(Message::Dismiss)
        .command(MenuCommand::new("Display only"))
        .command(MenuCommand::new("Save").on_press(Message::Save))
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(320.0, 120.0));
    harness.focus_next();

    let activated = harness.update(key_pressed(key::Named::Enter, key::Code::Enter));

    assert_eq!(activated.messages, vec![Message::Save, Message::Dismiss]);
}

#[test]
fn keyboard_activation_publishes_controlled_checkbox_and_radio_values() {
    let checkbox = Menu::new(Space::new())
        .checkbox(
            MenuCheckbox::new("Pinned", CheckboxState::Unchecked)
                .on_toggle(Message::Toggle)
                .dismiss_policy(MenuDismissPolicy::KeepOpen),
        )
        .into_content();
    let mut checkbox = WidgetHarness::new(checkbox, Size::new(320.0, 120.0));
    checkbox.focus_next();
    let toggled = checkbox.update(key_pressed(key::Named::Enter, key::Code::Enter));
    assert_eq!(
        toggled.messages,
        vec![Message::Toggle(CheckboxState::Checked)]
    );

    let radio = Menu::new(Space::new())
        .on_dismiss(Message::Dismiss)
        .radio_group(
            MenuRadioGroup::new(Some(1))
                .option(MenuRadioOption::new(1, "One"))
                .option(MenuRadioOption::new(2, "Two"))
                .on_select(Message::Select),
        )
        .into_content();
    let mut radio = WidgetHarness::new(radio, Size::new(320.0, 120.0));
    radio.focus_next();
    assert_eq!(radio.state::<widget::MenuListState>().highlight, Some(1));
    let selected = radio.update(key_pressed(key::Named::Space, key::Code::Space));
    assert_eq!(
        selected.messages,
        vec![Message::Select(2), Message::Dismiss]
    );
}

#[test]
fn touch_activation_publishes_once() {
    let content = Menu::new(Space::new())
        .on_dismiss(Message::Dismiss)
        .command(MenuCommand::new("Save").on_press(Message::Save))
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(320.0, 120.0));
    let position = Point::new(12.0, 12.0);
    let finger = touch::Finger(7);
    let pressed = harness.update(Event::Touch(touch::Event::FingerPressed {
        id: finger,
        position,
    }));
    assert!(pressed.messages.is_empty());
    let released = harness.update(Event::Touch(touch::Event::FingerLifted {
        id: finger,
        position,
    }));

    assert_eq!(released.messages, vec![Message::Save, Message::Dismiss]);
}

#[test]
fn outside_press_requests_exactly_one_root_dismissal() {
    let menu = Menu::new(Space::new().width(40).height(24))
        .open(true)
        .on_dismiss(Message::Dismiss)
        .command(MenuCommand::new("Save").on_press(Message::Save));
    let mut harness = WidgetHarness::new(menu.into(), Size::new(320.0, 200.0));
    harness.set_cursor(Point::new(319.0, 199.0));

    let dismissed = harness
        .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )))
        .expect("open Menu overlay");

    assert_eq!(dismissed.messages, vec![Message::Dismiss]);
    assert!(dismissed.captured);
}

#[test]
fn typeahead_wraps_search_and_skips_ineligible_rows() {
    let content = Menu::new(Space::new())
        .command(MenuCommand::new("Placeholder"))
        .command(MenuCommand::new("Save").on_press(Message::Save))
        .separator()
        .checkbox(MenuCheckbox::new("Pinned", CheckboxState::Unchecked).on_toggle(Message::Toggle))
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(320.0, 160.0));
    harness.focus_next();
    harness.update(Event::Window(iced::window::Event::RedrawRequested(
        iced::time::Instant::now(),
    )));

    harness.update(text_key("p", key::Code::KeyP));

    assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(3));
}

#[test]
fn typeahead_keeps_the_prefix_through_700ms_and_resets_afterward() {
    let content = Menu::new(Space::new())
        .command(MenuCommand::new("Save").on_press(Message::Save))
        .command(MenuCommand::new("Print").on_press(Message::Save))
        .command(MenuCommand::new("Pin").on_press(Message::Save))
        .into_content();
    let mut harness = WidgetHarness::new(content, Size::new(320.0, 140.0));
    harness.focus_next();
    let start = iced::time::Instant::now();
    harness.update(Event::Window(iced::window::Event::RedrawRequested(start)));

    harness.update(text_key("p", key::Code::KeyP));
    assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(1));
    harness.update(Event::Window(iced::window::Event::RedrawRequested(
        start + Duration::from_millis(700),
    )));
    harness.update(text_key("i", key::Code::KeyI));
    assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(2));

    harness.update(Event::Window(iced::window::Event::RedrawRequested(
        start + Duration::from_millis(1_401),
    )));
    harness.update(text_key("s", key::Code::KeyS));
    assert_eq!(harness.state::<widget::MenuListState>().highlight, Some(0));
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

fn text_key(value: &str, code: key::Code) -> Event {
    let key = keyboard::Key::Character(value.into());
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: key::Physical::Code(code),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::NONE,
        text: Some(value.into()),
        repeat: false,
    })
}
