use crate::theme::{ControlSize, Theme, ThemeDensity, ThemeMode, ToneRole};
use crate::widgets::navigation::overflow::{Overflow, OverflowDirection};
use crate::widgets::primitives::IconRole;

use super::content::item_tooltip;
use super::item::{VerticalRailBadge, VerticalRailItem};
use super::label::rotation_radians;
use super::layout::{ellipsize_label, item_layout, metrics_for_theme, RailMetrics};
use super::widget::{VerticalRail, CHEVRON_SCROLL_STEP_FACTOR};
use super::RailSide;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Message {
    Select(&'static str),
}

fn test_metrics() -> RailMetrics {
    RailMetrics {
        size: ControlSize::Sm,
        width: 28.0,
        radius: 6.0,
        item_padding_v: 4.0,
        gap: 3.0,
        icon_size: 14.0,
        font_size: 12.0,
        line_height: 14.0,
        min_label_track: 40.0,
        max_label_track: 70.0,
    }
}

#[test]
fn item_metadata_builders_store_contract_fields() {
    let item = VerticalRailItem::new("explorer", "Explorer")
        .icon(IconRole::Folder)
        .selected(true)
        .disabled(true)
        .badge(
            VerticalRailBadge::new("3")
                .warning()
                .description("3 warnings"),
        )
        .tooltip("Open explorer");
    let badge = item.badge.as_ref().expect("badge should be stored");

    assert_eq!(item.id(), &"explorer");
    assert_eq!(item.label(), "Explorer");
    assert_eq!(item.icon, Some(IconRole::Folder));
    assert!(item.selected);
    assert!(item.disabled);
    assert_eq!(badge.label(), "3");
    assert_eq!(badge.tone_role(), ToneRole::Warning);
    assert_eq!(badge.description_text(), Some("3 warnings"));
    assert_eq!(item.tooltip.as_deref(), Some("Open explorer"));
}

#[test]
fn side_maps_to_expected_rotation_direction() {
    assert_eq!(
        rotation_radians(RailSide::Left),
        -std::f32::consts::FRAC_PI_2
    );
    assert_eq!(
        rotation_radians(RailSide::Right),
        std::f32::consts::FRAC_PI_2
    );
}

#[test]
fn independent_selection_is_per_item() {
    let items = [
        VerticalRailItem::new("a", "A").selected(true),
        VerticalRailItem::new("b", "B").selected(true),
        VerticalRailItem::new("c", "C"),
    ];

    assert_eq!(items.iter().filter(|item| item.selected).count(), 2);
}

#[test]
fn enabled_activation_maps_through_rail_callback() {
    let rail = VerticalRail::new(RailSide::Left).on_select(Message::Select);
    let enabled = VerticalRailItem::new("enabled", "Enabled");

    assert_eq!(
        rail.item_activation(&enabled),
        Some(Message::Select("enabled"))
    );
}

#[test]
fn disabled_activation_suppresses_rail_callback() {
    let rail = VerticalRail::new(RailSide::Left).on_select(Message::Select);
    let disabled = VerticalRailItem::new("disabled", "Disabled").disabled(true);

    assert_eq!(rail.item_activation(&disabled), None);
}

#[test]
fn item_height_includes_same_vertical_padding_used_by_content() {
    let metrics = test_metrics();
    let item = VerticalRailItem::new("explorer", "Explorer").icon(IconRole::Folder);
    let layout = item_layout(&item, metrics);

    assert_eq!(metrics.item_padding_v, 4.0);
    assert_eq!(
        layout.height,
        (metrics.item_padding_v * 2.0 + layout.label_track + metrics.icon_size + metrics.gap)
            .ceil()
    );
}

#[test]
fn truncation_adds_ellipsis_and_tooltip_fallback_can_use_full_label() {
    let metrics = test_metrics();
    let label = "Very long vertical rail label";
    let (visible, truncated) = ellipsize_label(label, 42.0, metrics);

    assert!(truncated);
    assert!(visible.ends_with('…'));

    let item = VerticalRailItem::new("long", label);
    let tooltip = item_tooltip(&item, truncated);

    assert_eq!(tooltip.as_deref(), Some(label));
}

#[test]
fn explicit_tooltip_overrides_truncation_fallback() {
    let item = VerticalRailItem::new("long", "Very long vertical rail label").tooltip("Custom");
    let tooltip = item_tooltip(&item, true);

    assert_eq!(tooltip.as_deref(), Some("Custom"));
}

#[test]
fn badge_description_composes_item_tooltip() {
    let item = VerticalRailItem::new("problems", "Problems")
        .badge(VerticalRailBadge::new("3").danger().description("3 errors"));

    assert_eq!(
        item_tooltip(&item, false).as_deref(),
        Some("Problems — 3 errors")
    );
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

#[test]
fn width_matches_control_metrics_across_densities_and_sizes() {
    for density in ThemeDensity::ALL {
        let theme = Theme::builder("VerticalRail metric test", ThemeMode::Dark)
            .density(density)
            .build();

        for size in [
            ControlSize::Xs,
            ControlSize::Sm,
            ControlSize::Md,
            ControlSize::Lg,
        ] {
            assert_eq!(
                metrics_for_theme(theme, size).width,
                theme.control_metrics(size).height
            );
        }
    }
}
