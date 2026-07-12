//! Narrow vertical edge rail widget.
//!
//! `VerticalRail` is a controlled navigation primitive for professional
//! desktop shells. Applications own selection state; the rail renders each
//! item's `selected` flag independently and emits `on_press` for enabled items.

use std::borrow::Cow;

use iced::{
    advanced::{
        layout::{self, Layout, Node},
        mouse, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Shell, Widget,
    },
    widget::{canvas, container, Column},
    Alignment, Background, Border, Event, Length, Point, Radians, Rectangle, Shadow, Size, Vector,
};

use crate::theme::{self, ControlSize, SurfaceRole, TextRole, ToneRole, TypographyRole};
use crate::widgets::controls::button::{self, GroupedItemKind, GroupedItemSpec};
use crate::widgets::display::Badge;
use crate::widgets::primitives::{icon as icon_widget, IconRole, ToneDot};
use crate::Element;

const CHEVRON_SCROLL_STEP_FACTOR: f32 = 0.8;
const HIDDEN_AFFORDANCE_HEIGHT: f32 = 0.1;
const MAX_LABEL_TRACK_FACTOR: f32 = 4.75;
const MIN_LABEL_TRACK_FACTOR: f32 = 1.7;
const AVG_LABEL_ADVANCE_FACTOR: f32 = 0.56;
const ELLIPSIS: &str = "…";

/// Window edge where a [`VerticalRail`] is rendered.
///
/// `Left` renders labels counter-clockwise so text reads bottom-to-top. `Right`
/// renders labels clockwise so text reads top-to-bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailSide {
    /// Left window edge; labels read bottom-to-top.
    Left,
    /// Right window edge; labels read top-to-bottom.
    Right,
}

/// A narrow vertical rail for left and right window edges.
///
/// The rail lays out items in an axis-aligned vertical strip and scrolls
/// overflow using the same clipped-strip/chevron model as `TabBar`. It owns
/// only ephemeral scroll state; selection is independent per item and remains
/// application-owned through [`VerticalRailItem::selected`].
///
/// Each item is constructed with a mandatory text label. The label is both the
/// visible rotated label and the accessible label. Long labels are truncated
/// with an ellipsis; if no explicit tooltip is supplied, a truncated item falls
/// back to the full label text as its tooltip.
pub struct VerticalRail<'a, Message> {
    side: RailSide,
    items: Vec<VerticalRailItem<'a, Message>>,
    size: ControlSize,
    height: Option<Length>,
}

/// Data for one vertical rail entry.
///
/// `VerticalRailItem` requires a text label so icon-forward entries still carry
/// an accessible label. Optional icon, badge, and status tone render upright at
/// the icon end of the item; only the label is rotated. Activation emits
/// `on_press` for enabled items and never changes selection internally.
#[derive(Debug, Clone)]
pub struct VerticalRailItem<'a, Message> {
    label: Cow<'a, str>,
    icon: Option<IconRole>,
    selected: bool,
    disabled: bool,
    badge: Option<Cow<'a, str>>,
    status: Option<ToneRole>,
    tooltip: Option<Cow<'a, str>>,
    on_press: Option<Message>,
}

#[derive(Debug, Clone, Default)]
struct VerticalRailState {
    scroll_offset: f32,
    max_scroll: f32,
    content_height: f32,
    strip_height: f32,
    has_overflow: bool,
    item_bounds: Vec<Rectangle>,
    up_chevron: Option<Rectangle>,
    down_chevron: Option<Rectangle>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RailMetrics {
    size: ControlSize,
    width: f32,
    radius: f32,
    padding: f32,
    gap: f32,
    icon_size: f32,
    font_size: f32,
    line_height: f32,
    min_label_track: f32,
    max_label_track: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct RailLabelCanvas {
    text: String,
    side: RailSide,
    font_size: f32,
    line_height: f32,
}

impl<'a, Message> VerticalRail<'a, Message>
where
    Message: Clone + 'a,
{
    /// Builds an empty vertical rail for the given window edge.
    pub fn new(side: RailSide) -> Self {
        Self {
            side,
            items: Vec::new(),
            size: ControlSize::Sm,
            height: None,
        }
    }

    /// Sets the control size used to derive rail width and item metrics.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Uses the extra-small rail size.
    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    /// Uses the small rail size.
    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    /// Uses the medium rail size.
    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    /// Adds one rail item.
    pub fn push(mut self, item: VerticalRailItem<'a, Message>) -> Self {
        self.items.push(item);
        self
    }

    /// Adds one rail item.
    pub fn item(self, item: VerticalRailItem<'a, Message>) -> Self {
        self.push(item)
    }

    crate::impl_layout_builders!(height_opt, fill_height_opt);

    fn content_element(&self, state: &VerticalRailState) -> Element<'_, Message> {
        let metrics = metrics(self.size);
        let up_visible = state.has_overflow && state.scroll_offset > 0.0;
        let down_visible = state.has_overflow && state.scroll_offset < state.max_scroll;

        let mut items = Column::new()
            .spacing(metrics.gap)
            .align_x(Alignment::Center)
            .width(Length::Fixed(metrics.width))
            .height(Length::Shrink);

        for item in &self.items {
            items = items.push(self.item_element(item, metrics));
        }

        let strip = container(items)
            .width(Length::Fixed(metrics.width))
            .height(Length::Fill)
            .clip(true);

        let rail = Column::new()
            .spacing(0.0)
            .align_x(Alignment::Center)
            .width(Length::Fixed(metrics.width))
            .height(self.height.unwrap_or(Length::Fill))
            .push(self.chevron_button(metrics, IconRole::NiveDisclosureUp, up_visible))
            .push(strip)
            .push(self.chevron_button(metrics, IconRole::NiveDisclosureDown, down_visible));

        container(rail)
            .style(rail_container_style(SurfaceRole::Chrome))
            .width(Length::Fixed(metrics.width))
            .height(self.height.unwrap_or(Length::Fill))
            .into()
    }

    fn chevron_button(
        &self,
        metrics: RailMetrics,
        role: IconRole,
        visible: bool,
    ) -> Element<'_, Message> {
        button::icon(role)
            .width(Length::Fixed(metrics.width))
            .into_grouped_item(GroupedItemSpec {
                size: metrics.size,
                radius: metrics.radius.into(),
                height: if visible {
                    metrics.width
                } else {
                    HIDDEN_AFFORDANCE_HEIGHT
                },
                padding_h: 0.0,
                selected: false,
                destructive: false,
                kind: GroupedItemKind::Embedded,
            })
    }

    fn item_element<'b>(
        &'b self,
        item: &'b VerticalRailItem<'a, Message>,
        metrics: RailMetrics,
    ) -> Element<'b, Message> {
        let layout = item_layout(item, metrics);
        let (visible_label, truncated) = ellipsize_label(&item.label, layout.label_track, metrics);
        let tooltip = item
            .tooltip
            .clone()
            .or_else(|| truncated.then(|| Cow::Owned(item.label.to_string())));

        let mut content = Column::new()
            .spacing(metrics.gap)
            .align_x(Alignment::Center)
            .width(Length::Fixed(metrics.width))
            .height(Length::Fixed(layout.height));

        if let Some(icon) = item.icon {
            content = content.push(
                icon_widget::role(icon)
                    .size(metrics.icon_size)
                    .color_maybe(None),
            );
        }

        if let Some(status) = item.status {
            content = content.push(ToneDot::new(status).size(metrics.size));
        }

        if let Some(badge) = &item.badge {
            content = content.push(
                Badge::new(badge.clone())
                    .tone(item.status.unwrap_or(ToneRole::Neutral))
                    .xs(),
            );
        }

        content = content.push(
            canvas::Canvas::new(RailLabelCanvas {
                text: visible_label,
                side: self.side,
                font_size: metrics.font_size,
                line_height: metrics.line_height,
            })
            .width(Length::Fixed(metrics.width))
            .height(Length::Fixed(layout.label_track)),
        );

        let button = button::Button::custom(content.into())
            .disabled(item.disabled)
            .tooltip_maybe(tooltip)
            .on_press_maybe(item.activation());

        button.into_grouped_item(GroupedItemSpec {
            size: metrics.size,
            radius: metrics.radius.into(),
            height: layout.height,
            padding_h: 0.0,
            selected: item.selected,
            destructive: false,
            kind: GroupedItemKind::Selectable,
        })
    }
}

impl<'a, Message> From<VerticalRail<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(rail: VerticalRail<'a, Message>) -> Self {
        Element::new(rail)
    }
}

impl<'a, Message> VerticalRailItem<'a, Message>
where
    Message: Clone + 'a,
{
    /// Builds an item with its mandatory visible and accessible label.
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            selected: false,
            disabled: false,
            badge: None,
            status: None,
            tooltip: None,
            on_press: None,
        }
    }

    /// Sets the upright icon rendered at the icon end of the item.
    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the selected visual state. Selection remains app-owned.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Disables pointer and keyboard activation.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the upright badge text.
    pub fn badge(mut self, badge: impl Into<Cow<'a, str>>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// Sets the upright status tone.
    pub fn status(mut self, status: ToneRole) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the tooltip shown for this item.
    ///
    /// Explicit tooltips override the full-label fallback used for truncated
    /// labels.
    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Sets the activation message emitted for enabled items.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Conditionally sets the activation message emitted for enabled items.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    fn activation(&self) -> Option<Message> {
        (!self.disabled).then(|| self.on_press.clone()).flatten()
    }
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer> for VerticalRail<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<VerticalRailState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(VerticalRailState::default())
    }

    fn children(&self) -> Vec<Tree> {
        let state = VerticalRailState::default();
        vec![Tree::new(self.content_element(&state))]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_ref::<VerticalRailState>();
        let content = self.content_element(state);

        if tree.children.is_empty() {
            tree.children.push(Tree::new(&content));
        } else {
            tree.children[0].diff(content.as_widget());
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fixed(metrics(self.size).width),
            self.height.unwrap_or(Length::Fill),
        )
    }

    fn size_hint(&self) -> Size<Length> {
        self.size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> Node {
        let state = tree.state.downcast_ref::<VerticalRailState>();
        let mut content = self.content_element(state);
        let node = content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        let (content_height, strip_height, translated_node, item_bounds) =
            measure_and_translate(node, state.scroll_offset);

        let state = tree.state.downcast_mut::<VerticalRailState>();
        state.content_height = content_height;
        state.strip_height = strip_height;
        state.max_scroll = (content_height - strip_height).max(0.0);
        state.has_overflow = content_height > strip_height + 0.5;
        state.scroll_offset = state.scroll_offset.clamp(0.0, state.max_scroll);
        state.item_bounds = item_bounds;

        translated_node
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_ref::<VerticalRailState>();
        let mut content = self.content_element(state);
        content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        {
            let state = tree.state.downcast_ref::<VerticalRailState>();
            let mut content = self.content_element(state);
            content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        let hit_geometry = hit_geometry(layout);
        let state = tree.state.downcast_mut::<VerticalRailState>();
        state.up_chevron = hit_geometry.up_chevron;
        state.down_chevron = hit_geometry.down_chevron;

        if !cursor.is_over(layout.bounds()) {
            return;
        }

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if state.has_overflow => {
                let delta_y = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y * 24.0,
                    mouse::ScrollDelta::Pixels { y, .. } => *y,
                };
                state.scroll_offset = (state.scroll_offset - delta_y).clamp(0.0, state.max_scroll);
                if delta_y != 0.0 {
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if state.has_overflow =>
            {
                if state
                    .up_chevron
                    .is_some_and(|bounds| cursor.is_over(bounds))
                {
                    let step = CHEVRON_SCROLL_STEP_FACTOR * state.strip_height;
                    state.scroll_offset = (state.scroll_offset - step).max(0.0);
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                } else if state
                    .down_chevron
                    .is_some_and(|bounds| cursor.is_over(bounds))
                {
                    let step = CHEVRON_SCROLL_STEP_FACTOR * state.strip_height;
                    state.scroll_offset = (state.scroll_offset + step).min(state.max_scroll);
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            _ => {}
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
        let state = tree.state.downcast_ref::<VerticalRailState>();
        let content = self.content_element(state);
        let interaction = content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        );

        if interaction != mouse::Interaction::None {
            return interaction;
        }

        if state.has_overflow
            && (state
                .up_chevron
                .is_some_and(|bounds| cursor.is_over(bounds))
                || state
                    .down_chevron
                    .is_some_and(|bounds| cursor.is_over(bounds)))
        {
            return mouse::Interaction::Pointer;
        }

        mouse::Interaction::None
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<VerticalRailState>();
        let content = self.content_element(state);
        content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }
}

impl<Message> canvas::Program<Message, crate::theme::Theme, iced::Renderer> for RailLabelCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        theme: &crate::theme::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let color = theme.text(TextRole::Secondary).color;
        let angle = rotation_radians(self.side);
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);

        frame.with_save(|frame| {
            frame.translate(Vector::new(center.x, center.y));
            frame.rotate(Radians(angle));
            frame.fill_text(canvas::Text {
                content: self.text.clone(),
                position: Point::ORIGIN,
                max_width: bounds.height,
                color,
                size: iced::Pixels(self.font_size),
                line_height: iced::widget::text::LineHeight::Absolute(iced::Pixels(
                    self.line_height,
                )),
                font: theme.typography(TypographyRole::Label).font,
                align_x: iced::advanced::text::Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                shaping: iced::advanced::text::Shaping::Auto,
            });
        });

        vec![frame.into_geometry()]
    }
}

fn metrics(size: ControlSize) -> RailMetrics {
    let control = theme::control_metrics(size);
    let label = theme::typography(TypographyRole::Label);

    RailMetrics {
        size,
        width: control.height,
        radius: control.radius,
        padding: control.gap,
        gap: (control.gap * 0.75).max(2.0),
        icon_size: control.icon_size,
        font_size: label.size,
        line_height: label.line_height,
        min_label_track: control.height * MIN_LABEL_TRACK_FACTOR,
        max_label_track: control.height * MAX_LABEL_TRACK_FACTOR,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ItemLayout {
    height: f32,
    label_track: f32,
}

fn item_layout<Message>(item: &VerticalRailItem<'_, Message>, metrics: RailMetrics) -> ItemLayout {
    let label_advance = estimated_text_advance(&item.label, metrics);
    let label_track = label_advance
        .clamp(metrics.min_label_track, metrics.max_label_track)
        .ceil();
    let mut height = metrics.padding * 2.0 + label_track;

    if item.icon.is_some() {
        height += metrics.icon_size + metrics.gap;
    }
    if item.status.is_some() {
        height += tone_dot_size(metrics.size) + metrics.gap;
    }
    if item.badge.is_some() {
        height += badge_height(metrics.size) + metrics.gap;
    }

    ItemLayout {
        height: height.max(metrics.width).ceil(),
        label_track,
    }
}

fn estimated_text_advance(label: &str, metrics: RailMetrics) -> f32 {
    label.chars().count() as f32 * metrics.font_size * AVG_LABEL_ADVANCE_FACTOR
}

fn ellipsize_label(label: &str, max_advance: f32, metrics: RailMetrics) -> (String, bool) {
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

fn rotation_radians(side: RailSide) -> f32 {
    match side {
        RailSide::Left => -std::f32::consts::FRAC_PI_2,
        RailSide::Right => std::f32::consts::FRAC_PI_2,
    }
}

fn rail_container_style(role: SurfaceRole) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let surface = theme.surface(role);

        container::Style {
            text_color: Some(surface.foreground),
            background: Some(Background::Color(surface.background)),
            border: Border {
                color: surface.border.color,
                width: surface.border.width,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

fn tone_dot_size(size: ControlSize) -> f32 {
    match size {
        ControlSize::Xs => 4.0,
        ControlSize::Sm => 6.0,
        ControlSize::Md => 8.0,
        ControlSize::Lg => 10.0,
    }
}

fn badge_height(size: ControlSize) -> f32 {
    match size {
        ControlSize::Xs => 14.0,
        ControlSize::Sm => 16.0,
        ControlSize::Md | ControlSize::Lg => 18.0,
    }
}

fn measure_and_translate(node: Node, scroll_offset: f32) -> (f32, f32, Node, Vec<Rectangle>) {
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

    let rail_children: Vec<Node> = rail_column
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
struct HitGeometry {
    up_chevron: Option<Rectangle>,
    down_chevron: Option<Rectangle>,
}

fn hit_geometry(layout: Layout<'_>) -> HitGeometry {
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

#[cfg(test)]
mod vertical_rail_tests {
    use super::*;

    #[allow(dead_code)]
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Message {
        Pressed(&'static str),
    }

    fn test_metrics() -> RailMetrics {
        RailMetrics {
            size: ControlSize::Sm,
            width: 28.0,
            radius: 6.0,
            padding: 4.0,
            gap: 3.0,
            icon_size: 14.0,
            font_size: 12.0,
            line_height: 14.0,
            min_label_track: 40.0,
            max_label_track: 70.0,
        }
    }

    #[test]
    fn item_metadata_builders_store_contract_fields() {
        let item = VerticalRailItem::new("Explorer")
            .icon(IconRole::Folder)
            .selected(true)
            .disabled(true)
            .badge("3")
            .status(ToneRole::Warning)
            .tooltip("Open explorer")
            .on_press(Message::Pressed("explorer"));

        assert_eq!(item.label, "Explorer");
        assert_eq!(item.icon, Some(IconRole::Folder));
        assert!(item.selected);
        assert!(item.disabled);
        assert_eq!(item.badge.as_deref(), Some("3"));
        assert_eq!(item.status, Some(ToneRole::Warning));
        assert_eq!(item.tooltip.as_deref(), Some("Open explorer"));
        assert_eq!(item.activation(), None);
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
            VerticalRailItem::<Message>::new("A").selected(true),
            VerticalRailItem::<Message>::new("B").selected(true),
            VerticalRailItem::<Message>::new("C"),
        ];

        assert_eq!(items.iter().filter(|item| item.selected).count(), 2);
    }

    #[test]
    fn activation_ignores_disabled_items() {
        let enabled = VerticalRailItem::new("Enabled").on_press(Message::Pressed("enabled"));
        let disabled = VerticalRailItem::new("Disabled")
            .disabled(true)
            .on_press(Message::Pressed("disabled"));

        assert_eq!(enabled.activation(), Some(Message::Pressed("enabled")));
        assert_eq!(disabled.activation(), None);
    }

    #[test]
    fn truncation_adds_ellipsis_and_tooltip_fallback_can_use_full_label() {
        let metrics = test_metrics();
        let label = "Very long vertical rail label";
        let (visible, truncated) = ellipsize_label(label, 42.0, metrics);

        assert!(truncated);
        assert!(visible.ends_with(ELLIPSIS));

        let item = VerticalRailItem::<Message>::new(label);
        let tooltip = item
            .tooltip
            .clone()
            .or_else(|| truncated.then(|| Cow::Owned(item.label.to_string())));

        assert_eq!(tooltip.as_deref(), Some(label));
    }

    #[test]
    fn explicit_tooltip_overrides_truncation_fallback() {
        let item =
            VerticalRailItem::<Message>::new("Very long vertical rail label").tooltip("Custom");
        let tooltip = item
            .tooltip
            .clone()
            .or_else(|| Some(Cow::Owned(item.label.to_string())));

        assert_eq!(tooltip.as_deref(), Some("Custom"));
    }

    #[test]
    fn overflow_state_clamps_offsets_and_chevrons_transition() {
        let mut state = VerticalRailState {
            has_overflow: true,
            strip_height: 100.0,
            max_scroll: 80.0,
            scroll_offset: 5.0,
            ..VerticalRailState::default()
        };

        state.scroll_offset =
            (state.scroll_offset - CHEVRON_SCROLL_STEP_FACTOR * state.strip_height).max(0.0);
        assert_eq!(state.scroll_offset, 0.0);

        let show_up = state.has_overflow && state.scroll_offset > 0.0;
        let show_down = state.has_overflow && state.scroll_offset < state.max_scroll;
        assert!(!show_up);
        assert!(show_down);

        state.scroll_offset = (state.scroll_offset
            + CHEVRON_SCROLL_STEP_FACTOR * state.strip_height)
            .min(state.max_scroll);
        assert_eq!(state.scroll_offset, 80.0);

        let show_up = state.has_overflow && state.scroll_offset > 0.0;
        let show_down = state.has_overflow && state.scroll_offset < state.max_scroll;
        assert!(show_up);
        assert!(!show_down);
    }
}
