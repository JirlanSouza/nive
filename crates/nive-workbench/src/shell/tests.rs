use iced::widget::{text, Space};
use nive_ui::IconRole;

use super::*;
use crate::panels::PanelAction;

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Workbench(WorkbenchEvent<&'static str, &'static str, &'static str>),
}

#[test]
fn constructs_shell_with_full_regions() {
    let state = WorkbenchLayoutState::default().with_active_document("readme");
    let _view = WorkbenchShell::new(state, Message::Workbench)
        .toolbar(text("Toolbar"))
        .left_panels([WorkbenchPanel::new("files", "Files", Space::new())])
        .documents([WorkbenchDocument::new("readme", "README.md")])
        .document_content(text("Document"))
        .right_panels([WorkbenchPanel::new("inspector", "Inspector", Space::new())])
        .bottom_panels([WorkbenchPanel::new("problems", "Problems", Space::new())])
        .status(StatusBar::new())
        .view();
}

#[test]
fn constructs_shell_with_missing_optional_regions() {
    let state = WorkbenchLayoutState::<&str, &str>::default();
    let _view = WorkbenchShell::<_, _, &str, Message>::new(state, Message::Workbench)
        .document_content(text("Empty"))
        .view();
}

#[test]
fn custom_panel_actions_compile_through_shell() {
    let panel = WorkbenchPanel::new("custom", "Custom", Space::new()).action(PanelAction::icon(
        "refresh",
        IconRole::ViewRefresh,
        "Refresh",
    ));

    let state = WorkbenchLayoutState::<&str, &str>::default();
    let _view = WorkbenchShell::new(state, Message::Workbench)
        .left_panels([panel])
        .view();
}

#[test]
fn workbench_event_applies_builtin_view_state_transitions() {
    let mut state = WorkbenchLayoutState::<&str, &str>::default();

    WorkbenchEvent::<&str, &str, &str>::Layout(WorkbenchLayoutChange::SplitRatioChanged {
        region: WorkbenchRegion::Left,
        ratio: 0.4,
    })
    .apply_to(&mut state);
    WorkbenchEvent::<&str, &str, &str>::Panel(WorkbenchPanelEvent::Selected {
        region: WorkbenchRegion::Left,
        panel_id: "files",
    })
    .apply_to(&mut state);
    WorkbenchEvent::<&str, &str, &str>::Document(WorkbenchDocumentEvent::Select("readme"))
        .apply_to(&mut state);

    assert_eq!(state.split_ratios().left, 0.4);
    assert_eq!(state.active_panel(WorkbenchRegion::Left), Some(&"files"));
    assert_eq!(state.active_document(), Some(&"readme"));
}
