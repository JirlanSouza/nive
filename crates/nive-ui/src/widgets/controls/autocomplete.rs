mod builder;
mod render;
mod suggestion;
mod widget;

#[cfg(test)]
mod tests;

use std::borrow::Cow;

use crate::{
    theme::{ControlSize, FieldValidation},
    widgets::primitives::IconRole,
    Element,
};
use iced::{widget::Id, Length};

/// One typed application value rendered by [`Autocomplete`].
///
/// The value is durable identity and must be unique within a Suggestions
/// result. Leading icons and trailing secondary text are presentation metadata;
/// a disabled suggestion remains visible but cannot be activated.
#[derive(Debug, Clone)]
pub struct AutocompleteSuggestion<'a, T>
where
    T: Clone + Eq,
{
    value: T,
    label: Cow<'a, str>,
    leading: Option<IconRole>,
    trailing: Option<Cow<'a, str>>,
    disabled: bool,
}

/// One atomic result state supplied to [`Autocomplete`].
///
/// The application supplies exactly one complete state per view. Loading,
/// Empty, and Error remain visible popup content with no selectable rows.
/// Retrieval Error is independent from [`FieldValidation::Invalid`].
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutocompleteResults<'a, T>
where
    T: Clone + Eq,
{
    /// The complete ordered suggestion set for this view.
    Suggestions(Vec<AutocompleteSuggestion<'a, T>>),
    /// Results are being retrieved.
    Loading,
    /// Retrieval completed without suggestions and supplies visible help text.
    Empty(Cow<'a, str>),
    /// Retrieval failed and supplies visible popup error text.
    Error(Cow<'a, str>),
}

/// Initial logical highlight policy for a fresh suggestion session.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AutocompleteHighlight {
    /// Open a fresh suggestion session without an implicit choice.
    #[default]
    None,
    /// Highlight the first eligible suggestion in a fresh session.
    First,
}

/// A controlled typed query input with one atomic popup result model.
///
/// Query editing and committed selection remain application-owned. Semantic
/// names and selected values are retained as preparatory metadata; Nive does
/// not currently claim native expanded or active-descendant emission. The
/// application also owns filtering, ordering, retrieval, and result-state
/// transitions; retrieval Error does not invalidate the Field.
///
/// Actual focus and caret remain in the Input while suggestions use logical
/// highlight. Arrow navigation is bounded and does not change query text;
/// Enter without a highlight remains available to Input submit. Pointer
/// selection publishes `on_select(T)` before blur processing and does not also
/// publish `on_dismiss`.
///
/// Each optional callback controls only its own capability. Missing change,
/// clear, select, or dismiss callbacks do not imply disabled styling or create
/// hidden application state. Explicit `disabled(true)` wins over all callbacks
/// and suppresses interaction while preserving frame and content geometry.
/// A dismiss-capable Escape, outside press, Tab, or real blur publishes exactly
/// one dismissal and latches that semantic query/results/focus session closed;
/// without dismissal capability no latch or simulated close is created.
/// Programmatic `open` changes are silent.
///
/// Popup, result, and trailing-slot visuals update immediately with no local
/// animator or motion preference. Retained names, open state, values, result
/// state, and logical highlight are preparatory metadata only: Autocomplete
/// does not emit native combobox roles, names, expanded state,
/// active-descendant relations, or announcements.
///
/// Legacy message adapters are intentionally absent:
///
/// ```compile_fail
/// use nive_ui::widgets::AutocompleteMessage;
/// ```
///
/// Arbitrary result content, independent item counts, and anchor adapters from
/// the former API are intentionally absent in favor of atomic typed results:
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let autocomplete = Autocomplete::<u8, ()>::new("", None, AutocompleteResults::Loading);
/// let _ = autocomplete.content(text("Rows"));
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let autocomplete = Autocomplete::<u8, ()>::new("", None, AutocompleteResults::Loading);
/// let _ = autocomplete.content_with(|_| text("Rows").into());
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let autocomplete = Autocomplete::<u8, ()>::new("", None, AutocompleteResults::Loading);
/// let _ = autocomplete.item_count(1);
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let input = Input::<()>::new("Search", "");
/// let _ = Autocomplete::with_anchor(input, Into::into);
/// ```
///
/// ```compile_fail
/// use nive_ui::widgets::controls::autocomplete::widget::{
///     AutocompleteCallbacks, AutocompleteHandles, AutocompleteWidget, HighlightVisibility,
/// };
/// ```
pub struct Autocomplete<'a, T, Message>
where
    T: Clone + Eq,
{
    query: Cow<'a, str>,
    selected: Option<T>,
    results: AutocompleteResults<'a, T>,
    placeholder: Cow<'a, str>,
    semantic_name: Option<Cow<'a, str>>,
    size: ControlSize,
    width: Length,
    validation: FieldValidation,
    disabled: bool,
    id: Option<Id>,
    open: bool,
    highlight: AutocompleteHighlight,
    on_change: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_select: Option<Box<dyn Fn(T) -> Message + 'a>>,
    on_clear: Option<Message>,
    on_submit: Option<Message>,
    on_blur: Option<Message>,
    on_dismiss: Option<Message>,
}

impl<'a, T, Message> From<Autocomplete<'a, T, Message>> for Element<'a, Message>
where
    T: Clone + Eq + 'a + 'static,
    Message: Clone + 'a,
{
    fn from(autocomplete: Autocomplete<'a, T, Message>) -> Self {
        autocomplete.into_element()
    }
}
