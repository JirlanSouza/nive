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
