use iced::{advanced::layout, Size};

use super::*;
use crate::test_support::WidgetHarness;

fn node_for(footer: DialogActionFooter<'static, ()>, width: f32) -> layout::Node {
    crate::test_support::layout(footer.into(), Size::new(width, 1000.0))
}

/// Resizing a window reaches widgets through `UserInterface::relayout`, which
/// re-runs `layout` on the retained tree with no `diff` in between. The stacked
/// branch lays every action out a second time against a fixed width, so an
/// action's own subtree sees two different limits in one pass — and a label that
/// truncates under one and not the other swaps a `Text` for a `Tooltip` beneath
/// it. Both passes have to leave that child tree matching what they built.
#[test]
fn resizing_across_the_stacking_threshold_keeps_action_subtrees_valid() {
    let footer = || -> Element<'static, ()> {
        DialogActionFooter::with_one(
            DialogAction::cancel("Discard every unsaved change", ()),
            DialogTerminalAction::primary("Save and close this document now", ()),
        )
        .status(iced::widget::text("Autosaved a moment ago"))
        .into()
    };

    let mut harness = WidgetHarness::new(footer(), Size::new(900.0, 1000.0));
    harness.draw();

    for width in [280.0, 900.0, 200.0, 900.0, 120.0] {
        harness.relayout(Size::new(width, 1000.0));
        harness.draw();
    }
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
