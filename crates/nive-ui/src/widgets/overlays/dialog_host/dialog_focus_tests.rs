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
