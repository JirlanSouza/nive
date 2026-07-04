use nive::prelude::*;
use nive::ui::theme::{SurfaceRole, ToneRole};
use nive::ui::widgets::{button as nbutton, text as ntext};
use nive::ui::{SelectionMode, TransferOperation};
use nive::widget::{column, row};

use crate::app::{DemoTab, DemoTreeNode, Message, WidgetGallery};
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
    // Keep `Tree` and `TreeItem` side by side: `Tree` owns hierarchy
    // interactions, while `TreeItem` remains the focused row primitive sample.
    let expanded = &app.layout.expanded_tree_nodes;
    let examples_expanded = expanded.contains(&DemoTreeNode::Examples);
    let widget_gallery_expanded = expanded.contains(&DemoTreeNode::WidgetGallery);

    let mode = app.layout.tree_selection_mode;
    let tree = Tree::new(tree_nodes(
        app.layout.tree_deferred_loaded,
        app.layout.tree_deferred_loading,
    ))
    .state(&app.layout.tree_state)
    .selection_mode(mode)
    .size(app.control_size)
    .drag(
        TreeDrag::enabled()
            .operations([TransferOperation::Move, TransferOperation::Copy])
            .can_drop(|drop| !matches!(drop.target, TreeDropTarget::Into(DemoTreeNode::Target))),
    )
    .height(260)
    .on_event(Message::TreeEvent);

    let tree_item_rows = column![
        TreeItem::new("examples")
            .expanded(examples_expanded)
            .leading_icon(IconName::Folder)
            .selected(true)
            .size(app.control_size)
            .on_toggle(Message::ToggleTree(DemoTreeNode::Examples)),
        TreeItem::new("widget-gallery")
            .depth(1)
            .expanded(widget_gallery_expanded)
            .leading_icon(IconName::Folder)
            .trailing_text("primitive")
            .size(app.control_size)
            .on_toggle(Message::ToggleTree(DemoTreeNode::WidgetGallery)),
        TreeItem::new("disabled target")
            .depth(1)
            .leaf()
            .disabled(true)
            .leading_icon(IconName::Folder)
            .trailing_text("ignored")
            .size(app.control_size),
    ]
    .spacing(2);

    row![
        Panel::new(
            column![
                row![
                    ntext::caption("Tree"),
                    Badge::info(match mode {
                        SelectionMode::Multiple => "Multi-select",
                        SelectionMode::Single => "Single-select",
                        SelectionMode::None => "Selection off",
                    }),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    nbutton::secondary("Single")
                        .shrink()
                        .on_press(Message::TreeSelectionModeChanged(SelectionMode::Single)),
                    nbutton::secondary("Multiple")
                        .shrink()
                        .on_press(Message::TreeSelectionModeChanged(SelectionMode::Multiple)),
                ]
                .spacing(8),
                tree,
                tree_feedback(app),
            ]
            .spacing(10)
        )
        .role(SurfaceRole::Panel)
        .padding(14)
        .width(Length::FillPortion(2)),
        Panel::new(column![ntext::caption("TreeItem primitive"), tree_item_rows].spacing(10))
            .role(SurfaceRole::Panel)
            .padding(14)
            .width(Length::FillPortion(1)),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

fn tree_nodes(deferred_loaded: bool, deferred_loading: bool) -> Vec<TreeNode<'static, DemoTreeNode>> {
    let remote_branch = if deferred_loaded {
        TreeNode::branch(
            DemoTreeNode::RemotePackages,
            "remote-packages",
            [
                TreeNode::leaf(DemoTreeNode::RemoteSchema, "schema.json")
                    .leading_icon(IconName::Edit)
                    .trailing_text("loaded"),
                TreeNode::leaf(DemoTreeNode::RemoteCache, "cache.bin")
                    .leading_icon(IconName::Copy)
                    .trailing_text("2 MB"),
            ],
        )
        .leading_icon(IconName::Folder)
        .tone(ToneRole::Success)
        .trailing_text("loaded")
    } else {
        TreeNode::branch_deferred(DemoTreeNode::RemotePackages, "remote-packages")
            .leading_icon(IconName::Inbox)
            .tone(ToneRole::Info)
            .trailing_text(if deferred_loading { "loading" } else { "deferred" })
    };

    vec![TreeNode::branch(
        DemoTreeNode::Examples,
        "examples",
        [
            TreeNode::branch(
                DemoTreeNode::WidgetGallery,
                "widget-gallery",
                [
                    TreeNode::leaf(DemoTreeNode::CargoToml, "Cargo.toml")
                        .leading_icon(IconName::Edit),
                    TreeNode::leaf(DemoTreeNode::Target, "target")
                        .leading_icon(IconName::Folder)
                        .disabled(true)
                        .trailing_text("disabled"),
                    TreeNode::branch(
                        DemoTreeNode::Src,
                        "src",
                        [
                            TreeNode::leaf(DemoTreeNode::AppRs, "app.rs")
                                .leading_icon(IconName::Edit)
                                .trailing_text("modified"),
                            TreeNode::branch(
                                DemoTreeNode::Pages,
                                "pages",
                                [
                                    TreeNode::leaf(DemoTreeNode::LayoutNavRs, "layout_nav.rs")
                                        .leading_icon(IconName::Edit)
                                        .tone(ToneRole::Warning),
                                    TreeNode::leaf(DemoTreeNode::InputsRs, "inputs.rs")
                                        .leading_icon(IconName::Edit),
                                ],
                            )
                            .leading_icon(IconName::Folder),
                        ],
                    )
                    .leading_icon(IconName::Folder),
                    remote_branch,
                ],
            )
            .leading_icon(IconName::Folder)
            .trailing_text("new"),
        ],
    )
    .leading_icon(IconName::Folder)]
}

fn tree_feedback(app: &WidgetGallery) -> Element<'_, Message> {
    column![
        ntext::body_small(&app.layout.tree_event_feedback),
        ntext::body_small(&app.layout.tree_context_feedback),
        ntext::body_small(&app.layout.tree_clipboard_feedback),
        ntext::body_small(&app.layout.tree_drop_feedback),
    ]
    .spacing(4)
    .into()
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
