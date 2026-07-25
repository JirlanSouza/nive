use nive::prelude::*;
use nive::ui::theme::{SurfaceRole, ToneRole};
use nive::ui::interaction::{Orientation, SelectionMode, TransferOperation};
use nive::ui::widgets::controls::button as nbutton;
use nive::ui::widgets::primitives::text as ntext;
use nive::widget::{column, container, row, stack};

use crate::app::{DemoTab, DemoTreeLoadError, DemoTreeNode, Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid};

pub fn view(app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::LayoutNav,
        column![
            section("Tabs and section headers", tabs(app)),
            section("Toolbar structure and overflow", toolbars()),
            section("Panels, scrollbars, and separators", structural_widgets()),
            section("Vertical rails", side_rails(app)),
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
    let empty_tabs: Element<'_, Message> = TabBar::<DemoTab, Message>::new(None)
        .fill_width()
        .into();

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
            "Empty TabBar",
            column![
                empty_tabs,
                ntext::body_small("The strip keeps its Chrome height and bottom seam without placeholder tabs."),
            ]
            .spacing(8),
        ),
        example_cell(
            "SectionHeader",
            SectionHeader::new(
                "Resources with an intentionally long title that yields before controls",
            )
                .title_tooltip(
                    "Resources with an intentionally long title that yields before controls",
                )
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

fn toolbars() -> Element<'static, Message> {
    Toolbar::new()
        .group(
            ToolbarGroup::new()
                .action(
                    ToolbarAction::icon_label(IconRole::ViewRefresh, "Refresh")
                        .on_press(Message::Noop),
                )
                .action(
                    ToolbarAction::icon_label(IconRole::EditFind, "Selected")
                        .selected(true)
                        .on_press(Message::Noop),
                ),
        )
        .separator()
        .group(
            ToolbarGroup::new()
                .action(ToolbarAction::icon_label(IconRole::ViewMore, "Loading").loading(true))
                .action(
                    ToolbarAction::icon_label(IconRole::EditDelete, "Delete")
                        .destructive()
                        .on_press(Message::Noop),
                )
                .action(
                    ToolbarAction::icon_label(IconRole::PreferencesSystem, "Preferences")
                        .on_press(Message::Noop),
                ),
        )
        .width(360)
        .into()
}

fn structural_widgets() -> Element<'static, Message> {
    let overflow = column(
        (0..12).map(|index| ntext::body(format!("Scrollable row {index}")).into()),
    )
    .spacing(6)
    .padding(12);
    let scroll = scrollable(overflow)
        .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
        .height(110);

    column![
        row![
            Panel::new(ntext::body("Structural body inset"))
                .header(SectionHeader::new("Panel anatomy"))
                .body_padding(14)
                .role(SurfaceRole::Sidebar)
                .fill_width(),
            Panel::new(scroll)
                .header(SectionHeader::new("Edge-aligned scrollbar"))
                .shape_md()
                .bordered()
                .role(SurfaceRole::Elevated)
                .fill_width(),
        ]
        .spacing(12),
        Separator::horizontal().subtle(),
        Separator::horizontal().section().inset(24.0, 12.0),
        Separator::horizontal().text_column(40.0),
        row![
            Separator::vertical().subtle(),
            ntext::body_small("Subtle vertical"),
            Separator::vertical().section().inset(8.0, 8.0),
            ntext::body_small("Section vertical"),
        ]
        .spacing(12)
        .height(56),
    ]
    .spacing(12)
    .into()
}

fn side_rails(app: &WidgetGallery) -> Element<'_, Message> {
    let left = SideRail::new(RailSide::Left)
        .size(app.control_size)
        .height(220)
        .on_select(|_| Message::Noop)
        .item(rail_item("explorer", "Explorer", IconRole::Folder).selected(true))
        .item(rail_item("search", "Search", IconRole::EditFind).selected(true))
        .item(rail_item(
            "problems",
            "Problems",
            IconRole::DialogWarning,
        ))
        .item(rail_item(
            "long",
            "Very long tool window label that truncates",
            IconRole::DialogInformation,
        ))
        .item(rail_item("disabled", "Disabled", IconRole::ViewConceal).disabled(true));

    let right = SideRail::new(RailSide::Right)
        .size(app.control_size)
        .height(220)
        .on_select(|_| Message::Noop)
        .item(rail_item("outline", "Outline", IconRole::ListAdd).selected(true))
        .item(rail_item("run", "Run", IconRole::GoNext))
        .item(rail_item("console", "Console", IconRole::OpenMenu))
        .item(rail_item("preview", "Preview", IconRole::ViewReveal))
        .item(rail_item("logs", "Logs", IconRole::EditModify))
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
                        ntext::label_strong("SideRail"),
                        ntext::body_small(
                            "Both sides render a rotated label beside an upright icon, and carry no count marker. The selected item shows a full-height accent on its window-facing edge, opposite the panel-facing seam. The right rail overflows to show chevrons and mouse-wheel scrolling."
                        ),
                    ]
                    .spacing(8)
                )
                .role(SurfaceRole::Panel)
                .body_padding(14)
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
                .body_padding(14)
                .role(SurfaceRole::Canvas),
            Panel::new(ntext::body("Content pane"))
                .body_padding(14)
                .role(SurfaceRole::Elevated),
        )
        .orientation(Orientation::Horizontal)
        .ratio(app.layout.split_ratio)
        .min_sizes(160.0, 220.0)
        .on_change(Message::SplitRatioChanged)
        .snap(0.05)
        .height(160),
        SplitPane::new(
            Panel::new(ntext::body("Top")).body_padding(12),
            Panel::new(ntext::body("Bottom")).body_padding(12),
        )
        .orientation(Orientation::Vertical)
        .ratio(app.layout.vertical_split_ratio)
        .on_change(Message::VerticalSplitRatioChanged)
        .height(180),
        row![
            SplitPane::<Message>::new(ntext::body_small("Locked"), ntext::body_small("Inert"))
                .locked(true)
                .on_change(Message::SplitRatioChanged)
                .height(72),
            SplitPane::<Message>::new(
                ntext::body_small("Display-only"),
                ntext::body_small("No callback")
            )
            .min_sizes(f32::NAN, f32::INFINITY)
            .height(72),
        ]
        .spacing(12),
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
        app.layout.tree_config_failed,
        app.layout.tree_config_loading,
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
    let tree = tree_with_context_menu(app, tree.into());

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
                    Badge::status(match mode {
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
        .body_padding(14)
        .width(Length::FillPortion(2)),
        Panel::new(column![ntext::caption("TreeItem primitive"), tree_item_rows].spacing(10))
            .role(SurfaceRole::Panel)
            .body_padding(14)
            .width(Length::FillPortion(1)),
    ]
    .spacing(12)
    .width(Length::Fill)
    .into()
}

/// Hosts the canonical `Menu` at the Tree's captured context-request
/// position, demonstrating the context-menu-via-Menu boundary: Tree emits
/// `ContextRequested` only, and the application owns menu placement and
/// commands.
fn tree_with_context_menu<'a>(
    app: &'a WidgetGallery,
    tree: Element<'a, Message>,
) -> Element<'a, Message> {
    let Some(menu_state) = app.layout.tree_context_menu else {
        return tree;
    };

    let anchor = nive::widget::Space::new()
        .width(Length::Fixed(0.0))
        .height(Length::Fixed(0.0));

    let menu: Element<'_, Message> = Menu::new(anchor)
        .open(true)
        .on_dismiss(Message::TreeContextMenuDismissed)
        .command(MenuCommand::new("Rename").on_press(Message::TreeContextMenuAction("Rename")))
        .command(MenuCommand::new("Copy").on_press(Message::TreeContextMenuAction("Copy")))
        .command(
            MenuCommand::new("Delete")
                .on_press(Message::TreeContextMenuAction("Delete"))
                .destructive(),
        )
        .into();

    let overlay = container(menu)
        .padding(Padding {
            top: menu_state.position.y,
            right: 0.0,
            bottom: 0.0,
            left: menu_state.position.x,
        })
        .width(Length::Fill)
        .height(Length::Fill);

    stack![tree, overlay].into()
}

fn tree_nodes(
    deferred_loaded: bool,
    deferred_loading: bool,
    config_failed: bool,
    config_loading: bool,
) -> Vec<TreeNode<'static, DemoTreeNode>> {
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
        .status_text(ToneRole::Success, "Loaded")
        .trailing_text("loaded")
    } else {
        TreeNode::branch_deferred(DemoTreeNode::RemotePackages, "remote-packages")
            .leading_icon(IconRole::MailInbox)
            .trailing_text(if deferred_loading { "loading" } else { "deferred" })
    };

    let remote_config = if config_failed {
        TreeNode::branch_failed(DemoTreeNode::RemoteConfig, "remote-config", &DemoTreeLoadError)
            .leading_icon(IconRole::DialogWarning)
            .trailing_text("failed")
    } else {
        TreeNode::branch_deferred(DemoTreeNode::RemoteConfig, "remote-config")
            .leading_icon(IconRole::MailInbox)
            .trailing_text(if config_loading { "loading" } else { "deferred" })
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
                                        .status_text(ToneRole::Warning, "Modified"),
                                    TreeNode::leaf(DemoTreeNode::InputsRs, "inputs.rs")
                                        .leading_icon(IconRole::EditModify),
                                ],
                            )
                            .leading_icon(IconRole::Folder),
                        ],
                    )
                    .leading_icon(IconRole::Folder),
                    remote_branch,
                    remote_config,
                    TreeNode::branch(DemoTreeNode::Archived, "archived", Vec::new())
                        .leading_icon(IconRole::Folder),
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
                    .status_text(ToneRole::Danger, "Unavailable"),
                SelectableItem::new("Compact item")
                    .xs()
                    .shrink_width()
                    .tooltip("Compact selectable item")
                    .on_press(Message::SelectItem(2)),
                SelectableItem::new("Medium operational row")
                    .md()
                    .leading_icon(IconRole::DialogInformation)
                    .trailing_text("42 ms")
                    .on_press(Message::Noop),
                SelectableItem::new("Large semantic status row")
                    .lg()
                    .leading_icon(IconRole::DialogSuccess)
                    .status_text(ToneRole::Success, "Healthy")
                    .on_press(Message::Noop),
            ]
            .spacing(6),
        ),
    ])
}

fn rail_item(
    id: &'static str,
    label: &'static str,
    icon: IconRole,
) -> SideRailItem<'static, &'static str> {
    SideRailItem::new(id, label).icon(icon)
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
