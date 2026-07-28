use std::{borrow::Cow, cell::Cell, rc::Rc};

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        text::{self as advanced_text, Paragraph as _},
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    alignment, Event, Length, Pixels, Rectangle, Size, Vector,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    theme::{TextRole, TextStyle, TypographyRole},
    widgets::{overlays::tooltip, text},
    Element,
};

const ELLIPSIS: &str = "…";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EllipsisStrategy {
    End,
    Middle,
}

#[derive(Debug, Default)]
struct State {
    projection: String,
    finite_width: Option<f32>,
    truncated: bool,
}

impl State {
    /// The state a freshly built tree starts in: the whole string, nothing
    /// measured, nothing truncated.
    fn untruncated(original: &str) -> Self {
        Self {
            projection: original.to_string(),
            finite_width: None,
            truncated: false,
        }
    }
}

pub(crate) struct MeasuredText<'a, Message> {
    original: Cow<'a, str>,
    strategy: EllipsisStrategy,
    typography: TypographyRole,
    color: Option<TextRole>,
    maximum_width: Option<f32>,
    logical_focus: Option<Rc<Cell<bool>>>,
    content: Element<'a, Message>,
}

impl<'a, Message> MeasuredText<'a, Message>
where
    Message: 'a,
{
    pub(crate) fn new(
        original: impl Into<Cow<'a, str>>,
        strategy: EllipsisStrategy,
        typography: TypographyRole,
        color: TextRole,
    ) -> Self {
        let original = original.into();
        let content = measured_content(
            original.clone(),
            original.clone(),
            typography,
            Some(color),
            false,
            None,
        );

        Self {
            original,
            strategy,
            typography,
            color: Some(color),
            maximum_width: None,
            logical_focus: None,
            content,
        }
    }

    pub(crate) fn new_inherited(
        original: impl Into<Cow<'a, str>>,
        strategy: EllipsisStrategy,
        typography: TypographyRole,
    ) -> Self {
        let original = original.into();
        let content = measured_content(
            original.clone(),
            original.clone(),
            typography,
            None,
            false,
            None,
        );

        Self {
            original,
            strategy,
            typography,
            color: None,
            maximum_width: None,
            logical_focus: None,
            content,
        }
    }

    pub(crate) fn max_width(mut self, maximum_width: f32) -> Self {
        self.maximum_width = maximum_width.is_finite().then_some(maximum_width.max(0.0));
        self
    }

    pub(crate) fn logical_focus_candidate(mut self, focused: Rc<Cell<bool>>) -> Self {
        self.logical_focus = Some(focused);
        self
    }

    fn content_element(&self, state: &State) -> Element<'a, Message> {
        measured_content(
            Cow::Owned(state.projection.clone()),
            self.original.clone(),
            self.typography,
            self.color,
            state.truncated,
            self.logical_focus.clone(),
        )
    }
}

fn measured_content<'a, Message>(
    visible: Cow<'a, str>,
    tooltip_label: Cow<'a, str>,
    typography: TypographyRole,
    color: Option<TextRole>,
    truncated: bool,
    logical_focus: Option<Rc<Cell<bool>>>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let content =
        text::with_typography(visible, typography).wrapping(iced::widget::text::Wrapping::None);
    let content = if let Some(color) = color {
        content.style(crate::theme::text::style(color))
    } else {
        content
    };

    if truncated {
        let content: Element<'a, Message> = match logical_focus {
            Some(focused) => LogicalFocusCandidate::new(content, focused).into(),
            None => content.into(),
        };
        #[cfg(test)]
        {
            return tooltip::immediate_for_test(content, tooltip_label);
        }

        #[cfg(not(test))]
        {
            return tooltip::Tooltip::new(content, tooltip_label).into();
        }
    }

    content.into()
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer> for MeasuredText<'a, Message>
where
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::untruncated(&self.original))
    }

    /// Built from what [`Widget::state`] describes, never from `self.content`.
    ///
    /// `layout` reassigns `self.content` as truncation comes and goes, so
    /// building the child tree from it hands a fresh tree — whose state says
    /// nothing is truncated — a child describing the tooltip-wrapped variant.
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(
            self.content_element(&State::untruncated(&self.original)),
        )]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_ref::<State>();
        let content = self.content_element(state);
        tree.diff_children(&[content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let style = crate::theme::typography(self.typography);
        let host_width = limits.max().width;
        let finite_host = host_width.is_finite().then_some(host_width.max(0.0));
        let finite_width = match (finite_host, self.maximum_width) {
            (Some(host), Some(maximum)) => Some(host.min(maximum)),
            (Some(host), None) => Some(host),
            (None, Some(maximum)) => Some(maximum),
            (None, None) => None,
        };
        let available = finite_width.unwrap_or(f32::INFINITY);
        let projection = project(
            self.original.as_ref(),
            self.strategy,
            available,
            |candidate| measure_width(renderer, candidate, style),
        );

        let state = tree.state.downcast_mut::<State>();
        update_state(state, projection, finite_width);

        // Reconciled every pass rather than only when this widget's own state
        // moved. The tree handed in is not always one this widget last laid out
        // — `Dialog` measures its footer against a throwaway `Tree::new`, and
        // `UserInterface::relayout` re-runs layout with no `diff` in between —
        // so a delta against the previous pass says nothing about whether the
        // child tree matches the element about to be laid out against it.
        // `Tree::diff` rebuilds on a tag mismatch, which is exactly the
        // truncated/untruncated swap between a bare `Text` and a tooltip.
        let content = self.content_element(state);
        tree.children[0].diff(content.as_widget());
        self.content = content;

        let child_limits = finite_width.map_or(*limits, |width| limits.max_width(width));
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &child_limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_ref::<State>();
        self.content = self.content_element(state);

        self.content
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
        let state = tree.state.downcast_ref::<State>();
        if !state.truncated {
            return;
        }
        self.content = self.content_element(state);

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
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        if !state.truncated {
            return mouse::Interaction::None;
        }

        self.content_element(state).as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
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
        // Rebuilt from the stored projection instead of `self.content`, which
        // only becomes the truncated shape inside `layout`. Overlays are
        // rebuilt and painted against a cached layout, so a drawing instance
        // may never have run `layout` — see `UserInterface::draw`.
        let state = tree.state.downcast_ref::<State>();

        self.content_element(state).as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        let state = tree.state.downcast_ref::<State>();
        if !state.truncated {
            return None;
        }
        self.content = self.content_element(state);

        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<MeasuredText<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(value: MeasuredText<'a, Message>) -> Self {
        Element::new(value)
    }
}

mod focus_candidate;
mod projection;

#[cfg(test)]
mod tests;

use focus_candidate::LogicalFocusCandidate;
pub(crate) use projection::measure_width;
use projection::{project, update_state};
