use std::time::Duration;

use iced::{
    advanced::{
        layout::{Layout, Node},
        mouse,
    },
    Event, Point, Rectangle, Size, Vector,
};

#[cfg(test)]
use super::TabItem;
use super::{DisplayedTab, HitGeometry, TabDropTarget, TabRegion, INSERTION_MARKER_WIDTH};
use crate::interaction::CollectionTransferPayload;
use crate::widgets::navigation::overflow::OverflowDirection;

/// Room a tab label has before the tab would exceed `max_tab_width`.
///
/// Tab width is capped after layout, so a label measured against the full
/// content width gets sliced mid-word by the cap. Handing this budget to the
/// label lets it ellipsize while it is still being measured.
pub(super) fn label_budget(
    metrics: super::style::TabBarMetrics,
    leading_icons: usize,
    status_side: f32,
) -> f32 {
    let icons = leading_icons as f32 * (metrics.icon_size + metrics.gap);

    (metrics.max_tab_width - metrics.padding_h * 2.0 - status_side - metrics.gap - icons).max(0.0)
}

pub(super) fn snapshot_tab_region<Id>(
    tab_bounds: &[(Id, Rectangle, bool)],
    close_bounds: &[(Id, Rectangle)],
    left_chevron: Option<Rectangle>,
    right_chevron: Option<Rectangle>,
    all_tabs_button: Option<Rectangle>,
    position: Point,
) -> TabRegion {
    if let Some(bounds) = left_chevron {
        if bounds.contains(position) {
            return TabRegion::ChevronLeft;
        }
    }
    if let Some(bounds) = right_chevron {
        if bounds.contains(position) {
            return TabRegion::ChevronRight;
        }
    }
    if let Some(bounds) = all_tabs_button {
        if bounds.contains(position) {
            return TabRegion::AllTabsButton;
        }
    }
    for (index, (_, bounds)) in close_bounds.iter().enumerate() {
        if bounds.contains(position) {
            return TabRegion::Close(index);
        }
    }
    for (index, (_, bounds, _)) in tab_bounds.iter().enumerate() {
        if bounds.contains(position) {
            return TabRegion::Tab(index);
        }
    }
    TabRegion::Empty
}
pub(super) fn hit_geometry<Id: Clone + Eq>(
    layout: Layout<'_>,
    displayed: &[DisplayedTab<'_, '_, Id>],
    close_enabled: bool,
    close_side: f32,
) -> HitGeometry<Id> {
    let Some(bar_row) = layout.children().next() else {
        return HitGeometry::default();
    };

    let mut bar_children = bar_row.children();
    let left_chevron = bar_children.next();
    let strip = bar_children.next();
    let right_chevron = bar_children.next();
    let all_tabs_button = bar_children.next();

    let strip_bounds = strip.map(|layout| layout.bounds());
    // A scrolled tab keeps its full bounds, which reach past the strip and over
    // whatever sits beside the bar. Interaction follows what the strip shows,
    // so hover, press and close never answer outside the visible window.
    let visible = |bounds: Rectangle| match strip_bounds {
        Some(strip) => bounds.intersection(&strip),
        None => Some(bounds),
    };
    let mut tab_bounds: Vec<(Id, Rectangle, bool)> = Vec::new();
    let mut close_bounds: Vec<(Id, Rectangle)> = Vec::new();

    if let Some(tabs_row) = strip.and_then(|strip| strip.children().next()) {
        for (index, tab_layout) in tabs_row.children().enumerate() {
            let Some(displayed) = displayed.get(index) else {
                continue;
            };
            let item = displayed.item;
            let full = tab_layout.bounds();
            let Some(bounds) = visible(full) else {
                continue;
            };

            tab_bounds.push((item.id.clone(), bounds, item.pinned));

            if close_enabled && item.closable && !item.disabled {
                let close = Rectangle {
                    x: full.x + full.width - close_side,
                    y: full.y,
                    width: close_side,
                    height: full.height,
                };

                if let Some(close) = visible(close) {
                    close_bounds.push((item.id.clone(), close));
                }
            }
        }
    }

    HitGeometry {
        tab_bounds,
        close_bounds,
        left_chevron: visible_slot_bounds(left_chevron),
        right_chevron: visible_slot_bounds(right_chevron),
        all_tabs_button: visible_slot_bounds(all_tabs_button),
        strip_bounds,
    }
}

pub(super) fn event_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.position(),
        Event::Touch(iced::touch::Event::FingerPressed { position, .. }) => Some(*position),
        _ => None,
    }
}

pub(super) fn owns_wheel_event(
    has_overflow: bool,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> bool {
    has_overflow && cursor.is_over(bounds)
}

pub(super) fn visible_slot_bounds(layout: Option<Layout<'_>>) -> Option<Rectangle> {
    layout
        .map(|layout| layout.bounds())
        .filter(|bounds| bounds.width > 0.5)
}

pub(super) fn edge_scroll_direction(
    pointer: Point,
    strip: Rectangle,
    zone: f32,
    offset: f32,
    max_offset: f32,
) -> Option<OverflowDirection> {
    if !strip.contains(pointer) {
        return None;
    }
    if pointer.x <= strip.x + zone && offset > 0.0 {
        Some(OverflowDirection::Backward)
    } else if pointer.x >= strip.x + strip.width - zone && offset < max_offset {
        Some(OverflowDirection::Forward)
    } else {
        None
    }
}

pub(super) fn autoscroll_step(direction: OverflowDirection, elapsed: Duration) -> f32 {
    let distance = 360.0 * elapsed.min(Duration::from_millis(50)).as_secs_f32();
    match direction {
        OverflowDirection::Backward => -distance,
        OverflowDirection::Forward => distance,
    }
}
pub(super) fn singleton_payload<Id: Clone>(id: Id) -> CollectionTransferPayload<Id> {
    CollectionTransferPayload::flat([id])
}
#[cfg(test)]
pub(super) fn legal_reorder_target<Id>(
    dragged: &TabItem<'_, Id>,
    target: &TabDropTarget<Id>,
    tabs: &[TabItem<'_, Id>],
) -> bool
where
    Id: Eq,
{
    let target_pinned = match target {
        TabDropTarget::Before(id) | TabDropTarget::After(id) => tabs
            .iter()
            .find(|tab| &tab.id == id)
            .is_some_and(|tab| tab.pinned),
    };

    dragged.pinned == target_pinned
}
pub(super) fn gesture_to_pointer<Region>(
    gesture: &crate::interaction::PointerGesture<Region>,
    region: Region,
) -> crate::interaction::PointerGesture<Region> {
    crate::interaction::PointerGesture {
        kind: gesture.kind,
        button: gesture.button,
        region,
        position: gesture.position,
        modifiers: gesture.modifiers,
    }
}
/// Walks the freshly-laid-out content node to:
///  - measure total tab-strip content width and strip width,
///  - apply the scroll translation to the tab children,
///  - capture translated viewport-space tab bounds for active reveal.
pub(super) fn measure_and_translate(
    node: Node,
    scroll_offset: f32,
) -> (f32, f32, Node, Vec<Rectangle>) {
    let root_size = node.size();

    let Some(bar_row) = node.children().first() else {
        // Single-row constraint unsatisfied; return node unchanged.
        return (0.0, 0.0, node, Vec::new());
    };
    let Some(strip_container) = bar_row.children().get(1) else {
        return (0.0, 0.0, node, Vec::new());
    };

    let strip_width = strip_container.bounds().width;
    let Some(tabs_row) = strip_container.children().first() else {
        return (0.0, strip_width, node, Vec::new());
    };
    let translate = Vector::new(-scroll_offset, 0.0);

    let mut viewport_tab_bounds = Vec::with_capacity(tabs_row.children().len());
    let mut translated_tabs: Vec<Node> = Vec::with_capacity(tabs_row.children().len());
    // Tab widths are settled during layout — `MinWidth` carries the floor and
    // the ellipsized label keeps the cap — so this only applies the scroll.
    for tab in tabs_row.children() {
        let mut tab = tab.clone();
        tab.translate_mut(translate);
        viewport_tab_bounds.push(tab.bounds());
        translated_tabs.push(tab);
    }
    let content_width = if translated_tabs.is_empty() {
        0.0
    } else {
        tabs_row.bounds().width
    };

    let translated_tabs_row_size = Size::new(content_width, tabs_row.size().height);
    let translated_tabs_row_bounds = tabs_row.bounds();
    let translated_tabs_row = Node::with_children(translated_tabs_row_size, translated_tabs)
        .move_to(translated_tabs_row_bounds.position() + translate);

    let strip_container_size = strip_container.size();
    let strip_container_position = strip_container.bounds().position();
    let translated_strip_container =
        Node::with_children(strip_container_size, vec![translated_tabs_row])
            .move_to(strip_container_position);

    let new_bar_row_children: Vec<Node> = bar_row
        .children()
        .iter()
        .enumerate()
        .map(|(i, c)| {
            if i == 1 {
                translated_strip_container.clone()
            } else {
                c.clone()
            }
        })
        .collect();

    let bar_row_size = bar_row.size();
    let bar_row_position = bar_row.bounds().position();
    let new_bar_row =
        Node::with_children(bar_row_size, new_bar_row_children).move_to(bar_row_position);

    let _ = root_size;
    let new_root = Node::with_children(node.size(), vec![new_bar_row]);

    (content_width, strip_width, new_root, viewport_tab_bounds)
}

pub(super) fn insertion_marker_bounds<Id: Eq>(
    target: &TabDropTarget<Id>,
    tab_bounds: &[(Id, Rectangle, bool)],
    gap: f32,
) -> Option<Rectangle> {
    let (target_id, before) = match target {
        TabDropTarget::Before(id) => (id, true),
        TabDropTarget::After(id) => (id, false),
    };
    let (_, bounds, _) = tab_bounds.iter().find(|(id, _, _)| id == target_id)?;
    let x = if before {
        bounds.x - (gap + INSERTION_MARKER_WIDTH) / 2.0
    } else {
        bounds.x + bounds.width + (gap - INSERTION_MARKER_WIDTH) / 2.0
    };

    Some(Rectangle {
        x,
        y: bounds.y,
        width: INSERTION_MARKER_WIDTH,
        height: bounds.height,
    })
}
#[cfg(test)]
#[allow(dead_code)]
pub(super) fn translate_bounds(bounds: Rectangle, translation: Vector) -> Rectangle {
    Rectangle {
        x: bounds.x + translation.x,
        y: bounds.y + translation.y,
        width: bounds.width,
        height: bounds.height,
    }
}
