use iced::advanced::layout::{Layout, Node};
use iced::{Rectangle, Size, Vector};

use crate::test_support::layout as layout_node;
use crate::widgets::navigation::overflow::{Overflow, OverflowDirection};
use crate::widgets::primitives::IconRole;

use super::item::SideRailItem;
use super::layout::SELECTED_ACCENT_WIDTH;
use super::widget::{seam_bounds, SideRail, CHEVRON_SCROLL_STEP_FACTOR};
use super::RailSide;

const ORIGIN: Vector = Vector::new(50.0, 30.0);

#[derive(Clone, Debug, PartialEq, Eq)]
enum Message {
    Select(&'static str),
}

fn rail_node(side: RailSide) -> Node {
    let rail = SideRail::new(side)
        .on_select(Message::Select)
        .push(SideRailItem::new("a", "A").selected(true))
        .push(SideRailItem::new("b", "B"));

    layout_node(rail.into(), Size::new(400.0, 600.0))
}

fn item_layouts(node: &Node) -> Vec<Layout<'_>> {
    let root = Layout::with_offset(ORIGIN, node);
    let rail_column = root.children().next().expect("rail column");
    let strip = rail_column.children().nth(1).expect("strip");
    let items = strip.children().next().expect("items column");

    items.children().collect()
}

#[test]
fn enabled_activation_maps_through_rail_callback() {
    let rail = SideRail::new(RailSide::Left).on_select(Message::Select);
    let enabled = SideRailItem::new("enabled", "Enabled");

    assert_eq!(
        rail.item_activation(&enabled),
        Some(Message::Select("enabled"))
    );
}

#[test]
fn disabled_activation_suppresses_rail_callback() {
    let rail = SideRail::new(RailSide::Left).on_select(Message::Select);
    let disabled = SideRailItem::new("disabled", "Disabled").disabled(true);

    assert_eq!(rail.item_activation(&disabled), None);
}

#[test]
fn seam_follows_panel_facing_side() {
    let rail = Rectangle::new(iced::Point::new(10.0, 20.0), iced::Size::new(32.0, 300.0));

    assert_eq!(seam_bounds(rail, RailSide::Left).x, 41.0);
    assert_eq!(seam_bounds(rail, RailSide::Right).x, 10.0);
}

#[test]
fn selected_accent_sits_on_the_window_facing_edge_opposite_the_seam() {
    for side in [RailSide::Left, RailSide::Right] {
        let node = rail_node(side);
        let rail_bounds = Layout::with_offset(ORIGIN, &node).bounds();
        let items = item_layouts(&node);
        let selected = items.first().expect("selected item");
        let accent = selected
            .children()
            .nth(1)
            .and_then(|accent| accent.children().next())
            .expect("selected accent");
        let bounds = accent.bounds();
        let expected_x = match side {
            RailSide::Left => rail_bounds.x,
            RailSide::Right => rail_bounds.x + rail_bounds.width - SELECTED_ACCENT_WIDTH,
        };

        assert!(rail_bounds.x >= ORIGIN.x);
        assert_eq!(bounds.x, expected_x);
        assert_eq!(bounds.width, SELECTED_ACCENT_WIDTH);
        assert_ne!(bounds.x, seam_bounds(rail_bounds, side).x);
    }
}

/// Covers the accent's layout box only. How much of that box is painted is
/// decided by its fill mode, asserted in `side_rail_tests`.
#[test]
fn selected_accent_box_matches_the_item_bounds() {
    for side in [RailSide::Left, RailSide::Right] {
        let node = rail_node(side);
        let items = item_layouts(&node);
        let selected = items.first().expect("selected item");
        let accent = selected
            .children()
            .nth(1)
            .and_then(|accent| accent.children().next())
            .expect("selected accent");

        assert_eq!(accent.bounds().height, selected.bounds().height);
        assert_eq!(accent.bounds().y, selected.bounds().y);
    }
}

#[test]
fn unselected_items_compose_no_accent() {
    let node = rail_node(RailSide::Left);
    let items = item_layouts(&node);

    assert_eq!(items[0].children().count(), 2);
    assert_eq!(items[1].children().count(), 1);
}

#[test]
fn items_past_the_strip_stay_measurable_for_overflow() {
    let mut rail = SideRail::new(RailSide::Left).on_select(Message::Select);
    for id in ["a", "b", "c", "d", "e", "f"] {
        rail = rail.push(SideRailItem::new(id, id).icon(IconRole::Folder));
    }

    let node = layout_node(rail.into(), Size::new(400.0, 200.0));
    let root = Layout::new(&node);
    let rail_column = root.children().next().expect("rail column");
    let strip = rail_column.children().nth(1).expect("strip");
    let items = strip.children().next().expect("items column");
    let heights: Vec<f32> = items.children().map(|item| item.bounds().height).collect();
    let content_bottom = items
        .children()
        .map(|item| item.bounds().y + item.bounds().height)
        .fold(f32::MIN, f32::max);

    assert!(heights.iter().all(|height| *height > 0.0));
    assert!(content_bottom > strip.bounds().y + strip.bounds().height);
}

#[test]
fn overflow_state_clamps_offsets_and_chevrons_transition() {
    let mut overflow = Overflow::default();
    overflow.update_extents(180.0, 100.0);
    overflow.offset = 5.0;

    overflow.page_step(OverflowDirection::Backward, CHEVRON_SCROLL_STEP_FACTOR);
    assert_eq!(overflow.offset, 0.0);
    assert!(!overflow.show_start_chevron());
    assert!(overflow.show_end_chevron());

    overflow.page_step(OverflowDirection::Forward, CHEVRON_SCROLL_STEP_FACTOR);
    assert_eq!(overflow.offset, 80.0);
    assert!(overflow.show_start_chevron());
    assert!(!overflow.show_end_chevron());
}
