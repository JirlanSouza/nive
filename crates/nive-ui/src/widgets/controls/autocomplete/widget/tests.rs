use super::*;
use crate::widgets::controls::{AutocompleteResults, AutocompleteSuggestion};

fn results(order: &[u8]) -> AutocompleteResults<'static, u8> {
    AutocompleteResults::suggestions(
        order
            .iter()
            .map(|value| AutocompleteSuggestion::new(*value, format!("Value {value}")))
            .collect::<Vec<_>>(),
    )
}

fn widget(results: AutocompleteResults<'static, u8>) -> AutocompleteWidget<'static, u8, ()> {
    widget_with_policy(results, AutocompleteHighlight::None)
}

fn widget_with_policy(
    results: AutocompleteResults<'static, u8>,
    policy: AutocompleteHighlight,
) -> AutocompleteWidget<'static, u8, ()> {
    AutocompleteWidget::new(
        iced::widget::Space::new().into(),
        "v".into(),
        results,
        true,
        policy,
        AutocompleteHandles::new(),
        AutocompleteCallbacks::new(None, None),
    )
}

#[test]
fn default_policy_starts_without_a_logical_highlight() {
    let autocomplete = widget(results(&[1, 2, 3]));
    let mut state = AutocompleteState::default();

    autocomplete.sync_state(&mut state, false);

    assert_eq!(state.highlighted, None);
    assert_eq!(autocomplete.handles.highlighted_index.get(), None);
}

#[test]
fn first_policy_skips_disabled_and_falls_back_after_highlight_removal() {
    let initial = AutocompleteResults::suggestions(vec![
        AutocompleteSuggestion::new(1_u8, "One").disabled(true),
        AutocompleteSuggestion::new(2_u8, "Two"),
        AutocompleteSuggestion::new(3_u8, "Three"),
    ]);
    let autocomplete = widget_with_policy(initial, AutocompleteHighlight::First);
    let mut state = AutocompleteState::default();
    autocomplete.sync_state(&mut state, false);
    assert_eq!(state.highlighted, Some(2));

    assert!(autocomplete.navigate(&mut state, Navigation::Next));
    assert_eq!(state.highlighted, Some(3));

    let changed = widget_with_policy(results(&[1, 2]), AutocompleteHighlight::First);
    changed.sync_state(&mut state, false);

    assert_eq!(state.highlighted, Some(1));
    assert_eq!(changed.handles.highlighted_index.get(), Some(0));
}

#[test]
fn result_reordering_preserves_highlight_by_typed_value() {
    let mut state = AutocompleteState::default();
    let first = widget(results(&[1, 2, 3]));
    first.sync_state(&mut state, false);
    assert!(first.navigate(&mut state, Navigation::Next));
    assert!(first.navigate(&mut state, Navigation::Next));
    assert_eq!(state.highlighted, Some(2));

    let reordered = widget(results(&[3, 2, 1]));
    reordered.sync_state(&mut state, false);

    assert_eq!(state.highlighted, Some(2));
    assert_eq!(reordered.handles.highlighted_index.get(), Some(1));
}

#[test]
fn navigation_is_bounded_and_skips_disabled_values() {
    let results = AutocompleteResults::suggestions(vec![
        AutocompleteSuggestion::new(1_u8, "One").disabled(true),
        AutocompleteSuggestion::new(2_u8, "Two"),
        AutocompleteSuggestion::new(3_u8, "Three"),
    ]);
    let autocomplete = widget(results);
    let mut state = AutocompleteState::default();
    autocomplete.sync_state(&mut state, false);

    autocomplete.navigate(&mut state, Navigation::Next);
    assert_eq!(state.highlighted, Some(2));
    autocomplete.navigate(&mut state, Navigation::Previous);
    assert_eq!(state.highlighted, Some(2));
    autocomplete.navigate(&mut state, Navigation::Next);
    autocomplete.navigate(&mut state, Navigation::Next);
    assert_eq!(state.highlighted, Some(3));

    let mut reverse_state = AutocompleteState::default();
    autocomplete.sync_state(&mut reverse_state, false);
    autocomplete.navigate(&mut reverse_state, Navigation::Previous);
    assert_eq!(reverse_state.highlighted, Some(3));
    autocomplete.navigate(&mut reverse_state, Navigation::Previous);
    assert_eq!(reverse_state.highlighted, Some(2));
    autocomplete.navigate(&mut reverse_state, Navigation::Previous);
    assert_eq!(reverse_state.highlighted, Some(2));
}

#[test]
fn dismissal_latch_survives_equal_rebuilds_and_resets_on_session_changes() {
    let mut state = AutocompleteState::default();
    let initial = widget(results(&[1, 2, 3]));
    initial.sync_state(&mut state, false);
    initial.sync_state(&mut state, true);
    assert!(state.latch.is_some());
    assert!(initial.handles.local_closed.get());

    let equal = widget(results(&[1, 2, 3]));
    equal.sync_state(&mut state, false);
    assert!(state.latch.is_some());
    assert!(equal.handles.local_closed.get());

    let changed_query: AutocompleteWidget<'static, u8, ()> = AutocompleteWidget::new(
        iced::widget::Space::new().into(),
        "changed".into(),
        results(&[1, 2, 3]),
        true,
        AutocompleteHighlight::None,
        AutocompleteHandles::new(),
        AutocompleteCallbacks::new(None, None),
    );
    changed_query.sync_state(&mut state, false);
    assert!(state.latch.is_none());
    assert!(!changed_query.handles.local_closed.get());

    changed_query.sync_state(&mut state, true);
    let refocused = widget(results(&[1, 2, 3]));
    refocused.handles.input_focused.set(true);
    refocused.sync_state(&mut state, false);
    assert!(state.latch.is_none());
    assert_eq!(state.focus_generation, 1);

    refocused.sync_state(&mut state, true);
    let changed_results = widget(results(&[4, 5]));
    changed_results.handles.input_focused.set(true);
    changed_results.sync_state(&mut state, false);
    assert!(state.latch.is_none());

    changed_results.sync_state(&mut state, true);
    let closed: AutocompleteWidget<'static, u8, ()> = AutocompleteWidget::new(
        iced::widget::Space::new().into(),
        "v".into(),
        results(&[1, 2, 3]),
        false,
        AutocompleteHighlight::None,
        AutocompleteHandles::new(),
        AutocompleteCallbacks::new(None, None),
    );
    closed.sync_state(&mut state, false);
    assert!(state.latch.is_none());
}
