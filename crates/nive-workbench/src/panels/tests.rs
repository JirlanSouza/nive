use std::borrow::Cow;
use std::rc::Rc;

use iced::widget::Space;
use nive_ui::{
    widgets::{BadgeContent, RailSide, StatusIndicator},
    IconRole,
};

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
fn region_rail_side_matches_window_edges() {
    assert_eq!(WorkbenchRegion::Left.rail_side(), Some(RailSide::Left));
    assert_eq!(WorkbenchRegion::Right.rail_side(), Some(RailSide::Right));
    assert_eq!(WorkbenchRegion::Bottom.rail_side(), None);
    assert_eq!(WorkbenchRegion::Toolbar.rail_side(), None);
    assert_eq!(WorkbenchRegion::Center.rail_side(), None);
    assert_eq!(WorkbenchRegion::Status.rail_side(), None);
}

#[test]
fn host_mode_builders_map_to_presentation_modes() {
    let docked = WorkbenchPanelHostState::<&str>::new(WorkbenchRegion::Left);
    assert_eq!(docked.mode, PanelHostMode::Docked);
    assert_eq!(
        docked.clone().collapsed(true).mode,
        PanelHostMode::Collapsed
    );
    assert_eq!(docked.clone().collapsed(false).mode, PanelHostMode::Docked);
    assert_eq!(
        docked.mode(PanelHostMode::Maximized).mode,
        PanelHostMode::Maximized
    );
}

#[test]
fn rail_activation_restores_collapsed_region_with_target_panel() {
    let state = WorkbenchPanelHostState::new(WorkbenchRegion::Left).collapsed(true);

    let event: WorkbenchPanelEvent<&str, &str> = state.rail_activation_event("files");

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

    let event: WorkbenchPanelEvent<&str, &str> = state.rail_activation_event("files");

    assert_eq!(
        event,
        WorkbenchPanelEvent::CollapseRequested {
            region: WorkbenchRegion::Left,
            panel_id: "files"
        }
    );
}

#[test]
fn rail_activation_selects_panel_when_docked() {
    let state = WorkbenchPanelHostState::new(WorkbenchRegion::Right).active_panel("files");

    let event: WorkbenchPanelEvent<&str, &str> = state.rail_activation_event("logs");

    assert_eq!(
        event,
        WorkbenchPanelEvent::Selected {
            region: WorkbenchRegion::Right,
            panel_id: "logs"
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
        tooltip: None,
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
        RailSide::Left,
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
            .count_badge(3)
            .status_text(ToneRole::Warning, "Problems present")
            .disabled(true)
            .tooltip("Project problems");

    let tab = BottomHeaderTab::from(&panel);

    assert_eq!(tab.panel_id, "problems");
    assert!(matches!(tab.badge, Some(BadgeContent::Count(3))));
    assert_eq!(
        tab.status.as_ref().map(StatusIndicator::tone),
        Some(ToneRole::Warning)
    );
    assert_eq!(
        tab.status.as_ref().map(StatusIndicator::label),
        Some("Problems present")
    );
    assert!(tab.disabled);
}

#[test]
fn generic_badge_forwarder_remains_status_while_typed_count_stays_numeric() {
    let status: WorkbenchPanel<'_, &str, &str, ()> =
        WorkbenchPanel::new("status", "Status", Space::new()).badge(String::from("Review"));
    let count: WorkbenchPanel<'_, &str, &str, ()> =
        WorkbenchPanel::new("count", "Count", Space::new()).count_badge(120);

    assert!(matches!(
        status.badge_content_value(),
        Some(BadgeContent::Status(label)) if label == "Review"
    ));
    assert!(matches!(
        count.badge_content_value(),
        Some(BadgeContent::Count(120))
    ));
}

#[test]
fn panel_rail_maps_identity_and_preserves_disabled_items() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Message {
        Select(&'static str),
    }

    let disabled =
        PanelRailItem::new("problems", IconRole::DialogWarning, "Problems").disabled(true);
    let rail = PanelRail::new(
        RailSide::Right,
        [
            PanelRailItem::new("files", IconRole::Folder, "Files"),
            disabled.clone(),
        ],
    )
    .on_select(Message::Select);

    let mapper = rail.on_select.as_ref().expect("rail mapper");

    assert_eq!(mapper("files"), Message::Select("files"));
    assert_eq!(rail.items[0].label, "Files");
    assert!(rail.items[1].disabled);
    assert_eq!(rail.items[1].id(), disabled.id());
}
