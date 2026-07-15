use iced::{
    advanced::{
        layout::{self, Layout, Node},
        mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Renderer as _, Shell, Widget,
    },
    keyboard,
    widget::{container, text, Row},
    Alignment, Background, Border, Color, Event, Length, Padding, Rectangle, Shadow, Size, Vector,
};
use nive_ui::{
    theme::{BorderRole, ControlRole, ControlSize, ControlState, TextRole},
    widgets::{Badge, ToneDot},
    Element,
};

use super::BottomHeaderTab;

const MAX_TAB_WIDTH: f32 = 220.0;

pub(super) struct BottomPanelTabTrack<'a, Message> {
    items: Vec<TrackItem<'a, Message>>,
    size: ControlSize,
    active_index: usize,
    content: Element<'a, Message>,
}

struct TrackItem<'a, Message> {
    metadata: BottomHeaderTab<'a, ()>,
    message: Option<Message>,
}

#[derive(Debug, Default)]
struct TrackState {
    focused: bool,
    focused_index: Option<usize>,
    hovered_index: Option<usize>,
    item_bounds: Vec<Rectangle>,
    offset: f32,
    max_offset: f32,
    viewport_width: f32,
    last_active_index: Option<usize>,
}

impl<'a, Message> BottomPanelTabTrack<'a, Message>
where
    Message: Clone + 'a,
{
    pub(super) fn new(size: ControlSize, active_index: usize) -> Self {
        Self {
            items: Vec::new(),
            size,
            active_index,
            content: iced::widget::Space::new().into(),
        }
    }

    pub(super) fn push<PanelId>(
        mut self,
        tab: BottomHeaderTab<'a, PanelId>,
        message: Option<Message>,
    ) -> Self {
        self.items.push(TrackItem {
            metadata: BottomHeaderTab {
                panel_id: (),
                label: tab.label,
                icon: tab.icon,
                badge: tab.badge,
                status: tab.status,
                disabled: tab.disabled,
                tooltip: tab.tooltip,
            },
            message,
        });
        self.content = build_content(&self.items, self.size, self.active_index);
        self
    }

    fn metrics(&self) -> TrackMetrics {
        let control = nive_ui::theme::control_metrics(self.size);
        TrackMetrics {
            height: control.height,
            font_size: control.font_size.max(14.0),
            icon_size: control.icon_size,
            gap: control.gap,
            padding_h: nive_ui::theme::spacing().md,
            radius: control.radius,
        }
    }

    fn enabled_indices(&self) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (!item.metadata.disabled).then_some(index))
            .collect()
    }

    fn reconcile_focus(&self, state: &mut TrackState) {
        let enabled = self.enabled_indices();
        if !state
            .focused_index
            .is_some_and(|index| enabled.contains(&index))
        {
            state.focused_index = enabled
                .contains(&self.active_index)
                .then_some(self.active_index)
                .or_else(|| enabled.first().copied());
        }
    }
}

fn build_content<'a, Message>(
    items_data: &[TrackItem<'a, Message>],
    size: ControlSize,
    active_index: usize,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let control = nive_ui::theme::control_metrics(size);
    let metrics = TrackMetrics {
        height: control.height,
        font_size: control.font_size.max(14.0),
        icon_size: control.icon_size,
        gap: control.gap,
        padding_h: nive_ui::theme::spacing().md,
        radius: control.radius,
    };
    let mut items = Row::new()
        .spacing(0.0)
        .align_y(Alignment::Center)
        .height(Length::Fixed(metrics.height))
        .width(Length::Shrink);

    for (index, item) in items_data.iter().enumerate() {
        let active = index == active_index;
        let mut content = Row::new()
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Shrink);
        if let Some(icon) = item.metadata.icon {
            content =
                content.push(nive_ui::widgets::Icon::role(icon).custom_size(metrics.icon_size));
        }
        content = content.push(
            text(item.metadata.label.clone())
                .size(metrics.font_size)
                .wrapping(text::Wrapping::None),
        );
        if let Some(badge) = &item.metadata.badge {
            content = content.push(Badge::new(badge.clone()).xs());
        }
        if let Some(status) = item.metadata.status {
            content = content.push(ToneDot::new(status).xs());
        }

        let tab: Element<'_, Message> = container(content)
            .style(tab_content_style(active, item.metadata.disabled))
            .padding(Padding::ZERO.horizontal(metrics.padding_h))
            .height(Length::Fixed(metrics.height))
            .max_width(MAX_TAB_WIDTH)
            .clip(true)
            .into();
        let tooltip = item
            .metadata
            .tooltip
            .clone()
            .unwrap_or_else(|| item.metadata.label.clone());
        items = items.push(nive_ui::widgets::tooltip::bottom(tab, tooltip));
    }

    container(items)
        .width(Length::Fill)
        .height(Length::Fixed(metrics.height))
        .clip(true)
        .into()
}

#[derive(Debug, Clone, Copy)]
struct TrackMetrics {
    height: f32,
    font_size: f32,
    icon_size: f32,
    gap: f32,
    padding_h: f32,
    radius: f32,
}

impl operation::Focusable for TrackState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl<'a, Message> Widget<Message, nive_ui::Theme, nive_ui::Renderer>
    for BottomPanelTabTrack<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TrackState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TrackState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.metrics().height))
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &nive_ui::Renderer,
        limits: &layout::Limits,
    ) -> Node {
        let state = tree.state.downcast_ref::<TrackState>();
        let node = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let (content_width, viewport_width, item_bounds, translated) =
            translate_track(node, state.offset);
        let state = tree.state.downcast_mut::<TrackState>();
        state.viewport_width = viewport_width;
        state.max_offset = (content_width - viewport_width).max(0.0);
        state.offset = state.offset.clamp(0.0, state.max_offset);
        state.item_bounds = item_bounds;
        self.reconcile_focus(state);

        if state.last_active_index != Some(self.active_index) {
            state.last_active_index = Some(self.active_index);
            reveal_index(state, self.active_index);
        }
        translated
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &nive_ui::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        operation.focusable(
            None,
            layout.bounds(),
            tree.state.downcast_mut::<TrackState>(),
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &nive_ui::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if shell.is_event_captured() {
            return;
        }

        let state = tree.state.downcast_mut::<TrackState>();
        state.hovered_index = state
            .item_bounds
            .iter()
            .position(|bounds| cursor.is_over(*bounds));

        if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
            if state.max_offset > 0.0 && cursor.is_over(layout.bounds()) {
                let delta = horizontal_wheel(*delta);
                state.offset = (state.offset - delta).clamp(0.0, state.max_offset);
                if delta != 0.0 {
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            return;
        }

        if state.focused {
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named),
                repeat: false,
                ..
            }) = event
            {
                let enabled = self.enabled_indices();
                let current = state
                    .focused_index
                    .and_then(|focused| enabled.iter().position(|index| *index == focused))
                    .unwrap_or(0);
                let target = match named {
                    keyboard::key::Named::ArrowLeft => Some(current.saturating_sub(1)),
                    keyboard::key::Named::ArrowRight => {
                        Some((current + 1).min(enabled.len().saturating_sub(1)))
                    }
                    keyboard::key::Named::Home => Some(0),
                    keyboard::key::Named::End => Some(enabled.len().saturating_sub(1)),
                    _ => None,
                };
                if let Some(target) = target.and_then(|target| enabled.get(target).copied()) {
                    state.focused_index = Some(target);
                    reveal_index(state, target);
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                    return;
                }
                if matches!(
                    named,
                    keyboard::key::Named::Enter | keyboard::key::Named::Space
                ) {
                    if let Some(message) = state
                        .focused_index
                        .and_then(|index| self.items.get(index))
                        .and_then(|item| item.message.clone())
                    {
                        shell.publish(message);
                        shell.capture_event();
                    }
                    return;
                }
            }
        }

        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        ) {
            if let Some(message) = state
                .hovered_index
                .and_then(|index| self.items.get(index))
                .and_then(|item| item.message.clone())
            {
                shell.publish(message);
                shell.capture_event();
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        _layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &nive_ui::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<TrackState>();
        if state
            .item_bounds
            .iter()
            .any(|bounds| cursor.is_over(*bounds))
        {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut nive_ui::Renderer,
        theme: &nive_ui::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<TrackState>();
        for (index, bounds) in state.item_bounds.iter().enumerate() {
            let disabled = self.items[index].metadata.disabled;
            let active = index == self.active_index;
            let hovered = state.hovered_index == Some(index) && !disabled;
            let control_state = if disabled {
                ControlState::DISABLED.selected_if(active)
            } else if hovered {
                ControlState::HOVERED.selected_if(active)
            } else {
                ControlState::ENABLED.selected_if(active)
            };
            let background = if active || hovered {
                theme
                    .control(ControlRole::Selectable, control_state)
                    .background
            } else {
                Color::TRANSPARENT
            };
            renderer.fill_quad(
                renderer::Quad {
                    bounds: *bounds,
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: true,
                },
                background,
            );
        }
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
        let metrics = self.metrics();
        if let Some(bounds) = state.item_bounds.get(self.active_index) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x,
                        y: bounds.y + bounds.height - 2.0,
                        width: bounds.width,
                        height: 2.0,
                    },
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: true,
                },
                theme
                    .control(ControlRole::Selectable, ControlState::SELECTED)
                    .foreground,
            );
        }
        if state.focused {
            if let Some(bounds) = state
                .focused_index
                .and_then(|index| state.item_bounds.get(index))
            {
                let focus = theme.border(BorderRole::Focus);
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: *bounds,
                        border: Border {
                            color: focus.color,
                            width: focus.width,
                            radius: metrics.radius.into(),
                        },
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    Color::TRANSPARENT,
                );
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &nive_ui::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, nive_ui::Theme, nive_ui::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<BottomPanelTabTrack<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(track: BottomPanelTabTrack<'a, Message>) -> Self {
        Element::new(track)
    }
}

fn tab_content_style(active: bool, disabled: bool) -> impl Fn(&nive_ui::Theme) -> container::Style {
    move |theme| container::Style {
        text_color: Some(if disabled {
            theme
                .control(
                    ControlRole::Selectable,
                    ControlState::DISABLED.selected_if(active),
                )
                .foreground
        } else if active {
            theme.text(TextRole::Primary).color
        } else {
            theme.text(TextRole::Secondary).color
        }),
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border::default(),
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

fn horizontal_wheel(delta: mouse::ScrollDelta) -> f32 {
    match delta {
        mouse::ScrollDelta::Lines { x, y } => (if x.abs() > f32::EPSILON { x } else { y }) * 24.0,
        mouse::ScrollDelta::Pixels { x, y } => {
            if x.abs() > f32::EPSILON {
                x
            } else {
                y
            }
        }
    }
}

fn reveal_index(state: &mut TrackState, index: usize) {
    let Some(bounds) = state.item_bounds.get(index) else {
        return;
    };
    if bounds.x < 0.0 {
        state.offset = (state.offset + bounds.x).max(0.0);
    } else if bounds.x + bounds.width > state.viewport_width {
        state.offset =
            (state.offset + bounds.x + bounds.width - state.viewport_width).min(state.max_offset);
    }
}

fn translate_track(node: Node, offset: f32) -> (f32, f32, Vec<Rectangle>, Node) {
    let Some(row) = node.children().first() else {
        return (0.0, node.size().width, Vec::new(), node);
    };
    let viewport_width = node.size().width;
    let translation = Vector::new(-offset, 0.0);
    let mut children = Vec::with_capacity(row.children().len());
    let mut bounds = Vec::with_capacity(row.children().len());
    for child in row.children() {
        let mut child = child.clone();
        child.translate_mut(translation);
        bounds.push(child.bounds());
        children.push(child);
    }
    let content_width = row
        .children()
        .last()
        .map(|child| child.bounds().x + child.bounds().width - row.bounds().x)
        .unwrap_or(0.0);
    let translated_row =
        Node::with_children(row.size(), children).move_to(row.bounds().position() + translation);
    let translated = Node::with_children(node.size(), vec![translated_row]);
    (content_width, viewport_width, bounds, translated)
}

trait SelectedIf {
    fn selected_if(self, selected: bool) -> Self;
}

impl SelectedIf for ControlState {
    fn selected_if(self, selected: bool) -> Self {
        if selected {
            self.selected()
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    fn tab(label: &'static str, disabled: bool) -> BottomHeaderTab<'static, &'static str> {
        BottomHeaderTab {
            panel_id: label,
            label: Cow::Borrowed(label),
            icon: None,
            badge: None,
            status: None,
            disabled,
            tooltip: None,
        }
    }

    #[test]
    fn composite_focus_starts_active_and_skips_disabled_tabs() {
        let track = BottomPanelTabTrack::new(ControlSize::Sm, 1)
            .push(tab("Output", false), Some(1_u8))
            .push(tab("Problems", false), Some(2))
            .push(tab("Disabled", true), None)
            .push(tab("Terminal", false), Some(4));
        let mut state = TrackState::default();

        track.reconcile_focus(&mut state);
        assert_eq!(state.focused_index, Some(1));
        assert_eq!(track.enabled_indices(), vec![0, 1, 3]);
    }

    #[test]
    fn vertical_wheel_maps_to_horizontal_motion() {
        assert_eq!(
            horizontal_wheel(mouse::ScrollDelta::Lines { x: 0.0, y: 2.0 }),
            48.0
        );
        assert_eq!(
            horizontal_wheel(mouse::ScrollDelta::Pixels { x: 3.0, y: 20.0 }),
            3.0
        );
    }

    #[test]
    fn reveal_uses_minimum_displacement_and_clamps_offset() {
        let mut state = TrackState {
            item_bounds: vec![Rectangle::new(
                iced::Point::new(140.0, 0.0),
                Size::new(80.0, 28.0),
            )],
            viewport_width: 180.0,
            max_offset: 100.0,
            ..TrackState::default()
        };

        reveal_index(&mut state, 0);
        assert_eq!(state.offset, 40.0);
    }
}
