use std::time::Duration;

use iced::advanced::mouse;
use iced::Size;

use super::geometry::{
    autoscroll_step, edge_scroll_direction, insertion_marker_bounds, legal_reorder_target,
    owns_wheel_event, singleton_payload, snapshot_tab_region,
};
use super::*;
use crate::interaction::{ContextTarget, DropDecision};

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
enum Message {
    Select(u8),
    Close(TabCloseRequest<u8>),
    Context(ContextRequest<u8>),
    Drop(TabDrop<u8>),
    Tear(TabTearOff<u8>),
}

fn item(id: u8) -> TabItem<'static, u8> {
    TabItem::new(id, "tab")
}

fn state() -> TabBarState<u8> {
    TabBarState::default()
}

#[test]
fn item_requires_id_and_label_without_display_bound() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Id(u8);

    let item = TabItem::new(Id(1), "Overview");

    assert_eq!(item.id(), &Id(1));
    assert_eq!(item.label(), "Overview");
}

#[test]
fn pinned_tabs_render_as_leading_partition() {
    let bar: TabBar<'_, u8, Message> =
        TabBar::new(1).tabs([item(1), item(2).pinned(true), item(3), item(4).pinned(true)]);

    let display: Vec<u8> = bar
        .displayed_tabs()
        .into_iter()
        .map(|d| d.item.id)
        .collect();

    assert_eq!(display, vec![2, 4, 1, 3]);
}

#[test]
fn middle_click_close_requires_closable_item_and_callback() {
    let bar: TabBar<'_, u8, Message> = TabBar::new(1)
        .tabs([item(1).closable(true), item(2)])
        .on_close_request(Message::Close);

    assert_eq!(
        bar.close_request(TabRegion::Tab(0)),
        Some(TabCloseRequest {
            id: 1,
            trigger: TabCloseTrigger::MiddleClick
        })
    );
    assert_eq!(bar.close_request(TabRegion::Tab(1)), None);

    let disabled: TabBar<'_, u8, Message> = TabBar::new(1).tabs([item(1).closable(true)]);

    assert_eq!(disabled.close_request(TabRegion::Tab(0)), None);
}

#[test]
fn context_request_uses_singleton_or_empty_snapshot() {
    let bar: TabBar<'_, u8, Message> = TabBar::new(1).tabs([item(1), item(2)]);

    let tab = bar
        .context_request(TabRegion::Tab(1), Point::new(10.0, 20.0))
        .expect("context");
    assert_eq!(tab.target, ContextTarget::Item(2));
    assert_eq!(tab.selection.selected, vec![2]);
    assert_eq!(tab.selection.focused, Some(2));

    let empty = bar
        .context_request(TabRegion::Empty, Point::new(30.0, 20.0))
        .expect("context");
    assert_eq!(empty.target, ContextTarget::Empty);
    assert!(empty.selection.selected.is_empty());
}

#[test]
fn tab_pinned_icon_role_is_distinct_from_chevrons_and_menu() {
    assert!(IconRole::TabPinned != IconRole::NiveDisclosureLeft);
    assert!(IconRole::TabPinned != IconRole::NiveDisclosureRight);
    assert!(IconRole::TabPinned != IconRole::ViewMore);
    assert!(IconRole::TabPinned != IconRole::OpenMenu);
}

#[test]
fn default_state_has_no_overflow() {
    let st = state();

    assert!(!st.has_overflow);
    assert_eq!(st.scroll_offset, 0.0);
    assert_eq!(st.max_scroll, 0.0);
    assert!(st.dragged_id.is_none());
    assert!(st.insertion_target.is_none());
    assert!(!st.menu_open.get());
}

#[test]
fn wheel_ownership_requires_overflow_and_pointer_inside_tab_bar() {
    let bounds = Rectangle::new(Point::new(10.0, 20.0), Size::new(200.0, 28.0));

    assert!(owns_wheel_event(
        true,
        bounds,
        mouse::Cursor::Available(Point::new(30.0, 30.0))
    ));
    assert!(!owns_wheel_event(
        true,
        bounds,
        mouse::Cursor::Available(Point::new(30.0, 60.0))
    ));
    assert!(!owns_wheel_event(
        false,
        bounds,
        mouse::Cursor::Available(Point::new(30.0, 30.0))
    ));
    assert!(!owns_wheel_event(true, bounds, mouse::Cursor::Unavailable));
}

#[test]
fn singleton_payload_is_singleton() {
    let payload = singleton_payload(7_u8);

    assert_eq!(payload.ids, vec![7]);
    assert_eq!(payload.root_ids, vec![7]);
}

#[test]
fn legal_reorder_target_rejects_cross_zone() {
    let tabs = [
        TabItem::new(1_u8, "A").pinned(true),
        TabItem::new(2_u8, "B"),
    ];
    let dragged_pinned = &tabs[0];
    let dragged_unpinned = &tabs[1];

    assert!(legal_reorder_target(
        dragged_pinned,
        &TabDropTarget::Before(1),
        &tabs
    ));
    assert!(!legal_reorder_target(
        dragged_unpinned,
        &TabDropTarget::Before(1),
        &tabs
    ));
    assert!(!legal_reorder_target(
        dragged_pinned,
        &TabDropTarget::After(2),
        &tabs
    ));
    assert!(legal_reorder_target(
        dragged_unpinned,
        &TabDropTarget::After(2),
        &tabs
    ));
}

#[test]
fn tear_off_payload_shape_is_singleton() {
    let tear = TabTearOff {
        payload: singleton_payload(7),
        position: Point::new(100.0, 24.0),
    };

    assert_eq!(tear.payload.ids, vec![7]);
    assert_eq!(tear.payload.root_ids, vec![7]);
    assert_eq!(tear.position, Point::new(100.0, 24.0));
}

#[test]
fn reorder_per_segment_rejects_cross_zone_pinned_drop() {
    let bar: TabBar<'_, u8, Message> = TabBar::new(1)
        .tabs([item(1).pinned(true), item(2)])
        .on_reorder(Message::Drop);

    let _state = TabBarState::<u8>::default();

    // Place pinned at x=0, unpinned at x=200. Pointer over the unpinned
    // segment but dragged tab is pinned.
    let tab_bounds = vec![
        (
            1_u8,
            Rectangle::new(Point::new(0.0, 0.0), Size::new(100.0, 40.0)),
            true,
        ),
        (
            2_u8,
            Rectangle::new(Point::new(200.0, 0.0), Size::new(100.0, 40.0)),
            false,
        ),
    ];

    // Drag the pinned tab and ask for a decision at the unpinned drop
    // position; must reject.
    let decision_pinned_over_unpinned =
        bar.reorder_decision(1, Point::new(220.0, 0.0), &tab_bounds);
    assert!(!decision_pinned_over_unpinned.is_accept());

    // Drag the unpinned tab and ask for a decision at the pinned drop
    // position; must reject.
    let decision_unpinned_over_pinned = bar.reorder_decision(2, Point::new(20.0, 0.0), &tab_bounds);
    assert!(!decision_unpinned_over_pinned.is_accept());

    // Same segment valid drops must accept.
    let decision_pinned_in_pinned = bar.reorder_decision(1, Point::new(20.0, 0.0), &tab_bounds);
    assert!(decision_pinned_in_pinned.is_accept());
    let decision_unpinned_in_unpinned =
        bar.reorder_decision(2, Point::new(220.0, 0.0), &tab_bounds);
    assert!(decision_unpinned_in_unpinned.is_accept());
}

#[test]
fn drag_to_trailing_empty_space_accepts_after_last_same_segment() {
    let bar: TabBar<'_, u8, Message> = TabBar::new(1)
        .tabs([item(1), item(2), item(3)])
        .on_reorder(Message::Drop);
    let tab_bounds = vec![
        (
            1_u8,
            Rectangle::new(Point::new(0.0, 0.0), Size::new(50.0, 30.0)),
            false,
        ),
        (
            2_u8,
            Rectangle::new(Point::new(60.0, 0.0), Size::new(50.0, 30.0)),
            false,
        ),
        (
            3_u8,
            Rectangle::new(Point::new(120.0, 0.0), Size::new(50.0, 30.0)),
            false,
        ),
    ];

    assert_eq!(
        bar.reorder_decision(1, Point::new(220.0, 0.0), &tab_bounds),
        DropDecision::accept(TabDropTarget::After(3), TransferOperation::Move)
    );
}

#[test]
fn insertion_marker_bounds_before_and_after_geometry() {
    let tab_bounds = vec![(
        7_u8,
        Rectangle::new(Point::new(20.0, 10.0), Size::new(80.0, 30.0)),
        false,
    )];

    assert_eq!(
        insertion_marker_bounds(&TabDropTarget::Before(7), &tab_bounds, 4.0),
        Some(Rectangle::new(
            Point::new(17.0, 10.0),
            Size::new(INSERTION_MARKER_WIDTH, 30.0)
        ))
    );
    assert_eq!(
        insertion_marker_bounds(&TabDropTarget::After(7), &tab_bounds, 4.0),
        Some(Rectangle::new(
            Point::new(101.0, 10.0),
            Size::new(INSERTION_MARKER_WIDTH, 30.0)
        ))
    );
}

#[test]
fn menu_entries_reflect_active_set_after_tabs() {
    let entries = TabBar::<'_, u8, Message>::new(None)
        .tabs([item(1), item(2)])
        .active(2)
        .menu_entries();

    assert_eq!(
        entries,
        vec![
            AllTabsMenuEntry {
                id: 1,
                label: Cow::Borrowed("tab"),
                icon: None,
                active: false,
                dirty: false,
                pinned: false,
                disabled: false,
            },
            AllTabsMenuEntry {
                id: 2,
                label: Cow::Borrowed("tab"),
                icon: None,
                active: true,
                dirty: false,
                pinned: false,
                disabled: false,
            }
        ]
    );
}

#[test]
fn menu_entries_keep_disabled_items_visible_and_inert() {
    let entries = TabBar::<'_, u8, Message>::new(1)
        .tabs([item(1), item(2).disabled(true)])
        .menu_entries();

    assert_eq!(entries.len(), 2);
    assert!(entries[0].active);
    assert!(entries[1].disabled);
}

#[test]
fn composite_focus_starts_active_and_skips_disabled_items() {
    let bar: TabBar<'_, u8, Message> =
        TabBar::new(2).tabs([item(1), item(2), item(3).disabled(true), item(4)]);
    let mut state = state();

    bar.reconcile_focus(&mut state);
    assert_eq!(state.focused_id, Some(2));

    bar.move_focus(&mut state, FocusMovement::Next);
    assert_eq!(state.focused_id, Some(4));
    bar.move_focus(&mut state, FocusMovement::Last);
    assert_eq!(state.focused_id, Some(4));
    bar.move_focus(&mut state, FocusMovement::First);
    assert_eq!(state.focused_id, Some(1));
}

#[test]
fn composite_focus_survives_reorder_by_id_and_recovers_after_removal() {
    let mut state = state();
    state.focused_id = Some(2);
    state.previous_focus_order = vec![1, 2, 3];

    let reordered: TabBar<'_, u8, Message> = TabBar::new(1).tabs([item(3), item(2), item(1)]);
    reordered.reconcile_focus(&mut state);
    assert_eq!(state.focused_id, Some(2));

    let removed: TabBar<'_, u8, Message> = TabBar::new(3).tabs([item(3), item(1)]);
    removed.reconcile_focus(&mut state);
    assert_eq!(state.focused_id, Some(3));
}

#[test]
fn menu_entries_are_pinned_first_and_preserve_metadata() {
    let entries = TabBar::<'_, u8, Message>::new(2)
        .tabs([
            item(1).dirty(true),
            item(2).pinned(true).icon(IconRole::Folder),
            item(3).disabled(true),
        ])
        .menu_entries();

    assert_eq!(
        entries.iter().map(|entry| entry.id).collect::<Vec<_>>(),
        vec![2, 1, 3]
    );
    assert!(entries[0].active);
    assert!(entries[0].pinned);
    assert_eq!(entries[0].icon, Some(IconRole::Folder));
    assert!(entries[1].dirty);
    assert!(entries[2].disabled);
}

#[test]
fn snapshot_tab_region_uses_cached_bounds() {
    let tab_bounds = vec![
        (
            1_u8,
            Rectangle::new(Point::new(10.0, 0.0), Size::new(50.0, 30.0)),
            false,
        ),
        (
            2_u8,
            Rectangle::new(Point::new(70.0, 0.0), Size::new(50.0, 30.0)),
            false,
        ),
    ];

    assert_eq!(
        snapshot_tab_region(&tab_bounds, &[], None, None, None, Point::new(20.0, 10.0)),
        TabRegion::Tab(0)
    );
    assert_eq!(
        snapshot_tab_region(&tab_bounds, &[], None, None, None, Point::new(80.0, 10.0)),
        TabRegion::Tab(1)
    );
    assert_eq!(
        snapshot_tab_region(&tab_bounds, &[], None, None, None, Point::new(200.0, 10.0)),
        TabRegion::Empty
    );
}

#[test]
fn chevron_scroll_offsets_clamped() {
    let mut state = TabBarState::<u8>::default();
    state.overflow.update_extents(180.0, 100.0);
    state.overflow.offset = 5.0;

    state
        .overflow
        .page_step(OverflowDirection::Backward, CHEVRON_SCROLL_STEP_FACTOR);
    state.scroll_offset = state.overflow.offset;
    assert_eq!(state.scroll_offset, 0.0);
    assert!(!state.overflow.show_start_chevron());
    assert!(state.overflow.show_end_chevron());

    state
        .overflow
        .page_step(OverflowDirection::Forward, CHEVRON_SCROLL_STEP_FACTOR);
    state.scroll_offset = state.overflow.offset;
    assert_eq!(state.scroll_offset, 80.0);
    assert!(state.overflow.show_start_chevron());
    assert!(!state.overflow.show_end_chevron());
}

#[test]
fn edge_autoscroll_direction_respects_zone_and_endpoints() {
    let strip = Rectangle::new(Point::new(10.0, 20.0), Size::new(200.0, 28.0));

    assert_eq!(
        edge_scroll_direction(Point::new(15.0, 30.0), strip, 28.0, 40.0, 100.0),
        Some(OverflowDirection::Backward)
    );
    assert_eq!(
        edge_scroll_direction(Point::new(205.0, 30.0), strip, 28.0, 40.0, 100.0),
        Some(OverflowDirection::Forward)
    );
    assert_eq!(
        edge_scroll_direction(Point::new(15.0, 30.0), strip, 28.0, 0.0, 100.0),
        None
    );
}

#[test]
fn edge_autoscroll_step_clamps_suspended_frames_to_fifty_ms() {
    assert_eq!(
        autoscroll_step(OverflowDirection::Forward, Duration::from_millis(500)),
        18.0
    );
    assert_eq!(
        autoscroll_step(OverflowDirection::Backward, Duration::from_millis(50)),
        -18.0
    );
}
