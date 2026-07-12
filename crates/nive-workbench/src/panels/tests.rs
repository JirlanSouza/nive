use std::borrow::Cow;
use std::rc::Rc;

use iced::widget::Space;
use nive_ui::IconRole;

use crate::layout::WorkbenchRegion;
use nive_ui::theme::ToneRole;

use super::*;

#[test]
fn selector_defaults_match_regions() {
    assert_eq!(
        PanelSelectorPlacement::default_for_region(WorkbenchRegion::Left),
        PanelSelectorPlacement::SideRail
    );
    assert_eq!(
        PanelSelectorPlacement::default_for_region(WorkbenchRegion::Right),
        PanelSelectorPlacement::SideRail
    );
    assert_eq!(
        PanelSelectorPlacement::default_for_region(WorkbenchRegion::Bottom),
        PanelSelectorPlacement::HeaderTabs
    );
}

#[test]
fn rail_activation_restores_collapsed_region_with_target_panel() {
    let state = WorkbenchPanelHostState::new(WorkbenchRegion::Left);

    let event: WorkbenchPanelEvent<&str, &str> = state.rail_activation_event("files", true);

    assert_eq!(
        event,
        WorkbenchPanelEvent::RestoreRequested {
            region: WorkbenchRegion::Left,
            panel_id: "files"
        }
    );
}

#[test]
fn rail_activation_can_collapse_active_panel() {
    let state = WorkbenchPanelHostState::new(WorkbenchRegion::Left)
        .active_panel("files")
        .collapse_on_active_click(true);

    let event: WorkbenchPanelEvent<&str, &str> = state.rail_activation_event("files", false);

    assert_eq!(
        event,
        WorkbenchPanelEvent::CollapseRequested {
            region: WorkbenchRegion::Left,
            panel_id: "files"
        }
    );
}

#[test]
fn panel_action_icon_constructor_requires_accessible_label() {
    let action = PanelAction::icon("refresh", IconRole::ViewRefresh, "Refresh project");

    assert_eq!(action.accessible_label(), "Refresh project");
}

#[test]
fn panel_chrome_accepts_owned_accessible_labels() {
    let action = PanelAction::icon(
        "refresh",
        IconRole::ViewRefresh,
        String::from("Refresh project"),
    );
    assert!(matches!(action.accessible_label, Cow::Owned(_)));

    let header = PanelHeaderBar {
        region: WorkbenchRegion::Left,
        panel_id: "files",
        title: Cow::Borrowed("Files"),
        icon: None,
        badge: None,
        status: None,
        actions: vec![action],
        collapsible: false,
        restorable: false,
        maximizable: false,
        closable: false,
    };
    let mapper = Rc::new(|_: WorkbenchPanelEvent<&str, &str>| ());
    let _header: nive_ui::Element<'_, ()> = header.view(mapper);

    let rail = PanelRail::new(
        WorkbenchRegion::Left,
        [PanelRailItem::new(
            "files",
            IconRole::Folder,
            String::from("Files"),
        )],
    );
    let _rail: nive_ui::Element<'_, ()> = rail.view();

    let tab: BottomHeaderTab<'_, &str> = BottomHeaderTab::from(
        &WorkbenchPanel::<_, &str, ()>::new("problems", "Problems", Space::new())
            .tooltip(String::from("Project problems")),
    );
    assert!(matches!(tab.tooltip, Some(Cow::Owned(_))));
}

#[test]
fn bottom_header_tab_carries_metadata() {
    let panel: WorkbenchPanel<'_, &str, &str, ()> =
        WorkbenchPanel::new("problems", "Problems", Space::new())
            .icon(IconRole::DialogWarning)
            .badge("3")
            .status(ToneRole::Warning)
            .disabled(true)
            .tooltip("Project problems");

    let tab = BottomHeaderTab::from(&panel);

    assert_eq!(tab.panel_id, "problems");
    assert_eq!(tab.badge.as_deref(), Some("3"));
    assert_eq!(tab.status, Some(ToneRole::Warning));
    assert!(tab.disabled);
}
