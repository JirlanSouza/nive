use super::*;

#[test]
fn pane_sizes_are_normalized() {
    let sizes = WorkbenchPaneSizes::new(-1.0, f32::INFINITY, f32::NAN);

    assert_eq!(sizes.left, 0.0);
    assert_eq!(sizes.right, 0.0);
    assert_eq!(sizes.bottom, 0.0);
    assert_eq!(WorkbenchPaneSizes::new(280.0, 320.0, 200.0).left, 280.0);
}

#[test]
fn region_sizes_round_trip_only_for_panel_regions() {
    let mut sizes = WorkbenchPaneSizes::default();
    sizes.set_region_size(WorkbenchRegion::Left, 300.0);
    sizes.set_region_size(WorkbenchRegion::Center, 999.0);

    assert_eq!(sizes.region_size(WorkbenchRegion::Left), Some(300.0));
    assert_eq!(sizes.region_size(WorkbenchRegion::Center), None);
    assert_eq!(
        sizes
            .with_region_size(WorkbenchRegion::Bottom, 180.0)
            .bottom,
        180.0
    );
}

#[test]
fn collapse_and_restore_are_pure_panel_region_transitions() {
    let mut state = WorkbenchLayoutState::<&str, &str>::default();

    assert_eq!(
        state.collapse_region(WorkbenchRegion::Left),
        Some(WorkbenchLayoutChange::RegionCollapsed {
            region: WorkbenchRegion::Left,
            restore_size: None,
        })
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
    state.set_region_size(WorkbenchRegion::Left, 310.0);
    state.collapse_region(WorkbenchRegion::Right);
    state.active_document = Some("doc-a");
    state.set_active_panel(WorkbenchRegion::Left, "files");
    state.set_panel_order(WorkbenchRegion::Left, vec!["files", "search"]);

    state.maximize_panel(WorkbenchRegion::Left, "files");
    state.set_region_size(WorkbenchRegion::Left, 500.0);
    state.restore_region(WorkbenchRegion::Right);
    state.active_document = Some("doc-b");
    state.set_active_panel(WorkbenchRegion::Left, "search");
    state.set_panel_order(WorkbenchRegion::Left, vec!["search", "files"]);

    assert_eq!(
        state.restore_maximized(),
        Some(WorkbenchLayoutChange::PanelRestored)
    );
    assert_eq!(state.pane_sizes().left, 310.0);
    assert!(state.is_collapsed(WorkbenchRegion::Right));
    assert_eq!(state.active_document(), Some(&"doc-a"));
    assert_eq!(state.active_panel(WorkbenchRegion::Left), Some(&"files"));
    assert_eq!(
        state.panel_order(WorkbenchRegion::Left),
        Some(["files", "search"].as_slice())
    );
    assert!(state.maximized().is_none());
}
