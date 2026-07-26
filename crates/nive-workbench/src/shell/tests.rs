use iced::widget::{text, Space};
use nive_ui::{
    theme::ControlSize,
    widgets::{SplitStack, SplitStackPane, Toolbar},
    IconRole,
};

use super::*;
use crate::panels::PanelAction;

mod layout;

#[derive(Debug, Clone, PartialEq)]
enum Message {
    Workbench(WorkbenchEvent<&'static str, &'static str, &'static str>),
}

#[test]
fn constructs_shell_with_full_regions() {
    let state = WorkbenchLayoutState::default().with_active_document("readme");
    let _view = WorkbenchShell::new(state, Message::Workbench)
        .toolbar(Toolbar::new())
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
fn chrome_size_defaults_to_small() {
    let shell = WorkbenchShell::new(
        WorkbenchLayoutState::<&str, &str>::default(),
        Message::Workbench,
    );

    assert_eq!(shell.chrome_size, ControlSize::Sm);
}

#[test]
fn pane_constraints_default_and_normalize_app_overrides() {
    let defaults = WorkbenchPaneConstraints::default();
    assert_eq!(
        (defaults.left(), defaults.center(), defaults.right()),
        (160.0, 240.0, 160.0)
    );
    assert_eq!((defaults.upper(), defaults.bottom()), (160.0, 96.0));

    let configured = WorkbenchPaneConstraints::default()
        .left_min(-1.0)
        .center_min(f32::NAN)
        .right_min(180.0)
        .upper_min(f32::INFINITY)
        .bottom_min(120.0);
    assert_eq!(
        (
            configured.left(),
            configured.center(),
            configured.right(),
            configured.upper(),
            configured.bottom(),
        ),
        (0.0, 0.0, 180.0, 0.0, 120.0)
    );
}

#[test]
fn typed_toolbar_and_status_are_retained_until_rendering() {
    let shell = WorkbenchShell::new(
        WorkbenchLayoutState::<&str, &str>::default(),
        Message::Workbench,
    )
    .toolbar(Toolbar::new().xs())
    .status(StatusBar::new())
    .chrome_size(ControlSize::Lg);

    assert_eq!(shell.chrome_size, ControlSize::Lg);
    assert!(shell.toolbar.is_some());
    assert!(shell.status_bar.is_some());
}

#[test]
fn chrome_size_flows_through_all_managed_shell_paths() {
    let state = WorkbenchLayoutState::default().with_active_document("readme");
    let _view = WorkbenchShell::new(state, Message::Workbench)
        .chrome_size(ControlSize::Lg)
        .toolbar(Toolbar::new())
        .left_panels([WorkbenchPanel::new("files", "Files", Space::new())])
        .documents([WorkbenchDocument::new("readme", "README.md")])
        .document_content(text("Document"))
        .right_panels([WorkbenchPanel::new("inspector", "Inspector", Space::new())])
        .bottom_panels([WorkbenchPanel::new("problems", "Problems", Space::new())])
        .status(StatusBar::new())
        .view();
}

#[test]
fn typed_shell_builder_signatures_compile() {
    type Shell = WorkbenchShell<'static, &'static str, &'static str, &'static str, Message>;

    let _toolbar: fn(Shell, Toolbar<'static, Message>) -> Shell = Shell::toolbar;
    let _status: fn(Shell, StatusBar<'static>) -> Shell = Shell::status;
    let _chrome_size: fn(Shell, ControlSize) -> Shell = Shell::chrome_size;
}

#[test]
fn typed_toolbar_builder_order_and_shell_size_precedence_compile() {
    let state = WorkbenchLayoutState::<&str, &str>::default();
    let _configured_before_toolbar = WorkbenchShell::new(state, Message::Workbench)
        .chrome_size(ControlSize::Lg)
        .toolbar(Toolbar::new().xs())
        .status(StatusBar::new())
        .view();

    let state = WorkbenchLayoutState::<&str, &str>::default();
    let _configured_after_toolbar = WorkbenchShell::new(state, Message::Workbench)
        .toolbar(Toolbar::new().xs())
        .status(StatusBar::new())
        .chrome_size(ControlSize::Lg)
        .view();
}

#[test]
fn public_chrome_size_builders_compile() {
    let _shell = WorkbenchShell::new(
        WorkbenchLayoutState::<&str, &str>::default(),
        Message::Workbench,
    )
    .chrome_size(ControlSize::Md);

    let _split_stack = SplitStack::<()>::horizontal()
        .pane(SplitStackPane::fixed(Space::new(), 120.0).minimum(60.0))
        .pane(SplitStackPane::fill(Space::new()))
        .size(ControlSize::Xs)
        .xs()
        .sm()
        .md()
        .lg();
}

#[test]
fn workbench_event_applies_builtin_view_state_transitions() {
    let mut state = WorkbenchLayoutState::<&str, &str>::default();

    WorkbenchEvent::<&str, &str, &str>::Layout(WorkbenchLayoutChange::RegionResized {
        region: WorkbenchRegion::Left,
        size: 400.0,
    })
    .apply_to(&mut state);
    WorkbenchEvent::<&str, &str, &str>::Panel(WorkbenchPanelEvent::Selected {
        region: WorkbenchRegion::Left,
        panel_id: "files",
    })
    .apply_to(&mut state);
    WorkbenchEvent::<&str, &str, &str>::Document(WorkbenchDocumentEvent::Select("readme"))
        .apply_to(&mut state);

    assert_eq!(state.pane_sizes().left, 400.0);
    assert_eq!(state.active_panel(WorkbenchRegion::Left), Some(&"files"));
    assert_eq!(state.active_document(), Some(&"readme"));
}
