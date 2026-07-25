use iced::widget::rule;

use crate::theme::{
    ControlRole, ControlSize, ControlState, TextRole, Theme, ThemeDensity, ThemeMode,
};
use crate::widgets::primitives::IconRole;

use super::content::item_tooltip;
use super::item::SideRailItem;
use super::label::{rotation_radians, RailLabelCanvas};
use super::layout::{ellipsize_label, item_layout, metrics_for_theme, RailMetrics};
use super::style::selected_accent_style;
use super::RailSide;

fn test_label(selected: bool, disabled: bool) -> RailLabelCanvas {
    RailLabelCanvas {
        text: "Explorer".to_string(),
        side: RailSide::Left,
        font_size: 12.0,
        line_height: 14.0,
        selected,
        disabled,
    }
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
fn active_label_reads_secondary() {
    let theme = Theme::Dark;
    let label = test_label(false, false);

    assert_eq!(
        label.label_color(&theme),
        theme.text(TextRole::Secondary).color
    );
}

#[test]
fn selected_label_reflects_selected_state() {
    let theme = Theme::Dark;
    let label = test_label(true, false);
    let selected = theme.control(
        crate::theme::ControlRole::Selectable,
        crate::theme::ControlState::SELECTED,
    );

    assert_eq!(label.label_color(&theme), selected.foreground);
    assert_ne!(
        label.label_color(&theme),
        theme.text(TextRole::Secondary).color
    );
}

#[test]
fn disabled_label_reflects_disabled_state() {
    let theme = Theme::Dark;
    let label = test_label(false, true);
    let disabled = theme.control(
        crate::theme::ControlRole::Selectable,
        crate::theme::ControlState::DISABLED,
    );

    assert_eq!(label.label_color(&theme), disabled.foreground);
    assert_ne!(
        label.label_color(&theme),
        theme.text(TextRole::Secondary).color
    );
}

#[test]
fn item_metadata_builders_store_contract_fields() {
    let item = SideRailItem::new("explorer", "Explorer")
        .icon(IconRole::Folder)
        .selected(true)
        .disabled(true)
        .tooltip("Open explorer");

    assert_eq!(item.id(), &"explorer");
    assert_eq!(item.label(), "Explorer");
    assert_eq!(item.icon, Some(IconRole::Folder));
    assert!(item.selected);
    assert!(item.disabled);
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
        SideRailItem::new("a", "A").selected(true),
        SideRailItem::new("b", "B").selected(true),
        SideRailItem::new("c", "C"),
    ];

    assert_eq!(items.iter().filter(|item| item.selected).count(), 2);
}

#[test]
fn item_height_accounts_for_every_band_it_lays_out() {
    let metrics = test_metrics();
    let labelled = item_layout(&SideRailItem::<&str>::new("explorer", "Explorer"), metrics);
    let with_icon = item_layout(
        &SideRailItem::new("explorer", "Explorer").icon(IconRole::Folder),
        metrics,
    );

    assert_eq!(metrics.item_padding_v, 4.0);
    assert_eq!(
        labelled.height,
        (metrics.item_padding_v * 2.0 + labelled.label_track).ceil()
    );
    assert_eq!(
        with_icon.height,
        (metrics.item_padding_v * 2.0 + with_icon.label_track + metrics.icon_size + metrics.gap)
            .ceil()
    );
}

#[test]
fn truncation_adds_ellipsis_and_tooltip_fallback_can_use_full_label() {
    let metrics = test_metrics();
    let label = "Very long side rail label";
    let (visible, truncated) = ellipsize_label(label, 42.0, metrics);

    assert!(truncated);
    assert!(visible.ends_with('…'));

    let item = SideRailItem::new("long", label);
    let tooltip = item_tooltip(&item, truncated);

    assert_eq!(tooltip.as_deref(), Some(label));
}

#[test]
fn explicit_tooltip_overrides_truncation_fallback() {
    let item = SideRailItem::new("long", "Very long side rail label").tooltip("Custom");
    let tooltip = item_tooltip(&item, true);

    assert_eq!(tooltip.as_deref(), Some("Custom"));
}

#[test]
fn an_untruncated_item_without_an_explicit_tooltip_has_none() {
    let item = SideRailItem::new("short", "Logs");

    assert_eq!(item_tooltip(&item, false), None);
}

#[test]
fn selected_accent_paints_its_complete_box() {
    // The fill mode, not the layout box, decides how much of the rule is
    // painted: a full-height box with a padded fill still draws a short stub.
    let style = selected_accent_style()(&Theme::Dark);

    assert_eq!(style.fill_mode, rule::FillMode::Full);
    assert_eq!(
        style.color,
        Theme::Dark
            .control(ControlRole::Selectable, ControlState::SELECTED)
            .foreground
    );
}

#[test]
fn width_matches_control_metrics_across_densities_and_sizes() {
    for density in ThemeDensity::ALL {
        let theme = Theme::builder("SideRail metric test", ThemeMode::Dark)
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
