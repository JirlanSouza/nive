use nive_ui::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
enum DialogMessage {
    Close,
    Save,
    Delete,
}

#[test]
fn prelude_exposes_the_complete_canonical_dialog_family_contract() {
    let owned_title = String::from("Owned title");

    // Every DialogSize, Cow-owned/borrowed header copy, an icon, and a safe
    // named close affordance.
    for size in [DialogSize::Sm, DialogSize::Md, DialogSize::Lg] {
        let header = DialogHeader::new(owned_title.clone())
            .description("Borrowed description")
            .icon(IconRole::DialogWarning)
            .close("Close dialog", DialogMessage::Close);

        let _: Element<'_, DialogMessage> =
            Dialog::new(text("Body")).size(size).header(header).into();
    }

    // Plain DialogFooter for non-action footer content.
    let _: Element<'_, DialogMessage> = Dialog::new(text("Body"))
        .footer(DialogFooter::new(text("Custom footer content")))
        .into();

    // Zero, one, and two preceding actions plus one required terminal
    // action, through every DialogActionRole and every ergonomic static
    // constructor.
    let _: Element<'_, DialogMessage> = Dialog::new(text("Body"))
        .footer(DialogActionFooter::new(DialogTerminalAction::primary(
            "Save",
            DialogMessage::Save,
        )))
        .into();

    let _: Element<'_, DialogMessage> = Dialog::new(text("Body"))
        .footer(DialogActionFooter::with_one(
            DialogAction::cancel("Cancel", DialogMessage::Close),
            DialogTerminalAction::primary("Save", DialogMessage::Save),
        ))
        .into();

    let _: Element<'_, DialogMessage> = Dialog::new(text("Body"))
        .footer(
            DialogActionFooter::with_two(
                [
                    DialogAction::cancel("Cancel", DialogMessage::Close),
                    DialogAction::secondary("More", DialogMessage::Close).disabled(true),
                ],
                DialogTerminalAction::destructive("Delete", DialogMessage::Delete).disabled(false),
            )
            .status(text("Status/help content")),
        )
        .into();

    // Fallible dynamic construction and its typed error.
    let dynamic: Result<DialogActionFooter<'_, DialogMessage>, DialogActionFooterError> =
        DialogActionFooter::try_from_parts(
            vec![DialogAction::cancel("Cancel", DialogMessage::Close)],
            DialogTerminalAction::primary("Save", DialogMessage::Save),
        );
    assert!(dynamic.is_ok());

    // DialogInitialFocus and a stable action Id.
    let _: DialogInitialFocus = DialogInitialFocus::First;
    let _: DialogInitialFocus = DialogInitialFocus::Target(iced::widget::Id::new("field"));
    let _ =
        DialogAction::cancel("Cancel", DialogMessage::Close).id(iced::widget::Id::new("cancel"));

    // Canonical UI-only hosting: DialogHost with initial focus and a stable
    // declarative session id, entirely through `nive_ui::prelude::*`.
    let dialog: Element<'_, DialogMessage> = Dialog::new(text("Body"))
        .footer(DialogActionFooter::new(DialogTerminalAction::primary(
            "Save",
            DialogMessage::Save,
        )))
        .into();
    let _: Element<'_, DialogMessage> = DialogHost::new(text("Base content"))
        .dialog(
            dialog,
            Some(DialogMessage::Close),
            Some(DialogMessage::Close),
            DialogInitialFocus::Target(iced::widget::Id::new("field")),
        )
        .dialog_id(iced::widget::Id::new("workflow-step"))
        .into();

    // ErrorDetailsDialog reuses the same canonical composition.
    let _: Element<'_, DialogMessage> = ErrorDetailsDialog::new("stack trace")
        .on_close(DialogMessage::Close)
        .into();
}
