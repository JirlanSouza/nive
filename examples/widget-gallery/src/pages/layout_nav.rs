use nive::prelude::*;
use nive::ui::theme::{SurfaceRole, ToneRole};
use nive::ui::interaction::{Orientation, SelectionMode, TransferOperation};
use nive::ui::widgets::controls::button as nbutton;
use nive::ui::widgets::primitives::text as ntext;
use nive::widget::{column, row};

use crate::app::{DemoTab, DemoTreeNode, Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid};

pub fn view(app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::LayoutNav,
        column![
            section("Tabs and section headers", tabs(app)),
            section("Vertical rails", vertical_rails(app)),
            section("SplitPane", split_pane(app)),
            section("Trees", trees(app)),
            section("Selectable controls", selectable(app)),
        ]
        .spacing(18),
    )
}

fn tabs(app: &WidgetGallery) -> Element<'_, Message> {
    let tab_items: Vec<TabItem<'static, DemoTab>> = app
        .layout
        .tab_order
        .iter()
        .map(|&demo_tab| make_tab(demo_tab, app.layout.dirty_tab))
        .collect();

    variant_grid([
        example_cell(
            "TabBar",
            column![
                TabBar::new(app.layout.tab)
                    .tabs(tab_items)
                    .on_select(Message::SelectTab)
                    .on_close_request(Message::TabCloseRequested)
                    .on_context(Message::TabContextRequested)
                    .on_reorder(Message::TabReordered)
                    .on_tear_off(Message::TabTornOff)
                    .fill_width(),
                ntext::body_small(&app.layout.tab_feedback),
                nbutton::secondary("Toggle dirty tab")
                    .shrink_width()
                    .on_press(Message::ToggleDirtyTab),
            ]
            .spacing(10),
        ),
        example_cell(
            "SectionHeader",
            SectionHeader::new("Resources")
                .status(SectionHeaderStatus::refreshing("Refreshing").tone(ToneRole::Info))
                .action(
                    SectionHeaderAction::icon(IconRole::ViewRefresh)
                        .tooltip("Refresh")
                        .on_press(Message::Noop),
                )
                .action(
                    SectionHeaderAction::icon(IconRole::ListAdd)
                        .tooltip("Add")
                        .on_press(Message::Noop),
                ),
        ),
        example_cell(
            "Status header",
            SectionHeader::new("Validation")
                .status(SectionHeaderStatus::icon_label(
                    IconRole::DialogSuccess,
                    "Healthy",
                    ToneRole::Success,
                ))
                .xs(),
        ),
    ])
}

fn vertical_rails(app: &WidgetGallery) -> Element<'_, Message> {
    let left = VerticalRail::new(RailSide::Left)
        .size(app.control_size)
        .height(220)
        .on_select(|_| Message::Noop)
        .item(
            rail_item("explorer", "Explorer", IconRole::Folder)
                .selected(true)
                .badge(VerticalRailBadge::new("3").success().description("3 healthy services")),
        )
        .item(rail_item("search", "Search", IconRole::EditFind).selected(true))
        .item(
            rail_item("problems", "Problems", IconRole::DialogWarning)
                .badge(VerticalRailBadge::new("!").warning().description("Warnings available")),
        )
        .item(rail_item(
            "long",
            "Very long tool window label that truncates",
            IconRole::DialogInformation,
        ))
        .item(rail_item("disabled", "Disabled", IconRole::ViewConceal).disabled(true));

    let right = VerticalRail::new(RailSide::Right)
        .size(app.control_size)
        .height(220)
        .on_select(|_| Message::Noop)
        .item(rail_item("outline", "Outline", IconRole::ListAdd).selected(true))
        .item(
            rail_item("run", "Run", IconRole::GoNext)
                .badge(VerticalRailBadge::new("1").info().description("1 running task")),
        )
        .item(rail_item("console", "Console", IconRole::OpenMenu))
        .item(
            rail_item("preview", "Preview", IconRole::ViewReveal)
                .badge(VerticalRailBadge::new("12").description("12 previews")),
        )
        .item(
            rail_item("logs", "Logs", IconRole::EditModify)
                .badge(VerticalRailBadge::new("2").danger().description("2 log errors")),
        )
        .item(rail_item("history", "History", IconRole::ViewRefresh))
        .item(rail_item("packages", "Packages", IconRole::MailInbox))
        .item(rail_item("settings", "Settings", IconRole::PreferencesSystem));

    row![
        example_cell(
            "Left + right",
            row![
                left,
                Panel::new(
                    column![
                        ntext::label_strong("VerticalRail"),
                        ntext::body_small(
                            "Both sides render rotated labels with upright metadata. The right rail overflows to show chevrons and mouse-wheel scrolling."
                        ),
                    ]
                    .spacing(8)
                )
                .role(SurfaceRole::Panel)
                .padding(14)
                .width(Length::Fill),
                right,
            ]
            .spacing(12)
            .height(240),
        ),
    ]
    .into()
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
        .orientation(Orientation::Horizontal)
        .ratio(app.layout.split_ratio)
        .min_sizes(160.0, 220.0)
        .on_change(Message::SplitRatioChanged)
        .snap(0.05)
        .height(160),
        SplitPane::new(
            Panel::new(ntext::body("Top")).padding(12),
            Panel::new(ntext::body("Bottom")).padding(12),
        )
        .orientation(Orientation::Vertical)
        .ratio(app.layout.vertical_split_ratio)
        .on_change(Message::VerticalSplitRatioChanged)
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
            .leading_icon(IconRole::Folder)
            .selected(true)
            .size(app.control_size)
            .on_toggle(Message::ToggleTree(DemoTreeNode::Examples)),
        TreeItem::new("widget-gallery")
            .depth(1)
            .expanded(widget_gallery_expanded)
            .leading_icon(IconRole::Folder)
            .trailing_text("primitive")
            .size(app.control_size)
            .on_toggle(Message::ToggleTree(DemoTreeNode::WidgetGallery)),
        TreeItem::new("disabled target")
            .depth(1)
            .leaf()
            .disabled(true)
            .leading_icon(IconRole::Folder)
            .trailing_text("ignored")
            .size(app.control_size),
    ]
    .spacing(2);

    row![
        Panel::new(
            column![
                row![
                    ntext::caption("Tree"),
                    Badge::new(match mode {
                        SelectionMode::Multiple => "Multi-select",
                        SelectionMode::Single => "Single-select",
                        SelectionMode::None => "Selection off",
                    })
                    .info(),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                row![
                    nbutton::secondary("Single")
                        .shrink_width()
                        .on_press(Message::TreeSelectionModeChanged(SelectionMode::Single)),
                    nbutton::secondary("Multiple")
                        .shrink_width()
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
                    .leading_icon(IconRole::EditModify)
                    .trailing_text("loaded"),
                TreeNode::leaf(DemoTreeNode::RemoteCache, "cache.bin")
                    .leading_icon(IconRole::EditCopy)
                    .trailing_text("2 MB"),
            ],
        )
        .leading_icon(IconRole::Folder)
        .tone(ToneRole::Success)
        .trailing_text("loaded")
    } else {
        TreeNode::branch_deferred(DemoTreeNode::RemotePackages, "remote-packages")
            .leading_icon(IconRole::MailInbox)
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
                        .leading_icon(IconRole::EditModify),
                    TreeNode::leaf(DemoTreeNode::Target, "target")
                        .leading_icon(IconRole::Folder)
                        .disabled(true)
                        .trailing_text("disabled"),
                    TreeNode::branch(
                        DemoTreeNode::Src,
                        "src",
                        [
                            TreeNode::leaf(DemoTreeNode::AppRs, "app.rs")
                                .leading_icon(IconRole::EditModify)
                                .trailing_text("modified"),
                            TreeNode::branch(
                                DemoTreeNode::Pages,
                                "pages",
                                [
                                    TreeNode::leaf(DemoTreeNode::LayoutNavRs, "layout_nav.rs")
                                        .leading_icon(IconRole::EditModify)
                                        .tone(ToneRole::Warning),
                                    TreeNode::leaf(DemoTreeNode::InputsRs, "inputs.rs")
                                        .leading_icon(IconRole::EditModify),
                                ],
                            )
                            .leading_icon(IconRole::Folder),
                        ],
                    )
                    .leading_icon(IconRole::Folder),
                    remote_branch,
                ],
            )
            .leading_icon(IconRole::Folder)
            .trailing_text("new"),
        ],
    )
    .leading_icon(IconRole::Folder)]
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
                    .leading_icon(IconRole::ActionConfirm)
                    .trailing_text("active")
                    .on_press(Message::SelectItem(0)),
                SelectableItem::new("Disabled item with long label")
                    .disabled(true)
                    .leading_color(Color::from_rgb8(218, 78, 78)),
                SelectableItem::new("Compact item")
                    .xs()
                    .shrink_width()
                    .tooltip("Compact selectable item")
                    .on_press(Message::SelectItem(2)),
            ]
            .spacing(6),
        ),
    ])
}

fn rail_item(
    id: &'static str,
    label: &'static str,
    icon: IconRole,
) -> VerticalRailItem<'static, &'static str> {
    VerticalRailItem::new(id, label).icon(icon)
}

fn tab(tab: DemoTab, icon: IconRole) -> TabItem<'static, DemoTab> {
    TabItem::new(tab, tab.label()).icon(icon)
}

fn make_tab(demo_tab: DemoTab, dirty_tab: bool) -> TabItem<'static, DemoTab> {
    match demo_tab {
        DemoTab::PinnedNotes => tab(demo_tab, IconRole::DialogInformation).pinned(true),
        DemoTab::Overview => tab(demo_tab, IconRole::MailInbox),
        DemoTab::Details => tab(demo_tab, IconRole::DialogInformation)
            .dirty(dirty_tab)
            .closable(true),
        DemoTab::Console => tab(demo_tab, IconRole::OpenMenu).closable(true),
        DemoTab::Search => tab(demo_tab, IconRole::ViewRefresh).closable(true),
        DemoTab::Preview => tab(demo_tab, IconRole::DialogSuccess).closable(true),
        DemoTab::Logs => tab(demo_tab, IconRole::ListAdd).disabled(true),
        DemoTab::LongLabel => TabItem::new(demo_tab, demo_tab.label()).closable(true),
    }
}
