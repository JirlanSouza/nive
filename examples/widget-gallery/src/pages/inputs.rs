use nive::prelude::*;
use nive::ui::{
    theme::{ControlSize, SurfaceRole},
    widgets::controls::button as nbutton,
    widgets::primitives::text as ntext,
};
use nive::widget::{column, Id};

use crate::app::{Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid};

const PLANS: &[&str] = &["Free", "Pro", "Enterprise"];

pub(crate) fn programmatic_input_id() -> Id {
    Id::new("gallery-programmatic-input")
}

pub(crate) fn invalid_input_id() -> Id {
    Id::new("gallery-invalid-input")
}

pub fn view(app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::Inputs,
        column![
            section("Fields and input states", text_inputs(app)),
            section("Grouped input controls", grouped_inputs(app)),
            section("Field groups", field_groups(app)),
            section("Choice controls", choices(app)),
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
                Input::new("Type to search commands", &app.form.search)
                    .on_change(Message::InputSearchChanged),
            )
            .open(!app.form.search.is_empty())
            .suggestions(vec![
                "Open settings".to_owned(),
                "Refresh project".to_owned(),
                "Delete project".to_owned(),
            ])
            .on_select(Message::InputSearchChanged)
            .content_with(|highlighted| suggestions(highlighted)),
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

fn suggestions(highlighted: Option<usize>) -> Element<'static, Message> {
    let labels = ["Open settings", "Refresh project", "Delete project"];
    let mut rows = column![].spacing(2).padding(6).width(Length::Fill);

    for (index, label) in labels.iter().enumerate() {
        rows = rows.push(
            SelectableItem::new(label)
                .selected(highlighted == Some(index))
                .leading_icon(if index == 2 {
                    IconRole::EditDelete
                } else {
                    IconRole::EditFind
                })
                .on_press(Message::InputSearchChanged((*label).to_owned())),
        );
    }

    Panel::new(rows)
        .role(SurfaceRole::Popover)
        .width(260)
        .into()
}

fn choices(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell(
            "Checkbox / Switch",
            column![
                Checkbox::new("Checked option", app.form.checked).on_toggle(Message::ToggleChecked),
                Checkbox::new("Disabled option", true).disabled(true),
                Switch::new(app.form.enabled)
                    .label("Enable previews")
                    .on_toggle(Message::ToggleEnabled),
                Switch::new(false).label("Disabled switch").disabled(true),
            ]
            .spacing(10),
        ),
        example_cell(
            "Select",
            Select::new(PLANS.to_vec(), app.form.selected_plan)
                .placeholder("Plan")
                .on_select(Message::SelectPlan)
                .fill_width(),
        ),
        example_cell(
            "SegmentedControl",
            SegmentedControl::new()
                .item(segment("Preview", app.form.segment))
                .item(segment("Code", app.form.segment))
                .item(segment("Tests", app.form.segment).icon(IconRole::ActionConfirm))
                .fill_width(),
        ),
        example_cell(
            "SegmentedControl flat",
            SegmentedControl::new()
                .flat()
                .item(segment("Preview", app.form.segment))
                .item(segment("Code", app.form.segment))
                .item(segment("Tests", app.form.segment).icon(IconRole::ActionConfirm))
                .fill_width(),
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

fn sizes(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell("XS", size_stack(ControlSize::Xs, app)),
        example_cell("SM", size_stack(ControlSize::Sm, app)),
        example_cell("MD", size_stack(ControlSize::Md, app)),
        example_cell("LG", size_stack(ControlSize::Lg, app)),
    ])
}

fn size_stack(size: ControlSize, app: &WidgetGallery) -> Element<'_, Message> {
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
        Switch::new(app.form.enabled)
            .size(size)
            .label("Switch")
            .on_toggle(Message::ToggleEnabled),
        Select::new(PLANS.to_vec(), app.form.selected_plan)
            .size(size)
            .on_select(Message::SelectPlan),
    ]
    .spacing(8)
    .into()
}

fn segment(label: &'static str, selected: &'static str) -> SegmentedItem<'static, Message> {
    SegmentedItem::new(label)
        .selected(label == selected)
        .on_press(Message::SelectSegment(label))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_form_review_matrices_build_at_extreme_sizes() {
        let app = WidgetGallery::test_fixture();
        let _: Element<'_, Message> = text_inputs(&app);
        let _: Element<'_, Message> = grouped_inputs(&app);
        let _: Element<'_, Message> = field_groups(&app);
        let _: Element<'_, Message> = sizes(&app);
        let _: Element<'_, Message> = size_stack(ControlSize::Xs, &app);
        let _: Element<'_, Message> = size_stack(ControlSize::Lg, &app);
    }

    #[test]
    fn deterministic_focus_targets_are_distinct_and_stable() {
        assert_ne!(programmatic_input_id(), invalid_input_id());
        assert_eq!(programmatic_input_id(), programmatic_input_id());
        assert_eq!(invalid_input_id(), invalid_input_id());
    }
}
