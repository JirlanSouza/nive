use std::borrow::Cow;

use nive::prelude::*;
use nive::ui::widgets::controls::button as nbutton;
use nive::ui::widgets::primitives::text as ntext;
use nive::widget::{column, row};

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
            section("Tooltip matrix", tooltip_matrix()),
            section("Popover geometry", popover_geometry(app)),
            section("Popover focus and lifecycle", popover_lifecycle(app)),
            section("Dropdown and command palette", command_palette(app)),
        ]
        .spacing(18),
    )
}

pub fn dialog(kind: DialogKind) -> Element<'static, Message> {
    let header = match kind {
        DialogKind::Basic => DialogHeader::new("Confirm action")
            .description(
                "This dialog uses Dialog, DialogHeader, DialogActionFooter, and typed DialogAction roles.",
            )
            .close("Close dialog", Message::CloseDialog),
        DialogKind::Destructive => DialogHeader::new("Delete project")
            .description("The destructive action stays last and is never the initial-focus or Enter default."),
        DialogKind::LongContent => DialogHeader::new("Long content").description(
            "The Dialog body owns the only vertical scroll region; header and footer stay fixed.",
        ),
        DialogKind::NestedOverlay => DialogHeader::new("Assign reviewer").description(
            "The Select popup below is Dialog-owned content and receives Escape/outside-press priority over the Dialog itself.",
        ),
    };

    let body: Element<'static, Message> = match kind {
        DialogKind::LongContent => column(
            long_content_paragraphs()
                .iter()
                .map(|paragraph| ntext::body(*paragraph).into()),
        )
        .spacing(12)
        .into(),
        DialogKind::NestedOverlay => column![
            ntext::body(
                "Open the Select below, then press Escape: only the popup closes on the first press."
            ),
            Select::new(
                vec![
                    SelectOption::new("alex", "Alex Chen"),
                    SelectOption::new("priya", "Priya Nair"),
                    SelectOption::new("sam", "Sam Okafor"),
                ],
                None,
            )
            .placeholder("Choose a reviewer")
            .on_select(|_choice: &'static str| Message::Noop),
        ]
        .spacing(12)
        .into(),
        _ => ntext::body("Review the current dialog baseline before accepting the action.").into(),
    };

    let footer = match kind {
        DialogKind::Destructive => DialogActionFooter::with_one(
            DialogAction::cancel("Cancel", Message::CloseDialog),
            DialogTerminalAction::destructive("Delete", Message::CloseDialog),
        ),
        _ => DialogActionFooter::with_one(
            DialogAction::cancel("Cancel", Message::CloseDialog),
            DialogTerminalAction::primary("Confirm", Message::CloseDialog),
        ),
    }
    .status(ntext::caption("Esc or backdrop dismisses the dialog"));

    let size = match kind {
        DialogKind::LongContent => DialogSize::Lg,
        _ => DialogSize::Sm,
    };

    Dialog::new(body)
        .header(header)
        .footer(footer)
        .size(size)
        .into()
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
        example_cell(
            "Nested overlay",
            nbutton::secondary("Open dialog with Select")
                .on_press(Message::ShowDialog(DialogKind::NestedOverlay)),
        ),
    ])
}

fn tooltip_matrix() -> Element<'static, Message> {
    let scoped_neighbors = TooltipScope::new(
        row![
            Tooltip::new(
                nbutton::icon(IconRole::GoPrevious, "Previous item"),
                "Previous item in this group",
            )
            .placement(TooltipPlacement::Top),
            Tooltip::new(
                nbutton::icon(IconRole::GoNext, "Next item"),
                "Next item in this group",
            )
            .placement(TooltipPlacement::Top),
        ]
        .spacing(6),
    );

    variant_grid([
        example_cell(
            "Pointer · Top",
            Tooltip::new(
                nbutton::icon(IconRole::DialogInformation, "Inspect deployment"),
                "Deployment details",
            )
            .placement(TooltipPlacement::Top),
        ),
        example_cell(
            "Keyboard focus + Escape · Right",
            Tooltip::new(
                nbutton::secondary("Focus with Tab").on_press(Message::Noop),
                "Press Escape while this tooltip is visible",
            )
            .placement(TooltipPlacement::Right),
        ),
        example_cell(
            "Long text + flip / shift · Bottom",
            Tooltip::new(
                nbutton::icon(IconRole::DialogInformation, "Explain constrained layout"),
                "This intentionally long explanation wraps and flips or shifts when the viewport cannot honor its preferred placement.",
            )
            .placement(TooltipPlacement::Bottom),
        ),
        example_cell(
            "Disabled explanation · Left",
            Tooltip::new(
                nbutton::secondary("Unavailable action").disabled(true),
                "This action requires a writable workspace",
            )
            .placement(TooltipPlacement::Left),
        ),
        example_cell("Scoped neighbors", scoped_neighbors),
        example_cell(
            "Complete visible label",
            column![
                nbutton::secondary("No duplicate tooltip").on_press(Message::Noop),
                ntext::caption("The visible label is already complete."),
            ]
            .spacing(6),
        ),
    ])
}

fn popover_geometry(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell(
            "Standard inset · Content width",
            popover_button(
                app,
                PopoverKind::Start,
                "Open standard",
                PopoverPlacement::BottomStart,
                PopoverInset::Standard,
                PopoverWidth::Content,
            ),
        ),
        example_cell(
            "Standard inset · At least anchor",
            popover_button(
                app,
                PopoverKind::End,
                "Open at-least",
                PopoverPlacement::RightEnd,
                PopoverInset::Standard,
                PopoverWidth::AtLeastAnchor,
            ),
        ),
        example_cell(
            "Compact inset · Match anchor",
            popover_button(
                app,
                PopoverKind::MatchAnchor,
                "A wider match-anchor trigger",
                PopoverPlacement::TopStart,
                PopoverInset::Compact,
                PopoverWidth::MatchAnchor,
            ),
        ),
        example_cell(
            "Edge-to-edge · Content width",
            popover_button(
                app,
                PopoverKind::EdgeToEdge,
                "Open edge-to-edge",
                PopoverPlacement::LeftStart,
                PopoverInset::EdgeToEdge,
                PopoverWidth::Content,
            ),
        ),
        example_cell(
            "Fixed 300px",
            popover_button(
                app,
                PopoverKind::Wide,
                "Open fixed width",
                PopoverPlacement::BottomEnd,
                PopoverInset::Standard,
                PopoverWidth::Fixed(300.0),
            ),
        ),
        example_cell(
            "Viewport collision · FlipAndShift",
            popover_button(
                app,
                PopoverKind::Collision,
                "Open near an edge",
                PopoverPlacement::TopEnd,
                PopoverInset::Standard,
                PopoverWidth::Fixed(280.0),
            ),
        ),
    ])
}

fn popover_lifecycle(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell(
            "Retain anchor · outside / Escape / reactivation",
            lifecycle_popover(
                app,
                PopoverKind::RetainAnchor,
                "Open retain-anchor",
                PopoverFocusPolicy::RetainAnchor,
                popover_content("RetainAnchor", "Dismiss outside or with Escape, then reopen."),
            ),
        ),
        example_cell(
            "Focus first",
            lifecycle_popover(
                app,
                PopoverKind::FocusFirst,
                "Open focus-first",
                PopoverFocusPolicy::FocusFirst,
                focus_content(app, "The first action receives logical focus."),
            ),
        ),
        example_cell(
            "Trap",
            lifecycle_popover(
                app,
                PopoverKind::Trap,
                "Open focus trap",
                PopoverFocusPolicy::Trap,
                focus_content(app, "Tab cycles inside until dismissal."),
            ),
        ),
        example_cell("Nested priority", nested_popover(app)),
        example_cell(
            "Low viewport · one Popover scroll owner",
            Popover::new(
                nbutton::secondary("Open tall content")
                    .on_press(Message::TogglePopover(PopoverKind::LowHeight)),
            )
            .content(tall_popover_content())
            .open(app.overlays.active_popover == Some(PopoverKind::LowHeight))
            .placement(PopoverPlacement::BottomStart)
            .collision(PopoverCollision::FlipAndShift)
            .on_dismiss(Message::ClosePopover),
        ),
    ])
}

fn popover_button<'a>(
    app: &'a WidgetGallery,
    kind: PopoverKind,
    trigger: &'static str,
    placement: PopoverPlacement,
    inset: PopoverInset,
    width: PopoverWidth,
) -> Element<'a, Message> {
    Popover::new(nbutton::secondary(trigger).on_press(Message::TogglePopover(kind)))
        .content(popover_content(
            "Popover",
            "Real overlay content with current theme roles.",
        ))
        .open(app.overlays.active_popover == Some(kind))
        .placement(placement)
        .collision(PopoverCollision::FlipAndShift)
        .inset(inset)
        .width(width)
        .gap(8.0)
        .on_dismiss(Message::ClosePopover)
        .into()
}

fn lifecycle_popover<'a>(
    app: &WidgetGallery,
    kind: PopoverKind,
    trigger: &'static str,
    focus_policy: PopoverFocusPolicy,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    Popover::new(nbutton::secondary(trigger).on_press(Message::TogglePopover(kind)))
        .content(content)
        .open(app.overlays.active_popover == Some(kind))
        .placement(PopoverPlacement::BottomStart)
        .collision(PopoverCollision::FlipAndShift)
        .focus_policy(focus_policy)
        .on_dismiss(Message::ClosePopover)
        .into()
}

fn focus_content<'a>(app: &'a WidgetGallery, description: &'static str) -> Element<'a, Message> {
    column![
        nbutton::secondary("First action").on_press(Message::Noop),
        Input::new("Filter", &app.form.search).on_change(Message::InputSearchChanged),
        nbutton::secondary("Last action").on_press(Message::Noop),
        ntext::caption(description),
    ]
    .spacing(8)
    .into()
}

fn nested_popover(app: &WidgetGallery) -> Element<'_, Message> {
    let child = Popover::new(
        nbutton::secondary("Open nested popover").on_press(Message::ToggleNestedPopover),
    )
    .content(popover_content(
        "Nested child",
        "Escape and outside dismissal target this child before its parent.",
    ))
    .open(app.overlays.nested_popover_open)
    .placement(PopoverPlacement::RightStart)
    .collision(PopoverCollision::FlipAndShift)
    .on_dismiss(Message::CloseNestedPopover);

    Popover::new(
        nbutton::secondary("Open parent popover")
            .on_press(Message::TogglePopover(PopoverKind::Nested)),
    )
    .content(
        column![
            ntext::label_strong("Parent popover"),
            ntext::body_small("Open the child, then dismiss one level at a time."),
            child,
        ]
        .spacing(8),
    )
    .open(app.overlays.active_popover == Some(PopoverKind::Nested))
    .placement(PopoverPlacement::BottomStart)
    .collision(PopoverCollision::FlipAndShift)
    .on_dismiss(Message::ClosePopover)
    .into()
}

fn long_content_paragraphs() -> &'static [&'static str] {
    &[
        "The gallery exposes long content behavior in real dialog surfaces.",
        "Additional copy keeps height pressure visible while the app-owned dialog remains dismissible.",
        "No wrapper is correcting spacing, radius, or scroll ownership here.",
        "Paragraph 4 keeps pushing the body past the available viewport height.",
        "Paragraph 5 confirms the header and footer stay pinned while this text scrolls underneath.",
        "Paragraph 6 is here purely to add vertical pressure, not to demonstrate new copy.",
        "Paragraph 7 continues the same pattern so the scrollable region has real overflow to own.",
        "Paragraph 8 checks that focus and keyboard scrolling both stay inside the body region.",
        "Paragraph 9 verifies that the Dialog frame itself never grows past its DialogSize::Lg cap.",
        "Paragraph 10 is the midpoint marker for this long-content fixture.",
        "Paragraph 11 keeps the column growing so the scrollbar affordance stays visible.",
        "Paragraph 12 reiterates that only the body owns a Scrollable, never the header or footer.",
        "Paragraph 13 exists to keep the total height comfortably past the Lg dialog's fixed cap.",
        "Paragraph 14 is filler copy standing in for a real changelog or terms-of-service body.",
        "Paragraph 15 confirms scrolling to the end still respects the dialog's rounded frame.",
        "Paragraph 16 keeps stacking so the seam above the footer has a real overflow signal to react to.",
        "Paragraph 17 is nearly at the end of this fixture's long body.",
        "Paragraph 18 is the last paragraph before the closing line below.",
        "This final line marks the true end of the scrollable body content.",
    ]
}

fn tall_popover_content() -> Element<'static, Message> {
    column![
        ntext::label_strong("Popover-owned scrolling"),
        ntext::body_small("Row 01 · no caller Scrollable"),
        ntext::body_small("Row 02 · constrained by available height"),
        ntext::body_small("Row 03 · shared surface owns the viewport"),
        ntext::body_small("Row 04 · keyboard content remains reachable"),
        ntext::body_small("Row 05 · narrow and low layouts stay finite"),
        ntext::body_small("Row 06 · one inset and one border"),
        ntext::body_small("Row 07 · no nested Panel"),
        ntext::body_small("Row 08 · no second scroll owner"),
        ntext::body_small("Row 09 · collision remains active"),
        ntext::body_small("Row 10 · Escape dismisses once"),
        ntext::body_small("Row 11 · outside press dismisses once"),
        ntext::body_small("Row 12 · trigger can reactivate"),
    ]
    .spacing(6)
    .into()
}

fn popover_content(title: &'static str, body: &'static str) -> Element<'static, Message> {
    column![ntext::label_strong(title), ntext::body_small(body)]
        .spacing(8)
        .into()
}

fn command_palette(app: &WidgetGallery) -> Element<'_, Message> {
    let filtered = command_palette_filter(&app.overlays.command_query, COMMAND_ROWS);
    let visible: Vec<CommandPaletteRow<'static, Message>> = filtered
        .into_iter()
        .map(|index| COMMAND_ROWS[index].clone())
        .collect();

    let highlighted = (!visible.is_empty()).then_some(0);
    let on_submit = highlighted
        .and_then(|index| visible.get(index))
        .and_then(|row| row.activated())
        .cloned();

    variant_grid([
        example_cell(
            "Canonical anchored Menu",
            super::actions::view_menu_only(app),
        ),
        example_cell(
            "Command palette",
            column![
                command_palette_view(
                    "Search commands",
                    &app.overlays.command_query,
                    visible.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_and_popover_matrices_build_every_controlled_fixture() {
        let mut app = WidgetGallery::test_fixture();
        let _: Element<'_, Message> = tooltip_matrix();

        for kind in [
            PopoverKind::Start,
            PopoverKind::End,
            PopoverKind::Wide,
            PopoverKind::Collision,
            PopoverKind::MatchAnchor,
            PopoverKind::EdgeToEdge,
            PopoverKind::LowHeight,
            PopoverKind::RetainAnchor,
            PopoverKind::FocusFirst,
            PopoverKind::Trap,
            PopoverKind::Nested,
        ] {
            app.overlays.active_popover = Some(kind);
            app.overlays.nested_popover_open = kind == PopoverKind::Nested;
            let _: Element<'_, Message> = popover_geometry(&app);
            let _: Element<'_, Message> = popover_lifecycle(&app);
        }
    }

    #[test]
    fn dialog_builds_for_every_kind_including_nested_overlay() {
        for kind in [
            DialogKind::Basic,
            DialogKind::Destructive,
            DialogKind::LongContent,
            DialogKind::NestedOverlay,
        ] {
            let _: Element<'_, Message> = dialog(kind);
        }
    }
}
