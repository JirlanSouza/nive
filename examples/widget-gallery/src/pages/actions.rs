use nive::{
    prelude::*,
    ui::{
        theme::{ControlSize, SurfaceRole},
        widgets::{button as nbutton, text as ntext},
    },
    widget::column,
};

use crate::app::{Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid, variant_row};

pub fn view(app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::Actions,
        column![
            section("Button variants", buttons(app.control_size)),
            section("Action states", button_states()),
            section(
                "SegmentedControl",
                segmented_controls(app.control_size, app.form.segment)
            ),
            section("ActionGroup", action_groups(app.control_size)),
            section("Toolbar", toolbar(app.control_size)),
            section("DropdownMenu", dropdown_menu()),
            section("ActionCard", action_cards()),
        ]
        .spacing(18),
    )
}

fn buttons(size: ControlSize) -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Primary",
            nbutton::primary("Create")
                .size(size)
                .on_press(Message::Noop),
        ),
        example_cell(
            "Secondary",
            nbutton::secondary("Duplicate")
                .size(size)
                .on_press(Message::Noop),
        ),
        example_cell(
            "Outline",
            nbutton::outline("Inspect")
                .size(size)
                .on_press(Message::Noop),
        ),
        example_cell(
            "Ghost",
            nbutton::ghost("Preview").size(size).on_press(Message::Noop),
        ),
        example_cell(
            "Destructive",
            nbutton::destructive("Delete")
                .size(size)
                .on_press(Message::Noop),
        ),
        example_cell(
            "Link",
            nbutton::link("Open details")
                .size(size)
                .on_press(Message::Noop),
        ),
    ])
}

fn button_states() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Icon placement",
            variant_row([
                nbutton::primary("Back")
                    .leading_icon(IconName::ArrowLeft)
                    .on_press(Message::Noop)
                    .into(),
                nbutton::secondary("Next")
                    .trailing_icon(IconName::ArrowRight)
                    .on_press(Message::Noop)
                    .into(),
            ]),
        ),
        example_cell(
            "Icon-only",
            variant_row([
                nbutton::icon(IconName::Search)
                    .tooltip("Search")
                    .on_press(Message::Noop)
                    .into(),
                nbutton::icon(IconName::Settings)
                    .tooltip("Settings")
                    .on_press(Message::Noop)
                    .into(),
            ]),
        ),
        example_cell(
            "Disabled / loading",
            variant_row([
                nbutton::primary("Disabled")
                    .disabled(true)
                    .on_press(Message::Noop)
                    .into(),
                nbutton::secondary("Loading")
                    .loading(true)
                    .on_press(Message::Noop)
                    .into(),
            ]),
        ),
        example_cell(
            "Long label",
            nbutton::outline("Export selected records with a very long command label")
                .align_start()
                .width(Length::Fill)
                .on_press(Message::Noop),
        ),
        example_cell(
            "Narrow layout",
            container(
                nbutton::primary("A narrow action with wrapping pressure")
                    .width(140)
                    .on_press(Message::Noop),
            )
            .width(160),
        ),
    ])
}

fn segmented_controls(size: ControlSize, selected: &'static str) -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Default",
            SegmentedControl::new()
                .size(size)
                .item(segment("Preview", selected))
                .item(segment("Code", selected))
                .item(segment("Tests", selected).icon(IconName::Check))
                .fill(),
        ),
        example_cell(
            "Flat",
            SegmentedControl::new()
                .flat()
                .size(size)
                .item(segment("Preview", selected))
                .item(segment("Code", selected))
                .item(segment("Tests", selected).icon(IconName::Check))
                .fill(),
        ),
        example_cell(
            "Inline",
            variant_row([
                nbutton::secondary("Run")
                    .size(size)
                    .on_press(Message::Noop)
                    .into(),
                SegmentedControl::new()
                    .size(size)
                    .item(segment("Preview", selected))
                    .item(segment("Code", selected))
                    .item(segment("Tests", selected))
                    .into(),
                nbutton::icon(IconName::Settings)
                    .size(size)
                    .tooltip("Settings")
                    .on_press(Message::Noop)
                    .into(),
            ]),
        ),
    ])
}

fn action_groups(size: ControlSize) -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Inline actions",
            ActionGroup::new()
                .size(size)
                .action(
                    ToolbarAction::icon(IconName::ArrowLeft)
                        .tooltip("Back")
                        .on_press(Message::Noop),
                )
                .action(
                    ToolbarAction::icon(IconName::ArrowRight)
                        .tooltip("Forward")
                        .on_press(Message::Noop),
                )
                .separator()
                .action(ToolbarAction::icon_label(IconName::RefreshCw, "Refresh").loading(true)),
        ),
        example_cell(
            "Selectable actions",
            ActionGroup::new()
                .size(size)
                .action(
                    ToolbarAction::label("Preview")
                        .selected(true)
                        .on_press(Message::Noop),
                )
                .action(ToolbarAction::label("Code").on_press(Message::Noop))
                .action(
                    ToolbarAction::icon(IconName::Trash)
                        .disabled(true)
                        .tooltip("Delete"),
                ),
        ),
        example_cell(
            "Inline with button",
            variant_row([
                nbutton::secondary("Run")
                    .size(size)
                    .on_press(Message::Noop)
                    .into(),
                ActionGroup::new()
                    .size(size)
                    .action(
                        ToolbarAction::icon(IconName::ArrowLeft)
                            .tooltip("Back")
                            .on_press(Message::Noop),
                    )
                    .action(
                        ToolbarAction::icon(IconName::ArrowRight)
                            .tooltip("Forward")
                            .on_press(Message::Noop),
                    )
                    .into(),
                nbutton::icon(IconName::Settings)
                    .size(size)
                    .tooltip("Settings")
                    .on_press(Message::Noop)
                    .into(),
            ]),
        ),
    ])
}

fn toolbar(size: ControlSize) -> Element<'static, Message> {
    Panel::new(
        column![
            Toolbar::new()
                .size(size)
                .group(
                    ToolbarGroup::new()
                        .action(
                            ToolbarAction::icon(IconName::ArrowLeft)
                                .tooltip("Back")
                                .on_press(Message::Noop),
                        )
                        .action(
                            ToolbarAction::icon(IconName::ArrowRight)
                                .tooltip("Forward")
                                .on_press(Message::Noop),
                        )
                        .separator()
                        .action(
                            ToolbarAction::icon_label(IconName::RefreshCw, "Refresh").loading(true)
                        ),
                )
                .group(
                    ToolbarGroup::new()
                        .action(
                            ToolbarAction::label("Preview")
                                .selected(true)
                                .on_press(Message::Noop),
                        )
                        .action(
                            ToolbarAction::icon(IconName::Trash)
                                .disabled(true)
                                .tooltip("Delete"),
                        ),
                )
                .fill(),
            container(ntext::body_small("Selected project activity")).padding(16)
        ]
        .spacing(0),
    )
    .role(SurfaceRole::Panel)
    .padding(0)
    .into()
}

fn segment(label: &'static str, selected: &'static str) -> SegmentedItem<'static, Message> {
    SegmentedItem::new(label)
        .selected(label == selected)
        .on_press(Message::SelectSegment(label))
}

fn dropdown_menu() -> Element<'static, Message> {
    view_menu_only()
}

pub fn view_menu_only() -> Element<'static, Message> {
    DropdownMenu::new()
        .item(
            DropdownMenuItem::new("Rename")
                .icon(IconName::Edit)
                .trailing("Enter")
                .on_press(Message::Noop),
        )
        .item(
            DropdownMenuItem::new("Copy link")
                .icon(IconName::Copy)
                .trailing("Cmd+C")
                .selected(true)
                .on_press(Message::Noop),
        )
        .separator()
        .item(DropdownMenuItem::new("Disabled command").disabled(true))
        .item(
            DropdownMenuItem::new("Delete")
                .icon(IconName::Trash)
                .destructive(true)
                .on_press(Message::Noop),
        )
        .width(260)
        .into()
}

fn action_cards() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Default",
            ActionCard::new(
                column![
                    ntext::label_strong("Run import"),
                    ntext::body_small("Starts a background operation")
                ]
                .spacing(4),
            )
            .on_press(Message::Noop),
        ),
        example_cell(
            "Elevated",
            ActionCard::new(
                column![
                    Icon::new(IconName::Plus).lg(),
                    ntext::label_strong("New project")
                ]
                .spacing(8),
            )
            .role(SurfaceRole::Elevated)
            .on_press(Message::Noop),
        ),
        example_cell(
            "Disabled",
            ActionCard::new(ntext::body("Unavailable until setup completes")).disabled(true),
        ),
    ])
}
