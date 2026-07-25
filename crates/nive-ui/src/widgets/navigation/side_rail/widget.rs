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
use crate::widgets::navigation::overflow::Overflow;
use crate::Element;

use super::item::SideRailItem;
use super::layout::{measure_and_translate, metrics};
use super::RailSide;

mod draw;
mod pointer;
mod update;

pub(super) type SelectCallback<'a, Id, Message> = Box<dyn Fn(Id) -> Message + 'a>;

pub(super) const CHEVRON_SCROLL_STEP_FACTOR: f32 = 0.8;

pub(super) fn seam_bounds(bounds: Rectangle, side: RailSide) -> Rectangle {
    Rectangle {
        x: match side {
            RailSide::Left => bounds.x + bounds.width - 1.0,
            RailSide::Right => bounds.x,
        },
        y: bounds.y,
        width: 1.0,
        height: bounds.height,
    }
}

/// A narrow vertical rail for left and right window edges.
///
/// `SideRail` owns rail layout policy, overflow state, and item activation
/// mapping. Items carry identity and metadata; enabled item activation maps
/// through rail-level `on_select` or `on_select_maybe`. Selection remains
/// application-owned through each [`SideRailItem::selected`] flag, so more
/// than one item may be selected at the same time.
///
/// Item spacing is an internal side rail metric. Labels rotate per side:
/// [`RailSide::Left`] reads bottom-to-top and [`RailSide::Right`] reads
/// top-to-bottom. Disabled items render inert and suppress selection messages.
pub struct SideRail<'a, Id, Message> {
    pub(super) side: RailSide,
    pub(super) items: Vec<SideRailItem<'a, Id>>,
    pub(super) size: ControlSize,
    pub(super) height: Option<Length>,
    pub(super) on_select: Option<SelectCallback<'a, Id, Message>>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct SideRailState {
    pub(super) overflow: Overflow,
    scroll_offset: f32,
    up_chevron: Option<Rectangle>,
    down_chevron: Option<Rectangle>,
}

impl<'a, Id, Message> SideRail<'a, Id, Message>
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
    pub fn push(mut self, item: SideRailItem<'a, Id>) -> Self {
        self.items.push(item);
        self
    }

    /// Adds one rail item.
    pub fn item(self, item: SideRailItem<'a, Id>) -> Self {
        self.push(item)
    }

    crate::impl_layout_builders!(height_opt, fill_height_opt);
}

impl<'a, Id, Message> From<SideRail<'a, Id, Message>> for Element<'a, Message>
where
    Id: Clone + 'a,
    Message: Clone + 'a,
{
    fn from(rail: SideRail<'a, Id, Message>) -> Self {
        Element::new(rail)
    }
}

impl<'a, Id, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for SideRail<'a, Id, Message>
where
    Id: Clone + 'a,
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<SideRailState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(SideRailState::default())
    }

    fn children(&self) -> Vec<Tree> {
        let state = SideRailState::default();
        vec![Tree::new(self.content_element(&state))]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_ref::<SideRailState>();
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
        let state = tree.state.downcast_ref::<SideRailState>();
        let mut content = self.content_element(state);
        let node = content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let (content_height, strip_height, translated_node) =
            measure_and_translate(node, state.scroll_offset);
        let state = tree.state.downcast_mut::<SideRailState>();

        state.overflow.offset = state.scroll_offset;
        state.overflow.update_extents(content_height, strip_height);
        state.scroll_offset = state.overflow.offset;

        translated_node
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_ref::<SideRailState>();
        let mut content = self.content_element(state);
        content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    #[allow(clippy::too_many_arguments)]
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
        self.update_impl(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.mouse_interaction_impl(tree, layout, cursor, viewport, renderer)
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
        self.draw_impl(
            tree,
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }
}
