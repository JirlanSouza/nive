use nive_ui::prelude::*;

#[test]
fn prelude_exposes_complete_typed_form_contract() {
    let control: FieldControl<'_, String> = Input::new(String::from("Name"), String::from("Ada"))
        .semantic_name(String::from("Account name"))
        .on_change(|value| value)
        .into();
    let field = Field::new(String::from("Name"), control)
        .required(String::from("Required"))
        .hint(String::from("Use your public name"))
        .reserve_support_line(true)
        .lg();
    let grouped = Field::new(
        "Amount",
        InputGroup::new(Input::new("Amount", "42").read_only(true))
            .prefix(String::from("USD"))
            .unit(String::from("monthly"))
            .trailing_slot(text("custom")),
    )
    .optional(String::from("Optional"));
    let _: Element<'_, String> = FieldGroup::new(String::from("Profile"), [field, grouped])
        .description(String::from("Public account details"))
        .error(String::from("Review highlighted fields"))
        .layout(FieldGroupLayout::Wrap {
            min_field_width: 240.0,
        })
        .disabled(false)
        .fill_width()
        .into();
    let _: Element<'_, String> = Field::custom("Custom", text("escape hatch")).into();
    let _: Element<'_, String> =
        nive_ui::widgets::button::icon(IconRole::ValidationError, "Validation details")
            .on_press(String::from("open"))
            .into();
    let _: theme::FormControlMetrics = theme::active().form_control_metrics(theme::ControlSize::Lg);
}

#[test]
fn prelude_exposes_typed_select_and_field_conversion() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Plan(u8);

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    enum Message {
        Selected(Plan),
        Opened,
        Closed,
    }

    let owned = String::from("Professional");
    let options = vec![
        SelectOption::new(Plan(1), "Free"),
        SelectOption::new(Plan(2), owned).disabled(false),
    ];
    let control: FieldControl<'_, Message> = Select::new(options, Some(Plan(1)))
        .placeholder("Choose a plan")
        .semantic_name("Billing plan")
        .validation(FieldValidation::Invalid)
        .lg()
        .fill_width()
        .on_select(Message::Selected)
        .on_open_maybe(Some(Message::Opened))
        .on_close_maybe(Some(Message::Closed))
        .into();

    let _: Element<'_, Message> = Field::new("Plan", control).error("Required").into();
}

#[test]
fn prelude_exposes_typed_autocomplete_and_atomic_results() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Project(String);

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    enum Message {
        Query(String),
        Selected(Project),
        Clear,
        Submit,
        Blur,
        Dismiss,
    }

    let owned_label = String::from("Nive Runtime");
    let suggestions = AutocompleteResults::suggestions(vec![
        AutocompleteSuggestion::new(Project("core".into()), "Nive Core")
            .leading(IconRole::EditFind)
            .trailing("Rust"),
        AutocompleteSuggestion::new(Project("runtime".into()), owned_label).disabled(false),
    ]);
    let control: FieldControl<'_, Message> =
        Autocomplete::new("niv", Some(Project("core".into())), suggestions)
            .placeholder(String::from("Search projects"))
            .semantic_name("Project")
            .highlight(AutocompleteHighlight::First)
            .open(true)
            .on_change(Message::Query)
            .on_select(Message::Selected)
            .on_clear_maybe(Some(Message::Clear))
            .on_submit_maybe(Some(Message::Submit))
            .on_blur_maybe(Some(Message::Blur))
            .on_dismiss_maybe(Some(Message::Dismiss))
            .into();
    let _: Element<'_, Message> = Field::new("Project", control).into();

    let _: Element<'_, Message> =
        Autocomplete::new("", None::<Project>, AutocompleteResults::Loading).into();
    let _: Element<'_, Message> = Autocomplete::new(
        "",
        None::<Project>,
        AutocompleteResults::empty("No projects"),
    )
    .into();
    let _: Element<'_, Message> = Autocomplete::new(
        "",
        None::<Project>,
        AutocompleteResults::error(String::from("Offline")),
    )
    .into();
}

#[test]
fn prelude_exposes_complete_selection_contract() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Choice {
        None,
        First,
        Second,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Message {
        Checkbox(CheckboxState),
        Radio(Choice),
        Switch(bool),
        Segment(Choice),
    }

    let _: Element<'_, Message> = Checkbox::new(String::from("Owned"), CheckboxState::Mixed)
        .description("Borrowed description")
        .error(String::from("Review this choice"))
        .on_toggle(Message::Checkbox)
        .into();
    let _: Element<'_, Message> = Checkbox::new("Boolean convenience", true).into();
    let _: Element<'_, Message> = RadioGroup::new(
        String::from("Typed options"),
        Some(Choice::None),
        [
            RadioOption::new(Choice::None, "No preference"),
            RadioOption::new(Choice::First, String::from("First"))
                .description("Borrowed description"),
            RadioOption::new(Choice::Second, "Second").disabled(true),
        ],
    )
    .optional(String::from("Optional"))
    .layout(RadioGroupLayout::HorizontalWrap)
    .on_select(Message::Radio)
    .fill_width()
    .into();
    let _: Element<'_, Message> = Switch::inline(String::from("Immediate"), true)
        .on_toggle(Message::Switch)
        .into();
    let _: Element<'_, Message> = Switch::setting("Setting row", false)
        .description(String::from("Takes effect immediately"))
        .into();
    let _: Element<'_, Message> = SegmentedControl::new(
        String::from("Mode"),
        Choice::First,
        [
            SegmentedOption::new(Choice::First, "First"),
            SegmentedOption::new(Choice::Second, String::from("Second"))
                .icon(IconRole::ActionConfirm),
        ],
    )
    .linked()
    .on_select(Message::Segment)
    .fill_width()
    .into();
}
