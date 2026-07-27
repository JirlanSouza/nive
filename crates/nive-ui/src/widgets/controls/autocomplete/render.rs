use std::borrow::Cow;
use std::{cell::Cell, ops::Range, rc::Rc};

use iced::{
    widget::{column, container, rich_text, row, span, text, Space},
    Alignment, Length, Padding,
};
use unicode_segmentation::UnicodeSegmentation;

use super::widget::HighlightVisibility;
use super::AutocompleteResults;
use crate::widgets::overlays::anchored_overlay::scroll::EnsureVisibleHandle;
use crate::{
    theme::{
        self,
        choice::{self, ChoicePersistentState, ChoiceStateInput},
        FieldValidation, TextRole, ToneRole, TypographyRole,
    },
    widgets::{
        display::measured_text::{EllipsisStrategy, MeasuredText},
        navigation::menu::{
            self, MENU_COLUMN_GAP, MENU_ICON_SIZE, MENU_LIST_INSET, MENU_ROW_HEIGHT,
            MENU_ROW_PADDING_H, MENU_ROW_RADIUS,
        },
        primitives::{icon, text as nive_text},
    },
    Element,
};

pub(super) fn results_content<'a, T, Message>(
    results: AutocompleteResults<'a, T>,
    query: Cow<'a, str>,
    highlighted: Rc<Cell<Option<usize>>>,
    ensure_pending: Rc<Cell<bool>>,
    ensure_visible: EnsureVisibleHandle,
    local_closed: Rc<Cell<bool>>,
    on_select: Option<Rc<dyn Fn(T) -> Message + 'a>>,
) -> Element<'a, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    let mut suggestions_meta = Vec::new();
    let mut content = column![].padding(MENU_LIST_INSET).width(Length::Fill);
    match results {
        AutocompleteResults::Suggestions(suggestions) => {
            for (index, suggestion) in suggestions.into_iter().enumerate() {
                let suggestion_disabled = suggestion.disabled;
                suggestions_meta.push((suggestion.value.clone(), suggestion_disabled));
                let leading: Element<'a, Message> = suggestion.leading.map_or_else(
                    || Space::new().width(Length::Fixed(MENU_ICON_SIZE)).into(),
                    |role| {
                        container(icon::reference(role).custom_size(MENU_ICON_SIZE))
                            .width(Length::Fixed(MENU_ICON_SIZE))
                            .center_y(Length::Fill)
                            .into()
                    },
                );
                let trailing: Element<'a, Message> = suggestion.trailing.map_or_else(
                    || Space::new().into(),
                    |trailing| {
                        nive_text::with_typography(trailing, TypographyRole::Control)
                            .style(theme::text::style(TextRole::Secondary))
                            .into()
                    },
                );
                let row = row![
                    leading,
                    container(suggestion_label(suggestion.label, query.as_ref()))
                        .width(Length::Fill)
                        .clip(true),
                    trailing,
                ]
                .spacing(MENU_COLUMN_GAP)
                .align_y(Alignment::Center)
                .height(Length::Fill)
                .width(Length::Fill);
                content = content.push(
                    container(row)
                        .style({
                            let highlighted = Rc::clone(&highlighted);
                            move |theme| {
                                let resolved = choice::resolve_state(ChoiceStateInput {
                                    // Suggestions are transient candidates, so
                                    // the highlight is the only state a row
                                    // carries — never committed selection.
                                    persistent: ChoicePersistentState::Unselected,
                                    validation: FieldValidation::Valid,
                                    callback_present: !suggestion_disabled,
                                    disabled: suggestion_disabled,
                                    hovered: highlighted.get() == Some(index),
                                    pressed: false,
                                    focused: false,
                                });
                                menu::style::row_style_with_fill(theme, resolved, MENU_ROW_RADIUS)
                            }
                        })
                        .padding(Padding::ZERO.horizontal(MENU_ROW_PADDING_H))
                        .height(Length::Fixed(MENU_ROW_HEIGHT))
                        .width(Length::Fill),
                );
            }
        }
        AutocompleteResults::Loading => {
            content = content.push(status_row("Loading…", TextRole::Secondary));
        }
        AutocompleteResults::Empty(message) => {
            content = content.push(status_row(message, TextRole::Secondary));
        }
        AutocompleteResults::Error(message) => {
            content = content.push(error_status_row(message));
        }
    }

    HighlightVisibility::new(
        container(content).width(Length::Fill).into(),
        highlighted,
        ensure_pending,
        ensure_visible,
        suggestions_meta,
        local_closed,
        on_select,
    )
    .into()
}

pub(super) fn suggestion_label<'a, Message>(
    label: Cow<'a, str>,
    query: &str,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let Some(matched) = first_grapheme_match(label.as_ref(), query) else {
        return MeasuredText::new_inherited(label, EllipsisStrategy::End, TypographyRole::Control)
            .into();
    };
    let normal = theme::typography(TypographyRole::Control);
    let strong = theme::typography(TypographyRole::ControlStrong);
    let mut spans: Vec<text::Span<'a, (), iced::Font>> = Vec::with_capacity(3);
    if matched.start > 0 {
        spans.push(
            span(label[..matched.start].to_owned())
                .font(normal.font)
                .size(normal.size)
                .line_height(normal.line_height),
        );
    }
    spans.push(
        span(label[matched.clone()].to_owned())
            .font(strong.font)
            .size(strong.size)
            .line_height(strong.line_height),
    );
    if matched.end < label.len() {
        spans.push(
            span(label[matched.end..].to_owned())
                .font(normal.font)
                .size(normal.size)
                .line_height(normal.line_height),
        );
    }

    rich_text(spans)
        .wrapping(text::Wrapping::None)
        .width(Length::Shrink)
        .into()
}

pub(super) fn first_grapheme_match(label: &str, query: &str) -> Option<Range<usize>> {
    let query: Vec<String> = query
        .graphemes(true)
        .map(|grapheme| grapheme.to_lowercase())
        .collect();
    if query.is_empty() {
        return None;
    }

    let label: Vec<(usize, &str, String)> = label
        .grapheme_indices(true)
        .map(|(start, grapheme)| (start, grapheme, grapheme.to_lowercase()))
        .collect();
    label
        .windows(query.len())
        .find(|window| {
            window
                .iter()
                .zip(&query)
                .all(|((_, _, candidate), expected)| candidate == expected)
        })
        .map(|window| {
            let start = window[0].0;
            let (last_start, last_grapheme, _) = &window[window.len() - 1];
            start..last_start + last_grapheme.len()
        })
}

pub(super) fn error_status_row<'a, Message>(
    message: impl Into<Cow<'a, str>>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    container(
        nive_text::with_typography(message.into(), TypographyRole::Control)
            .style(theme::text::tone(ToneRole::Danger)),
    )
    .padding(Padding::ZERO.horizontal(MENU_ROW_PADDING_H))
    .center_y(Length::Fixed(MENU_ROW_HEIGHT))
    .width(Length::Fill)
    .into()
}

pub(super) fn status_row<'a, Message>(
    message: impl Into<Cow<'a, str>>,
    role: TextRole,
) -> Element<'a, Message>
where
    Message: 'a,
{
    container(
        nive_text::with_typography(message.into(), TypographyRole::Control)
            .style(theme::text::style(role)),
    )
    .padding(Padding::ZERO.horizontal(MENU_ROW_PADDING_H))
    .center_y(Length::Fixed(MENU_ROW_HEIGHT))
    .width(Length::Fill)
    .into()
}
