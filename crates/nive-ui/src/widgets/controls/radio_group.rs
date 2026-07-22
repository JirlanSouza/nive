mod builder;
mod group_widget;

#[cfg(test)]
mod tests;

use std::borrow::Cow;

use iced::{widget, Length};

use crate::advanced::focus::FocusState;
use crate::theme::{ControlSize, FieldValidation};
use crate::Element;

use super::FieldRequirement;

/// A typed, non-renderable option owned by a [`RadioGroup`].
///
/// Values must be unique within their group. The visible label is required and
/// may be supplemented by a wrapping description.
pub struct RadioOption<'a, T> {
    value: T,
    label: Cow<'a, str>,
    description: Option<Cow<'a, str>>,
    disabled: bool,
}

/// Layout policy for complete radio option rows.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadioGroupLayout {
    /// Stack options vertically.
    #[default]
    Vertical,
    /// Wrap between complete options when finite width is exhausted.
    HorizontalWrap,
}

/// A controlled, typed one-of-many choice with one composite focus entry.
///
/// The group owns its legend, requirement, description, error, selection, and
/// callback. `None` means no selected value; model a user-selectable “None” as
/// an ordinary inner `T` value. Duplicate option values produce a finite
/// display-only fallback. Physical LTR arrows navigate enabled options
/// circularly and Space activates the focused value. Native accessibility-tree
/// roles and relationships are not emitted yet.
pub struct RadioGroup<'a, T, Message> {
    legend: Cow<'a, str>,
    selected: Option<T>,
    options: Vec<RadioOption<'a, T>>,
    requirement: Option<FieldRequirement<'a>>,
    description: Option<Cow<'a, str>>,
    error: Option<Cow<'a, str>>,
    layout: RadioGroupLayout,
    size: ControlSize,
    width: Length,
    disabled: bool,
    id: Option<widget::Id>,
    on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
}

impl<'a, T, Message> From<RadioGroup<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    fn from(group: RadioGroup<'a, T, Message>) -> Self {
        group.into_element()
    }
}

struct RadioGroupWidget<'a, T, Message> {
    selected: Option<T>,
    options: Vec<RadioOption<'a, T>>,
    layout: RadioGroupLayout,
    size: ControlSize,
    width: Length,
    validation: FieldValidation,
    disabled: bool,
    id: Option<widget::Id>,
    on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
}

#[derive(Debug, Default)]
struct RadioGroupState {
    focus: FocusState,
    focused_index: Option<usize>,
}

struct RadioGroupFocus<'a> {
    focus: &'a mut FocusState,
    focused_index: &'a mut Option<usize>,
}

impl<'a, T, Message> From<RadioGroupWidget<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    fn from(group: RadioGroupWidget<'a, T, Message>) -> Self {
        Element::new(group)
    }
}
