use std::{cell::Cell, rc::Rc};

use super::layout::FieldGrid;
use super::style::{error_style, hint_style, label_style, metrics, sanitize_minimum, style};
use super::*;
use crate::test_support::{named_probe, WidgetHarness};
use crate::theme::{self, SpaceStep, TextRole, ToneRole, TypographyRole};
use crate::theme::{Theme, ThemeBuilder, ThemeDensity, ThemeMode};
use crate::widgets::controls::{
    choice_test_support::key_pressed, AutocompleteResults, AutocompleteSuggestion, SelectOption,
};
use iced::{
    keyboard::key,
    mouse,
    widget::{container, Space},
    Event, Point, Size,
};

#[test]
fn style_uses_primary_text_color() {
    let theme = Theme::Dark;
    let style = style(&theme);

    assert_eq!(style.text_color, Some(theme.text(TextRole::Primary).color));
}

#[test]
fn hint_and_error_styles_use_app_theme() {
    let theme = Theme::Dark;

    assert_eq!(
        hint_style()(&theme).color,
        Some(theme.text(TextRole::Secondary).color)
    );
    assert_eq!(
        error_style()(&theme).color,
        Some(theme.tone(ToneRole::Danger).color)
    );
}

#[test]
fn label_style_uses_app_theme() {
    let theme = Theme::Dark;

    assert_eq!(
        label_style()(&theme).color,
        Some(theme.text(TextRole::Primary).color)
    );
}

#[test]
fn typed_boundary_retains_owned_label_and_private_control_kind() {
    let field = Field::<String>::new(
        String::from("Account name"),
        Input::new("Name", "Ada").on_change(|value| value),
    )
    .required(String::from("Required"));

    assert_eq!(field.label.as_ref(), "Account name");
    assert!(matches!(field.control.kind, FieldControlKind::Input(_)));
    assert!(matches!(
        field.requirement,
        Some(FieldRequirement::Required(ref value)) if value == "Required"
    ));
}

#[test]
fn custom_boundary_is_explicit_and_keeps_canonical_defaults() {
    let field = Field::<()>::custom("Custom", Space::new());

    assert!(matches!(field.control.kind, FieldControlKind::Custom(_)));
    assert_eq!(field.size, ControlSize::Sm);
    assert_eq!(field.width, Length::Fill);
}

#[test]
fn empty_and_whitespace_errors_normalize_to_absence() {
    assert_eq!(normalized_error(None), None);
    assert_eq!(normalized_error(Some(Cow::Borrowed(""))), None);
    assert_eq!(normalized_error(Some(Cow::Borrowed("  \n"))), None);
    assert_eq!(
        normalized_error(Some(Cow::Borrowed("Required"))).as_deref(),
        Some("Required")
    );
}

#[test]
fn reserved_support_adds_one_line_and_error_replaces_hint_without_double_height() {
    let plain: Element<'_, ()> = Field::new("Name", Input::new("Name", "Ada")).into();
    let reserved: Element<'_, ()> = Field::new("Name", Input::new("Name", "Ada"))
        .reserve_support_line(true)
        .into();
    let hint: Element<'_, ()> = Field::new("Name", Input::new("Name", "Ada"))
        .hint("Helpful")
        .reserve_support_line(true)
        .into();
    let error: Element<'_, ()> = Field::new("Name", Input::new("Name", "Ada"))
        .hint("Helpful")
        .error("Required")
        .reserve_support_line(true)
        .into();
    let size = Size::new(320.0, 200.0);
    let plain = WidgetHarness::new(plain, size).bounds().height;
    let reserved = WidgetHarness::new(reserved, size).bounds().height;
    let hint = WidgetHarness::new(hint, size).bounds().height;
    let error = WidgetHarness::new(error, size).bounds().height;

    assert!(reserved > plain);
    assert_eq!(hint, reserved);
    assert_eq!(error, hint);
}

#[test]
fn activating_an_already_focused_label_keeps_focus_without_blur() {
    let id = Id::new("name");
    let field: Element<'_, &'static str> = Field::new(
        "Name",
        Input::new("Name", "Ada")
            .id(id.clone())
            .on_change(|_| "change")
            .on_blur("blur"),
    )
    .into();
    let mut harness = WidgetHarness::new(field, Size::new(320.0, 120.0));
    harness.focus(id);
    harness.set_cursor(Point::new(4.0, 4.0));

    let result = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));

    assert!(result.messages.is_empty());
    assert_eq!(harness.focused_widgets(), 1);
}

#[test]
fn generated_label_target_keeps_the_input_anchor_across_view_rebuilds() {
    fn view() -> Element<'static, &'static str> {
        let content = iced::widget::column![
            Input::new("Search", "")
                .id(Id::new("sidebar-search"))
                .on_change(|_| "search"),
            Field::new(
                "Empty value",
                Input::new("Enter a value", "").on_change(|_| "changed"),
            )
            .probe_name("empty-field"),
            crate::widgets::button::primary("After")
                .id(Id::new("after-empty-field"))
                .on_press("after"),
        ]
        .spacing(12);

        crate::accessibility::FocusRoot::new(iced::widget::scrollable(content)).into()
    }

    let mut harness = WidgetHarness::new(view(), Size::new(400.0, 240.0));
    let field = harness.named_bounds("empty-field").expect("empty field");
    harness.set_cursor(Point::new(field.x + 20.0, field.y + field.height - 10.0));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert_eq!(
        harness
            .managed_focus()
            .entries
            .iter()
            .filter(|entry| entry.active)
            .count(),
        1
    );

    harness.set_cursor(Point::new(380.0, 220.0));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert_eq!(
        harness
            .managed_focus()
            .entries
            .iter()
            .filter(|entry| entry.anchor_only)
            .count(),
        1
    );

    harness.replace(view());
    assert_eq!(
        harness
            .managed_focus()
            .entries
            .iter()
            .filter(|entry| entry.anchor_only)
            .count(),
        1
    );

    harness.focus_next();
    assert_eq!(harness.focused_ids(), [Id::new("after-empty-field")]);
}

#[test]
fn enabled_label_uses_default_cursor_and_still_focuses_its_input() {
    let field: Element<'_, ()> = Field::new(
        "Name",
        Input::new("Name", "Ada")
            .id(Id::new("name"))
            .on_change(|_| ()),
    )
    .into();
    let mut harness = WidgetHarness::new(field, Size::new(320.0, 120.0));
    harness.set_cursor(Point::new(4.0, 4.0));

    assert_eq!(harness.mouse_interaction(), mouse::Interaction::Idle);

    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));

    assert_eq!(harness.focused_widgets(), 1);
}

#[test]
fn activating_a_sibling_label_blurs_once_and_moves_the_single_focus() {
    let first_id = Id::new("first");
    let second_id = Id::new("second");
    let first_visual_focus = Rc::new(Cell::new(false));
    let second_visual_focus = Rc::new(Cell::new(false));
    let fields: Element<'_, &'static str> = iced::widget::column![
        Field::new(
            "First",
            Input::new("First", "One")
                .id(first_id.clone())
                .track_focus(Rc::clone(&first_visual_focus))
                .on_change(|_| "first change")
                .on_blur("first blur"),
        ),
        named_probe(
            "second-field",
            Field::new(
                "Second",
                Input::new("Second", "Two")
                    .id(second_id)
                    .track_focus(Rc::clone(&second_visual_focus))
                    .on_change(|_| "second change")
                    .on_blur("second blur"),
            ),
        ),
    ]
    .spacing(12)
    .into();
    let mut harness = WidgetHarness::new(fields, Size::new(320.0, 240.0));
    harness.focus(first_id);
    let second = harness.named_bounds("second-field").expect("second field");
    harness.set_cursor(Point::new(second.x + 4.0, second.y + 4.0));

    let result = harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));

    assert_eq!(result.messages, vec!["first blur"]);
    assert!(!first_visual_focus.get());
    assert!(second_visual_focus.get());
}

#[test]
fn disabled_typed_field_label_does_not_focus_its_input() {
    let field: Element<'_, ()> = Field::new(
        "Name",
        Input::new("Name", "Ada")
            .id(Id::new("disabled-name"))
            .on_change(|_| ()),
    )
    .disabled(true)
    .into();
    let mut harness = WidgetHarness::new(field, Size::new(320.0, 120.0));
    harness.set_cursor(Point::new(4.0, 4.0));

    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));

    assert_eq!(harness.focused_widgets(), 0);
}

#[test]
#[should_panic(expected = "nonempty visible legend")]
fn field_group_rejects_an_empty_visible_legend() {
    let _ = FieldGroup::<()>::new("  ", Vec::new());
}

#[test]
fn field_group_api_retains_heading_layout_and_context() {
    let group = FieldGroup::<()>::new(
        String::from("Profile"),
        [Field::new("Name", Input::new("Name", "Ada")).error("Local error")],
    )
    .description(String::from("Account identity"))
    .error(String::from("Review this group"))
    .wrap(f32::NAN)
    .lg()
    .disabled(true);

    assert_eq!(group.legend.as_ref(), "Profile");
    assert_eq!(group.description.as_deref(), Some("Account identity"));
    assert_eq!(group.error.as_deref(), Some("Review this group"));
    assert_eq!(group.fields[0].error.as_deref(), Some("Local error"));
    assert_eq!(group.size, ControlSize::Lg);
    assert!(group.disabled);
    assert!(matches!(
        group.layout,
        FieldGroupLayout::Wrap { min_field_width } if min_field_width.is_nan()
    ));
    assert_eq!(sanitize_minimum(f32::NAN), 240.0);
    assert_eq!(sanitize_minimum(0.0), 240.0);
    assert_eq!(sanitize_minimum(180.0), 180.0);
}

#[test]
fn wrap_uses_the_exact_finite_column_formula_and_narrow_fallback() {
    fn field<'a>(name: &'static str) -> Field<'a, ()> {
        Field::custom(
            name,
            named_probe(
                name,
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fixed(12.0)),
            ),
        )
    }

    let group: Element<'_, ()> = FieldGroup::new("Profile", [field("first"), field("second")])
        .wrap(240.0)
        .into();
    let mut harness = WidgetHarness::new(group, Size::new(500.0, 240.0));
    let first = harness.named_bounds("first").expect("first");
    let second = harness.named_bounds("second").expect("second");

    assert_eq!(first.width, 244.0);
    assert_eq!(second.width, 244.0);
    assert_eq!(second.x - first.x, 256.0);

    harness.relayout(Size::new(200.0, 300.0));
    let first = harness.named_bounds("first").expect("narrow first");
    let second = harness.named_bounds("second").expect("narrow second");
    assert_eq!(first.width, 200.0);
    assert_eq!(second.width, 200.0);
    assert_eq!(second.x, first.x);
    assert!(second.y > first.y);
}

#[test]
fn typed_context_retains_label_metadata_and_propagates_size_validation_and_disabled() {
    let (input, _) = Input::<()>::new("Name", "Ada").apply_field_context(
        Cow::Owned(String::from("Account name")),
        ControlSize::Lg,
        FieldValidation::Invalid,
        true,
    );

    assert_eq!(input.semantic_name_value(), Some("Account name"));
    assert_eq!(input.control_size(), ControlSize::Lg);
    assert_eq!(input.field_validation(), FieldValidation::Invalid);
    assert!(input.is_disabled());

    let optional = Field::<()>::new("Reference", Input::new("Reference", "A-1"))
        .optional(String::from("Optional"));
    assert!(matches!(
        optional.requirement,
        Some(FieldRequirement::Optional(ref value)) if value == "Optional"
    ));
}

#[test]
fn multiline_support_grows_beyond_the_reserved_minimum() {
    let short: Element<'_, ()> = Field::new("Name", Input::new("Name", "Ada"))
        .error("Required")
        .reserve_support_line(true)
        .into();
    let long: Element<'_, ()> = Field::new("Name", Input::new("Name", "Ada"))
        .error("This long validation message must wrap by word or glyph inside a narrow field")
        .reserve_support_line(true)
        .into();
    let short = WidgetHarness::new(short, Size::new(150.0, 300.0));
    let long = WidgetHarness::new(long, Size::new(150.0, 300.0));

    assert!(long.bounds().height > short.bounds().height);
}

#[test]
fn custom_field_label_does_not_claim_or_create_a_focus_target() {
    let field: Element<'_, ()> = Field::custom("Custom", Space::new()).into();
    let mut harness = WidgetHarness::new(field, Size::new(200.0, 100.0));
    harness.set_cursor(Point::new(4.0, 4.0));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));

    assert_eq!(harness.focused_widgets(), 0);
}

#[test]
fn anatomy_gaps_follow_every_active_density_step() {
    for density in ThemeDensity::ALL {
        let theme = ThemeBuilder::new("Field density", ThemeMode::Light)
            .density(density)
            .build();
        let _guard = crate::theme::testing::ThemeTestGuard::activate(theme);
        let field: Element<'_, ()> = Field::custom(
            "Label",
            named_probe(
                "control",
                container(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fixed(10.0)),
            ),
        )
        .into();
        let mut harness = WidgetHarness::new(field, Size::new(240.0, 120.0));
        let control = harness.named_bounds("control").expect("control");
        let label_line = theme.typography(TypographyRole::BodyStrong);

        assert_eq!(
            control.y,
            label_line.size * label_line.line_height + theme.space(SpaceStep::Sm)
        );
        assert_eq!(metrics().requirement_gap, theme.space(SpaceStep::Xs));
        assert_eq!(metrics().control_to_support_gap, theme.space(SpaceStep::Xs));
    }
}

#[test]
fn field_group_wrap_crosses_every_exact_standard_threshold() {
    fn probed(name: &'static str) -> Field<'static, ()> {
        Field::custom(name, Space::new()).probe_name(name)
    }

    for (width, expected_columns) in [(491.0, 1), (492.0, 2), (743.0, 2), (744.0, 3)] {
        let group: Element<'_, ()> = FieldGroup::new(
            "Thresholds",
            [probed("one"), probed("two"), probed("three")],
        )
        .wrap(240.0)
        .into();
        let mut harness = WidgetHarness::new(group, Size::new(width, 400.0));
        let one = harness.named_bounds("one").expect("one");
        let two = harness.named_bounds("two").expect("two");
        let three = harness.named_bounds("three").expect("three");
        let first_row = [one, two, three]
            .into_iter()
            .filter(|bounds| bounds.y == one.y)
            .count();

        assert_eq!(first_row, expected_columns);
    }
}

#[test]
fn field_group_vertical_gap_tracks_size_and_density_without_own_surface() {
    for density in ThemeDensity::ALL {
        let theme = ThemeBuilder::new("Group density", ThemeMode::Light)
            .density(density)
            .build();
        let _guard = crate::theme::testing::ThemeTestGuard::activate(theme);

        for (size, expected_step) in [
            (ControlSize::Sm, SpaceStep::Lg),
            (ControlSize::Md, SpaceStep::Xl),
        ] {
            let group: Element<'_, ()> = FieldGroup::new(
                "Vertical",
                [
                    Field::custom("One", Space::new()).probe_name("one"),
                    Field::custom("Two", Space::new()).probe_name("two"),
                ],
            )
            .size(size)
            .into();
            let mut harness = WidgetHarness::new(group, Size::new(320.0, 300.0));
            let one = harness.named_bounds("one").expect("one");
            let two = harness.named_bounds("two").expect("two");

            assert_eq!(two.y - one.y - one.height, theme.space(expected_step));
            assert_eq!(one.x, 0.0);
            assert_eq!(one.width, 320.0);
        }
    }
}

#[test]
fn typed_input_group_receives_field_size_and_label_focus() {
    let id = Id::new("group-input");
    let field: Element<'_, ()> = Field::new(
        "Amount",
        InputGroup::new(Input::new("Amount", "42").id(id.clone()).on_change(|_| ())).prefix("USD"),
    )
    .lg()
    .into();
    let mut harness = WidgetHarness::new(field, Size::new(320.0, 120.0));
    let input = harness.focusable_bounds(&id).expect("typed group input");
    assert_eq!(
        input.height,
        theme::form_control_metrics(ControlSize::Lg).height
    );

    harness.set_cursor(Point::new(4.0, 4.0));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert_eq!(harness.focused_widgets(), 1);
}

#[test]
fn typed_select_receives_field_context_and_label_focus_before_erasure() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Message {
        Opened,
        Selected(u8),
    }

    let field: Element<'_, Message> = Field::new(
        "Billing plan",
        Select::new(vec![SelectOption::new(1_u8, "Professional")], Some(1))
            .on_select(Message::Selected)
            .on_open(Message::Opened),
    )
    .error("Choose a valid plan")
    .lg()
    .into();
    let mut harness = WidgetHarness::new(field, Size::new(320.0, 160.0));
    harness.set_cursor(Point::new(4.0, 4.0));

    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert_eq!(harness.focused_widgets(), 1);
    let opened = harness.update(key_pressed(key::Named::Enter, key::Code::Enter));

    assert_eq!(opened.messages, vec![Message::Opened]);
    assert!(harness.has_overlay());
}

#[test]
fn typed_autocomplete_receives_field_context_and_label_focus_before_erasure() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Message {
        Query(String),
    }

    let id = Id::new("project-autocomplete");
    let field: Element<'_, Message> = Field::new(
        "Project",
        Autocomplete::new(
            "niv",
            None,
            AutocompleteResults::suggestions(vec![AutocompleteSuggestion::new(1_u8, "Nive Core")]),
        )
        .id(id.clone())
        .on_change(Message::Query),
    )
    .error("Choose a valid project")
    .lg()
    .into();
    let mut harness = WidgetHarness::new(field, Size::new(320.0, 160.0));
    let input = harness
        .focusable_bounds(&id)
        .expect("autocomplete input focus target");

    assert_eq!(
        input.height,
        theme::form_control_metrics(ControlSize::Lg).height
    );
    harness.set_cursor(Point::new(4.0, 4.0));
    harness.update(Event::Mouse(mouse::Event::ButtonPressed(
        mouse::Button::Left,
    )));
    assert_eq!(harness.focused_widgets(), 1);
    assert_eq!(harness.focused_ids(), vec![id]);
}

#[test]
fn unbounded_wrap_falls_back_to_vertical_order() {
    let children = vec![
        named_probe(
            "first-unbounded",
            Space::new()
                .width(Length::Fixed(80.0))
                .height(Length::Fixed(10.0)),
        ),
        named_probe(
            "second-unbounded",
            Space::new()
                .width(Length::Fixed(100.0))
                .height(Length::Fixed(10.0)),
        ),
    ];
    let grid: Element<'_, ()> = FieldGrid::new(children, 12.0, Some(240.0)).into();
    let mut harness = WidgetHarness::new(grid, Size::INFINITE);
    let first = harness
        .named_bounds("first-unbounded")
        .expect("first unbounded");
    let second = harness
        .named_bounds("second-unbounded")
        .expect("second unbounded");

    assert_eq!(first.x, second.x);
    assert_eq!(second.y - first.y - first.height, 12.0);
}

#[test]
fn group_context_is_monotonic_and_preserves_local_error() {
    let field = Field::<()>::new("Name", Input::new("Name", "Ada"))
        .error("Local error")
        .disabled(true)
        .apply_group_context(ControlSize::Lg, false);

    assert_eq!(field.size, ControlSize::Lg);
    assert!(field.disabled);
    assert_eq!(field.error.as_deref(), Some("Local error"));
}
