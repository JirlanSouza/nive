use super::*;

#[test]
fn new_has_zero_preceding_actions() {
    let footer = DialogActionFooter::<()>::new(DialogTerminalAction::primary("Save", ()));
    assert!(footer.preceding.is_empty());
    assert_eq!(footer.terminal.role, DialogActionRole::Primary);
}

#[test]
fn with_one_and_with_two_bound_preceding_count() {
    let one = DialogActionFooter::with_one(
        DialogAction::cancel("Cancel", ()),
        DialogTerminalAction::primary("Save", ()),
    );
    assert_eq!(one.preceding.len(), 1);

    let two = DialogActionFooter::with_two(
        [
            DialogAction::cancel("Cancel", ()),
            DialogAction::secondary("More", ()),
        ],
        DialogTerminalAction::primary("Save", ()),
    );
    assert_eq!(two.preceding.len(), 2);
}

#[test]
fn try_from_parts_rejects_more_than_two_preceding_actions() {
    let result = DialogActionFooter::try_from_parts(
        vec![
            DialogAction::cancel("A", ()),
            DialogAction::secondary("B", ()),
            DialogAction::secondary("C", ()),
        ],
        DialogTerminalAction::primary("Save", ()),
    );

    match result {
        Err(error) => {
            assert_eq!(error, DialogActionFooterError::TooManyPrecedingActions(3));
        }
        Ok(_) => panic!("expected TooManyPrecedingActions"),
    }
}

#[test]
fn try_from_parts_rejects_invalid_preceding_role() {
    let invalid = DialogAction::new(DialogActionRole::Destructive, "Delete", ());
    let result = DialogActionFooter::try_from_parts(
        vec![invalid],
        DialogTerminalAction::primary("Save", ()),
    );

    match result {
        Err(error) => {
            assert_eq!(
                error,
                DialogActionFooterError::InvalidPrecedingRole(DialogActionRole::Destructive)
            );
        }
        Ok(_) => panic!("expected InvalidPrecedingRole"),
    }
}

#[test]
fn try_from_parts_accepts_zero_to_two_valid_preceding_actions() {
    assert!(
        DialogActionFooter::try_from_parts(vec![], DialogTerminalAction::primary("Save", ()))
            .is_ok()
    );
    assert!(DialogActionFooter::try_from_parts(
        vec![DialogAction::cancel("Cancel", ())],
        DialogTerminalAction::primary("Save", ())
    )
    .is_ok());
}

#[test]
fn enter_default_message_is_none_for_destructive_terminal() {
    let footer = DialogActionFooter::new(DialogTerminalAction::destructive("Delete", "delete"));
    assert!(footer.enter_default_message().is_none());
}

#[test]
fn enter_default_message_is_none_when_primary_disabled() {
    let footer =
        DialogActionFooter::new(DialogTerminalAction::primary("Save", "save").disabled(true));
    assert!(footer.enter_default_message().is_none());
}

#[test]
fn enter_default_message_is_the_enabled_primary_message() {
    let footer = DialogActionFooter::new(DialogTerminalAction::primary("Save", "save"));
    assert_eq!(footer.enter_default_message(), Some(&"save"));
}

#[test]
fn ordering_places_preceding_actions_before_the_terminal_action() {
    let footer = DialogActionFooter::with_one(
        DialogAction::cancel("Cancel", 1),
        DialogTerminalAction::primary("Save", 2),
    );
    let ordered: Vec<_> = footer.all_actions().map(|action| action.message).collect();

    assert_eq!(ordered, vec![1, 2]);
}
