use nive::prelude::*;
use nive::ui::{
    theme::{ControlSize, SurfaceRole},
    widgets::{button as nbutton, text as ntext},
};
use nive::widget::column;

use crate::app::{Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid};

const PLANS: &[&str] = &["Free", "Pro", "Enterprise"];

pub fn view(app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::Inputs,
        column![
            section("Fields and input states", text_inputs(app)),
            section("Grouped input controls", grouped_inputs(app)),
            section("Choice controls", choices(app)),
            section("Color and path controls", color_path(app)),
            section("Control sizes", sizes(app)),
        ]
        .spacing(18),
    )
}

fn text_inputs(app: &WidgetGallery) -> Element<'_, Message> {
    let email_invalid = !app.form.email.is_empty() && !app.form.email.contains('@');

    variant_grid([
        example_cell(
            "Filled",
            Field::new(Input::new("Name", &app.form.name).on_input(Message::NameChanged))
                .label("Name")
                .hint("Editable field with helper text"),
        ),
        example_cell(
            "Invalid",
            Field::new(
                Input::new("Email", &app.form.email)
                    .on_input(Message::EmailChanged)
                    .invalid(email_invalid),
            )
            .label("Email")
            .error(if email_invalid { "Email must contain @" } else { "" }),
        ),
        example_cell(
            "Secure",
            Field::new(
                Input::new("Password", &app.form.secret)
                    .secure(true)
                    .on_input(Message::SecretChanged),
            )
            .label("Password"),
        ),
        example_cell(
            "Placeholder-only",
            Input::new("Empty placeholder", "").on_input(Message::InputSearchChanged),
        ),
        example_cell(
            "Disabled",
            Input::new("Disabled", "Read only value").disabled(true),
        ),
        example_cell(
            "Long value",
            Input::new(
                "Long",
                "A very long value that keeps text clipping and scroll behavior visible inside the input",
            )
            .on_input(Message::InputSearchChanged),
        ),
        example_cell(
            "Field parts",
            FieldGroup::new(
                column![
                    FieldLabel::new("Standalone label"),
                    Input::new("Grouped input", &app.form.name).on_input(Message::NameChanged),
                    FieldHint::new("FieldHint inside FieldGroup"),
                    FieldError::new("FieldError baseline"),
                ]
                .spacing(6),
            ),
        ),
    ])
}

fn grouped_inputs(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell(
            "InputGroup",
            InputGroup::new(
                Input::new("Search", &app.form.search).on_input(Message::InputSearchChanged),
            )
            .leading_icon(IconName::Search)
            .trailing_text("⌘K"),
        ),
        example_cell(
            "Ghost group",
            InputGroup::new(
                Input::new("Filter", &app.form.search).on_input(Message::InputSearchChanged),
            )
            .leading_text("repo:")
            .trailing_action(
                nbutton::icon(IconName::Close).on_press(Message::InputSearchChanged(String::new())),
            )
            .ghost(),
        ),
        example_cell(
            "Autocomplete",
            Autocomplete::new(
                Input::new("Type to search commands", &app.form.search)
                    .on_input(Message::InputSearchChanged),
            )
            .open(!app.form.search.is_empty())
            .item_count(3)
            .on_select(|index| match index {
                0 => Message::InputSearchChanged("Open settings".to_owned()),
                1 => Message::InputSearchChanged("Refresh project".to_owned()),
                _ => Message::InputSearchChanged("Delete project".to_owned()),
            })
            .content_with(|highlighted| suggestions(highlighted)),
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
                    IconName::Trash
                } else {
                    IconName::Search
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
                .fill(),
        ),
        example_cell(
            "SegmentedControl",
            SegmentedControl::new()
                .item(segment("Preview", app.form.segment))
                .item(segment("Code", app.form.segment))
                .item(segment("Tests", app.form.segment).icon(IconName::Check))
                .fill(),
        ),
        example_cell(
            "SegmentedControl flat",
            SegmentedControl::new()
                .flat()
                .item(segment("Preview", app.form.segment))
                .item(segment("Code", app.form.segment))
                .item(segment("Tests", app.form.segment).icon(IconName::Check))
                .fill(),
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
                .leading_icon(IconName::Folder)
                .on_input(Message::PathChanged)
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
        Input::new("Input", &app.form.name)
            .size(size)
            .on_input(Message::NameChanged),
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
