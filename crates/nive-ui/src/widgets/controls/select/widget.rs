use std::{cell::Cell, rc::Rc, time::Duration};

use iced::{touch, widget::Id, Length};

use crate::{
    advanced::focus::FocusState, widgets::overlays::anchored_overlay::scroll::EnsureVisibleHandle,
    Element,
};

use super::SelectOption;

const TYPEAHEAD_TIMEOUT: Duration = Duration::from_millis(700);

mod helpers;
mod list;
mod trigger;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
enum SelectEvent<T> {
    Commit(T),
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TriggerPress {
    Pointer,
    Touch(touch::Finger),
}

#[derive(Debug, Default)]
pub(super) struct SelectState {
    focus: FocusState,
    open: bool,
    pressed: Option<TriggerPress>,
}

pub(super) struct SelectWidget<'a, T, Message>
where
    T: Clone + Eq,
{
    closed_trigger: Element<'a, Message>,
    open_trigger: Element<'a, Message>,
    popup: Element<'a, SelectEvent<T>>,
    options: Vec<SelectOption<'a, T>>,
    selected: Option<T>,
    width: Length,
    height: f32,
    disabled: bool,
    model_valid: bool,
    id: Option<Id>,
    on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    ensure_visible: EnsureVisibleHandle,
    focus_visible: Rc<Cell<bool>>,
}

impl<'a, T, Message> From<SelectWidget<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    fn from(select: SelectWidget<'a, T, Message>) -> Self {
        Element::new(select)
    }
}

#[derive(Debug, Default)]
pub(super) struct SelectListState {
    highlight: Option<usize>,
    highlighted_label: Option<String>,
    pressed: Option<usize>,
    typeahead: String,
    typeahead_deadline: Option<iced::time::Instant>,
    now: Option<iced::time::Instant>,
    ensure_pending: bool,
}

struct SelectList<'a, T>
where
    T: Clone + Eq,
{
    content: Element<'a, SelectEvent<T>>,
    options: Vec<SelectOption<'a, T>>,
    selected: Option<T>,
    ensure_visible: EnsureVisibleHandle,
    focus_visible: Rc<Cell<bool>>,
    selection_capable: bool,
}

impl<'a, T> From<SelectList<'a, T>> for Element<'a, SelectEvent<T>>
where
    T: Clone + Eq + 'a,
{
    fn from(list: SelectList<'a, T>) -> Self {
        Element::new(list)
    }
}
