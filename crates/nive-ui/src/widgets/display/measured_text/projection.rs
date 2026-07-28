//! Fitting a string into a measured width: how much is visible, and whether
//! anything was dropped.

use super::*;

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

pub(super) fn update_state(state: &mut State, projection: Projection, finite_width: Option<f32>) {
    state.projection = projection.visible;
    state.finite_width = finite_width;
    state.truncated = projection.truncated;
}

#[cfg(test)]
pub(super) fn tooltip_source<'a>(state: &State, original: &'a str) -> Option<&'a str> {
    state.truncated.then_some(original)
}

pub(crate) fn measure_width(_renderer: &iced::Renderer, content: &str, style: TextStyle) -> f32 {
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
