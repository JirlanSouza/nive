mod widget;

use std::borrow::Cow;
use std::{cell::Cell, ops::Range, rc::Rc};

use iced::{
    widget::{column, container, rich_text, row, span, text, Id, Space},
    Alignment, Length, Padding,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    theme::{self, ControlSize, FieldValidation, TextRole, ToneRole, TypographyRole},
    widgets::{
        display::measured_text::{EllipsisStrategy, MeasuredText},
        navigation::menu::{
            self, MENU_COLUMN_GAP, MENU_ICON_SIZE, MENU_LIST_INSET, MENU_ROW_HEIGHT,
            MENU_ROW_PADDING_H, MENU_ROW_RADIUS,
        },
        overlays::{Popover, PopoverCollision, PopoverInset, PopoverPlacement},
        primitives::{icon, text as nive_text, IconRole},
    },
    Element,
};

use super::{button, Input, InputGroup};
use crate::widgets::overlays::anchored_overlay::scroll::EnsureVisibleHandle;
use widget::{AutocompleteCallbacks, AutocompleteHandles, AutocompleteWidget, HighlightVisibility};

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

impl<'a, T> AutocompleteSuggestion<'a, T>
where
    T: Clone + Eq,
{
    pub fn new(value: T, label: impl Into<Cow<'a, str>>) -> Self {
        let label = label.into();
        debug_assert!(
            !label.trim().is_empty(),
            "AutocompleteSuggestion requires a nonempty visible label"
        );

        Self {
            value,
            label,
            leading: None,
            trailing: None,
            disabled: false,
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn label(&self) -> &str {
        self.label.as_ref()
    }

    pub fn leading_icon(&self) -> Option<IconRole> {
        self.leading
    }

    pub fn trailing_text(&self) -> Option<&str> {
        self.trailing.as_deref()
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn leading(mut self, icon: IconRole) -> Self {
        self.leading = Some(icon);
        self
    }

    pub fn trailing(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.trailing = Some(text.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<T> PartialEq for AutocompleteSuggestion<'_, T>
where
    T: Clone + Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.label == other.label
            && self.leading == other.leading
            && self.trailing == other.trailing
            && self.disabled == other.disabled
    }
}

impl<T> Eq for AutocompleteSuggestion<'_, T> where T: Clone + Eq {}

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

impl<'a, T> AutocompleteResults<'a, T>
where
    T: Clone + Eq,
{
    pub fn suggestions(suggestions: impl Into<Vec<AutocompleteSuggestion<'a, T>>>) -> Self {
        Self::Suggestions(suggestions.into())
    }

    pub fn empty(message: impl Into<Cow<'a, str>>) -> Self {
        Self::Empty(message.into())
    }

    pub fn error(message: impl Into<Cow<'a, str>>) -> Self {
        Self::Error(message.into())
    }

    pub fn as_suggestions(&self) -> Option<&[AutocompleteSuggestion<'a, T>]> {
        match self {
            Self::Suggestions(suggestions) => Some(suggestions),
            Self::Loading | Self::Empty(_) | Self::Error(_) => None,
        }
    }

    pub fn has_unique_values(&self) -> bool {
        let Some(suggestions) = self.as_suggestions() else {
            return true;
        };

        suggestions.iter().enumerate().all(|(index, suggestion)| {
            suggestions[..index]
                .iter()
                .all(|previous| previous.value() != suggestion.value())
        })
    }
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

impl<'a, T, Message> Autocomplete<'a, T, Message>
where
    T: Clone + Eq + 'a,
    Message: Clone + 'a,
{
    pub fn new(
        query: impl Into<Cow<'a, str>>,
        selected: Option<T>,
        results: AutocompleteResults<'a, T>,
    ) -> Self {
        Self {
            query: query.into(),
            selected,
            results,
            placeholder: Cow::Borrowed("Search"),
            semantic_name: None,
            size: ControlSize::Sm,
            width: Length::Fill,
            validation: FieldValidation::Valid,
            disabled: false,
            id: None,
            open: false,
            highlight: AutocompleteHighlight::None,
            on_change: None,
            on_select: None,
            on_clear: None,
            on_submit: None,
            on_blur: None,
            on_dismiss: None,
        }
    }

    pub fn query(&self) -> &str {
        self.query.as_ref()
    }

    pub fn selected(&self) -> Option<&T> {
        self.selected.as_ref()
    }

    pub fn results(&self) -> &AutocompleteResults<'a, T> {
        &self.results
    }

    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn semantic_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.semantic_name = Some(name.into());
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    pub fn lg(self) -> Self {
        self.size(ControlSize::Lg)
    }

    crate::impl_layout_builders!(width_direct, fill_width_direct, shrink_width_direct);

    pub fn validation(mut self, validation: FieldValidation) -> Self {
        self.validation = validation;
        self
    }

    pub fn invalid(self, invalid: bool) -> Self {
        self.validation(if invalid {
            FieldValidation::Invalid
        } else {
            FieldValidation::Valid
        })
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn highlight(mut self, highlight: AutocompleteHighlight) -> Self {
        self.highlight = highlight;
        self
    }

    pub fn on_change(mut self, callback: impl Fn(String) -> Message + 'a) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn on_change_maybe(mut self, callback: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.on_change = callback.map(|callback| Box::new(callback) as _);
        self
    }

    pub fn on_select(mut self, callback: impl Fn(T) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    pub fn on_select_maybe(mut self, callback: Option<impl Fn(T) -> Message + 'a>) -> Self {
        self.on_select = callback.map(|callback| Box::new(callback) as _);
        self
    }

    pub fn on_clear(mut self, message: Message) -> Self {
        self.on_clear = Some(message);
        self
    }

    pub fn on_clear_maybe(mut self, message: Option<Message>) -> Self {
        self.on_clear = message;
        self
    }

    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    pub fn on_submit_maybe(mut self, message: Option<Message>) -> Self {
        self.on_submit = message;
        self
    }

    pub fn on_blur(mut self, message: Message) -> Self {
        self.on_blur = Some(message);
        self
    }

    pub fn on_blur_maybe(mut self, message: Option<Message>) -> Self {
        self.on_blur = message;
        self
    }

    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }

    pub fn on_dismiss_maybe(mut self, message: Option<Message>) -> Self {
        self.on_dismiss = message;
        self
    }

    pub fn control_size(&self) -> ControlSize {
        self.size
    }

    pub fn field_validation(&self) -> FieldValidation {
        self.validation
    }

    pub fn semantic_name_value(&self) -> Option<&str> {
        self.semantic_name.as_deref()
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn widget_id(&self) -> Option<&Id> {
        self.id.as_ref()
    }

    pub fn highlight_policy(&self) -> AutocompleteHighlight {
        self.highlight
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub(crate) fn apply_field_context(
        mut self,
        label: Cow<'a, str>,
        size: ControlSize,
        validation: FieldValidation,
        disabled: bool,
    ) -> (Self, Id) {
        let id = self.id.clone().unwrap_or_else(Id::unique);
        self.id = Some(id.clone());
        self.semantic_name = Some(label);
        self.size = size;
        self.validation = validation;
        self.disabled |= disabled;
        (self, id)
    }

    fn into_element(self) -> Element<'a, Message>
    where
        T: 'static,
    {
        let Self {
            query,
            selected: _selected,
            results,
            placeholder,
            semantic_name,
            size,
            width,
            validation,
            disabled,
            id,
            open,
            highlight,
            on_change,
            on_select,
            on_clear,
            on_submit,
            on_blur,
            on_dismiss,
        } = self;
        let loading = matches!(&results, AutocompleteResults::Loading);
        let show_clear = !loading && !query.is_empty() && on_clear.is_some();
        let match_query = query.clone();
        let widget_query = query.to_string();
        let widget_results = results.clone();
        let handles = AutocompleteHandles::new();
        let input_focused = handles.input_focused();
        let highlighted = handles.highlighted_index();
        let ensure_pending = handles.ensure_pending();
        let local_closed = handles.local_closed();
        let ensure_visible = EnsureVisibleHandle::new();
        let on_select: Option<Rc<dyn Fn(T) -> Message + 'a>> = widget_results
            .has_unique_values()
            .then(|| on_select.map(Rc::from))
            .flatten();
        let mut input = Input::new(placeholder, query)
            .size(size)
            .validation(validation)
            .disabled(disabled)
            .track_focus(Rc::clone(&input_focused));
        if let Some(id) = id {
            input = input.id(id);
        }
        if let Some(name) = semantic_name {
            input = input.semantic_name(name);
        }
        if let Some(on_change) = on_change {
            input = input.on_change(on_change);
        }
        if let Some(message) = on_submit {
            input = input.on_submit(message);
        }
        if let Some(message) = on_blur {
            input = input.on_blur(message);
        }
        let mut anchor = InputGroup::new(input)
            .size(size)
            .width(width)
            .validation(validation)
            .disabled(disabled)
            .activity(loading);
        if show_clear {
            anchor = anchor.clear_action(
                button::icon(IconRole::WindowClose, "Clear")
                    .on_press(on_clear.expect("clear capability checked above")),
            );
        }

        let popover_dismiss = on_dismiss.clone();
        let callbacks = AutocompleteCallbacks::new(on_select.clone(), on_dismiss);
        let popover: Element<'a, Message> = Popover::new(anchor)
            .content(results_content(
                results,
                match_query,
                Rc::clone(&highlighted),
                Rc::clone(&ensure_pending),
                ensure_visible.clone(),
                Rc::clone(&local_closed),
                on_select.clone(),
            ))
            .open(open && !disabled)
            .placement(PopoverPlacement::BottomStart)
            .capped_anchor_width(360.0)
            .max_height(280.0)
            .collision(PopoverCollision::FlipAndShift)
            .inset(PopoverInset::EdgeToEdge)
            .on_dismiss_maybe(popover_dismiss)
            .ensure_visible(ensure_visible)
            .into();

        AutocompleteWidget::new(
            popover,
            widget_query,
            widget_results,
            open && !disabled,
            highlight,
            handles,
            callbacks,
        )
        .into()
    }
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

fn results_content<'a, T, Message>(
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
                        container(icon::role(role).custom_size(MENU_ICON_SIZE))
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
                                menu::style::row_style(
                                    false,
                                    highlighted.get() == Some(index),
                                    suggestion_disabled,
                                    MENU_ROW_RADIUS,
                                )(theme)
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

fn suggestion_label<'a, Message>(label: Cow<'a, str>, query: &str) -> Element<'a, Message>
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

fn first_grapheme_match(label: &str, query: &str) -> Option<Range<usize>> {
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

fn error_status_row<'a, Message>(message: impl Into<Cow<'a, str>>) -> Element<'a, Message>
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

fn status_row<'a, Message>(message: impl Into<Cow<'a, str>>, role: TextRole) -> Element<'a, Message>
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::WidgetHarness;
    use crate::widgets::controls::{choice_test_support::key_pressed, Field, FieldControl};
    use iced::{keyboard::key, mouse, window, Event, Point, Size};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Project(u8);

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Message {
        Query(String),
        Selected(Project),
        Clear,
        Submit,
        Blur,
        Dismiss,
    }

    fn suggestions() -> AutocompleteResults<'static, Project> {
        AutocompleteResults::suggestions(vec![
            AutocompleteSuggestion::new(Project(1), "Nive Core"),
            AutocompleteSuggestion::new(Project(2), String::from("Nive Runtime"))
                .leading(IconRole::EditFind)
                .trailing("Rust")
                .disabled(true),
        ])
    }

    #[test]
    fn suggestion_keeps_typed_value_and_owned_or_borrowed_presentation() {
        let suggestions = suggestions();
        let values = suggestions.as_suggestions().expect("suggestions");

        assert_eq!(values[0].value(), &Project(1));
        assert_eq!(values[0].label(), "Nive Core");
        assert_eq!(values[1].leading_icon(), Some(IconRole::EditFind));
        assert_eq!(values[1].trailing_text(), Some("Rust"));
        assert!(values[1].is_disabled());
    }

    #[test]
    fn results_represent_exactly_one_atomic_state() {
        assert_eq!(suggestions().as_suggestions().map(<[_]>::len), Some(2));
        assert!(AutocompleteResults::<Project>::Loading
            .as_suggestions()
            .is_none());
        assert!(matches!(
            AutocompleteResults::<Project>::empty("No projects"),
            AutocompleteResults::Empty(_)
        ));
        assert!(matches!(
            AutocompleteResults::<Project>::error("Could not load projects"),
            AutocompleteResults::Error(_)
        ));
        assert_eq!(
            AutocompleteHighlight::default(),
            AutocompleteHighlight::None
        );
    }

    #[test]
    fn duplicate_values_are_deterministically_diagnosable() {
        let duplicate = AutocompleteResults::suggestions(vec![
            AutocompleteSuggestion::new(Project(1), "First"),
            AutocompleteSuggestion::new(Project(1), "Duplicate"),
        ]);

        assert!(!duplicate.has_unique_values());
        assert!(suggestions().has_unique_values());
        assert!(AutocompleteResults::<Project>::Loading.has_unique_values());
    }

    #[test]
    fn contiguous_match_is_case_insensitive_first_and_grapheme_safe() {
        let label = "CAFÉ café";
        let matched = first_grapheme_match(label, "fé").expect("contiguous match");

        assert_eq!(&label[matched.clone()], "FÉ");
        assert!(label.is_char_boundary(matched.start));
        assert!(label.is_char_boundary(matched.end));

        let decomposed = "Cafe\u{301} Noir";
        let matched = first_grapheme_match(decomposed, "FE\u{301}").expect("combined grapheme");
        assert_eq!(&decomposed[matched], "fe\u{301}");
    }

    #[test]
    fn fuzzy_noncontiguous_and_empty_queries_do_not_claim_emphasis() {
        assert_eq!(first_grapheme_match("Nive Runtime", "NR"), None);
        assert_eq!(first_grapheme_match("Nive Runtime", ""), None);
    }

    #[test]
    fn emoji_match_never_slices_the_joined_grapheme() {
        let label = "Team 👩‍💻 tools";
        let matched = first_grapheme_match(label, "👩‍💻").expect("emoji grapheme");

        assert_eq!(&label[matched.clone()], "👩‍💻");
        assert!(label.is_char_boundary(matched.start));
        assert!(label.is_char_boundary(matched.end));
    }

    #[test]
    fn typed_api_retains_controlled_query_selection_results_and_callbacks() {
        let autocomplete = Autocomplete::new("niv", Some(Project(1)), suggestions())
            .placeholder("Search projects")
            .semantic_name("Project")
            .lg()
            .invalid(true)
            .open(true)
            .highlight(AutocompleteHighlight::First)
            .on_change(Message::Query)
            .on_select(Message::Selected)
            .on_clear(Message::Clear)
            .on_submit(Message::Submit)
            .on_blur(Message::Blur)
            .on_dismiss(Message::Dismiss);

        assert_eq!(autocomplete.query(), "niv");
        assert_eq!(autocomplete.selected(), Some(&Project(1)));
        assert_eq!(
            autocomplete.results().as_suggestions().map(<[_]>::len),
            Some(2)
        );
        assert_eq!(autocomplete.semantic_name_value(), Some("Project"));
        assert_eq!(autocomplete.control_size(), ControlSize::Lg);
        assert_eq!(autocomplete.field_validation(), FieldValidation::Invalid);
        assert_eq!(
            autocomplete.highlight_policy(),
            AutocompleteHighlight::First
        );
        assert!(autocomplete.is_open());
    }

    #[test]
    fn optional_callbacks_cover_the_complete_controlled_surface() {
        let autocomplete = Autocomplete::new("niv", None, suggestions())
            .on_change_maybe(Some(Message::Query as fn(String) -> Message))
            .on_select_maybe(Some(Message::Selected as fn(Project) -> Message))
            .on_clear_maybe(Some(Message::Clear))
            .on_submit_maybe(Some(Message::Submit))
            .on_blur_maybe(Some(Message::Blur))
            .on_dismiss_maybe(Some(Message::Dismiss));
        let _: Element<'_, Message> = autocomplete.into();
    }

    #[test]
    fn field_context_overrides_size_validation_and_semantic_name_and_keeps_disabled_monotonic() {
        let (autocomplete, id) = Autocomplete::<_, Message>::new("niv", None, suggestions())
            .semantic_name("Standalone search")
            .invalid(true)
            .disabled(true)
            .apply_field_context(
                Cow::Borrowed("Project"),
                ControlSize::Lg,
                FieldValidation::Valid,
                false,
            );

        assert_eq!(autocomplete.semantic_name_value(), Some("Project"));
        assert_eq!(autocomplete.control_size(), ControlSize::Lg);
        assert_eq!(autocomplete.field_validation(), FieldValidation::Valid);
        assert!(autocomplete.is_disabled());
        assert_eq!(autocomplete.widget_id(), Some(&id));
    }

    #[test]
    fn autocomplete_converts_through_the_opaque_field_control_boundary() {
        let autocomplete = Autocomplete::<_, Message>::new("niv", None, suggestions());
        let _: FieldControl<'_, Message> = autocomplete.into();

        let field: Element<'_, Message> = Field::new(
            "Project",
            Autocomplete::new("niv", None, suggestions()).on_change(Message::Query),
        )
        .into();
        let _ = WidgetHarness::new(field, Size::new(320.0, 120.0));
    }

    #[test]
    fn atomic_non_suggestion_states_render_without_invalidating_the_input() {
        for results in [
            AutocompleteResults::Loading,
            AutocompleteResults::empty("No projects"),
            AutocompleteResults::error("Offline"),
        ] {
            let autocomplete: Element<'_, Message> =
                Autocomplete::new("niv", None::<Project>, results)
                    .on_change(Message::Query)
                    .open(true)
                    .into();
            let mut harness = WidgetHarness::new(autocomplete, Size::new(320.0, 140.0));

            assert!(harness.has_overlay());
            assert_eq!(
                harness.bounds().height,
                theme::form_control_metrics(ControlSize::Sm).height
            );
        }
    }

    #[test]
    fn loading_clear_and_empty_reservation_keep_the_same_input_geometry() {
        fn bounds(results: AutocompleteResults<'static, Project>) -> iced::Rectangle {
            let autocomplete: Element<'_, Message> = Autocomplete::new("niv", None, results)
                .on_clear(Message::Clear)
                .shrink_width()
                .into();
            WidgetHarness::new(autocomplete, Size::new(500.0, 80.0)).bounds()
        }

        let clear = bounds(suggestions());
        let loading = bounds(AutocompleteResults::Loading);
        let empty = bounds(AutocompleteResults::empty("No projects"));

        assert_eq!(clear, loading);
        assert_eq!(loading, empty);
        assert_eq!(
            clear.height,
            theme::form_control_metrics(ControlSize::Sm).height
        );
    }

    #[test]
    fn clear_action_routes_once_and_disabled_precedence_makes_it_inert() {
        fn release_clear(disabled: bool) -> Vec<Message> {
            let autocomplete: Element<'_, Message> = Autocomplete::new("niv", None, suggestions())
                .on_clear(Message::Clear)
                .disabled(disabled)
                .width(Length::Fixed(240.0))
                .into();
            let mut harness = WidgetHarness::new(autocomplete, Size::new(320.0, 80.0));
            harness.set_cursor(Point::new(236.0, 14.0));
            assert!(harness
                .update(Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                )))
                .messages
                .is_empty());
            harness
                .update(Event::Mouse(mouse::Event::ButtonReleased(
                    mouse::Button::Left,
                )))
                .messages
        }

        assert_eq!(release_clear(false), vec![Message::Clear]);
        assert!(release_clear(true).is_empty());
    }

    #[test]
    fn popup_width_is_the_safe_minimum_of_input_and_compact_cap() {
        fn popup_width(input_width: f32, viewport_width: f32) -> f32 {
            let autocomplete: Element<'_, Message> = Autocomplete::new("niv", None, suggestions())
                .width(Length::Fixed(input_width))
                .open(true)
                .into();
            let mut harness = WidgetHarness::new(autocomplete, Size::new(viewport_width, 360.0));
            harness.overlay_bounds().expect("suggestion popup").width
        }

        assert_eq!(popup_width(240.0, 640.0), 240.0);
        assert_eq!(popup_width(480.0, 640.0), 360.0);
        assert_eq!(popup_width(480.0, 300.0), 284.0);
    }

    #[test]
    fn popup_height_caps_at_280_and_then_at_available_viewport_height() {
        fn tall_results() -> AutocompleteResults<'static, Project> {
            AutocompleteResults::suggestions(
                (0..40)
                    .map(|index| {
                        AutocompleteSuggestion::new(Project(index), format!("Project {index}"))
                    })
                    .collect::<Vec<_>>(),
            )
        }

        fn popup_height(viewport_height: f32) -> f32 {
            let autocomplete: Element<'_, Message> = Autocomplete::new("niv", None, tall_results())
                .width(Length::Fixed(240.0))
                .open(true)
                .into();
            let mut harness = WidgetHarness::new(autocomplete, Size::new(320.0, viewport_height));
            harness.overlay_bounds().expect("suggestion popup").height
        }

        assert_eq!(popup_height(640.0), 280.0);
        assert!(popup_height(120.0) <= 80.0);
    }

    #[test]
    fn arrow_navigation_keeps_native_input_focus_and_does_not_mutate_query() {
        let id = Id::new("autocomplete-query");
        let autocomplete: Element<'_, Message> = Autocomplete::new("niv", None, suggestions())
            .id(id.clone())
            .open(true)
            .on_change(Message::Query)
            .into();
        let mut harness = WidgetHarness::new(autocomplete, Size::new(320.0, 180.0));
        harness.focus(id.clone());

        let result = harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));

        assert!(result.messages.is_empty());
        assert_eq!(harness.focused_ids(), vec![id]);
        assert!(harness.has_overlay());
    }

    #[test]
    fn enter_selects_only_an_eligible_highlight_and_otherwise_reaches_input_submit() {
        fn press_enter(select: bool, highlight: AutocompleteHighlight) -> Vec<Message> {
            let id = Id::unique();
            let mut autocomplete = Autocomplete::new("niv", None, suggestions())
                .id(id.clone())
                .open(true)
                .highlight(highlight)
                .on_submit(Message::Submit);
            if select {
                autocomplete = autocomplete.on_select(Message::Selected);
            }
            let mut harness =
                WidgetHarness::new(Element::from(autocomplete), Size::new(320.0, 180.0));
            harness.focus(id);
            harness
                .update(key_pressed(key::Named::Enter, key::Code::Enter))
                .messages
        }

        assert_eq!(
            press_enter(true, AutocompleteHighlight::First),
            vec![Message::Selected(Project(1))]
        );
        assert_eq!(
            press_enter(true, AutocompleteHighlight::None),
            vec![Message::Submit]
        );
        assert_eq!(
            press_enter(false, AutocompleteHighlight::First),
            vec![Message::Submit]
        );
    }

    #[test]
    fn escape_dismisses_only_when_the_capability_exists() {
        fn press_escape(dismissible: bool) -> crate::test_support::UpdateResult<Message> {
            let mut autocomplete = Autocomplete::new("niv", None, suggestions()).open(true);
            if dismissible {
                autocomplete = autocomplete.on_dismiss(Message::Dismiss);
            }
            let mut harness =
                WidgetHarness::new(Element::from(autocomplete), Size::new(320.0, 180.0));
            harness
                .update_overlay(key_pressed(key::Named::Escape, key::Code::Escape))
                .expect("open Autocomplete overlay")
        }

        let dismissible = press_escape(true);
        assert_eq!(dismissible.messages, vec![Message::Dismiss]);
        assert!(dismissible.captured);

        let persistent = press_escape(false);
        assert!(persistent.messages.is_empty());
        assert!(!persistent.captured);
    }

    #[test]
    fn tab_never_selects_or_blocks_traversal_and_dismisses_only_with_capability() {
        fn press_tab(dismissible: bool) -> crate::test_support::UpdateResult<Message> {
            let id = Id::unique();
            let mut autocomplete = Autocomplete::new("niv", None, suggestions())
                .id(id.clone())
                .open(true)
                .highlight(AutocompleteHighlight::First)
                .on_select(Message::Selected);
            if dismissible {
                autocomplete = autocomplete.on_dismiss(Message::Dismiss);
            }
            let mut harness =
                WidgetHarness::new(Element::from(autocomplete), Size::new(320.0, 180.0));
            harness.focus(id);
            harness.update(key_pressed(key::Named::Tab, key::Code::Tab))
        }

        let dismissible = press_tab(true);
        assert_eq!(dismissible.messages, vec![Message::Dismiss]);
        assert!(!dismissible.captured);

        let persistent = press_tab(false);
        assert!(persistent.messages.is_empty());
        assert!(!persistent.captured);
    }

    #[test]
    fn tab_latch_survives_equal_rebuild_only_with_dismissal_capability() {
        fn element(id: Id, dismissible: bool) -> Element<'static, Message> {
            let autocomplete = Autocomplete::new("niv", None, suggestions())
                .id(id)
                .open(true)
                .highlight(AutocompleteHighlight::First)
                .on_select(Message::Selected);
            if dismissible {
                autocomplete.on_dismiss(Message::Dismiss).into()
            } else {
                autocomplete.into()
            }
        }

        fn run(dismissible: bool) -> (Vec<Message>, bool) {
            let id = Id::unique();
            let mut harness =
                WidgetHarness::new(element(id.clone(), dismissible), Size::new(320.0, 180.0));
            harness.focus(id.clone());
            let messages = harness
                .update(key_pressed(key::Named::Tab, key::Code::Tab))
                .messages;
            harness.replace(element(id, dismissible));
            (messages, harness.has_overlay())
        }

        assert_eq!(run(true), (vec![Message::Dismiss], false));
        assert_eq!(run(false), (Vec::new(), true));
    }

    #[test]
    fn suggestion_press_selects_once_before_blur_and_closes_the_local_surface() {
        let id = Id::unique();
        let autocomplete: Element<'_, Message> = Autocomplete::new("niv", None, suggestions())
            .id(id.clone())
            .open(true)
            .on_select(Message::Selected)
            .on_blur(Message::Blur)
            .on_dismiss(Message::Dismiss)
            .into();
        let mut harness = WidgetHarness::new(autocomplete, Size::new(320.0, 180.0));
        harness.focus(id.clone());
        let popup = harness.overlay_bounds().expect("suggestion popup");
        harness.set_cursor(Point::new(
            popup.x + MENU_LIST_INSET + 8.0,
            popup.y + MENU_LIST_INSET + MENU_ROW_HEIGHT / 2.0,
        ));

        let press = harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )))
            .expect("open Autocomplete overlay");

        assert_eq!(press.messages, vec![Message::Selected(Project(1))]);
        assert!(press.captured);
        assert_eq!(harness.focused_ids(), vec![id]);
        assert!(!harness.has_overlay());
    }

    #[test]
    fn suggestion_rows_remain_open_and_nonactivating_without_selection_capability() {
        let autocomplete: Element<'_, Message> = Autocomplete::new("niv", None, suggestions())
            .open(true)
            .into();
        let mut harness = WidgetHarness::new(autocomplete, Size::new(320.0, 180.0));
        let popup = harness.overlay_bounds().expect("suggestion popup");
        harness.set_cursor(Point::new(
            popup.x + MENU_LIST_INSET + 8.0,
            popup.y + MENU_LIST_INSET + MENU_ROW_HEIGHT / 2.0,
        ));

        let press = harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )))
            .expect("open Autocomplete overlay");

        assert!(press.messages.is_empty());
        assert!(harness.has_overlay());
    }

    #[test]
    fn duplicate_and_disabled_suggestions_are_pointer_inert() {
        fn press_row(
            results: AutocompleteResults<'static, Project>,
            row: usize,
        ) -> (Vec<Message>, bool) {
            let autocomplete: Element<'_, Message> = Autocomplete::new("niv", None, results)
                .open(true)
                .on_select(Message::Selected)
                .into();
            let mut harness = WidgetHarness::new(autocomplete, Size::new(320.0, 180.0));
            let popup = harness.overlay_bounds().expect("suggestion popup");
            harness.set_cursor(Point::new(
                popup.x + MENU_LIST_INSET + 8.0,
                popup.y + MENU_LIST_INSET + (row as f32 + 0.5) * MENU_ROW_HEIGHT,
            ));
            let messages = harness
                .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                )))
                .expect("open Autocomplete overlay")
                .messages;
            (messages, harness.has_overlay())
        }

        let duplicate = AutocompleteResults::suggestions(vec![
            AutocompleteSuggestion::new(Project(1), "First"),
            AutocompleteSuggestion::new(Project(1), "Duplicate"),
        ]);
        assert_eq!(press_row(duplicate, 0), (Vec::new(), true));
        assert_eq!(press_row(suggestions(), 1), (Vec::new(), true));
    }

    #[test]
    fn equal_rebuild_keeps_selection_latched_and_query_change_reopens() {
        fn element(id: Id, query: &'static str) -> Element<'static, Message> {
            Autocomplete::new(query, None, suggestions())
                .id(id)
                .open(true)
                .on_select(Message::Selected)
                .into()
        }

        let id = Id::unique();
        let mut harness = WidgetHarness::new(element(id.clone(), "niv"), Size::new(320.0, 180.0));
        harness.focus(id.clone());
        let popup = harness.overlay_bounds().expect("suggestion popup");
        harness.set_cursor(Point::new(
            popup.x + MENU_LIST_INSET + 8.0,
            popup.y + MENU_LIST_INSET + MENU_ROW_HEIGHT / 2.0,
        ));
        harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )))
            .expect("open Autocomplete overlay");

        harness.replace(element(id.clone(), "niv"));
        assert!(!harness.has_overlay());

        harness.replace(element(id, "nive"));
        assert!(harness.has_overlay());
    }

    #[test]
    fn escape_latch_survives_equal_open_and_resets_through_false_to_true() {
        fn element(open: bool) -> Element<'static, Message> {
            Autocomplete::new("niv", None, suggestions())
                .open(open)
                .on_dismiss(Message::Dismiss)
                .into()
        }

        let mut harness = WidgetHarness::new(element(true), Size::new(320.0, 180.0));
        let escape = harness
            .update_overlay(key_pressed(key::Named::Escape, key::Code::Escape))
            .expect("open Autocomplete overlay");
        assert_eq!(escape.messages, vec![Message::Dismiss]);

        harness.replace(element(true));
        assert!(!harness.has_overlay());

        harness.replace(element(false));
        assert!(!harness.has_overlay());
        harness.replace(element(true));
        assert!(harness.has_overlay());
    }

    #[test]
    fn outside_dismissal_latches_only_when_the_capability_exists() {
        fn element(dismissible: bool) -> Element<'static, Message> {
            let autocomplete = Autocomplete::new("niv", None, suggestions()).open(true);
            if dismissible {
                autocomplete.on_dismiss(Message::Dismiss).into()
            } else {
                autocomplete.into()
            }
        }

        fn run(dismissible: bool) -> (Vec<Message>, bool, bool) {
            let mut harness = WidgetHarness::new(element(dismissible), Size::new(320.0, 180.0));
            harness.set_cursor(Point::new(310.0, 170.0));
            let result = harness
                .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                )))
                .expect("open Autocomplete overlay");
            let captured = result.captured;
            let messages = result.messages;
            harness.replace(element(dismissible));
            (messages, captured, harness.has_overlay())
        }

        assert_eq!(run(true), (vec![Message::Dismiss], true, false));
        assert_eq!(run(false), (Vec::new(), false, true));
    }

    #[test]
    fn later_real_focus_entry_reopens_the_same_latched_session() {
        fn element(autocomplete_id: Id, next_id: Id) -> Element<'static, Message> {
            iced::widget::column![
                Autocomplete::new("niv", None, suggestions())
                    .id(autocomplete_id)
                    .open(true)
                    .on_select(Message::Selected),
                Input::new("Next", "").id(next_id).on_change(Message::Query),
            ]
            .into()
        }

        let autocomplete_id = Id::unique();
        let next_id = Id::unique();
        let mut harness = WidgetHarness::new(
            element(autocomplete_id.clone(), next_id.clone()),
            Size::new(320.0, 220.0),
        );
        harness.focus(autocomplete_id.clone());
        let popup = harness.overlay_bounds().expect("suggestion popup");
        harness.set_cursor(Point::new(
            popup.x + MENU_LIST_INSET + 8.0,
            popup.y + MENU_LIST_INSET + MENU_ROW_HEIGHT / 2.0,
        ));
        harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )))
            .expect("open Autocomplete overlay");
        harness.replace(element(autocomplete_id.clone(), next_id.clone()));
        assert!(!harness.has_overlay());

        harness.focus(next_id.clone());
        assert_eq!(harness.focused_ids(), vec![next_id]);
        harness.focus(autocomplete_id.clone());
        assert_eq!(harness.focused_ids(), vec![autocomplete_id]);
        assert!(harness.has_overlay());
    }

    #[test]
    fn real_focus_exit_dismisses_once_without_overriding_the_new_target() {
        let autocomplete_id = Id::unique();
        let next_id = Id::unique();
        let content: Element<'_, Message> = iced::widget::column![
            Autocomplete::new("niv", None, suggestions())
                .id(autocomplete_id.clone())
                .open(true)
                .on_dismiss(Message::Dismiss),
            Input::new("Next", "")
                .id(next_id.clone())
                .on_change(Message::Query),
        ]
        .into();
        let mut harness = WidgetHarness::new(content, Size::new(320.0, 220.0));
        harness.focus(autocomplete_id);
        assert!(harness.has_overlay());

        harness.focus(next_id.clone());
        assert_eq!(harness.focused_ids(), vec![next_id]);
        let first = harness.update(Event::Window(window::Event::RedrawRequested(
            iced::time::Instant::now(),
        )));
        assert_eq!(first.messages, vec![Message::Dismiss]);
        assert!(!harness.has_overlay());

        let second = harness.update(Event::Window(window::Event::RedrawRequested(
            iced::time::Instant::now(),
        )));
        assert!(second.messages.is_empty());
    }

    #[test]
    fn keyboard_highlight_is_ensured_visible_by_the_popover_scroll_owner() {
        let results = AutocompleteResults::suggestions(
            (0_u8..24)
                .map(|value| {
                    AutocompleteSuggestion::new(Project(value), format!("Project {value}"))
                })
                .collect::<Vec<_>>(),
        );
        let id = Id::new("long-autocomplete-query");
        let autocomplete: Element<'_, Message> = Autocomplete::new("project", None, results)
            .id(id.clone())
            .width(Length::Fixed(180.0))
            .open(true)
            .on_change(Message::Query)
            .into();
        let mut harness = WidgetHarness::new(autocomplete, Size::new(220.0, 90.0));
        harness.focus(id);
        for _ in 0..24 {
            harness.update(key_pressed(key::Named::ArrowDown, key::Code::ArrowDown));
        }

        harness
            .update_overlay(Event::Window(window::Event::RedrawRequested(
                iced::time::Instant::now(),
            )))
            .expect("open Autocomplete overlay");

        assert!(harness
            .overlay_scroll_offsets()
            .iter()
            .any(|offset| offset.y.abs() > f32::EPSILON));
    }

    #[test]
    #[should_panic(expected = "AutocompleteSuggestion requires a nonempty visible label")]
    fn suggestion_rejects_an_empty_visible_label() {
        let _ = AutocompleteSuggestion::new(Project(1), "  ");
    }
}
