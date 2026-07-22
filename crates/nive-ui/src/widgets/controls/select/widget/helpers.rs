use std::borrow::Cow;

use iced::{
    advanced::{
        mouse,
        widget::{tree, Tree},
    },
    touch,
    widget::{container, row, text},
    Alignment, Event, Length, Point, Rectangle,
};

use super::SelectListState;
use crate::{
    theme::{self, FormControlMetrics, TextRole},
    widgets::{
        controls::select::SelectOption,
        navigation::menu::{MENU_ICON_SIZE, MENU_LIST_INSET, MENU_ROW_HEIGHT},
        primitives::{icon, IconRole},
    },
    Element,
};

pub(super) fn trigger<'a, Message: 'a>(
    label: Cow<'a, str>,
    placeholder: bool,
    open: bool,
    width: Length,
    metrics: FormControlMetrics,
) -> Element<'a, Message> {
    let label = text(label)
        .font(metrics.text_style.font)
        .size(metrics.text_style.size)
        .line_height(text::LineHeight::Relative(metrics.text_style.line_height))
        .shaping(text::Shaping::Auto)
        .style(theme::text::style(if placeholder {
            TextRole::Muted
        } else {
            TextRole::Primary
        }));
    let chevron = icon::role(if open {
        IconRole::NiveDisclosureUp
    } else {
        IconRole::NiveDisclosureDown
    })
    .custom_size(MENU_ICON_SIZE);

    container(
        row![
            container(label).width(Length::Fill).clip(true),
            container(chevron)
                .width(Length::Fixed(MENU_ICON_SIZE))
                .height(Length::Fixed(MENU_ICON_SIZE)),
        ]
        .align_y(Alignment::Center),
    )
    .padding(metrics.padding)
    .width(width)
    .height(Length::Fixed(metrics.height))
    .into()
}

pub(super) fn find_list_state(tree: &mut Tree) -> Option<&mut SelectListState> {
    if tree.tag == tree::Tag::of::<SelectListState>() {
        return Some(tree.state.downcast_mut::<SelectListState>());
    }
    tree.children.iter_mut().find_map(find_list_state)
}

pub(super) fn unique_values<T: Eq>(options: &[SelectOption<'_, T>]) -> bool {
    options.iter().enumerate().all(|(index, option)| {
        options[..index]
            .iter()
            .all(|previous| previous.value() != option.value())
    })
}

pub(super) fn set_highlight<T>(
    options: &[SelectOption<'_, T>],
    state: &mut SelectListState,
    highlight: Option<usize>,
) {
    state.highlight = highlight;
    state.highlighted_label = highlight.map(|index| options[index].label().to_owned());
}

pub(super) fn first_enabled<T>(
    options: &[SelectOption<'_, T>],
    model_valid: bool,
) -> Option<usize> {
    model_valid
        .then(|| options.iter().position(|option| !option.is_disabled()))
        .flatten()
}

pub(super) fn last_enabled<T>(options: &[SelectOption<'_, T>], model_valid: bool) -> Option<usize> {
    model_valid
        .then(|| options.iter().rposition(|option| !option.is_disabled()))
        .flatten()
}

pub(super) fn is_enabled<T>(
    options: &[SelectOption<'_, T>],
    index: usize,
    model_valid: bool,
) -> bool {
    model_valid
        && options
            .get(index)
            .is_some_and(|option| !option.is_disabled())
}

pub(super) fn move_highlight<T>(
    options: &[SelectOption<'_, T>],
    current: Option<usize>,
    direction: isize,
    model_valid: bool,
) -> Option<usize> {
    if direction > 0 {
        ((current.map_or(0, |index| index.saturating_add(1)))..options.len())
            .find(|index| is_enabled(options, *index, model_valid))
            .or(current.filter(|index| is_enabled(options, *index, model_valid)))
    } else {
        let start = current.unwrap_or(options.len());
        (0..start)
            .rev()
            .find(|index| is_enabled(options, *index, model_valid))
            .or(current.filter(|index| is_enabled(options, *index, model_valid)))
    }
}

pub(super) fn typeahead_match<T>(
    options: &[SelectOption<'_, T>],
    current: Option<usize>,
    prefix: &str,
    model_valid: bool,
) -> Option<usize> {
    if options.is_empty() {
        return None;
    }
    let prefix = prefix.to_lowercase();
    let start = current.map_or(0, |index| (index + 1) % options.len());
    (0..options.len())
        .map(|offset| (start + offset) % options.len())
        .find(|index| {
            is_enabled(options, *index, model_valid)
                && options[*index].label().to_lowercase().starts_with(&prefix)
        })
}

pub(super) fn option_bounds(bounds: Rectangle, index: usize, count: usize) -> Option<Rectangle> {
    (index < count).then(|| Rectangle {
        x: bounds.x + MENU_LIST_INSET,
        y: bounds.y + MENU_LIST_INSET + index as f32 * MENU_ROW_HEIGHT,
        width: (bounds.width - MENU_LIST_INSET * 2.0).max(0.0),
        height: MENU_ROW_HEIGHT,
    })
}

pub(super) fn option_at(bounds: Rectangle, point: Point, count: usize) -> Option<usize> {
    if point.x < bounds.x + MENU_LIST_INSET
        || point.x > bounds.x + bounds.width - MENU_LIST_INSET
        || point.y < bounds.y + MENU_LIST_INSET
    {
        return None;
    }
    let index = ((point.y - bounds.y - MENU_LIST_INSET) / MENU_ROW_HEIGHT).floor() as usize;
    option_bounds(bounds, index, count)
        .is_some_and(|bounds| bounds.contains(point))
        .then_some(index)
}

pub(super) fn pointer_position(event: &Event, _cursor: mouse::Cursor) -> Option<Option<Point>> {
    match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => Some(Some(*position)),
        Event::Mouse(mouse::Event::CursorLeft) => Some(None),
        Event::Touch(touch::Event::FingerMoved { position, .. }) => Some(Some(*position)),
        _ => None,
    }
}

pub(super) fn is_primary_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
    )
}

pub(super) fn primary_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.position(),
        Event::Touch(touch::Event::FingerPressed { position, .. }) => Some(*position),
        _ => None,
    }
}

pub(super) fn release_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    match event {
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => cursor.position(),
        Event::Touch(touch::Event::FingerLifted { position, .. }) => Some(*position),
        _ => None,
    }
}

pub(super) fn is_release(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. })
    )
}
