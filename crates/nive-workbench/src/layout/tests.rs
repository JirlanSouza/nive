use super::*;

#[test]
fn split_ratios_are_clamped() {
    let ratios = WorkbenchSplitRatios::new(-1.0, 2.0, f32::NAN);

    assert_eq!(ratios.left, WorkbenchSplitRatios::MIN);
    assert_eq!(ratios.right, WorkbenchSplitRatios::MAX);
    assert_eq!(ratios.bottom, 0.5);
}

#[test]
fn collapse_and_restore_are_pure_panel_region_transitions() {
    let mut state = WorkbenchLayoutState::<&str, &str>::default();

    assert_eq!(
        state.collapse_region(WorkbenchRegion::Left),
        Some(WorkbenchLayoutChange::RegionCollapsed(
            WorkbenchRegion::Left
        ))
    );
    assert!(state.is_collapsed(WorkbenchRegion::Left));
    assert_eq!(state.collapse_region(WorkbenchRegion::Left), None);
    assert_eq!(state.collapse_region(WorkbenchRegion::Toolbar), None);

    assert_eq!(
        state.restore_region(WorkbenchRegion::Left),
        Some(WorkbenchLayoutChange::RegionRestored(WorkbenchRegion::Left))
    );
    assert!(!state.is_collapsed(WorkbenchRegion::Left));
}

#[test]
fn maximize_restore_preserves_view_state() {
    let mut state = WorkbenchLayoutState::<&str, &str>::default();
    state.set_split_ratio(WorkbenchRegion::Left, 0.31);
    state.collapse_region(WorkbenchRegion::Right);
    state.active_document = Some("doc-a");
    state.set_active_panel(WorkbenchRegion::Left, "files");
    state.set_panel_order(WorkbenchRegion::Left, vec!["files", "search"]);

    state.maximize_panel(WorkbenchRegion::Left, "files");
    state.set_split_ratio(WorkbenchRegion::Left, 0.5);
    state.restore_region(WorkbenchRegion::Right);
    state.active_document = Some("doc-b");
    state.set_active_panel(WorkbenchRegion::Left, "search");
    state.set_panel_order(WorkbenchRegion::Left, vec!["search", "files"]);

    assert_eq!(
        state.restore_maximized(),
        Some(WorkbenchLayoutChange::PanelRestored)
    );
    assert_eq!(state.split_ratios().left, 0.31);
    assert!(state.is_collapsed(WorkbenchRegion::Right));
    assert_eq!(state.active_document(), Some(&"doc-a"));
    assert_eq!(state.active_panel(WorkbenchRegion::Left), Some(&"files"));
    assert_eq!(
        state.panel_order(WorkbenchRegion::Left),
        Some(["files", "search"].as_slice())
    );
    assert!(state.maximized().is_none());
}
