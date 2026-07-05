use iced::{
    advanced::{
        layout, mouse, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    time::Duration,
    window, Event, Length, Rectangle, Size,
};

use crate::Element;

use super::runner::AnimationState;
use super::timeline::{Animation, AnimationFrame};

type ViewFn<'a, Message> = dyn Fn(AnimationFrame) -> Element<'a, Message> + 'a;

/// A self-contained widget that re-renders its content as a function of the
/// animation frame, **without** affecting layout.
///
/// Use it for purely visual motion — rotation, opacity, colour, the loading
/// dots. The content must keep a stable size between frames; to animate
/// width/height use `AnimatedLayout` instead.
pub struct AnimatedVisual<'a, Message> {
    animation: Animation,
    frame_interval: Option<Duration>,
    view: Box<ViewFn<'a, Message>>,
}

impl<'a, Message> AnimatedVisual<'a, Message> {
    pub fn new(view: impl Fn(AnimationFrame) -> Element<'a, Message> + 'a) -> Self {
        Self {
            animation: Animation::default(),
            frame_interval: None,
            view: Box::new(view),
        }
    }

    pub fn animation(mut self, animation: Animation) -> Self {
        self.animation = animation;
        self
    }

    /// Pins the redraw cadence to a fixed interval instead of the display's
    /// native refresh rate (the default).
    pub fn frame_interval(mut self, frame_interval: Duration) -> Self {
        self.frame_interval = Some(frame_interval);
        self
    }

    fn content(&self, frame: AnimationFrame) -> Element<'a, Message> {
        (self.view)(frame)
    }

    fn frame_from_tree(&self, tree: &Tree) -> AnimationFrame {
        tree.state
            .downcast_ref::<AnimationState>()
            .frame(self.animation)
    }
}

impl<Message> Widget<Message, crate::theme::Theme, iced::Renderer> for AnimatedVisual<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<AnimationState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(AnimationState::default())
    }

    fn children(&self) -> Vec<Tree> {
        let content = self.content(AnimationFrame::initial());

        vec![Tree::new(&content)]
    }

    fn diff(&self, tree: &mut Tree) {
        let content = self.content(self.frame_from_tree(tree));

        tree.diff_children(&[content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.content(AnimationFrame::initial()).as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content(AnimationFrame::initial())
            .as_widget()
            .size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mut content = self.content(self.frame_from_tree(tree));
        tree.children[0].diff(content.as_widget());

        content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let mut content = self.content(self.frame_from_tree(tree));
        tree.children[0].diff(content.as_widget());

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
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let state = tree.state.downcast_mut::<AnimationState>();
            state.advance(*now, self.animation, self.frame_interval, shell);
        }

        let mut content = self.content(self.frame_from_tree(tree));
        tree.children[0].diff(content.as_widget());

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

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let content = self.content(self.frame_from_tree(tree));

        content
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
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
        let content = self.content(self.frame_from_tree(tree));

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

impl<'a, Message> From<AnimatedVisual<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(animated: AnimatedVisual<'a, Message>) -> Self {
        Element::new(animated)
    }
}
