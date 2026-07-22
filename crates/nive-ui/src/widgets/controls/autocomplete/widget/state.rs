use std::{cell::Cell, rc::Rc};

use iced::advanced::widget::{tree, Tree};

use super::{
    AutocompleteCallbacks, AutocompleteHandles, AutocompleteState, HighlightVisibilityState,
    ResultsSnapshot, SuggestionSnapshot,
};
use crate::widgets::controls::autocomplete::AutocompleteResults;

impl<'a, T, Message> AutocompleteCallbacks<'a, T, Message> {
    pub(in crate::widgets::controls::autocomplete) fn new(
        on_select: Option<Rc<dyn Fn(T) -> Message + 'a>>,
        on_dismiss: Option<Message>,
    ) -> Self {
        Self {
            on_select,
            on_dismiss,
        }
    }
}

impl AutocompleteHandles {
    pub(in crate::widgets::controls::autocomplete) fn new() -> Self {
        Self {
            input_focused: Rc::new(Cell::new(false)),
            highlighted_index: Rc::new(Cell::new(None)),
            ensure_pending: Rc::new(Cell::new(true)),
            local_closed: Rc::new(Cell::new(false)),
        }
    }

    pub(in crate::widgets::controls::autocomplete) fn input_focused(&self) -> Rc<Cell<bool>> {
        Rc::clone(&self.input_focused)
    }

    pub(in crate::widgets::controls::autocomplete) fn highlighted_index(
        &self,
    ) -> Rc<Cell<Option<usize>>> {
        Rc::clone(&self.highlighted_index)
    }

    pub(in crate::widgets::controls::autocomplete) fn ensure_pending(&self) -> Rc<Cell<bool>> {
        Rc::clone(&self.ensure_pending)
    }

    pub(in crate::widgets::controls::autocomplete) fn local_closed(&self) -> Rc<Cell<bool>> {
        Rc::clone(&self.local_closed)
    }
}

impl<T> ResultsSnapshot<T>
where
    T: Clone + Eq,
{
    pub(in crate::widgets::controls::autocomplete) fn from_results(
        results: &AutocompleteResults<'_, T>,
    ) -> Self {
        match results {
            AutocompleteResults::Suggestions(suggestions) => Self::Suggestions(
                suggestions
                    .iter()
                    .map(|suggestion| SuggestionSnapshot {
                        value: suggestion.value().clone(),
                        label: suggestion.label().to_owned(),
                        leading: suggestion.leading_icon(),
                        trailing: suggestion.trailing_text().map(str::to_owned),
                        disabled: suggestion.is_disabled(),
                    })
                    .collect(),
            ),
            AutocompleteResults::Loading => Self::Loading,
            AutocompleteResults::Empty(message) => Self::Empty(message.to_string()),
            AutocompleteResults::Error(message) => Self::Error(message.to_string()),
        }
    }
}

impl<T> Default for AutocompleteState<T> {
    fn default() -> Self {
        Self {
            highlighted: None,
            query: String::new(),
            results: None,
            was_open: false,
            initialized: false,
            focus_generation: 0,
            input_was_focused: false,
            latch: None,
            dismissal_message_pending: false,
        }
    }
}

pub(super) fn take_selection_request(tree: &mut Tree) -> bool {
    if tree.tag == tree::Tag::of::<HighlightVisibilityState>() {
        return std::mem::take(
            &mut tree
                .state
                .downcast_mut::<HighlightVisibilityState>()
                .selection_requested,
        );
    }
    tree.children.iter_mut().any(take_selection_request)
}
