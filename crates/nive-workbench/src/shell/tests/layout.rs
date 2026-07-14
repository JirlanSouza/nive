use std::collections::BTreeMap;

use iced::{
    advanced::{
        layout::{Layout, Limits},
        mouse,
        widget::Tree,
    },
    widget::{container, Space},
    Font, Length, Pixels, Point, Rectangle, Size,
};
use nive_ui::{
    theme::{testing::ThemeTestGuard, ControlSize, GapRole, Theme, ThemeDensity, ThemeMode},
    widgets::{Toolbar, ToolbarAction, ToolbarGroup},
    Element, IconRole,
};

use super::Message;
use crate::layout_probe;
use crate::panels::PanelAction;
use crate::{
    StatusBar, StatusItem, WorkbenchDocument, WorkbenchLayoutState, WorkbenchPanel,
    WorkbenchRegion, WorkbenchShell,
};

#[derive(Clone, Copy)]
enum ShellMode {
    Expanded,
    Collapsed(WorkbenchRegion),
    Maximized,
}

struct LayoutSnapshot {
    root: Rectangle,
    viewport: Rectangle,
    primary_extent: f32,
    toolbar_extent: f32,
    status_extent: f32,
    content_gap: f32,
    tight_gap: f32,
    probes: BTreeMap<&'static str, Rectangle>,
}

impl LayoutSnapshot {
    fn probe(&self, name: &'static str) -> Rectangle {
        *self
            .probes
            .get(name)
            .unwrap_or_else(|| panic!("missing layout probe: {name}"))
    }

    fn maybe_probe(&self, name: &'static str) -> Option<Rectangle> {
        self.probes.get(name).copied()
    }
}

fn snapshot(
    density: ThemeDensity,
    size: ControlSize,
    viewport_size: Size,
    mode: ShellMode,
    long_content: bool,
) -> LayoutSnapshot {
    let theme = Theme::builder("Workbench layout harness", ThemeMode::Dark)
        .density(density)
        .build();
    let primary_extent = theme.control_metrics(size).height;
    // `Toolbar` owns its bottom edge: a 1px hairline is appended below the
    // fixed-height toolbar content (see `widgets::navigation::toolbar`).
    const TOOLBAR_BOTTOM_EDGE: f32 = 1.0;
    let toolbar_extent =
        primary_extent + theme.spacing().xxs * 2.0 + theme.spacing().xs * 2.0 + TOOLBAR_BOTTOM_EDGE;
    // `StatusBar` owns its top edge: a 1px hairline is prepended above the
    // fixed-height status content (see `crate::status`).
    const STATUS_TOP_EDGE: f32 = 1.0;
    let status_extent = primary_extent + STATUS_TOP_EDGE;
    let content_gap = theme.gap(GapRole::Content);
    let tight_gap = theme.spacing().xs;
    let _theme_guard = ThemeTestGuard::activate(theme);
    let viewport = Rectangle::with_size(viewport_size);
    let mut element = build_shell(size, mode, long_content);
    let mut tree = Tree::new(&element);
    let renderer = test_renderer();

    element.as_widget_mut().diff(&mut tree);
    let node = element.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &Limits::new(Size::ZERO, viewport_size),
    );
    let root = Layout::new(&node).bounds();

    layout_probe::clear();
    let _ = element.as_widget().mouse_interaction(
        &tree,
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &viewport,
        &renderer,
    );

    LayoutSnapshot {
        root,
        viewport,
        primary_extent,
        toolbar_extent,
        status_extent,
        content_gap,
        tight_gap,
        probes: layout_probe::snapshot(),
    }
}

fn build_shell(
    size: ControlSize,
    mode: ShellMode,
    long_content: bool,
) -> Element<'static, Message> {
    let mut state = WorkbenchLayoutState::default().with_active_document("readme");
    match mode {
        ShellMode::Expanded => {}
        ShellMode::Collapsed(region) => {
            let _ = state.collapse_region(region);
        }
        ShellMode::Maximized => {
            let _ = state.maximize_panel(WorkbenchRegion::Left, "files");
        }
    }

    let long_label = "A deliberately long workbench chrome label that must remain contained";
    let document_label = if long_content {
        long_label
    } else {
        "README.md"
    };
    let bottom_label = if long_content {
        "A long bottom selector label that must remain inside its tab track"
    } else {
        "Problems"
    };
    let status_label = if long_content {
        "A long status item that must stay inside the fixed footer band"
    } else {
        "Ready"
    };

    WorkbenchShell::new(state, Message::Workbench)
        .chrome_size(size)
        .toolbar(
            Toolbar::new().xs().group(
                ToolbarGroup::new()
                    .action(ToolbarAction::icon(IconRole::ViewRefresh).tooltip("Refresh")),
            ),
        )
        .left_panels([WorkbenchPanel::new("files", "Files", fill_space())
            .icon(IconRole::Folder)
            .action(PanelAction::icon(
                "refresh-files",
                IconRole::ViewRefresh,
                "Refresh files",
            ))])
        .documents([WorkbenchDocument::new("readme", document_label)])
        .document_content(fill_space())
        .right_panels([WorkbenchPanel::new("inspector", "Inspector", fill_space())
            .icon(IconRole::NiveDisclosureRight)
            .action(PanelAction::icon(
                "refresh-inspector",
                IconRole::ViewRefresh,
                "Refresh inspector",
            ))])
        .bottom_panels([
            WorkbenchPanel::new("problems", bottom_label, fill_space()).action(
                PanelAction::icon_text("clear-problems", IconRole::WindowClose, "Clear problems"),
            ),
            WorkbenchPanel::new("logs", "Logs", fill_space()),
        ])
        .status(
            StatusBar::new()
                .item(StatusItem::text(status_label))
                .item(StatusItem::Spacer)
                .item(StatusItem::operation_summary(3, status_label)),
        )
        .view()
}

fn fill_space() -> Element<'static, Message> {
    container(Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn test_renderer() -> iced::Renderer {
    iced_renderer::fallback::Renderer::Secondary(iced_tiny_skia::Renderer::new(
        Font::default(),
        Pixels(14.0),
    ))
}

fn left_split_interaction(
    shell_size: ControlSize,
    expected_hit_size: ControlSize,
) -> mouse::Interaction {
    let theme = Theme::builder("Split propagation harness", ThemeMode::Dark)
        .density(ThemeDensity::Standard)
        .build();
    let _theme_guard = ThemeTestGuard::activate(theme);
    let mut element = build_shell(shell_size, ShellMode::Expanded, false);
    let mut tree = Tree::new(&element);
    let renderer = test_renderer();
    let viewport = Rectangle::with_size(Size::new(1440.0, 900.0));

    element.as_widget_mut().diff(&mut tree);
    let node = element.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &Limits::new(Size::ZERO, viewport.size()),
    );

    layout_probe::clear();
    let _ = element.as_widget().mouse_interaction(
        &tree,
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &viewport,
        &renderer,
    );
    let probes = layout_probe::snapshot();
    let leading = probes["left_split_leading"];
    let trailing = probes["left_split_trailing"];
    let hit_thickness = 12.0_f32.max(theme.control_metrics(expected_hit_size).icon_size);
    let point = Point::new(
        trailing.x - 0.5 - hit_thickness / 2.0 + 0.25,
        leading.y + leading.height / 2.0,
    );

    element.as_widget().mouse_interaction(
        &tree,
        Layout::new(&node),
        mouse::Cursor::Available(point),
        &viewport,
        &renderer,
    )
}

fn assert_in_viewport(bounds: Rectangle, viewport: Rectangle, name: &str) {
    assert!(bounds.width >= 0.0, "{name} width is negative: {bounds:?}");
    assert!(
        bounds.height >= 0.0,
        "{name} height is negative: {bounds:?}"
    );
    assert!(
        bounds.x >= viewport.x,
        "{name} escapes viewport left: {bounds:?}"
    );
    assert!(
        bounds.y >= viewport.y,
        "{name} escapes viewport top: {bounds:?}"
    );
    assert!(
        bounds.x + bounds.width <= viewport.x + viewport.width,
        "{name} escapes viewport right: {bounds:?}"
    );
    assert!(
        bounds.y + bounds.height <= viewport.y + viewport.height,
        "{name} escapes viewport bottom: {bounds:?}"
    );
}

fn assert_common_expanded_geometry(snapshot: &LayoutSnapshot) {
    assert_eq!(snapshot.root, snapshot.viewport);
    assert_in_viewport(snapshot.root, snapshot.viewport, "root");

    for name in [
        "toolbar",
        "body",
        "status",
        "document_tabs",
        "document_content",
        "left_rail",
        "right_rail",
        "left_panel_header",
        "left_panel_content",
        "right_panel_header",
        "right_panel_content",
        "bottom_selector",
        "bottom_tab_track",
        "bottom_controls",
        "bottom_content",
        "left_split_leading",
        "left_split_trailing",
        "right_split_leading",
        "right_split_trailing",
        "bottom_split_leading",
        "bottom_split_trailing",
    ] {
        assert_in_viewport(snapshot.probe(name), snapshot.viewport, name);
    }

    let toolbar = snapshot.probe("toolbar");
    let body = snapshot.probe("body");
    let status = snapshot.probe("status");
    assert_eq!(toolbar.height, snapshot.toolbar_extent);
    assert_eq!(toolbar.y + toolbar.height, body.y);
    assert_eq!(body.y + body.height, status.y);

    let tabs = snapshot.probe("document_tabs");
    let document_content = snapshot.probe("document_content");
    assert_eq!(tabs.height, snapshot.primary_extent);
    assert_eq!(tabs.y + tabs.height, document_content.y);

    let left_rail = snapshot.probe("left_rail");
    let right_rail = snapshot.probe("right_rail");
    let left_header = snapshot.probe("left_panel_header");
    let left_content = snapshot.probe("left_panel_content");
    let right_header = snapshot.probe("right_panel_header");
    let right_content = snapshot.probe("right_panel_content");
    assert_eq!(left_rail.width, snapshot.primary_extent);
    assert_eq!(right_rail.width, snapshot.primary_extent);
    assert_eq!(left_header.height, snapshot.primary_extent);
    assert_eq!(right_header.height, snapshot.primary_extent);
    assert_eq!(left_rail.x + left_rail.width, left_header.x);
    assert_eq!(right_header.x + right_header.width, right_rail.x);
    assert_eq!(
        left_content.y - (left_header.y + left_header.height),
        snapshot.content_gap
    );
    assert_eq!(
        right_content.y - (right_header.y + right_header.height),
        snapshot.content_gap
    );

    let bottom_selector = snapshot.probe("bottom_selector");
    let bottom_track = snapshot.probe("bottom_tab_track");
    let bottom_controls = snapshot.probe("bottom_controls");
    let bottom_content = snapshot.probe("bottom_content");
    assert_eq!(bottom_selector.height, snapshot.primary_extent);
    assert_eq!(bottom_track.height, snapshot.primary_extent);
    assert_eq!(bottom_track.y, bottom_controls.y);
    assert!(
        bottom_track.width > 0.0,
        "bottom tab track has no remaining width"
    );
    assert_eq!(
        bottom_track.x + bottom_track.width + snapshot.tight_gap,
        bottom_controls.x
    );
    assert!(bottom_track.x >= bottom_selector.x);
    assert!(bottom_track.x + bottom_track.width <= bottom_controls.x);
    assert_eq!(
        bottom_content.y - (bottom_selector.y + bottom_selector.height),
        snapshot.content_gap
    );

    assert_split_gap(snapshot, "left_split_leading", "left_split_trailing", true);
    assert_split_gap(
        snapshot,
        "right_split_leading",
        "right_split_trailing",
        true,
    );
    assert_split_gap(
        snapshot,
        "bottom_split_leading",
        "bottom_split_trailing",
        false,
    );

    let status_content = snapshot.probe("status_content");
    assert_eq!(status.height, snapshot.status_extent);
    assert!(status_content.x >= status.x);
    assert!(status_content.y >= status.y);
    assert!(status_content.y + status_content.height <= status.y + status.height);
    assert!(status.width <= snapshot.viewport.width);
}

fn assert_split_gap(
    snapshot: &LayoutSnapshot,
    leading_name: &'static str,
    trailing_name: &'static str,
    horizontal: bool,
) {
    let leading = snapshot.probe(leading_name);
    let trailing = snapshot.probe(trailing_name);
    let gap = if horizontal {
        trailing.x - (leading.x + leading.width)
    } else {
        trailing.y - (leading.y + leading.height)
    };

    assert_eq!(gap, 1.0, "{leading_name}/{trailing_name} divider gap");
    if horizontal {
        assert!(leading.x + leading.width <= trailing.x);
    } else {
        assert!(leading.y + leading.height <= trailing.y);
    }
}

#[test]
fn standard_density_propagates_every_chrome_size_through_the_expanded_shell() {
    for size in [
        ControlSize::Xs,
        ControlSize::Sm,
        ControlSize::Md,
        ControlSize::Lg,
    ] {
        let snapshot = snapshot(
            ThemeDensity::Standard,
            size,
            Size::new(1440.0, 900.0),
            ShellMode::Expanded,
            false,
        );

        assert_common_expanded_geometry(&snapshot);
    }
}

#[test]
fn shell_chrome_size_overrides_a_pre_sized_toolbar_and_reaches_split_interaction() {
    assert_eq!(
        left_split_interaction(ControlSize::Lg, ControlSize::Lg),
        mouse::Interaction::ResizingColumn
    );
    assert_ne!(
        left_split_interaction(ControlSize::Xs, ControlSize::Lg),
        mouse::Interaction::ResizingColumn
    );
}

#[test]
fn constrained_compact_and_comfortable_shells_contain_long_chrome_content() {
    for (density, viewport) in [
        (ThemeDensity::Compact, Size::new(800.0, 600.0)),
        (ThemeDensity::Comfortable, Size::new(1024.0, 480.0)),
    ] {
        let snapshot = snapshot(
            density,
            ControlSize::Sm,
            viewport,
            ShellMode::Expanded,
            true,
        );

        assert_common_expanded_geometry(&snapshot);
    }
}

#[test]
fn collapsed_regions_omit_only_their_split_without_expanding_the_shell() {
    for region in [
        WorkbenchRegion::Left,
        WorkbenchRegion::Right,
        WorkbenchRegion::Bottom,
    ] {
        let snapshot = snapshot(
            ThemeDensity::Standard,
            ControlSize::Sm,
            Size::new(800.0, 600.0),
            ShellMode::Collapsed(region),
            false,
        );

        assert_eq!(snapshot.root, snapshot.viewport);
        assert_in_viewport(snapshot.probe("toolbar"), snapshot.viewport, "toolbar");
        assert_in_viewport(snapshot.probe("body"), snapshot.viewport, "body");
        assert_in_viewport(snapshot.probe("status"), snapshot.viewport, "status");
        assert_eq!(snapshot.probe("status").height, snapshot.status_extent);

        match region {
            WorkbenchRegion::Left => {
                assert!(snapshot.maybe_probe("left_split_leading").is_none());
                assert_in_viewport(snapshot.probe("left_rail"), snapshot.viewport, "left rail");
            }
            WorkbenchRegion::Right => {
                assert!(snapshot.maybe_probe("right_split_leading").is_none());
                assert_in_viewport(
                    snapshot.probe("right_rail"),
                    snapshot.viewport,
                    "right rail",
                );
            }
            WorkbenchRegion::Bottom => {
                assert!(snapshot.maybe_probe("bottom_split_leading").is_none());
                assert!(snapshot.maybe_probe("bottom_selector").is_none());
            }
            WorkbenchRegion::Toolbar | WorkbenchRegion::Center | WorkbenchRegion::Status => {
                unreachable!("only panel regions are collapsed")
            }
        }
    }
}

#[test]
fn maximized_panel_preserves_toolbar_status_and_viewport_bounds() {
    let snapshot = snapshot(
        ThemeDensity::Standard,
        ControlSize::Sm,
        Size::new(1440.0, 900.0),
        ShellMode::Maximized,
        false,
    );

    assert_eq!(snapshot.root, snapshot.viewport);
    assert_in_viewport(snapshot.probe("toolbar"), snapshot.viewport, "toolbar");
    assert_in_viewport(snapshot.probe("body"), snapshot.viewport, "body");
    assert_in_viewport(snapshot.probe("status"), snapshot.viewport, "status");
    assert_eq!(snapshot.probe("status").height, snapshot.status_extent);
    assert_in_viewport(
        snapshot.probe("left_panel_header"),
        snapshot.viewport,
        "maximized panel header",
    );
    assert_in_viewport(
        snapshot.probe("left_panel_content"),
        snapshot.viewport,
        "maximized panel content",
    );
}
