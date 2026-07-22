use super::render::first_grapheme_match;
use super::*;
use crate::test_support::WidgetHarness;
use crate::theme;
use crate::widgets::controls::{choice_test_support::key_pressed, Field, FieldControl, Input};
use crate::widgets::navigation::menu::{MENU_LIST_INSET, MENU_ROW_HEIGHT};
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
        let autocomplete: Element<'_, Message> = Autocomplete::new("niv", None::<Project>, results)
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
        let mut harness = WidgetHarness::new(Element::from(autocomplete), Size::new(320.0, 180.0));
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
        let mut harness = WidgetHarness::new(Element::from(autocomplete), Size::new(320.0, 180.0));
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
        let mut harness = WidgetHarness::new(Element::from(autocomplete), Size::new(320.0, 180.0));
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
            .map(|value| AutocompleteSuggestion::new(Project(value), format!("Project {value}")))
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
