use std::borrow::Cow;

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

pub(crate) struct MeasuredText<'a, Message> {
    original: Cow<'a, str>,
    strategy: EllipsisStrategy,
    typography: TypographyRole,
    color: Option<TextRole>,
    maximum_width: Option<f32>,
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
        );

        Self {
            original,
            strategy,
            typography,
            color: Some(color),
            maximum_width: None,
            content,
        }
    }

    pub(crate) fn new_inherited(
        original: impl Into<Cow<'a, str>>,
        strategy: EllipsisStrategy,
        typography: TypographyRole,
    ) -> Self {
        let original = original.into();
        let content = measured_content(original.clone(), original.clone(), typography, None, false);

        Self {
            original,
            strategy,
            typography,
            color: None,
            maximum_width: None,
            content,
        }
    }

    pub(crate) fn max_width(mut self, maximum_width: f32) -> Self {
        self.maximum_width = maximum_width.is_finite().then_some(maximum_width.max(0.0));
        self
    }

    fn content_element(&self, state: &State) -> Element<'a, Message> {
        measured_content(
            Cow::Owned(state.projection.clone()),
            self.original.clone(),
            self.typography,
            self.color,
            state.truncated,
        )
    }
}

fn measured_content<'a, Message>(
    visible: Cow<'a, str>,
    tooltip_label: Cow<'a, str>,
    typography: TypographyRole,
    color: Option<TextRole>,
    truncated: bool,
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
        tree::State::new(State {
            projection: self.original.to_string(),
            finite_width: None,
            truncated: false,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
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
        let was_truncated = state.truncated;
        let changed = update_state(state, projection, finite_width);

        let content = self.content_element(state);
        if was_truncated && !state.truncated {
            tree.children[0] = Tree::new(&content);
        } else if changed {
            tree.children[0].diff(content.as_widget());
        }
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
        if !tree.state.downcast_ref::<State>().truncated {
            return;
        }
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
        if !tree.state.downcast_ref::<State>().truncated {
            return mouse::Interaction::None;
        }
        self.content.as_widget().mouse_interaction(
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
        self.content.as_widget().draw(
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
        if !tree.state.downcast_ref::<State>().truncated {
            return None;
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Projection {
    pub(super) visible: String,
    pub(super) truncated: bool,
}

pub(super) fn project(
    original: &str,
    strategy: EllipsisStrategy,
    available_width: f32,
    mut measure: impl FnMut(&str) -> f32,
) -> Projection {
    let available_width = if available_width.is_finite() {
        available_width.max(0.0)
    } else {
        f32::INFINITY
    };

    if measure(original) <= available_width {
        return Projection {
            visible: original.to_string(),
            truncated: false,
        };
    }

    let graphemes: Vec<&str> = original.graphemes(true).collect();
    let visible = match strategy {
        EllipsisStrategy::End => end_projection(&graphemes, available_width, &mut measure),
        EllipsisStrategy::Middle => middle_projection(&graphemes, available_width, &mut measure),
    };

    Projection {
        visible,
        truncated: true,
    }
}

fn end_projection(
    graphemes: &[&str],
    available_width: f32,
    measure: &mut impl FnMut(&str) -> f32,
) -> String {
    for retained in (1..graphemes.len()).rev() {
        let candidate = format!("{}{}", graphemes[..retained].concat(), ELLIPSIS);
        if measure(&candidate) <= available_width {
            return candidate;
        }
    }

    fallback_projection(available_width, measure)
}

fn middle_projection(
    graphemes: &[&str],
    available_width: f32,
    measure: &mut impl FnMut(&str) -> f32,
) -> String {
    let mut best: Option<(usize, usize, usize, String)> = None;

    for prefix in 1..graphemes.len() {
        for suffix in 1..(graphemes.len() - prefix + 1) {
            if prefix + suffix >= graphemes.len() {
                continue;
            }
            let candidate = format!(
                "{}{}{}",
                graphemes[..prefix].concat(),
                ELLIPSIS,
                graphemes[graphemes.len() - suffix..].concat()
            );
            if measure(&candidate) > available_width {
                continue;
            }

            let score = (prefix + suffix, prefix.abs_diff(suffix), prefix);
            let replace = best
                .as_ref()
                .is_none_or(|(retained, imbalance, best_prefix, _)| {
                    score.0 > *retained
                        || (score.0 == *retained && score.1 < *imbalance)
                        || (score.0 == *retained && score.1 == *imbalance && score.2 > *best_prefix)
                });
            if replace {
                best = Some((score.0, score.1, score.2, candidate));
            }
        }
    }

    best.map_or_else(
        || fallback_projection(available_width, measure),
        |(_, _, _, value)| value,
    )
}

fn fallback_projection(available_width: f32, measure: &mut impl FnMut(&str) -> f32) -> String {
    if measure(ELLIPSIS) <= available_width {
        ELLIPSIS.to_string()
    } else {
        String::new()
    }
}

fn update_state(state: &mut State, projection: Projection, finite_width: Option<f32>) -> bool {
    let changed = state.projection != projection.visible
        || state.finite_width != finite_width
        || state.truncated != projection.truncated;
    state.projection = projection.visible;
    state.finite_width = finite_width;
    state.truncated = projection.truncated;
    changed
}

#[cfg(test)]
fn tooltip_source<'a>(state: &State, original: &'a str) -> Option<&'a str> {
    state.truncated.then_some(original)
}

pub(super) fn measure_width(_renderer: &iced::Renderer, content: &str, style: TextStyle) -> f32 {
    type Paragraph = <iced::Renderer as advanced_text::Renderer>::Paragraph;

    Paragraph::with_text(advanced_text::Text {
        content,
        bounds: Size::new(f32::INFINITY, f32::INFINITY),
        size: Pixels(style.size),
        line_height: advanced_text::LineHeight::Relative(style.line_height),
        font: style.font,
        align_x: advanced_text::Alignment::Default,
        align_y: alignment::Vertical::Center,
        shaping: advanced_text::Shaping::Auto,
        wrapping: advanced_text::Wrapping::None,
    })
    .min_width()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::WidgetHarness;
    use crate::widgets::controls::choice_test_support::pointer_move;
    use iced::{Point, Size};

    fn grapheme_width(value: &str) -> f32 {
        value.graphemes(true).count() as f32
    }

    #[test]
    fn end_projection_preserves_graphemes_and_fallback_ladder() {
        assert_eq!(
            project("Ame\u{301}lie", EllipsisStrategy::End, 4.0, grapheme_width),
            Projection {
                visible: "Ame\u{301}…".to_string(),
                truncated: true,
            }
        );
        assert_eq!(
            project("long", EllipsisStrategy::End, 1.0, grapheme_width).visible,
            "…"
        );
        assert_eq!(
            project("long", EllipsisStrategy::End, 0.0, grapheme_width).visible,
            ""
        );
    }

    #[test]
    fn middle_projection_uses_balance_then_longer_prefix_tie_break() {
        assert_eq!(
            project("abcdef", EllipsisStrategy::Middle, 5.0, grapheme_width).visible,
            "ab…ef"
        );
        assert_eq!(
            project("abcde", EllipsisStrategy::Middle, 4.0, grapheme_width).visible,
            "ab…e"
        );
    }

    #[test]
    fn complete_and_empty_values_retain_exact_content() {
        assert_eq!(
            project("", EllipsisStrategy::End, 0.0, grapheme_width),
            Projection {
                visible: String::new(),
                truncated: false,
            }
        );
        assert_eq!(
            project("界", EllipsisStrategy::Middle, 1.0, grapheme_width),
            Projection {
                visible: "界".to_string(),
                truncated: false,
            }
        );
    }

    #[test]
    fn unicode_cases_never_split_graphemes() {
        let cases = [
            ("e\u{301}clair", 2.0, "e\u{301}…"),
            ("👩‍💻studio", 2.0, "👩‍💻…"),
            ("界面", 3.0, "界…"),
        ];

        for (source, width, expected) in cases {
            assert_eq!(
                project(source, EllipsisStrategy::End, width, |value| {
                    value
                        .graphemes(true)
                        .map(|grapheme| {
                            if grapheme.contains(['界', '面']) {
                                2.0
                            } else {
                                1.0
                            }
                        })
                        .sum()
                })
                .visible,
                expected
            );
        }
    }

    #[test]
    fn measured_layout_state_recomputes_wide_narrow_wide_and_closes_tooltip() {
        let original = "build-2026.07.15";
        let mut state = State::default();

        let wide = project(original, EllipsisStrategy::Middle, 32.0, grapheme_width);
        assert!(update_state(&mut state, wide, Some(32.0)));
        assert_eq!(tooltip_source(&state, original), None);

        let narrow = project(original, EllipsisStrategy::Middle, 7.0, grapheme_width);
        assert!(update_state(&mut state, narrow, Some(7.0)));
        assert_eq!(tooltip_source(&state, original), Some(original));
        assert_ne!(state.projection, original);

        let restored = project(original, EllipsisStrategy::Middle, 32.0, grapheme_width);
        assert!(update_state(&mut state, restored, Some(32.0)));
        assert_eq!(state.projection, original);
        assert_eq!(tooltip_source(&state, original), None);
    }

    #[test]
    fn exact_fit_is_complete_but_sub_ellipsis_width_is_truncated() {
        assert!(!project("exact", EllipsisStrategy::End, 5.0, grapheme_width).truncated);
        let sub_ellipsis = project("x", EllipsisStrategy::End, 0.5, grapheme_width);
        assert!(sub_ellipsis.truncated);
        assert!(sub_ellipsis.visible.is_empty());
    }

    #[test]
    fn truncated_tooltip_keeps_one_tree_through_event_overlay_and_draw() {
        let label = || -> Element<'static, ()> {
            MeasuredText::new_inherited(
                "A deliberately long constrained label",
                EllipsisStrategy::End,
                TypographyRole::ControlStrong,
            )
            .max_width(48.0)
            .into()
        };
        let mut harness = WidgetHarness::new(label(), Size::new(48.0, 40.0));
        let point = Point::new(4.0, 8.0);

        harness.set_cursor(point);
        harness.update(pointer_move(point));
        harness.draw();
        assert!(harness.draw_overlay());

        harness.replace(label());
        harness.set_cursor(point);
        harness.update(pointer_move(point));
        harness.draw();
        assert!(harness.draw_overlay());
    }
}
