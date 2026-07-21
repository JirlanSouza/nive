use std::borrow::Cow;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    widget::{text, Column, Space},
    Event, Length, Point, Rectangle, Size, Vector,
};

use crate::{
    theme::{ControlSize, TextRole, ToneRole, TypographyRole},
    widgets::{text as ntext, ToneDot},
    Element,
};

use super::{
    super::measured_text::{EllipsisStrategy, MeasuredText},
    style as theme_metadata,
};

const DEFAULT_LABEL_WIDTH: f32 = 96.0;

enum MetadataValue<'a, Message> {
    Text(Cow<'a, str>),
    Code(Cow<'a, str>),
    Custom(Element<'a, Message>),
}

/// Static, surface-neutral definition-list composition.
///
/// The host owns fill, perimeter, radius, shadow, and outer padding. Label
/// width belongs to the complete list, not individual items. Text and Code
/// values use complete 14 px hierarchy and wrap; Custom values retain caller
/// ownership of interaction and accessibility. Status is orthogonal and must
/// already have complete visible wording in the value.
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = KeyValueList::<()>::new().role(SurfaceRole::Panel);
/// ```
pub struct KeyValueList<'a, Message> {
    items: Vec<MetadataItem<'a, Message>>,
    size: ControlSize,
    width: Option<Length>,
    label_width: f32,
}

/// One labelled Text, Code, or caller-styled Custom metadata value.
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = MetadataItem::<()>::new("Name", "Nive").label_width(80.0);
/// ```
pub struct MetadataItem<'a, Message> {
    label: Cow<'a, str>,
    value: MetadataValue<'a, Message>,
    status: Option<ToneRole>,
}

impl<'a, Message> KeyValueList<'a, Message>
where
    Message: 'a,
{
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            size: ControlSize::Sm,
            width: None,
            label_width: DEFAULT_LABEL_WIDTH,
        }
    }

    pub fn push(mut self, item: MetadataItem<'a, Message>) -> Self {
        self.items.push(item);
        self
    }

    pub fn item(self, item: MetadataItem<'a, Message>) -> Self {
        self.push(item)
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    pub fn lg(self) -> Self {
        self.size(ControlSize::Lg)
    }

    /// Requests one logical-pixel label column for every item.
    ///
    /// Negative and non-finite values sanitize to zero; finite hosts clamp the
    /// effective column to 40% of the row track.
    pub fn label_width(mut self, width: f32) -> Self {
        self.label_width = sanitize_label_width(width);
        self
    }

    crate::impl_layout_builders!(width_opt, fill_width_opt, shrink_width_opt);

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_metadata::metrics(self.size);
        let reserve_status = self.items.iter().any(MetadataItem::renders_status);
        let mut content = Column::new().spacing(metrics.list_gap);

        for item in self.items {
            content = content.push(item.into_row(
                self.label_width,
                metrics.slot_gap,
                metrics.minimum_height,
                self.size,
                reserve_status,
            ));
        }

        if let Some(width) = self.width {
            content = content.width(width);
        }

        content.into()
    }
}

impl<'a, Message> Default for KeyValueList<'a, Message>
where
    Message: 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> From<KeyValueList<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(list: KeyValueList<'a, Message>) -> Self {
        list.into_element()
    }
}

impl<'a, Message> MetadataItem<'a, Message>
where
    Message: 'a,
{
    pub fn new(label: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            value: MetadataValue::Text(value.into()),
            status: None,
        }
    }

    /// Replaces the value with framework-styled 14 px Body text.
    pub fn text_value(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.value = MetadataValue::Text(value.into());
        self
    }

    /// Replaces the value with framework-styled semantic code text.
    pub fn code_value(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.value = MetadataValue::Code(value.into());
        self
    }

    /// Uses arbitrary caller-owned content in the value slot.
    ///
    /// The caller owns its typography, wrapping, interaction, and complete
    /// visible status/accessibility meaning.
    pub fn custom_value(mut self, value: impl Into<Element<'a, Message>>) -> Self {
        self.value = MetadataValue::Custom(value.into());
        self
    }

    /// Adds a value-side dot as the caller's assertion that the value contains
    /// complete visible neutral status meaning.
    pub fn status(mut self, tone: ToneRole) -> Self {
        self.status = Some(tone);
        self
    }

    fn renders_status(&self) -> bool {
        self.status.is_some()
            && match &self.value {
                MetadataValue::Text(value) | MetadataValue::Code(value) => !value.trim().is_empty(),
                MetadataValue::Custom(_) => true,
            }
    }

    fn into_row(
        self,
        label_width: f32,
        gap: f32,
        minimum_height: f32,
        size: ControlSize,
        reserve_status: bool,
    ) -> KeyValueRow<'a, Message> {
        let render_status = self.renders_status();
        let label = MeasuredText::new(
            self.label,
            EllipsisStrategy::End,
            TypographyRole::Body,
            TextRole::Secondary,
        )
        .into();
        let status: Element<'a, Message> = if reserve_status {
            self.status.filter(|_| render_status).map_or_else(
                || {
                    let diameter = crate::widgets::primitives::tone_dot::dot_size(size);
                    Space::new()
                        .width(Length::Fixed(diameter))
                        .height(Length::Fixed(diameter))
                        .into()
                },
                |tone| ToneDot::new(tone).size(size).into(),
            )
        } else {
            Space::new().into()
        };
        let value: Element<'a, Message> = match self.value {
            MetadataValue::Text(value) => {
                ntext::with_role(value, TypographyRole::Body, TextRole::Primary)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .width(Length::Fill)
                    .into()
            }
            MetadataValue::Code(value) => {
                ntext::with_role(value, TypographyRole::Code, TextRole::Primary)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .width(Length::Fill)
                    .into()
            }
            MetadataValue::Custom(value) => value,
        };

        KeyValueRow {
            children: [label, status, value],
            requested_label_width: label_width,
            gap,
            minimum_height,
            reserve_status,
            status_width: crate::widgets::primitives::tone_dot::dot_size(size),
        }
    }
}

fn sanitize_label_width(width: f32) -> f32 {
    if width.is_finite() && width >= 0.0 {
        width
    } else {
        0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Tracks {
    label: f32,
    gap: f32,
    value: f32,
}

fn finite_tracks(request: f32, semantic_gap: f32, width: f32) -> Tracks {
    let width = width.max(0.0);
    let label = sanitize_label_width(request).min(width * 0.4);
    let gap = semantic_gap.max(0.0).min((width - label).max(0.0));
    Tracks {
        label,
        gap,
        value: (width - label - gap).max(0.0),
    }
}

struct KeyValueRow<'a, Message> {
    children: [Element<'a, Message>; 3],
    requested_label_width: f32,
    gap: f32,
    minimum_height: f32,
    reserve_status: bool,
    status_width: f32,
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer> for KeyValueRow<'a, Message>
where
    Message: 'a,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &self
                .children
                .iter()
                .map(Element::as_widget)
                .collect::<Vec<_>>(),
        );
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let maximum_width = limits.max().width;
        let width = if maximum_width.is_finite() {
            maximum_width.max(0.0)
        } else {
            self.requested_label_width + self.gap + 240.0
        };
        let tracks = if maximum_width.is_finite() {
            finite_tracks(self.requested_label_width, self.gap, width)
        } else {
            Tracks {
                label: self.requested_label_width,
                gap: self.gap,
                value: (width - self.requested_label_width - self.gap).max(0.0),
            }
        };
        let label_width = tracks.label;
        let gap = tracks.gap;
        let value_track = tracks.value;
        let status_width = if self.reserve_status {
            self.status_width.min(value_track)
        } else {
            0.0
        };
        let status_gap = if self.reserve_status {
            self.gap.min((value_track - status_width).max(0.0))
        } else {
            0.0
        };
        let value_width = (value_track - status_width - status_gap).max(0.0);

        let label = self.children[0].as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(label_width, f32::INFINITY))
                .width(Length::Fixed(label_width)),
        );
        let status = self.children[1].as_widget_mut().layout(
            &mut tree.children[1],
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(status_width, f32::INFINITY)),
        );
        let value = self.children[2].as_widget_mut().layout(
            &mut tree.children[2],
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(value_width, f32::INFINITY))
                .width(Length::Fill),
        );
        let height = self
            .minimum_height
            .max(label.size().height)
            .max(status.size().height)
            .max(value.size().height);
        let center = |node: layout::Node, x: f32| {
            let y = (height - node.size().height) / 2.0;
            node.move_to(Point::new(x, y))
        };

        layout::Node::with_children(
            Size::new(width, height),
            vec![
                center(label, 0.0),
                center(status, label_width + gap),
                center(value, label_width + gap + status_width + status_gap),
            ],
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child
                .as_widget_mut()
                .operate(tree, layout, renderer, operation);
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
            if shell.is_event_captured() {
                break;
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let clip = layout.bounds().intersection(viewport).unwrap_or_default();
        renderer.with_layer(clip, |renderer| {
            for ((child, tree), layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
            {
                child
                    .as_widget()
                    .draw(tree, renderer, theme, style, layout, cursor, &clip);
            }
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        let children = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((child, tree), layout)| {
                child
                    .as_widget_mut()
                    .overlay(tree, layout, renderer, viewport, translation)
            })
            .collect::<Vec<_>>();
        (!children.is_empty()).then(|| overlay::Group::with_children(children).overlay())
    }
}

impl<'a, Message> From<KeyValueRow<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(row: KeyValueRow<'a, Message>) -> Self {
        Element::new(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::{mouse, Event, Size};

    #[test]
    fn invalid_label_widths_sanitize_to_zero() {
        for width in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(sanitize_label_width(width), 0.0);
        }
        assert_eq!(sanitize_label_width(96.0), 96.0);
    }

    #[test]
    fn finite_tracks_follow_exact_shared_column_and_impossible_width_formula() {
        assert_eq!(
            finite_tracks(96.0, 8.0, 300.0),
            Tracks {
                label: 96.0,
                gap: 8.0,
                value: 196.0,
            }
        );
        assert_eq!(
            finite_tracks(96.0, 8.0, 100.0),
            Tracks {
                label: 40.0,
                gap: 8.0,
                value: 52.0,
            }
        );
        assert_eq!(
            finite_tracks(96.0, 8.0, 5.0),
            Tracks {
                label: 2.0,
                gap: 3.0,
                value: 0.0,
            }
        );
    }

    #[test]
    fn typed_empty_status_is_omitted_but_custom_is_caller_asserted() {
        assert!(!MetadataItem::<()>::new("Status", " ")
            .status(ToneRole::Info)
            .renders_status());
        assert!(MetadataItem::<()>::new("Status", "Ready")
            .status(ToneRole::Info)
            .renders_status());
    }

    #[test]
    fn real_layout_uses_one_shared_clamped_track_and_stable_status_slots() {
        let node = crate::test_support::layout(
            KeyValueList::<()>::new()
                .item(MetadataItem::new("Short", "Ready").status(ToneRole::Success))
                .item(MetadataItem::new(
                    "A substantially longer label",
                    "A wrapped textual value that needs more than one line in this narrow host",
                ))
                .fill_width()
                .into(),
            Size::new(160.0, 400.0),
        );
        let rows = node.children();
        assert_eq!(node.size().width, 160.0);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].children()[0].size().width, 64.0);
        assert_eq!(rows[1].children()[0].size().width, 64.0);
        assert_eq!(
            rows[0].children()[1].bounds().x,
            rows[1].children()[1].bounds().x
        );
        assert!(rows[1].size().height > theme_metadata::metrics(ControlSize::Sm).minimum_height);
        for row in rows {
            assert!(row.bounds().x + row.bounds().width <= node.size().width);
        }
    }

    #[test]
    fn real_layout_survives_sub_gap_width_without_non_finite_geometry() {
        let node = crate::test_support::layout(
            KeyValueList::<()>::new()
                .item(MetadataItem::new("Label", "Value").status(ToneRole::Info))
                .fill_width()
                .into(),
            Size::new(5.0, 100.0),
        );
        assert_eq!(node.size().width, 5.0);
        assert!(node.children()[0]
            .children()
            .iter()
            .all(|child| child.bounds().width.is_finite()));
    }

    #[test]
    fn custom_value_event_is_delegated_to_the_owned_child() {
        let messages = crate::test_support::event_messages(
            KeyValueList::new()
                .item(
                    MetadataItem::new("Custom", "unused")
                        .custom_value(crate::test_support::event_probe(7_u8)),
                )
                .fill_width()
                .into(),
            Size::new(240.0, 80.0),
            Event::Mouse(mouse::Event::CursorEntered),
        );
        assert_eq!(messages, vec![7]);
    }
}
