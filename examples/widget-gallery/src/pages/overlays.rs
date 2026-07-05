use std::borrow::Cow;

use nive::prelude::*;
use nive::ui::theme::SurfaceRole;
use nive::ui::widgets::controls::button as nbutton;
use nive::ui::widgets::primitives::text as ntext;
use nive::widget::column;

use crate::app::{DialogKind, Message, PopoverKind, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid};

static COMMAND_ROWS: &[CommandPaletteRow<'static, Message>] = &[
    CommandPaletteRow {
        id: "open-settings",
        label: "Open settings",
        description: Some("Navigate to app settings"),
        shortcut_label: Some(Cow::Borrowed("Cmd+,")),
        enabled: true,
        message: Some(Message::SelectCommand("open-settings")),
    },
    CommandPaletteRow {
        id: "refresh",
        label: "Refresh project",
        description: Some("Reload the active project"),
        shortcut_label: Some(Cow::Borrowed("Cmd+R")),
        enabled: true,
        message: Some(Message::SelectCommand("refresh")),
    },
    CommandPaletteRow {
        id: "delete",
        label: "Delete project",
        description: Some("Disabled destructive command"),
        shortcut_label: Some(Cow::Borrowed("Del")),
        enabled: false,
        message: None,
    },
];

pub fn view(app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::Overlays,
        column![
            section("Dialogs", dialog_triggers()),
            section("Popovers and tooltip", popovers(app)),
            section("Dropdown and command palette", command_palette(app)),
        ]
        .spacing(18),
    )
}

pub fn dialog(kind: DialogKind) -> Element<'static, Message> {
    let header = match kind {
        DialogKind::Basic => DialogHeader::new("Confirm action").description(
            "This dialog uses Dialog, DialogHeader, DialogFooter, and DialogActionFooter.",
        ),
        DialogKind::Destructive => DialogHeader::new("Delete project")
            .description("This destructive confirmation keeps current dialog styling visible."),
        DialogKind::LongContent => DialogHeader::new("Long content").description(
            "Scrollable text pressures the dialog content area without custom styling.",
        ),
    };

    let body: Element<'static, Message> = match kind {
        DialogKind::LongContent => scrollable(column![
            ntext::body("The gallery should expose long content behavior in real dialog surfaces."),
            ntext::body("Additional copy keeps height pressure visible while the app-owned dialog remains dismissible."),
            ntext::body("No wrapper is correcting spacing, radius, or token behavior here."),
        ]
        .spacing(12))
        .height(180)
        .into(),
        _ => ntext::body("Review the current dialog baseline before accepting the action.").into(),
    };

    let footer = DialogActionFooter::new(ntext::caption("Esc or backdrop dismisses the dialog"))
        .action(nbutton::secondary("Cancel").on_press(Message::CloseDialog))
        .action(match kind {
            DialogKind::Destructive => {
                nbutton::destructive("Delete").on_press(Message::CloseDialog)
            }
            _ => nbutton::primary("Confirm").on_press(Message::CloseDialog),
        });

    Dialog::new(column![header, body, DialogFooter::new(footer)].spacing(16)).into()
}

fn dialog_triggers() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Basic",
            nbutton::primary("Open basic dialog").on_press(Message::ShowDialog(DialogKind::Basic)),
        ),
        example_cell(
            "Destructive",
            nbutton::destructive("Open destructive dialog")
                .on_press(Message::ShowDialog(DialogKind::Destructive)),
        ),
        example_cell(
            "Long content",
            nbutton::secondary("Open long dialog")
                .on_press(Message::ShowDialog(DialogKind::LongContent)),
        ),
    ])
}

fn popovers(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell(
            "Bottom start",
            popover_button(
                app,
                PopoverKind::Start,
                PopoverPlacement::BottomStart,
                PopoverWidth::Content,
            ),
        ),
        example_cell(
            "Right end",
            popover_button(
                app,
                PopoverKind::End,
                PopoverPlacement::RightEnd,
                PopoverWidth::AtLeastAnchor,
            ),
        ),
        example_cell(
            "Fixed width",
            popover_button(
                app,
                PopoverKind::Wide,
                PopoverPlacement::BottomEnd,
                PopoverWidth::Fixed(300.0),
            ),
        ),
        example_cell(
            "Collision behavior",
            Popover::new(
                nbutton::secondary("Flip / shift")
                    .on_press(Message::TogglePopover(PopoverKind::Collision)),
            )
            .content(popover_content(
                "Collision",
                "Uses FlipAndShift near viewport edges.",
            ))
            .open(app.overlays.active_popover == Some(PopoverKind::Collision))
            .placement(PopoverPlacement::TopEnd)
            .collision(PopoverCollision::FlipAndShift)
            .gap(8.0)
            .on_dismiss(Message::ClosePopover),
        ),
        example_cell(
            "Tooltip",
            nbutton::icon(IconName::Info)
                .tooltip("Tooltip rendered by the current widget helper")
                .on_press(Message::Noop),
        ),
    ])
}

fn popover_button(
    app: &WidgetGallery,
    kind: PopoverKind,
    placement: PopoverPlacement,
    width: PopoverWidth,
) -> Element<'_, Message> {
    Popover::new(nbutton::secondary("Toggle popover").on_press(Message::TogglePopover(kind)))
        .content(popover_content(
            "Popover",
            "Real overlay content with current theme roles.",
        ))
        .open(app.overlays.active_popover == Some(kind))
        .placement(placement)
        .width(width)
        .gap(8.0)
        .on_dismiss(Message::ClosePopover)
        .into()
}

fn popover_content(title: &'static str, body: &'static str) -> Element<'static, Message> {
    Panel::new(column![ntext::label_strong(title), ntext::body_small(body)].spacing(8))
        .role(SurfaceRole::Popover)
        .padding(12)
        .into()
}

fn command_palette(app: &WidgetGallery) -> Element<'_, Message> {
    let filtered = command_palette_filter(&app.overlays.command_query, COMMAND_ROWS);
    let visible: Vec<&CommandPaletteRow<'static, Message>> = filtered
        .into_iter()
        .map(|index| &COMMAND_ROWS[index])
        .collect();

    let highlighted = (!visible.is_empty()).then_some(0);
    let on_submit = highlighted
        .and_then(|index| visible.get(index))
        .and_then(|row| row.activated())
        .cloned();

    variant_grid([
        example_cell(
            "DropdownMenu static overlay surface",
            super::actions::view_menu_only(),
        ),
        example_cell(
            "Command palette",
            column![
                command_palette_view(
                    "Search commands",
                    &app.overlays.command_query,
                    visible.iter().copied(),
                    Some(0),
                    Message::CommandQueryChanged,
                    on_submit,
                ),
                ntext::caption(
                    app.overlays
                        .selected_command
                        .unwrap_or("No command selected yet"),
                ),
            ]
            .spacing(8),
        ),
    ])
}
