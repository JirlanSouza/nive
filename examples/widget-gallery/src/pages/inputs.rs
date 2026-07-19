use nive::prelude::*;
use nive::ui::{
    theme::ControlSize,
    widgets::controls::button as nbutton,
    widgets::primitives::text as ntext,
};
use nive::widget::{column, Id};

use crate::app::{AutocompleteKind, Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid};

const PLANS: &[&str] = &["Free", "Pro", "Enterprise"];

pub(crate) fn programmatic_input_id() -> Id {
    Id::new("gallery-programmatic-input")
}

pub(crate) fn invalid_input_id() -> Id {
    Id::new("gallery-invalid-input")
}

pub(crate) fn select_focus_id() -> Id {
    Id::new("gallery-select-focus")
}

pub fn view(app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::Inputs,
        column![
            section("Fields and input states", text_inputs(app)),
            section("Grouped input controls", grouped_inputs(app)),
            section("Field groups", field_groups(app)),
            section("Choice controls", choices(app)),
            section("Select matrix", select_matrix(app)),
            section("Autocomplete matrix", autocomplete_matrix(app)),
            section("Color and path controls", color_path(app)),
            section("Control sizes", sizes(app)),
        ]
        .spacing(18),
    )
}

fn text_inputs(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell(
            "Empty",
            Field::new(
                "Empty value",
                Input::new("Enter a value", "").on_change(Message::InputSearchChanged),
            )
            .hint("Visible label independent from placeholder"),
        ),
        example_cell(
            "Filled",
            Field::new(
                "Name",
                Input::new("Name", &app.form.name).on_change(Message::NameChanged),
            )
                .hint("Editable field with helper text"),
        ),
        example_cell(
            "Invalid",
            Field::new(
                "Email",
                Input::new("Email", "ada.example.com").on_change(|_| Message::Noop),
            )
            .error("Email must contain @")
            .reserve_support_line(true),
        ),
        example_cell(
            "Invalid + focus target",
            column![
                Field::new(
                    "Focused invalid email",
                    Input::new("Email", "invalid.example.com")
                        .id(invalid_input_id())
                        .on_change(|_| Message::Noop),
                )
                .error("Email must contain @")
                .reserve_support_line(true),
                nbutton::tertiary("Focus invalid input")
                    .on_press(Message::FocusInvalidInput),
            ]
            .spacing(8),
        ),
        example_cell(
            "Secure",
            Field::new(
                "Password",
                Input::new("Password", &app.form.secret)
                    .secure(true)
                    .on_change(Message::SecretChanged),
            ),
        ),
        example_cell(
            "Explicit read-only",
            Field::new(
                "Account reference",
                Input::new("Reference", "ACC-1042").read_only(true),
            )
            .optional("Read only")
            .hint("Selection and copy remain available"),
        ),
        example_cell(
            "Callback-less read-only",
            Field::new(
                "Generated identifier",
                Input::new("Identifier", "generated-8f31"),
            )
            .optional("Read only")
            .hint("No change callback means read-only, not disabled"),
        ),
        example_cell(
            "Disabled",
            Field::new(
                "Provisioning key",
                Input::new("Disabled", "Unavailable").disabled(true),
            )
            .optional("Disabled")
            .hint("Disabled blocks focus and selection"),
        ),
        example_cell(
            "Long / restricted width",
            container(Field::new(
                "Long project reference that wraps within a finite field",
                Input::new(
                    "Long",
                    "A very long one-line value that keeps horizontal caret scrolling visible",
                )
                .on_change(Message::InputSearchChanged),
            )
            .hint("Long support content wraps without widening the field"))
            .width(190),
        ),
        example_cell(
            "Programmatic focus",
            column![
                Field::new(
                    "Reusable focus Id",
                    Input::new("Focus target", &app.form.search)
                        .id(programmatic_input_id())
                        .on_change(Message::InputSearchChanged),
                ),
                nbutton::secondary("Focus input").on_press(Message::FocusProgrammaticInput),
            ]
            .spacing(8),
        ),
        example_cell(
            "Required / reserved",
            Field::new(
                "Organization",
                Input::new("Organization", &app.form.name).on_change(Message::NameChanged),
            )
            .required("Required")
            .reserve_support_line(true),
        ),
        example_cell(
            "Whitespace error = absence",
            Field::new(
                "Normalized support",
                Input::new("Value", &app.form.name).on_change(Message::NameChanged),
            )
            .hint("Hint remains because whitespace is not an error")
            .error("   "),
        ),
    ])
}

fn grouped_inputs(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell(
            "Typed slots",
            InputGroup::new(
                Input::new("Search", &app.form.search).on_change(Message::InputSearchChanged),
            )
            .prefix("repo:")
            .semantic_icon(IconRole::EditFind)
            .unit("⌘K")
            .status(StatusIndicator::new(
                theme::roles::ToneRole::Success,
                "Ready",
            )),
        ),
        example_cell(
            "Ghost group",
            InputGroup::new(
                Input::new("Filter", &app.form.search).on_change(Message::InputSearchChanged),
            )
            .prefix("repo:")
            .trailing_action(
                nbutton::icon(IconRole::WindowClose, "Clear search")
                    .on_press(Message::InputSearchChanged(String::new())),
            )
            .ghost(),
        ),
        example_cell(
            "Named clear cell",
            InputGroup::new(
                Input::new("Search", &app.form.search).on_change(Message::InputSearchChanged),
            )
            .clear_action(
                nbutton::icon(IconRole::WindowClose, "Clear search")
                    .on_press(Message::InputSearchChanged(String::new())),
            )
            .activity(false),
        ),
        example_cell(
            "Stable activity cell",
            InputGroup::new(Input::new("Loading", "Resolving workspace"))
                .clear_action(nbutton::icon(IconRole::WindowClose, "Clear result"))
                .activity(true),
        ),
        example_cell(
            "Read-only + action",
            InputGroup::new(Input::new("Reference", "ACC-1042").read_only(true))
                .prefix("ID")
                .trailing_action(
                    nbutton::icon(IconRole::EditCopy, "Copy reference")
                        .on_press(Message::Noop),
                ),
        ),
        example_cell(
            "Invalid group",
            InputGroup::new(
                Input::new("Amount", "invalid").on_change(|_| Message::Noop),
            )
            .prefix("USD")
            .unit("monthly")
            .invalid(true),
        ),
        example_cell(
            "Disabled group",
            InputGroup::new(Input::new("Disabled", "No actions"))
                .semantic_icon(IconRole::Identity)
                .trailing_action(
                    nbutton::icon(IconRole::ViewReveal, "Reveal value").on_press(Message::Noop),
                )
                .disabled(true),
        ),
        example_cell(
            "Custom slot limitation",
            InputGroup::new(
                Input::new("Custom", &app.form.search).on_change(Message::InputSearchChanged),
            )
            .leading_slot(ntext::caption("caller-owned"))
            .trailing_slot(ntext::caption("rectangular")),
        ),
        example_cell(
            "Fixed allocation",
            InputGroup::new(
                Input::new("Fixed", &app.form.search).on_change(Message::InputSearchChanged),
            )
            .prefix("scope:")
            .unit("⌘K")
            .width(220),
        ),
        example_cell(
            "Shrink allocation",
            InputGroup::new(Input::new("Shrink", "42"))
                .prefix("USD")
                .unit("kg")
                .shrink_width(),
        ),
        example_cell(
            "Autocomplete",
            Autocomplete::new(
                &app.form.search,
                None::<String>,
                AutocompleteResults::suggestions(vec![
                    AutocompleteSuggestion::new(
                        "Open settings".to_owned(),
                        "Open settings",
                    )
                    .leading(IconRole::EditFind),
                    AutocompleteSuggestion::new(
                        "Refresh project".to_owned(),
                        "Refresh project",
                    )
                    .leading(IconRole::ViewRefresh),
                    AutocompleteSuggestion::new(
                        "Delete project".to_owned(),
                        "Delete project",
                    )
                    .leading(IconRole::EditDelete),
                ]),
            )
            .placeholder("Type to search commands")
            .open(!app.form.search.is_empty())
            .highlight(AutocompleteHighlight::First)
            .on_change(Message::InputSearchChanged)
            .on_select(Message::InputSearchChanged)
            .on_clear(Message::InputSearchChanged(String::new())),
        ),
    ])
}

fn field_groups(app: &WidgetGallery) -> Element<'_, Message> {
    let vertical = FieldGroup::new(
        "Vertical account details",
        [
            Field::new(
                "Name",
                Input::new("Name", &app.form.name).on_change(Message::NameChanged),
            )
            .required("Required")
            .hint("Local hint remains beside its field"),
            Field::new(
                "Email",
                Input::new("Email", "invalid.example.com").on_change(|_| Message::Noop),
            )
            .required("Required")
            .error("Local email error"),
        ],
    )
    .description("Description precedes the concise group error")
    .error("Review one field")
    .vertical();
    let wrapped = FieldGroup::new(
        "Responsive profile",
        [
            Field::new(
                "Name",
                Input::new("Name", &app.form.name).on_change(Message::NameChanged),
            ),
            Field::new(
                "Email",
                Input::new("Email", &app.form.email).on_change(Message::EmailChanged),
            ),
            Field::new("Reference", Input::new("Reference", "ACC-1042")),
        ],
    )
    .description("Wrap threshold: 220 logical pixels")
    .wrap(220.0);
    let disabled = FieldGroup::new(
        "Disabled group",
        [
            Field::new("Name", Input::new("Name", &app.form.name)),
            Field::new(
                "Amount",
                InputGroup::new(Input::new("Amount", "42")).prefix("USD"),
            ),
        ],
    )
    .lg()
    .disabled(true);

    variant_grid([
        example_cell("Vertical + errors", Card::new(vertical).outlined()),
        example_cell(
            "Wrap / exact threshold",
            Card::new(container(wrapped).width(452)).outlined(),
        ),
        example_cell("Lg disabled propagation", Card::new(disabled).outlined()),
        example_cell(
            "Custom Field escape",
            Field::custom(
                "Unsupported control",
                container(ntext::caption(
                    "Caller owns focus, state, sizing, semantics, and clipping",
                ))
                .width(Length::Fill),
            )
            .hint("Explicitly outside canonical typed guarantees"),
        ),
    ])
}

fn choices(app: &WidgetGallery) -> Element<'_, Message> {
    let binary_segment = if matches!(app.form.segment, "Preview" | "Code") {
        app.form.segment
    } else {
        "Preview"
    };

    variant_grid([
        example_cell(
            "Checkbox states",
            column![
                Checkbox::new("Controlled choice", app.form.checked)
                    .description("Indicator, label, and description share one target")
                    .on_toggle(Message::ToggleChecked),
                Checkbox::new("Unchecked display-only", CheckboxState::Unchecked),
                Checkbox::new("Mixed aggregate", CheckboxState::Mixed)
                    .on_toggle(Message::ToggleChecked),
                Checkbox::new(
                    "A long invalid choice label that wraps without moving the indicator away from its first line",
                    CheckboxState::Unchecked,
                )
                .error("This submitted choice requires review")
                .fill_width()
                .on_toggle(Message::ToggleChecked),
                Checkbox::new("Disabled mixed choice", CheckboxState::Mixed).disabled(true),
            ]
            .spacing(10),
        ),
        example_cell(
            "RadioGroup",
            column![
                RadioGroup::new(
                    "Deployment target",
                    app.form.radio,
                    [
                        RadioOption::new("none", "No automatic deployment")
                            .description("An ordinary visible clearing choice"),
                        RadioOption::new("preview", "Preview environment"),
                        RadioOption::new("production", "Production").disabled(true),
                    ],
                )
                .required("Required")
                .description("Choose one typed destination")
                .error_maybe(app.form.radio.is_none().then_some("Select a deployment target"))
                .layout(RadioGroupLayout::HorizontalWrap)
                .on_select(Message::SelectRadio)
                .fill_width(),
                RadioGroup::new(
                    "Optional vertical choice",
                    Some("none"),
                    [
                        RadioOption::new("none", "No preference"),
                        RadioOption::new(
                            "long",
                            "A deliberately long option that wraps as one complete row",
                        )
                        .description("The indicator remains attached to the first text line"),
                    ],
                )
                .optional("Optional")
                .on_select(Message::SelectRadio),
                RadioGroup::new(
                    "Disabled group",
                    Some("preview"),
                    [
                        RadioOption::new("preview", "Preview"),
                        RadioOption::new("production", "Production"),
                    ],
                )
                .disabled(true)
                .on_select(Message::SelectRadio),
            ]
            .spacing(12),
        ),
        example_cell(
            "Switch compositions",
            column![
                Switch::inline("Enable previews", app.form.enabled)
                    .on_toggle(Message::ToggleEnabled),
                Switch::setting("Synchronize automatically", app.form.enabled)
                    .description("Changes take effect immediately")
                    .on_toggle(Message::ToggleEnabled),
                Switch::inline("On display-only", true),
                Switch::setting("Disabled setting", true).disabled(true),
            ]
            .spacing(10),
        ),
        example_cell(
            "SegmentedControl",
            column![
                SegmentedControl::new(
                    "Two option intrinsic mode",
                    binary_segment,
                    [segment("Preview"), segment("Code")],
                )
                .on_select(Message::SelectSegment),
                SegmentedControl::new(
                    "Input preview mode",
                    app.form.segment,
                    [
                        segment("Preview"),
                        segment("Code"),
                        segment("Tests").icon(IconRole::ActionConfirm),
                    ],
                )
                .on_select(Message::SelectSegment)
                .fill_width(),
                SegmentedControl::new(
                    "Linked input preview mode",
                    app.form.segment,
                    [
                        segment("Preview"),
                        segment("Code"),
                        segment("Tests").icon(IconRole::ActionConfirm),
                    ],
                )
                .linked()
                .on_select(Message::SelectSegment)
                .fill_width(),
                SegmentedControl::new(
                    "Five constrained modes",
                    app.form.segment,
                    [
                        segment("Preview"),
                        segment("Code"),
                        segment("Tests").disabled(true),
                        segment("Long diagnostics"),
                        segment("Metadata"),
                    ],
                )
                .width(180),
                SegmentedControl::new(
                    "Disabled linked modes",
                    binary_segment,
                    [segment("Preview"), segment("Code")],
                )
                .linked()
                .disabled(true)
                .on_select(Message::SelectSegment),
            ]
            .spacing(10),
        ),
    ])
}

fn color_path(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell(
            "ColorInput",
            ColorInput::new(app.form.color)
                .tooltip("Pick accent color")
                .on_change(Message::ColorChanged),
        ),
        example_cell(
            "ColorPicker",
            column![
                ColorPicker::new(app.form.color).on_change(Message::ColorChanged),
                ntext::caption(format!(
                    "RgbHexColor: {}",
                    RgbHexColor::from_color(app.form.color)
                )),
            ]
            .spacing(8),
        ),
        example_cell(
            "PathInput",
            PathInput::new("Project path", &app.form.path)
                .semantic_name("Project path")
                .leading_icon(IconRole::Folder)
                .on_change(Message::PathChanged)
                .on_browse(Message::PickPath),
        ),
    ])
}

fn select_matrix(app: &WidgetGallery) -> Element<'_, Message> {
    let selected = Select::new(
        vec![
            SelectOption::new("free", "Free"),
            SelectOption::new("pro", "Pro"),
            SelectOption::new("enterprise", "Enterprise").disabled(true),
        ],
        app.form.selected_plan.map(|plan| match plan {
            "Free" => "free",
            "Pro" => "pro",
            "Enterprise" => "enterprise",
            _ => "missing",
        }),
    )
    .placeholder("Choose a plan")
    .id(select_focus_id())
    .on_select(|value| {
        Message::SelectPlan(match value {
            "free" => "Free",
            "pro" => "Pro",
            "enterprise" => "Enterprise",
            _ => "Free",
        })
    })
    .on_open(Message::Noop)
    .on_close(Message::Noop);

    let long_options = (1..=18)
        .map(|index| {
            SelectOption::new(
                index,
                format!("Environment {index:02} with a deliberately long visible label"),
            )
        })
        .collect::<Vec<_>>();

    variant_grid([
        example_cell(
            "Selected + focused + disabled option",
            column![
                Field::new("Plan", selected)
                    .hint("Open to inspect the selected check and skip disabled rows"),
                nbutton::tertiary("Focus Select").on_press(Message::FocusSelect),
            ]
            .spacing(8),
        ),
        example_cell(
            "Placeholder + lifecycle",
            Field::new(
                "Deployment target",
                Select::new(
                    vec![
                        SelectOption::new("preview", "Preview"),
                        SelectOption::new("production", "Production"),
                    ],
                    None,
                )
                .placeholder("Select a target")
                .on_select(Message::SelectRadio)
                .on_open(Message::Noop)
                .on_close(Message::Noop),
            )
            .required("Required"),
        ),
        example_cell(
            "Invalid Field integration",
            Field::new(
                "Region",
                Select::new(
                    vec![
                        SelectOption::new("us", "US East"),
                        SelectOption::new("eu", "EU West"),
                    ],
                    None,
                )
                .placeholder("Choose a region")
                .on_select(Message::SelectRadio),
            )
            .error("Select a region before continuing")
            .reserve_support_line(true),
        ),
        example_cell(
            "Callback-absent display-only",
            Field::new(
                "Current channel",
                Select::<_, Message>::new(
                    vec![
                        SelectOption::new("stable", "Stable"),
                        SelectOption::new("preview", "Preview"),
                    ],
                    Some("stable"),
                ),
            )
            .optional("Read only"),
        ),
        example_cell(
            "Disabled",
            Field::new(
                "Managed plan",
                Select::<_, Message>::new(
                    vec![SelectOption::new("enterprise", "Enterprise")],
                    Some("enterprise"),
                )
                .disabled(true)
                .on_select(Message::SelectRadio),
            )
            .optional("Policy managed"),
        ),
        example_cell(
            "Empty options",
            Field::new(
                "No environments",
                Select::<&str, Message>::new(Vec::new(), None)
                    .placeholder("No options available")
                    .on_select(Message::SelectRadio),
            )
            .hint("The popup model is empty without substituting another control"),
        ),
        example_cell(
            "Duplicate values",
            Field::new(
                "Ambiguous model",
                Select::new(
                    vec![
                        SelectOption::new("same", "First duplicate"),
                        SelectOption::new("same", "Second duplicate"),
                    ],
                    Some("same"),
                )
                .on_select(Message::SelectRadio),
            )
            .hint("Duplicate diagnostics preserve finite presentation"),
        ),
        example_cell(
            "Missing selected value",
            Field::new(
                "Stale selection",
                Select::new(
                    vec![
                        SelectOption::new("one", "One"),
                        SelectOption::new("two", "Two"),
                    ],
                    Some("missing"),
                )
                .on_select(Message::SelectRadio),
            )
            .hint("The app-owned value is not present in current options"),
        ),
        example_cell(
            "Long list + constrained width",
            container(Field::new(
                "Environment",
                Select::new(long_options, Some(12))
                    .on_select(|_| Message::Noop)
                    .on_open(Message::Noop)
                    .on_close(Message::Noop),
            ))
            .width(190),
        ),
    ])
}

fn autocomplete_matrix(app: &WidgetGallery) -> Element<'_, Message> {
    let callback_absent = Autocomplete::<String, Message>::new(
        &app.overlays.autocomplete_query,
        app.overlays.autocomplete_selected.clone(),
        AutocompleteResults::suggestions(vec![
            AutocompleteSuggestion::new("display-one".to_owned(), "Display-only result"),
            AutocompleteSuggestion::new("display-two".to_owned(), "Another result"),
        ]),
    )
    .placeholder("No callbacks installed")
    .open(app.overlays.active_autocomplete == Some(AutocompleteKind::CallbackAbsent));

    let duplicate = autocomplete_fixture(
        app,
        AutocompleteKind::Duplicate,
        AutocompleteResults::suggestions(vec![
            AutocompleteSuggestion::new("same".to_owned(), "First duplicate"),
            AutocompleteSuggestion::new("same".to_owned(), "Second duplicate"),
        ]),
        AutocompleteHighlight::First,
    );

    let disabled = autocomplete_fixture(
        app,
        AutocompleteKind::Disabled,
        AutocompleteResults::suggestions(command_suggestions()),
        AutocompleteHighlight::First,
    )
    .disabled(true);

    variant_grid([
        autocomplete_cell(
            "Suggestions · First · clear · long list",
            AutocompleteKind::SuggestionsFirst,
            autocomplete_fixture(
                app,
                AutocompleteKind::SuggestionsFirst,
                AutocompleteResults::suggestions(command_suggestions()),
                AutocompleteHighlight::First,
            ),
        ),
        autocomplete_cell(
            "Suggestions · None · Enter pass-through",
            AutocompleteKind::SuggestionsNone,
            autocomplete_fixture(
                app,
                AutocompleteKind::SuggestionsNone,
                AutocompleteResults::suggestions(vec![
                    AutocompleteSuggestion::new("nive-core".to_owned(), "Nive Core")
                        .leading(IconRole::Folder)
                        .trailing("Rust"),
                    AutocompleteSuggestion::new("nive-ui".to_owned(), "Nive UI")
                        .leading(IconRole::DialogInformation)
                        .trailing("Design system"),
                ]),
                AutocompleteHighlight::None,
            ),
        ),
        autocomplete_cell(
            "Loading · Spinner reservation",
            AutocompleteKind::Loading,
            autocomplete_fixture(
                app,
                AutocompleteKind::Loading,
                AutocompleteResults::Loading,
                AutocompleteHighlight::None,
            ),
        ),
        autocomplete_cell(
            "Empty results",
            AutocompleteKind::Empty,
            autocomplete_fixture(
                app,
                AutocompleteKind::Empty,
                AutocompleteResults::empty("No commands match this query"),
                AutocompleteHighlight::None,
            ),
        ),
        autocomplete_cell(
            "Retrieval error",
            AutocompleteKind::Error,
            autocomplete_fixture(
                app,
                AutocompleteKind::Error,
                AutocompleteResults::error("Could not retrieve remote commands"),
                AutocompleteHighlight::None,
            ),
        ),
        example_cell(
            "Field validation remains separate",
            column![
                nbutton::tertiary("Toggle validation fixture")
                    .on_press(Message::ToggleAutocomplete(AutocompleteKind::Validation)),
                Field::new(
                    "Required command",
                    autocomplete_fixture(
                        app,
                        AutocompleteKind::Validation,
                        AutocompleteResults::suggestions(command_suggestions()),
                        AutocompleteHighlight::First,
                    ),
                )
                .error("Choose an allowed command before submitting")
                .reserve_support_line(true),
            ]
            .spacing(8),
        ),
        autocomplete_cell(
            "Callback absence",
            AutocompleteKind::CallbackAbsent,
            callback_absent,
        ),
        autocomplete_cell("Duplicate values", AutocompleteKind::Duplicate, duplicate),
        autocomplete_cell("Disabled precedence", AutocompleteKind::Disabled, disabled),
    ])
}

fn autocomplete_fixture<'a>(
    app: &'a WidgetGallery,
    kind: AutocompleteKind,
    results: AutocompleteResults<'a, String>,
    highlight: AutocompleteHighlight,
) -> Autocomplete<'a, String, Message> {
    Autocomplete::new(
        &app.overlays.autocomplete_query,
        app.overlays.autocomplete_selected.clone(),
        results,
    )
    .placeholder("Search commands")
    .semantic_name("Command search")
    .open(app.overlays.active_autocomplete == Some(kind))
    .highlight(highlight)
    .on_change(Message::AutocompleteQueryChanged)
    .on_select(Message::AutocompleteSelected)
    .on_clear(Message::ClearAutocomplete)
    .on_submit(Message::Noop)
    .on_blur(Message::Noop)
    .on_dismiss(Message::CloseAutocomplete)
}

fn autocomplete_cell<'a>(
    label: &'a str,
    kind: AutocompleteKind,
    autocomplete: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    example_cell(
        label,
        column![
            nbutton::tertiary("Toggle result fixture")
                .on_press(Message::ToggleAutocomplete(kind)),
            autocomplete.into(),
        ]
        .spacing(8),
    )
}

fn command_suggestions() -> Vec<AutocompleteSuggestion<'static, String>> {
    let mut suggestions = vec![
        AutocompleteSuggestion::new("nive-core".to_owned(), "Nive Core")
            .leading(IconRole::Folder)
            .trailing("Rust"),
        AutocompleteSuggestion::new("café-telemetry".to_owned(), "Café telemetry")
            .leading(IconRole::DialogInformation)
            .trailing("Unicode"),
        AutocompleteSuggestion::new(
            "long-command".to_owned(),
            "A deliberately long command result that exercises stable columns and clipping",
        )
        .trailing("Remote"),
        AutocompleteSuggestion::new("disabled-command".to_owned(), "Disabled command")
            .leading(IconRole::EditDelete)
            .trailing("Unavailable")
            .disabled(true),
    ];
    suggestions.extend((1..=14).map(|index| {
        AutocompleteSuggestion::new(
            format!("project-{index:02}"),
            format!("Project command {index:02}"),
        )
        .trailing("Workspace")
    }));
    suggestions
}

fn sizes(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell("XS", size_stack(ControlSize::Xs, app)),
        example_cell("SM", size_stack(ControlSize::Sm, app)),
        example_cell("MD", size_stack(ControlSize::Md, app)),
        example_cell("LG", size_stack(ControlSize::Lg, app)),
    ])
}

fn size_stack(size: ControlSize, app: &WidgetGallery) -> Element<'_, Message> {
    let binary_segment = if matches!(app.form.segment, "Preview" | "Code") {
        app.form.segment
    } else {
        "Preview"
    };

    column![
        Field::new(
            "Input",
            Input::new("Input", &app.form.name).on_change(Message::NameChanged),
        )
        .size(size),
        Field::new(
            "InputGroup",
            InputGroup::new(
                Input::new("Amount", "42").on_change(|_| Message::Noop),
            )
            .prefix("USD")
            .unit("kg"),
        )
        .size(size),
        Checkbox::new("Checkbox", app.form.checked)
            .size(size)
            .on_toggle(Message::ToggleChecked),
        Switch::inline("Switch", app.form.enabled)
            .size(size)
            .on_toggle(Message::ToggleEnabled),
        RadioGroup::new(
            "Radio",
            app.form.radio,
            [
                RadioOption::new("none", "None"),
                RadioOption::new("preview", "Preview"),
            ],
        )
        .size(size)
        .on_select(Message::SelectRadio),
        SegmentedControl::new(
            "Sized mode",
            binary_segment,
            [segment("Preview"), segment("Code")],
        )
        .size(size)
        .on_select(Message::SelectSegment),
        Select::from_values(PLANS.to_vec(), app.form.selected_plan)
            .size(size)
            .on_select(Message::SelectPlan),
    ]
    .spacing(8)
    .into()
}

fn segment(label: &'static str) -> SegmentedOption<'static, &'static str> {
    SegmentedOption::new(label, label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_form_review_matrices_build_at_extreme_sizes() {
        let mut app = WidgetGallery::test_fixture();
        let _: Element<'_, Message> = text_inputs(&app);
        let _: Element<'_, Message> = grouped_inputs(&app);
        let _: Element<'_, Message> = field_groups(&app);
        let _: Element<'_, Message> = select_matrix(&app);
        let _: Element<'_, Message> = autocomplete_matrix(&app);
        let _: Element<'_, Message> = sizes(&app);
        let _: Element<'_, Message> = size_stack(ControlSize::Xs, &app);
        let _: Element<'_, Message> = size_stack(ControlSize::Lg, &app);

        for kind in [
            AutocompleteKind::SuggestionsFirst,
            AutocompleteKind::SuggestionsNone,
            AutocompleteKind::Loading,
            AutocompleteKind::Empty,
            AutocompleteKind::Error,
            AutocompleteKind::Validation,
            AutocompleteKind::CallbackAbsent,
            AutocompleteKind::Duplicate,
            AutocompleteKind::Disabled,
        ] {
            app.overlays.active_autocomplete = Some(kind);
            let _: Element<'_, Message> = autocomplete_matrix(&app);
        }
    }

    #[test]
    fn deterministic_focus_targets_are_distinct_and_stable() {
        assert_ne!(programmatic_input_id(), invalid_input_id());
        assert_ne!(programmatic_input_id(), select_focus_id());
        assert_ne!(invalid_input_id(), select_focus_id());
        assert_eq!(programmatic_input_id(), programmatic_input_id());
        assert_eq!(invalid_input_id(), invalid_input_id());
    }
}
