use iced::{
    advanced::{
        layout, mouse, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    animation::Animation as Tween,
    time::{Duration, Instant},
    window, Event, Length, Rectangle, Size,
};

use crate::Element;

use super::timeline::Easing;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Axis {
    Horizontal,
    Vertical,
}

/// A self-contained widget that animates the **size** of its content along one
/// axis, reflowing its siblings as it opens and closes.
///
/// Driven by the `open` flag: flip it and the box eases between `from` (closed)
/// and `to` (open), interrupting cleanly if toggled mid-flight. The content is
/// built once; only the box around it animates, revealing the content via a
/// clip. Reaching for this type makes its cost explicit: it relayouts the tree
/// on every frame while moving.
///
/// It is backed by a reversible [`iced::animation::Animation`], unlike the
/// stateless [`AnimatedVisual`]. To animate arbitrary, non-size values from your
/// `view` (colours, offsets), use that native animation directly instead.
///
/// [`AnimatedVisual`]: super::AnimatedVisual
pub struct AnimatedLayout<'a, Message> {
    content: Element<'a, Message>,
    axis: Axis,
    open: bool,
    from: f32,
    to: f32,
    duration: Duration,
    easing: Easing,
}

/// Tree state: a reversible transition plus the last clock and extent we saw.
#[derive(Debug)]
struct LayoutState {
    tween: Tween<bool>,
    now: Instant,
    last_extent: Option<f32>,
}

impl<'a, Message> AnimatedLayout<'a, Message> {
    const DEFAULT_DURATION: Duration = Duration::from_millis(250);

    /// Animates the height of `content` between `from` (closed) and `to` (open),
    /// toward the side selected by `open`.
    pub fn height(
        content: impl Into<Element<'a, Message>>,
        open: bool,
        from: f32,
        to: f32,
    ) -> Self {
        Self::new(content, Axis::Vertical, open, from, to)
    }

    /// Animates the width of `content` between `from` (closed) and `to` (open),
    /// toward the side selected by `open`.
    pub fn width(content: impl Into<Element<'a, Message>>, open: bool, from: f32, to: f32) -> Self {
        Self::new(content, Axis::Horizontal, open, from, to)
    }

    fn new(
        content: impl Into<Element<'a, Message>>,
        axis: Axis,
        open: bool,
        from: f32,
        to: f32,
    ) -> Self {
        Self {
            content: content.into(),
            axis,
            open,
            from,
            to,
            duration: Self::DEFAULT_DURATION,
            easing: Easing::EaseInOut,
        }
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    fn tween(&self) -> Tween<bool> {
        Tween::new(self.open)
            .duration(self.duration)
            .easing(self.easing)
    }

    /// The current extent, clamped to stay non-negative even when the easing
    /// overshoots (e.g. `EaseOutBack`/`Elastic`), which would otherwise yield a
    /// negative `Size`.
    fn extent(&self, state: &LayoutState) -> f32 {
        state
            .tween
            .interpolate(self.from, self.to, state.now)
            .max(0.0)
    }
}

/// Hides the cursor from the child when it falls outside the visible (clipped)
/// box, so content scrolled behind the reveal cannot be clicked or hovered.
fn visible_cursor(cursor: mouse::Cursor, bounds: Rectangle) -> mouse::Cursor {
    match cursor.position() {
        Some(point) if bounds.contains(point) => cursor,
        _ => mouse::Cursor::Unavailable,
    }
}

fn clipped(bounds: Rectangle, viewport: &Rectangle) -> Rectangle {
    bounds.intersection(viewport).unwrap_or(bounds)
}

impl<Message> Widget<Message, crate::theme::Theme, iced::Renderer> for AnimatedLayout<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<LayoutState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(LayoutState {
            tween: self.tween(),
            now: Instant::now(),
            last_extent: None,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        // The animated axis reports `Shrink` so its fill factor stays stable;
        // the real extent is carried by the laid-out node (see `layout`).
        let child = self.content.as_widget().size();

        match self.axis {
            Axis::Vertical => Size::new(child.width, Length::Shrink),
            Axis::Horizontal => Size::new(Length::Shrink, child.height),
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let extent = self.extent(tree.state.downcast_ref::<LayoutState>());

        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let child_size = child.size();

        let size = match self.axis {
            Axis::Vertical => Size::new(child_size.width, extent),
            Axis::Horizontal => Size::new(extent, child_size.height),
        };

        layout::Node::with_children(size, vec![child])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        if let Some(child_layout) = layout.children().next() {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                child_layout,
                renderer,
                operation,
            );
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
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let state = tree.state.downcast_mut::<LayoutState>();
            state.now = *now;

            // Pick up an `open` toggle and transition from wherever we are now,
            // so reversing mid-flight is seamless.
            if state.tween.value() != self.open {
                state.tween.go_mut(self.open, *now);
                shell.invalidate_layout();
            }

            if state.tween.is_animating(*now) {
                shell.request_redraw();

                // Relayout only when the extent actually changes, so the redraw
                // loop converges within the frame instead of thrashing layout.
                let extent = self.extent(state);
                if state.last_extent != Some(extent) {
                    state.last_extent = Some(extent);
                    shell.invalidate_layout();
                }
            }
        }

        if let Some(child_layout) = layout.children().next() {
            let bounds = layout.bounds();

            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                child_layout,
                visible_cursor(cursor, bounds),
                renderer,
                clipboard,
                shell,
                &clipped(bounds, viewport),
            );
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
        layout
            .children()
            .next()
            .map(|child_layout| {
                self.content.as_widget().mouse_interaction(
                    &tree.children[0],
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
                &tree.children[0],
                renderer,
                theme,
                inherited_style,
                child_layout,
                visible_cursor(cursor, bounds),
                &clip,
            );
        });
    }
}

impl<'a, Message> From<AnimatedLayout<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(animated: AnimatedLayout<'a, Message>) -> Self {
        Element::new(animated)
    }
}
