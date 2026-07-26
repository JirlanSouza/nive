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
    theme::{testing::ThemeTestGuard, ControlSize, Theme, ThemeDensity, ThemeMode},
    widgets::{Toolbar, ToolbarAction, ToolbarGroup},
    Element, IconRole,
};

use super::Message;
use crate::layout_probe;
use crate::panels::PanelAction;
use crate::{
    StatusBar, StatusItem, WorkbenchDocument, WorkbenchEvent, WorkbenchLayoutChange,
    WorkbenchLayoutState, WorkbenchPanel, WorkbenchRegion, WorkbenchShell,
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
    // Both structural hairlines are overlaid inside their managed extents.
    let toolbar_extent = primary_extent + theme.spacing().xs * 2.0;
    let status_extent = primary_extent;
    let content_gap = 0.0;
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
                .leading(StatusItem::text(status_label))
                .trailing(StatusItem::operation_summary(3, status_label)),
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
    let leading = probes["left_pane"];
    let trailing = probes["center_pane"];
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
        "left_pane",
        "center_pane",
        "right_pane",
        "upper_pane",
        "bottom_pane",
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
    assert!(bottom_controls.y >= bottom_selector.y);
    assert!(
        bottom_controls.y + bottom_controls.height <= bottom_selector.y + bottom_selector.height
    );
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

    assert_split_gap(snapshot, "left_pane", "center_pane", true);
    assert_split_gap(snapshot, "center_pane", "right_pane", true);
    assert_split_gap(snapshot, "upper_pane", "bottom_pane", false);

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

/// Split harness viewport, wide enough that no pane sits on a pixel minimum.
const SPLIT_VIEWPORT: Size = Size::new(1400.0, 900.0);

/// Renders a left/center/right shell and returns its layout probes.
struct SplitHarness {
    element: Element<'static, Message>,
    tree: Tree,
    node: iced::advanced::layout::Node,
    renderer: iced::Renderer,
    cursor: mouse::Cursor,
    viewport: Size,
}

impl SplitHarness {
    fn new(state: WorkbenchLayoutState<&'static str, &'static str>) -> Self {
        Self::sized(state, SPLIT_VIEWPORT)
    }

    fn sized(state: WorkbenchLayoutState<&'static str, &'static str>, viewport: Size) -> Self {
        let mut element = WorkbenchShell::new(state, Message::Workbench)
            .left_panels([
                WorkbenchPanel::new("files", "Files", fill_space()).icon(IconRole::Folder),
                WorkbenchPanel::new("search", "Search", fill_space()).icon(IconRole::EditFind),
            ])
            .documents([WorkbenchDocument::new("readme", "README.md")])
            .document_content(fill_space())
            .right_panels([WorkbenchPanel::new("inspector", "Inspector", fill_space())
                .icon(IconRole::NiveDisclosureRight)])
            .bottom_panels([WorkbenchPanel::new("problems", "Problems", fill_space())])
            .view();

        let mut tree = Tree::new(&element);
        let renderer = test_renderer();
        element.as_widget_mut().diff(&mut tree);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &Limits::new(Size::ZERO, viewport),
        );

        Self {
            element,
            tree,
            node,
            renderer,
            cursor: mouse::Cursor::Unavailable,
            viewport,
        }
    }

    fn probes(&self) -> BTreeMap<&'static str, Rectangle> {
        layout_probe::clear();
        let _ = self.element.as_widget().mouse_interaction(
            &self.tree,
            Layout::new(&self.node),
            mouse::Cursor::Unavailable,
            &Rectangle::with_size(self.viewport),
            &self.renderer,
        );

        layout_probe::snapshot()
    }

    /// Center of the one-pixel divider separating two probed regions.
    fn divider_x(&self, trailing: &'static str) -> f32 {
        self.probes()[trailing].x - 0.5
    }

    fn update(&mut self, event: iced::Event) -> Vec<Message> {
        if let iced::Event::Mouse(mouse::Event::CursorMoved { position }) = event {
            self.cursor = mouse::Cursor::Available(position);
        }

        let mut messages = Vec::new();
        let mut clipboard = iced::advanced::clipboard::Null;
        let mut shell = iced::advanced::Shell::new(&mut messages);
        self.element.as_widget_mut().update(
            &mut self.tree,
            &event,
            Layout::new(&self.node),
            self.cursor,
            &self.renderer,
            &mut clipboard,
            &mut shell,
            &Rectangle::with_size(self.viewport),
        );
        drop(shell);

        messages
    }

    /// Drags a divider horizontally and returns every emitted workbench event.
    ///
    /// The first move only crosses the gesture drag threshold and anchors the
    /// drag origin, so `delta` is measured from that anchor.
    fn drag_x(&mut self, from: Point, delta: f32) -> Vec<Message> {
        self.drag(
            from,
            |point, offset| Point::new(point.x + offset, point.y),
            delta,
        )
    }

    /// Drags a divider vertically, as [`Self::drag_x`] does along the other axis.
    fn drag_y(&mut self, from: Point, delta: f32) -> Vec<Message> {
        self.drag(
            from,
            |point, offset| Point::new(point.x, point.y + offset),
            delta,
        )
    }

    fn drag(
        &mut self,
        from: Point,
        offset: impl Fn(Point, f32) -> Point,
        delta: f32,
    ) -> Vec<Message> {
        let _ = self.update(iced::Event::Mouse(mouse::Event::CursorMoved {
            position: from,
        }));
        let _ = self.update(iced::Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        )));
        // Crosses the 4px drag threshold and becomes the drag anchor.
        let anchor = offset(from, 8.0);
        let _ = self.update(iced::Event::Mouse(mouse::Event::CursorMoved {
            position: anchor,
        }));

        self.update(iced::Event::Mouse(mouse::Event::CursorMoved {
            position: offset(anchor, delta),
        }))
    }
}

fn split_state(left: f32, right: f32) -> WorkbenchLayoutState<&'static str, &'static str> {
    let mut state = WorkbenchLayoutState::default().with_active_document("readme");
    let _ = state.set_region_size(WorkbenchRegion::Left, left);
    let _ = state.set_region_size(WorkbenchRegion::Right, right);
    state
}

fn applied(
    state: &WorkbenchLayoutState<&'static str, &'static str>,
    messages: &[Message],
) -> WorkbenchLayoutState<&'static str, &'static str> {
    let mut next = state.clone();
    for Message::Workbench(event) in messages {
        event.apply_to(&mut next);
    }
    next
}

/// Vertical pitch of a rail item under the harness theme: an icon, its rotated
/// label, and padding.
const RAIL_ITEM_EXTENT: f32 = 72.0;

/// Clicks the rail item at `index`, counting from the rail's top.
fn click_rail_item(harness: &mut SplitHarness, rail: &'static str, index: usize) -> Vec<Message> {
    let bounds = harness.probes()[rail];
    let point = Point::new(
        bounds.center_x(),
        bounds.y + RAIL_ITEM_EXTENT * (index as f32 + 0.5),
    );

    let _ = harness.update(iced::Event::Mouse(mouse::Event::CursorMoved {
        position: point,
    }));
    let _ = harness.update(iced::Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));

    harness.update(iced::Event::Mouse(mouse::Event::ButtonReleased(
        mouse::Button::Left,
    )))
}

fn split_theme() -> ThemeTestGuard {
    ThemeTestGuard::activate(
        Theme::builder("Split coupling harness", ThemeMode::Dark)
            .density(ThemeDensity::Standard)
            .build(),
    )
}

fn maximized_state(
    region: WorkbenchRegion,
    panel: &'static str,
) -> WorkbenchLayoutState<&'static str, &'static str> {
    let mut state = split_state(280.0, 320.0);
    let _ = state.maximize_panel(region, panel);
    let _ = state.set_active_panel(region, panel);
    state
}

#[test]
fn maximizing_keeps_every_rail_and_its_items() {
    let _guard = split_theme();
    let state = maximized_state(WorkbenchRegion::Left, "files");
    let mut harness = SplitHarness::new(state.clone());
    let probes = harness.probes();

    // The maximized region takes the content area but no rail disappears.
    assert!(probes.contains_key("maximized_pane"));
    assert!(probes.contains_key("left_rail"));
    assert!(
        probes.contains_key("right_rail"),
        "the opposite rail vanished while maximized"
    );

    // The maximized region keeps its whole panel list, so a sibling item is
    // still there to click; with only the maximized panel this lands on nothing.
    let switched = applied(&state, &click_rail_item(&mut harness, "left_rail", 1));
    assert_eq!(
        switched.active_panel(WorkbenchRegion::Left),
        Some(&"search")
    );
    assert!(
        switched.maximized().is_some(),
        "switching left the maximized view"
    );
}

#[test]
fn reaching_for_the_opposite_rail_leaves_the_maximized_view() {
    let _guard = split_theme();
    let state = maximized_state(WorkbenchRegion::Left, "files");
    let mut harness = SplitHarness::new(state.clone());
    let restored = applied(&state, &click_rail_item(&mut harness, "right_rail", 0));

    assert!(restored.maximized().is_none());
    assert_eq!(
        restored.active_panel(WorkbenchRegion::Right),
        Some(&"inspector")
    );
    // The ordinary three-region body is back.
    let probes = SplitHarness::new(restored).probes();
    assert!(probes.contains_key("left_pane"));
    assert!(probes.contains_key("right_pane"));
    assert!(!probes.contains_key("maximized_pane"));
}

#[test]
fn clicking_the_active_rail_item_toggles_its_region() {
    let _guard = split_theme();
    let expanded = split_state(280.0, 320.0);

    // The rail paints the first panel as active, so clicking it folds the region.
    let mut harness = SplitHarness::new(expanded.clone());
    let messages = click_rail_item(&mut harness, "left_rail", 0);
    let collapsed = applied(&expanded, &messages);

    assert!(
        collapsed.is_collapsed(WorkbenchRegion::Left),
        "clicking the active rail item did not collapse: {messages:?}"
    );
    assert!(!collapsed.is_collapsed(WorkbenchRegion::Right));

    // Clicking it again brings the region back at the width it kept.
    let mut harness = SplitHarness::new(collapsed.clone());
    let restored = applied(&collapsed, &click_rail_item(&mut harness, "left_rail", 0));

    assert!(!restored.is_collapsed(WorkbenchRegion::Left));
    assert_eq!(
        SplitHarness::new(restored).probes()["left_pane"].width,
        280.0
    );
}

#[test]
fn clicking_an_inactive_rail_item_selects_instead_of_collapsing() {
    let _guard = split_theme();
    let mut state = split_state(280.0, 320.0);
    // Make the second panel active, so the first rail item is now the inactive one.
    let _ = state.set_active_panel(WorkbenchRegion::Left, "search");

    let mut harness = SplitHarness::new(state.clone());
    let next = applied(&state, &click_rail_item(&mut harness, "left_rail", 0));

    assert!(!next.is_collapsed(WorkbenchRegion::Left));
    assert_eq!(next.active_panel(WorkbenchRegion::Left), Some(&"files"));
}

#[test]
fn dragging_a_side_divider_past_its_minimum_collapses_that_region() {
    let _guard = split_theme();
    let state = split_state(280.0, 320.0);
    let mut harness = SplitHarness::new(state.clone());
    let divider = harness.divider_x("center_pane");
    // The left region has 120 of slack above its 160 minimum; the next 32 are
    // over-travel and trip the collapse.
    let messages = harness.drag_x(Point::new(divider, 400.0), -200.0);
    let collapsed = applied(&state, &messages);

    assert!(collapsed.is_collapsed(WorkbenchRegion::Left));
    // The stored width is the one from before the drag, not the minimum the
    // same drag squeezed the region to.
    assert_eq!(collapsed.pane_sizes().left, 280.0);
    assert!(!collapsed.is_collapsed(WorkbenchRegion::Right));

    let probes = SplitHarness::new(collapsed.clone()).probes();
    assert!(!probes.contains_key("left_pane"));
    assert!(probes.contains_key("left_rail"));

    // Restoring through the rail brings the region back at its pre-drag width.
    let mut restored = collapsed;
    let _ = restored.restore_region(WorkbenchRegion::Left);
    assert_eq!(
        SplitHarness::new(restored).probes()["left_pane"].width,
        280.0
    );
}

#[test]
fn the_bottom_divider_stops_at_its_minimum_instead_of_collapsing() {
    let _guard = split_theme();
    let state = split_state(280.0, 320.0);
    let mut harness = SplitHarness::new(state.clone());
    let divider_y = harness.probes()["bottom_pane"].y - 0.5;
    let messages = harness.drag_y(Point::new(400.0, divider_y), 2_000.0);
    let dragged = applied(&state, &messages);

    assert!(!dragged.is_collapsed(WorkbenchRegion::Bottom));
    assert!(SplitHarness::new(dragged).probes()["bottom_pane"].height >= 95.0);
}

#[test]
fn resizing_one_region_never_moves_the_opposite_region() {
    let _guard = split_theme();
    let baseline = SplitHarness::new(split_state(280.0, 320.0)).probes();

    // Swept across each divider's whole travel. The range stops where the shell
    // can still seat both sides plus the centre minimum; past that the
    // documented reverse-order yield takes over, which the drag-to-limit test
    // covers instead.
    for step in 0..=60 {
        let size = 160.0 + 10.0 * step as f32;

        let left_moved = SplitHarness::new(split_state(size, 320.0)).probes();
        assert_eq!(
            left_moved["right_pane"].width, baseline["right_pane"].width,
            "left={size} moved the right region"
        );

        let right_moved = SplitHarness::new(split_state(280.0, size)).probes();
        assert_eq!(
            right_moved["left_pane"].width, baseline["left_pane"].width,
            "right={size} moved the left region"
        );
        assert!(
            right_moved["center_pane"].width >= 239.0,
            "right={size} crushed the centre: {}",
            right_moved["center_pane"].width
        );
    }
}

#[test]
fn dragging_the_right_divider_reports_only_the_right_region() {
    let _guard = split_theme();
    let state = split_state(280.0, 320.0);
    let mut harness = SplitHarness::new(state.clone());
    let divider = harness.divider_x("right_pane");
    let messages = harness.drag_x(Point::new(divider, 400.0), -200.0);

    assert!(!messages.is_empty(), "right divider drag emitted nothing");
    for Message::Workbench(event) in &messages {
        assert!(
            matches!(
                event,
                WorkbenchEvent::Layout(WorkbenchLayoutChange::RegionResized {
                    region: WorkbenchRegion::Right,
                    ..
                })
            ),
            "right divider drag touched another region: {event:?}"
        );
    }

    let dragged = applied(&state, &messages);
    assert_eq!(dragged.pane_sizes().left, state.pane_sizes().left);
    assert!(dragged.pane_sizes().right > state.pane_sizes().right);

    let after = SplitHarness::new(dragged).probes();
    let before = SplitHarness::new(state).probes();
    assert_eq!(
        after["left_pane"].width, before["left_pane"].width,
        "the left region followed the right divider"
    );
}

#[test]
fn a_region_divider_dragged_to_its_limit_still_leaves_the_other_alone() {
    let _guard = split_theme();

    for region in [WorkbenchRegion::Left, WorkbenchRegion::Right] {
        let state = split_state(280.0, 320.0);
        let before = SplitHarness::new(state.clone()).probes();
        let mut harness = SplitHarness::new(state.clone());

        // Far beyond the divider's travel in the direction that grows its region.
        let (probe, delta, opposite) = match region {
            WorkbenchRegion::Left => ("center_pane", 2_000.0, "right_pane"),
            _ => ("right_pane", -2_000.0, "left_pane"),
        };
        let divider = harness.divider_x(probe);
        let messages = harness.drag_x(Point::new(divider, 400.0), delta);
        let after = SplitHarness::new(applied(&state, &messages)).probes();

        assert_eq!(
            after[opposite].width, before[opposite].width,
            "{region:?} at its limit moved {opposite}"
        );
        assert!(
            after["center_pane"].width >= 239.0,
            "{region:?} crushed the centre: {}",
            after["center_pane"].width
        );
    }
}

#[test]
fn dragging_a_divider_moves_it_one_to_one_in_pixels() {
    let _guard = split_theme();
    let state = split_state(280.0, 320.0);
    let before = SplitHarness::new(state.clone()).probes();
    let mut harness = SplitHarness::new(state.clone());
    let divider = harness.divider_x("center_pane");
    let messages = harness.drag_x(Point::new(divider, 400.0), 120.0);

    assert!(!messages.is_empty(), "left divider drag emitted nothing");
    let dragged = applied(&state, &messages);
    assert_eq!(dragged.pane_sizes().right, state.pane_sizes().right);

    let after = SplitHarness::new(dragged.clone()).probes();
    let grew = after["left_pane"].width - before["left_pane"].width;
    assert!(
        (grew - 120.0).abs() <= 1.0,
        "left divider drag was not one-to-one: grew by {grew}"
    );
    assert_eq!(
        after["right_pane"].x, before["right_pane"].x,
        "the right region followed the left divider"
    );
    // The stored size is the region's own width in logical pixels.
    assert_eq!(dragged.pane_sizes().left, after["left_pane"].width);
}

#[test]
fn widening_the_shell_widens_only_the_centre() {
    let _guard = split_theme();
    let state = split_state(280.0, 320.0);
    let narrow = SplitHarness::sized(state.clone(), Size::new(1400.0, 900.0)).probes();
    let wide = SplitHarness::sized(state, Size::new(1920.0, 900.0)).probes();

    assert_eq!(wide["left_pane"].width, narrow["left_pane"].width);
    assert_eq!(wide["right_pane"].width, narrow["right_pane"].width);
    assert_eq!(
        wide["center_pane"].width - narrow["center_pane"].width,
        520.0
    );
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
                assert!(snapshot.maybe_probe("left_pane").is_none());
                assert_in_viewport(snapshot.probe("left_rail"), snapshot.viewport, "left rail");
            }
            WorkbenchRegion::Right => {
                assert!(snapshot.maybe_probe("right_pane").is_none());
                assert_in_viewport(
                    snapshot.probe("right_rail"),
                    snapshot.viewport,
                    "right rail",
                );
            }
            WorkbenchRegion::Bottom => {
                assert!(snapshot.maybe_probe("bottom_pane").is_none());
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
