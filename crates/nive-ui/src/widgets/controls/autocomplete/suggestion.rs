use std::borrow::Cow;

use super::{AutocompleteResults, AutocompleteSuggestion};
use crate::widgets::primitives::IconRole;

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
