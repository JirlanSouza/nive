use iced::{
    advanced::{layout::Layout, layout::Node},
    Rectangle, Size, Vector,
};

use crate::theme::{self, ControlSize, TypographyRole};

use super::item::VerticalRailItem;

pub(super) const HIDDEN_AFFORDANCE_HEIGHT: f32 = 0.1;

const MAX_LABEL_TRACK_FACTOR: f32 = 4.75;
const MIN_LABEL_TRACK_FACTOR: f32 = 1.7;
const AVG_LABEL_ADVANCE_FACTOR: f32 = 0.56;
const ELLIPSIS: &str = "…";

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct RailMetrics {
    pub(super) size: ControlSize,
    pub(super) width: f32,
    pub(super) radius: f32,
    pub(super) item_padding_v: f32,
    pub(super) gap: f32,
    pub(super) icon_size: f32,
    pub(super) font_size: f32,
    pub(super) line_height: f32,
    pub(super) min_label_track: f32,
    pub(super) max_label_track: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ItemLayout {
    pub(super) height: f32,
    pub(super) label_track: f32,
}

pub(super) fn metrics(size: ControlSize) -> RailMetrics {
    let control = theme::control_metrics(size);
    let label = theme::typography(TypographyRole::Label);

    RailMetrics {
        size,
        width: control.height,
        radius: control.radius,
        item_padding_v: control.gap,
        gap: (control.gap * 0.75).max(2.0),
        icon_size: control.icon_size,
        font_size: label.size,
        line_height: label.line_height,
        min_label_track: control.height * MIN_LABEL_TRACK_FACTOR,
        max_label_track: control.height * MAX_LABEL_TRACK_FACTOR,
    }
}

pub(super) fn item_layout<Id>(item: &VerticalRailItem<'_, Id>, metrics: RailMetrics) -> ItemLayout {
    let label_advance = estimated_text_advance(&item.label, metrics);
    let label_track = label_advance
        .clamp(metrics.min_label_track, metrics.max_label_track)
        .ceil();
    let mut height = metrics.item_padding_v * 2.0 + label_track;

    if item.icon.is_some() {
        height += metrics.icon_size + metrics.gap;
    }
    if item.badge.is_some() {
        height += badge_height(metrics.size) + metrics.gap;
    }

    ItemLayout {
        height: height.max(metrics.width).ceil(),
        label_track,
    }
}

pub(super) fn ellipsize_label(
    label: &str,
    max_advance: f32,
    metrics: RailMetrics,
) -> (String, bool) {
    if estimated_text_advance(label, metrics) <= max_advance {
        return (label.to_owned(), false);
    }

    let ellipsis_width = estimated_text_advance(ELLIPSIS, metrics);
    let available = (max_advance - ellipsis_width).max(0.0);
    let mut out = String::new();
    let mut used = 0.0;

    for ch in label.chars() {
        let ch_width = metrics.font_size * AVG_LABEL_ADVANCE_FACTOR;
        if used + ch_width > available {
            break;
        }
        out.push(ch);
        used += ch_width;
    }

    if out.is_empty() {
        (ELLIPSIS.to_owned(), true)
    } else {
        out.push_str(ELLIPSIS);
        (out, true)
    }
}

pub(super) fn measure_and_translate(
    node: Node,
    scroll_offset: f32,
) -> (f32, f32, Node, Vec<Rectangle>) {
    let Some(rail_column) = node.children().first() else {
        return (0.0, 0.0, node, Vec::new());
    };
    let Some(strip_container) = rail_column.children().get(1) else {
        return (0.0, 0.0, node, Vec::new());
    };
    let strip_height = strip_container.bounds().height;
    let Some(items_column) = strip_container.children().first() else {
        return (0.0, strip_height, node, Vec::new());
    };

    let column_y = items_column.bounds().y;
    let translate = Vector::new(0.0, -scroll_offset);
    let mut viewport_item_bounds = Vec::with_capacity(items_column.children().len());
    let mut translated_items = Vec::with_capacity(items_column.children().len());
    let mut content_bottom = column_y;

    for item in items_column.children() {
        let mut item = item.clone();
        item.translate_mut(translate);
        content_bottom = content_bottom.max(item.bounds().y + item.bounds().height + scroll_offset);
        viewport_item_bounds.push(item.bounds());
        translated_items.push(item);
    }

    let content_height = (content_bottom - column_y).max(0.0);
    let translated_items_column = Node::with_children(
        Size::new(items_column.size().width, content_height),
        translated_items,
    )
    .move_to(items_column.bounds().position() + translate);

    let translated_strip_container =
        Node::with_children(strip_container.size(), vec![translated_items_column])
            .move_to(strip_container.bounds().position());

    let rail_children = rail_column
        .children()
        .iter()
        .enumerate()
        .map(|(index, child)| {
            if index == 1 {
                translated_strip_container.clone()
            } else {
                child.clone()
            }
        })
        .collect();
    let translated_rail_column = Node::with_children(rail_column.size(), rail_children)
        .move_to(rail_column.bounds().position());
    let translated_root = Node::with_children(node.size(), vec![translated_rail_column])
        .move_to(node.bounds().position());

    (
        content_height,
        strip_height,
        translated_root,
        viewport_item_bounds,
    )
}

#[derive(Debug, Clone, Copy)]
pub(super) struct HitGeometry {
    pub(super) up_chevron: Option<Rectangle>,
    pub(super) down_chevron: Option<Rectangle>,
}

pub(super) fn hit_geometry(layout: Layout<'_>) -> HitGeometry {
    let Some(rail_column) = layout.children().next() else {
        return HitGeometry {
            up_chevron: None,
            down_chevron: None,
        };
    };
    let mut children = rail_column.children();
    let up = children.next().map(|layout| layout.bounds());
    let _strip = children.next();
    let down = children.next().map(|layout| layout.bounds());

    HitGeometry {
        up_chevron: up.filter(|bounds| bounds.height > HIDDEN_AFFORDANCE_HEIGHT),
        down_chevron: down.filter(|bounds| bounds.height > HIDDEN_AFFORDANCE_HEIGHT),
    }
}

fn estimated_text_advance(label: &str, metrics: RailMetrics) -> f32 {
    label.chars().count() as f32 * metrics.font_size * AVG_LABEL_ADVANCE_FACTOR
}

fn badge_height(size: ControlSize) -> f32 {
    match size {
        ControlSize::Xs => 14.0,
        ControlSize::Sm => 16.0,
        ControlSize::Md | ControlSize::Lg => 18.0,
    }
}
