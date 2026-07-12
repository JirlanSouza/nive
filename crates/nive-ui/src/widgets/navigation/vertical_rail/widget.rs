use iced::{
    advanced::{
        layout::{self, Layout},
        mouse, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Shell, Widget,
    },
    Event, Length, Rectangle, Size,
};

use crate::theme::ControlSize;
use crate::widgets::navigation::overflow::{
    wheel_delta, Overflow, OverflowAxis, OverflowDirection,
};
use crate::Element;

use super::item::VerticalRailItem;
use super::layout::{hit_geometry, measure_and_translate, metrics};
use super::RailSide;

pub(super) type SelectCallback<'a, Id, Message> = Box<dyn Fn(Id) -> Message + 'a>;

pub(super) const CHEVRON_SCROLL_STEP_FACTOR: f32 = 0.8;

/// A narrow vertical rail for left and right window edges.
///
/// `VerticalRail` owns rail layout policy, overflow state, and item activation
/// mapping. Items carry identity and metadata; enabled item activation maps
/// through rail-level `on_select` or `on_select_maybe`. Selection remains
/// application-owned through each [`VerticalRailItem::selected`] flag, so more
/// than one item may be selected at the same time.
///
/// Item spacing is an internal vertical rail metric. Labels rotate per side:
/// [`RailSide::Left`] reads bottom-to-top and [`RailSide::Right`] reads
/// top-to-bottom. Disabled items render inert and suppress selection messages.
pub struct VerticalRail<'a, Id, Message> {
    pub(super) side: RailSide,
    pub(super) items: Vec<VerticalRailItem<'a, Id>>,
    pub(super) size: ControlSize,
    pub(super) height: Option<Length>,
    pub(super) on_select: Option<SelectCallback<'a, Id, Message>>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct VerticalRailState {
    pub(super) overflow: Overflow,
    scroll_offset: f32,
    up_chevron: Option<Rectangle>,
    down_chevron: Option<Rectangle>,
}

impl<'a, Id, Message> VerticalRail<'a, Id, Message>
where
    Id: Clone + 'a,
    Message: Clone + 'a,
{
    /// Builds an empty vertical rail for the given window edge.
    pub fn new(side: RailSide) -> Self {
        Self {
            side,
            items: Vec::new(),
            size: ControlSize::Sm,
            height: None,
            on_select: None,
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

    /// Maps enabled item activation into app messages.
    pub fn on_select(mut self, mapper: impl Fn(Id) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(mapper));
        self
    }

    /// Conditionally maps enabled item activation into app messages.
    pub fn on_select_maybe<F>(mut self, mapper: Option<F>) -> Self
    where
        F: Fn(Id) -> Message + 'a,
    {
        self.on_select = mapper.map(|mapper| Box::new(mapper) as SelectCallback<'a, Id, Message>);
        self
    }

    /// Adds one rail item.
    pub fn push(mut self, item: VerticalRailItem<'a, Id>) -> Self {
        self.items.push(item);
        self
    }

    /// Adds one rail item.
    pub fn item(self, item: VerticalRailItem<'a, Id>) -> Self {
        self.push(item)
    }

    crate::impl_layout_builders!(height_opt, fill_height_opt);
}

impl<'a, Id, Message> From<VerticalRail<'a, Id, Message>> for Element<'a, Message>
where
    Id: Clone + 'a,
    Message: Clone + 'a,
{
    fn from(rail: VerticalRail<'a, Id, Message>) -> Self {
        Element::new(rail)
    }
}

impl<'a, Id, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for VerticalRail<'a, Id, Message>
where
    Id: Clone + 'a,
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
    ) -> layout::Node {
        let state = tree.state.downcast_ref::<VerticalRailState>();
        let mut content = self.content_element(state);
        let node = content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let (content_height, strip_height, translated_node, item_bounds) =
            measure_and_translate(node, state.scroll_offset);
        let state = tree.state.downcast_mut::<VerticalRailState>();

        state.overflow.offset = state.scroll_offset;
        state.overflow.update_extents(content_height, strip_height);
        state.scroll_offset = state.overflow.offset;
        let _ = item_bounds;

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
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if state.overflow.has_overflow => {
                let delta_y = wheel_delta(OverflowAxis::Vertical, *delta);
                state.overflow.offset = state.scroll_offset;
                state.overflow.scroll_by(delta_y);
                state.scroll_offset = state.overflow.offset;
                if delta_y != 0.0 {
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if state.overflow.has_overflow =>
            {
                if state
                    .up_chevron
                    .is_some_and(|bounds| cursor.is_over(bounds))
                {
                    state.overflow.offset = state.scroll_offset;
                    state
                        .overflow
                        .page_step(OverflowDirection::Backward, CHEVRON_SCROLL_STEP_FACTOR);
                    state.scroll_offset = state.overflow.offset;
                    shell.invalidate_layout();
                    shell.request_redraw();
                    shell.capture_event();
                } else if state
                    .down_chevron
                    .is_some_and(|bounds| cursor.is_over(bounds))
                {
                    state.overflow.offset = state.scroll_offset;
                    state
                        .overflow
                        .page_step(OverflowDirection::Forward, CHEVRON_SCROLL_STEP_FACTOR);
                    state.scroll_offset = state.overflow.offset;
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

        if state.overflow.has_overflow
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
