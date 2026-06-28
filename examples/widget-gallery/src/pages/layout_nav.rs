use nive::prelude::*;
use nive::ui::theme::{SurfaceRole, ToneRole};
use nive::ui::widgets::{button as nbutton, text as ntext};
use nive::widget::{column, row};

use crate::app::{DemoTab, Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid};

pub fn view(app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::LayoutNav,
        column![
            section("Tabs and section headers", tabs(app)),
            section("SplitPane", split_pane(app)),
            section("Trees", trees(app)),
            section("Selectable controls", selectable(app)),
        ]
        .spacing(18),
    )
}

fn tabs(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell(
            "TabBar",
            column![
                TabBar::new()
                    .tab(tab(
                        "Overview",
                        DemoTab::Overview,
                        app.layout.tab,
                        IconName::Inbox
                    ))
                    .tab(
                        TabItem::new("Details")
                            .icon(IconName::Info)
                            .selected(app.layout.tab == DemoTab::Details)
                            .dirty(app.layout.dirty_tab)
                            .on_press(Message::SelectTab(DemoTab::Details))
                            .on_close(Message::ToggleDirtyTab)
                    )
                    .tab(
                        TabItem::new("Very long tab label")
                            .selected(app.layout.tab == DemoTab::LongLabel)
                            .on_press(Message::SelectTab(DemoTab::LongLabel))
                    )
                    .tab(TabItem::new("Disabled").disabled(true))
                    .fill(),
                nbutton::secondary("Toggle dirty tab")
                    .shrink()
                    .on_press(Message::ToggleDirtyTab),
            ]
            .spacing(10),
        ),
        example_cell(
            "SectionHeader",
            SectionHeader::new("Resources")
                .status(SectionHeaderStatus::refreshing("Refreshing").tone(ToneRole::Info))
                .action(
                    SectionHeaderAction::icon(IconName::RefreshCw)
                        .tooltip("Refresh")
                        .on_press(Message::Noop),
                )
                .action(
                    SectionHeaderAction::icon(IconName::Plus)
                        .tooltip("Add")
                        .on_press(Message::Noop),
                ),
        ),
        example_cell(
            "Status header",
            SectionHeader::new("Validation")
                .status(SectionHeaderStatus::icon_label(
                    IconName::CheckCircle,
                    "Healthy",
                    ToneRole::Success,
                ))
                .xs(),
        ),
    ])
}

fn split_pane(app: &WidgetGallery) -> Element<'_, Message> {
    column![
        row![
            nbutton::secondary("40 / 60").on_press(Message::SplitRatioChanged(0.4)),
            nbutton::secondary("60 / 40").on_press(Message::SplitRatioChanged(0.6)),
        ]
        .spacing(8),
        SplitPane::new(
            Panel::new(ntext::body("Navigation pane"))
                .padding(14)
                .role(SurfaceRole::Canvas),
            Panel::new(ntext::body("Content pane"))
                .padding(14)
                .role(SurfaceRole::Elevated),
        )
        .ratio(app.layout.split_ratio)
        .min_sizes(160.0, 220.0)
        .available_length(720.0)
        .height(160),
        SplitPane::new(
            Panel::new(ntext::body("Top")).padding(12),
            Panel::new(ntext::body("Bottom")).padding(12),
        )
        .vertical()
        .ratio(0.35)
        .height(180),
    ]
    .spacing(12)
    .into()
}

fn trees(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell(
            "TreeItem",
            column![
                TreeItem::new("examples")
                    .expanded(app.layout.tree_expanded)
                    .leading_icon(IconName::Folder)
                    .selected(true)
                    .on_toggle(Message::ToggleTree),
                TreeItem::new("widget-gallery")
                    .depth(1)
                    .leading_icon(IconName::Folder)
                    .trailing_text("new"),
                TreeItem::new("disabled-file.rs")
                    .depth(2)
                    .disabled(true)
                    .leading_icon(IconName::Edit),
            ]
            .spacing(2),
        ),
        example_cell(
            "OutlineTreeItem",
            column![
                OutlineTreeItem::new("Public widgets")
                    .expanded(true)
                    .on_toggle(Message::Noop),
                OutlineTreeItem::new("Inputs")
                    .depth(1)
                    .selected(true)
                    .on_press(Message::Noop),
                OutlineTreeItem::new("A very long nested item label that tests layout").depth(2),
            ]
            .spacing(2),
        ),
    ])
}

fn selectable(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell(
            "SelectableCard",
            row![
                SelectableCard::new(
                    column![ntext::label_strong("Compact"), ntext::caption("Selected")].spacing(4)
                )
                .selected(app.layout.selected_card == 0)
                .on_press(Message::SelectCard(0)),
                SelectableCard::new(
                    column![ntext::label_strong("Detailed"), ntext::caption("Disabled")].spacing(4)
                )
                .disabled(true)
                .on_press(Message::SelectCard(1)),
            ]
            .spacing(10),
        ),
        example_cell(
            "SelectableItem",
            column![
                SelectableItem::new("Selected item")
                    .selected(app.layout.selected_item == 0)
                    .leading_icon(IconName::Check)
                    .trailing_text("active")
                    .on_press(Message::SelectItem(0)),
                SelectableItem::new("Disabled item with long label")
                    .disabled(true)
                    .leading_color(Color::from_rgb8(218, 78, 78)),
                SelectableItem::new("Compact item")
                    .xs()
                    .shrink()
                    .tooltip("Compact selectable item")
                    .on_press(Message::SelectItem(2)),
            ]
            .spacing(6),
        ),
    ])
}

fn tab(
    label: &'static str,
    tab: DemoTab,
    active: DemoTab,
    icon: IconName,
) -> TabItem<'static, Message> {
    TabItem::new(label)
        .icon(icon)
        .selected(active == tab)
        .on_press(Message::SelectTab(tab))
}
