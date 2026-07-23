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
    let mut host = DialogHost::new(base).dialog(dialog, None, None, DialogInitialFocus::default());
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
