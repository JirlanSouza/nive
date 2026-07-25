use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    Event, Length, Rectangle, Size, Vector,
};

use super::OverflowAxis;
use crate::Element;

/// The scrolling window of an overflow strip.
///
/// Two things a clipping `container` cannot do:
///
/// - `iced`'s `container(..).clip(true)` only narrows the viewport handed to
///   the child; quads keep painting at their own bounds. This wrapper adds the
///   real recorte through `Renderer::with_layer`, and hides the cursor from
///   content scrolled outside the window so it cannot be hovered or clicked.
/// - It measures the content unbounded along the scrolled axis. Under the
///   window's own limits `flex` hands the leftover space out in order, so items
///   past the fold collapse to zero instead of overflowing — the strip then
///   reads as "fits" and never scrolls.
///
/// Its node keeps the shape a clipping `container` produces — one child,
/// resolved size — so scroll translation that walks the layout tree is
/// unaffected.
pub(crate) struct ClipViewport<'a, Message> {
    content: Element<'a, Message>,
    axis: OverflowAxis,
    width: Length,
    height: Length,
}

impl<'a, Message> ClipViewport<'a, Message> {
    pub(crate) fn vertical(content: impl Into<Element<'a, Message>>) -> Self {
        Self::new(content, OverflowAxis::Vertical)
    }

    pub(crate) fn horizontal(content: impl Into<Element<'a, Message>>) -> Self {
        Self::new(content, OverflowAxis::Horizontal)
    }

    fn new(content: impl Into<Element<'a, Message>>, axis: OverflowAxis) -> Self {
        Self {
            content: content.into(),
            axis,
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    pub(crate) fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub(crate) fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

fn visible_cursor(cursor: mouse::Cursor, bounds: Rectangle) -> mouse::Cursor {
    match cursor.position() {
        Some(point) if bounds.contains(point) => cursor,
        _ => mouse::Cursor::Unavailable,
    }
}

fn clipped(bounds: Rectangle, viewport: &Rectangle) -> Rectangle {
    bounds.intersection(viewport).unwrap_or(bounds)
}

impl<Message> Widget<Message, crate::theme::Theme, iced::Renderer> for ClipViewport<'_, Message> {
    // Transparent in the state tree, exactly like the `container` this
    // replaces: it adds a layout node but never a `Tree` level, so widget state
    // below the strip keeps lining up with the rest of the bar.
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let max = limits.max();
        let content_max = match self.axis {
            OverflowAxis::Vertical => Size::new(max.width, f32::INFINITY),
            OverflowAxis::Horizontal => Size::new(f32::INFINITY, max.height),
        };
        let content = self.content.as_widget_mut().layout(
            tree,
            renderer,
            &layout::Limits::new(Size::ZERO, content_max),
        );

        layout::Node::with_children(
            limits.resolve(self.width, self.height, content.size()),
            vec![content],
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        if let Some(child_layout) = layout.children().next() {
            self.content
                .as_widget_mut()
                .operate(tree, child_layout, renderer, operation);
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
        let Some(child_layout) = layout.children().next() else {
            return;
        };
        let bounds = layout.bounds();

        self.content.as_widget_mut().update(
            tree,
            event,
            child_layout,
            visible_cursor(cursor, bounds),
            renderer,
            clipboard,
            shell,
            &clipped(bounds, viewport),
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
        layout
            .children()
            .next()
            .map(|child_layout| {
                self.content.as_widget().mouse_interaction(
                    tree,
                    child_layout,
                    visible_cursor(cursor, layout.bounds()),
                    viewport,
                    renderer,
                )
            })
            .unwrap_or_default()
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
        let Some(child_layout) = layout.children().next() else {
            return;
        };

        let bounds = layout.bounds();
        let clip = clipped(bounds, viewport);

        renderer.with_layer(clip, |renderer| {
            self.content.as_widget().draw(
                tree,
                renderer,
                theme,
                inherited_style,
                child_layout,
                visible_cursor(cursor, bounds),
                &clip,
            );
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
        let child_layout = layout.children().next()?;
        // Clipped too: an overlay anchored on content scrolled out of the
        // window must not position itself against what it cannot show.
        let viewport = clipped(layout.bounds(), viewport);

        self.content
            .as_widget_mut()
            .overlay(tree, child_layout, renderer, &viewport, translation)
    }
}

impl<'a, Message> From<ClipViewport<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(viewport: ClipViewport<'a, Message>) -> Self {
        Element::new(viewport)
    }
}

#[cfg(test)]
mod tests {
    use iced::{
        advanced::{
            layout::{Limits, Node},
            mouse,
            widget::Tree,
            Layout,
        },
        widget::container,
        Length, Point, Rectangle, Size,
    };

    use super::ClipViewport;
    use crate::test_support::layout;
    use crate::Element;

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Message {}

    fn body<'a>() -> Element<'a, Message> {
        let mut column = iced::widget::Column::new()
            .spacing(0.0)
            .width(Length::Fixed(120.0))
            .height(Length::Shrink);

        for _ in 0..4 {
            column = column.push(
                iced::widget::Space::new()
                    .width(Length::Fixed(120.0))
                    .height(Length::Fixed(100.0)),
            );
        }

        column.into()
    }

    fn window(content: Element<'_, Message>, maximum: Size) -> Node {
        layout(
            ClipViewport::vertical(content)
                .width(Length::Fixed(120.0))
                .height(Length::Fill)
                .into(),
            maximum,
        )
    }

    #[test]
    fn window_keeps_its_own_size_while_content_overflows() {
        let node = window(body(), Size::new(200.0, 150.0));
        let content = &node.children()[0];
        let last = content.children().last().expect("last child");

        assert_eq!(node.size(), Size::new(120.0, 150.0));
        assert_eq!(content.size().height, 400.0);
        assert_eq!(last.bounds().y + last.bounds().height, 400.0);
    }

    #[test]
    fn a_clipping_container_would_collapse_the_same_content() {
        let contained = layout(
            container(body())
                .width(Length::Fixed(120.0))
                .height(Length::Fill)
                .clip(true)
                .into(),
            Size::new(200.0, 150.0),
        );
        let heights: Vec<f32> = contained.children()[0]
            .children()
            .iter()
            .map(|child| child.size().height)
            .collect();

        assert_eq!(heights, vec![100.0, 50.0, 0.0, 0.0]);
    }

    #[test]
    fn cross_axis_stays_bounded_by_the_window() {
        let node = window(body(), Size::new(64.0, 150.0));

        assert_eq!(node.children()[0].size().width, 64.0);
    }

    #[test]
    fn horizontal_windows_free_the_measured_width() {
        let mut row = iced::widget::Row::<Message, crate::theme::Theme>::new().spacing(0.0);
        for _ in 0..4 {
            row = row.push(
                iced::widget::Space::new()
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(30.0)),
            );
        }
        let node = layout(
            ClipViewport::horizontal(row)
                .width(Length::Fill)
                .height(Length::Fixed(30.0))
                .into(),
            Size::new(150.0, 200.0),
        );

        assert_eq!(node.size(), Size::new(150.0, 30.0));
        assert_eq!(node.children()[0].size().width, 400.0);
    }

    #[test]
    fn state_tree_stays_transparent_like_a_container() {
        let shape = |tree: &Tree| -> Vec<usize> {
            tree.children
                .iter()
                .map(|child| child.children.len())
                .collect()
        };
        let bare = Tree::new(body());
        let wrapped = Tree::new(Element::from(ClipViewport::vertical(body())));
        let contained = Tree::new(Element::from(container(body())));

        assert_eq!(wrapped.children.len(), bare.children.len());
        assert_eq!(shape(&wrapped), shape(&bare));
        assert_eq!(shape(&wrapped), shape(&contained));
    }

    #[test]
    fn shrink_axes_follow_the_content() {
        let node = layout(
            ClipViewport::vertical(body()).into(),
            Size::new(500.0, 500.0),
        );

        assert_eq!(node.size(), Size::new(120.0, 400.0));
    }

    fn pointer_row<'a>() -> Element<'a, Message> {
        let inner = iced::widget::mouse_area(
            iced::widget::Space::new()
                .width(Length::Fixed(400.0))
                .height(Length::Fixed(30.0)),
        )
        .interaction(mouse::Interaction::Pointer);

        iced::widget::Row::<Message, crate::theme::Theme>::new()
            .push(inner)
            .into()
    }

    fn interaction_at(point: Point) -> mouse::Interaction {
        let mut element = Element::from(
            ClipViewport::horizontal(pointer_row())
                .width(Length::Fixed(150.0))
                .height(Length::Fixed(30.0)),
        );
        let renderer = crate::test_support::renderer();
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &Limits::new(Size::ZERO, Size::new(150.0, 30.0)),
        );
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(4096.0, 4096.0));

        element.as_widget().mouse_interaction(
            &tree,
            Layout::new(&node),
            mouse::Cursor::Available(point),
            &viewport,
            &renderer,
        )
    }

    #[test]
    fn the_cursor_is_masked_to_the_window() {
        // Inside the window the pointer child answers; past its right edge the
        // cursor is hidden, so content scrolled out of view cannot respond.
        assert_eq!(
            interaction_at(Point::new(20.0, 15.0)),
            mouse::Interaction::Pointer
        );
        assert_eq!(
            interaction_at(Point::new(300.0, 15.0)),
            mouse::Interaction::None
        );
    }
}
