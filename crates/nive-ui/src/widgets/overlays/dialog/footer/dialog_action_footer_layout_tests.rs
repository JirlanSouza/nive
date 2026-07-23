use iced::{advanced::layout, Size};

use super::*;

fn node_for(footer: DialogActionFooter<'static, ()>, width: f32) -> layout::Node {
    crate::test_support::layout(footer.into(), Size::new(width, 1000.0))
}

#[test]
fn wide_footer_renders_a_single_row() {
    let footer = DialogActionFooter::with_one(
        DialogAction::cancel("Cancel", ()),
        DialogTerminalAction::primary("Save", ()),
    )
    .status(iced::widget::text("Autosaved"));

    let node = node_for(footer, 900.0);
    assert_eq!(node.children().len(), 3);
}

#[test]
fn narrow_footer_stacks_status_above_actions() {
    let footer = DialogActionFooter::with_one(
        DialogAction::cancel("Cancel", ()),
        DialogTerminalAction::primary("Save", ()),
    )
    .status(iced::widget::text(
        "A rather long status message that will not fit beside the action buttons",
    ));

    let single_row = node_for(
        DialogActionFooter::with_one(
            DialogAction::cancel("Cancel", ()),
            DialogTerminalAction::primary("Save", ()),
        )
        .status(iced::widget::text("Short")),
        900.0,
    );
    let stacked = node_for(footer, 320.0);

    assert!(stacked.size().height > single_row.size().height);
}
