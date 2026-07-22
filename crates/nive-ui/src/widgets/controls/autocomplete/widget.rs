use std::{cell::Cell, rc::Rc};

mod highlight;
mod main;
mod state;

#[cfg(test)]
mod tests;

use super::{AutocompleteHighlight, AutocompleteResults};
use crate::{widgets::overlays::anchored_overlay::scroll::EnsureVisibleHandle, Element};

#[derive(Clone)]
pub(super) struct AutocompleteHandles {
    input_focused: Rc<Cell<bool>>,
    highlighted_index: Rc<Cell<Option<usize>>>,
    ensure_pending: Rc<Cell<bool>>,
    local_closed: Rc<Cell<bool>>,
}

pub(super) struct AutocompleteCallbacks<'a, T, Message> {
    on_select: Option<Rc<dyn Fn(T) -> Message + 'a>>,
    on_dismiss: Option<Message>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResultsSnapshot<T> {
    Suggestions(Vec<SuggestionSnapshot<T>>),
    Loading,
    Empty(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SuggestionSnapshot<T> {
    value: T,
    label: String,
    leading: Option<crate::IconRole>,
    trailing: Option<String>,
    disabled: bool,
}

#[derive(Debug)]
struct AutocompleteState<T> {
    highlighted: Option<T>,
    query: String,
    results: Option<ResultsSnapshot<T>>,
    was_open: bool,
    initialized: bool,
    focus_generation: u64,
    input_was_focused: bool,
    latch: Option<AutocompleteLatch<T>>,
    dismissal_message_pending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AutocompleteLatch<T> {
    query: String,
    results: ResultsSnapshot<T>,
    focus_generation: u64,
}

pub(super) struct AutocompleteWidget<'a, T, Message>
where
    T: Clone + Eq,
{
    content: Element<'a, Message>,
    query: String,
    results: AutocompleteResults<'a, T>,
    open: bool,
    policy: AutocompleteHighlight,
    handles: AutocompleteHandles,
    callbacks: AutocompleteCallbacks<'a, T, Message>,
}

#[derive(Debug, Clone, Copy)]
enum Navigation {
    Previous,
    Next,
}

pub(super) struct HighlightVisibility<'a, T, Message> {
    content: Element<'a, Message>,
    highlighted_index: Rc<Cell<Option<usize>>>,
    ensure_pending: Rc<Cell<bool>>,
    ensure_visible: EnsureVisibleHandle,
    suggestions: Vec<(T, bool)>,
    local_closed: Rc<Cell<bool>>,
    on_select: Option<Rc<dyn Fn(T) -> Message + 'a>>,
}

#[derive(Debug, Default)]
struct HighlightVisibilityState {
    selection_requested: bool,
}

impl<'a, T, Message> From<HighlightVisibility<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + 'a,
    Message: 'a,
{
    fn from(visibility: HighlightVisibility<'a, T, Message>) -> Self {
        Element::new(visibility)
    }
}

impl<'a, T, Message> From<AutocompleteWidget<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    fn from(widget: AutocompleteWidget<'a, T, Message>) -> Self {
        Element::new(widget)
    }
}
