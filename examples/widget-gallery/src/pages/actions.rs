use nive::{
    prelude::*,
    ui::{
        theme::{ControlSize, SurfaceRole},
        widgets::controls::button as nbutton,
        widgets::primitives::text as ntext,
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
            section("Card family", card_family()),
        ]
        .spacing(18),
    )
}

fn buttons(size: ControlSize) -> Element<'static, Message> {
    use nbutton::{ButtonIntent as Intent, ButtonVariant as Variant};

    variant_grid([
        button_combo("Neutral solid", "Neutral", Intent::Neutral, Variant::Solid, size),
        button_combo("Neutral subtle", "Neutral", Intent::Neutral, Variant::Subtle, size),
        button_combo("Neutral outline", "Neutral", Intent::Neutral, Variant::Outline, size),
        button_combo("Neutral ghost", "Neutral", Intent::Neutral, Variant::Ghost, size),
        button_combo("Suggested solid", "Suggested", Intent::Suggested, Variant::Solid, size),
        button_combo("Suggested subtle", "Suggested", Intent::Suggested, Variant::Subtle, size),
        button_combo("Suggested outline", "Suggested", Intent::Suggested, Variant::Outline, size),
        button_combo("Suggested ghost", "Suggested", Intent::Suggested, Variant::Ghost, size),
        button_combo(
            "Destructive solid",
            "Destructive",
            Intent::Destructive,
            Variant::Solid,
            size,
        ),
        button_combo(
            "Destructive subtle",
            "Destructive",
            Intent::Destructive,
            Variant::Subtle,
            size,
        ),
        button_combo(
            "Destructive outline",
            "Destructive",
            Intent::Destructive,
            Variant::Outline,
            size,
        ),
        button_combo(
            "Destructive ghost",
            "Destructive",
            Intent::Destructive,
            Variant::Ghost,
            size,
        ),
    ])
}

fn button_combo(
    title: &'static str,
    label: &'static str,
    intent: nbutton::ButtonIntent,
    variant: nbutton::ButtonVariant,
    size: ControlSize,
) -> Element<'static, Message> {
    example_cell(
        title,
        nbutton::secondary(label)
            .intent(intent)
            .variant(variant)
            .size(size)
            .on_press(Message::Noop),
    )
}

fn button_states() -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Icon placement",
            variant_row([
                nbutton::primary("Back")
                    .leading_icon(IconRole::GoPrevious)
                    .on_press(Message::Noop)
                    .into(),
                nbutton::secondary("Next")
                    .trailing_icon(IconRole::GoNext)
                    .on_press(Message::Noop)
                    .into(),
            ]),
        ),
        example_cell(
            "Icon-only",
            variant_row([
                nbutton::icon(IconRole::EditFind)
                    .tooltip("Search")
                    .on_press(Message::Noop)
                    .into(),
                nbutton::icon(IconRole::PreferencesSystem)
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
                .item(segment("Tests", selected).icon(IconRole::ActionConfirm))
                .fill_width(),
        ),
        example_cell(
            "Flat",
            SegmentedControl::new()
                .flat()
                .size(size)
                .item(segment("Preview", selected))
                .item(segment("Code", selected))
                .item(segment("Tests", selected).icon(IconRole::ActionConfirm))
                .fill_width(),
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
                nbutton::icon(IconRole::PreferencesSystem)
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
                    ContentAction::icon(IconRole::GoPrevious, "Back")
                        .tooltip("Back")
                        .on_press(Message::Noop),
                )
                .action(
                    ContentAction::icon(IconRole::GoNext, "Forward")
                        .tooltip("Forward")
                        .on_press(Message::Noop),
                )
                .separator()
                .action(ContentAction::icon_label(IconRole::ViewRefresh, "Refresh").loading(true)),
        ),
        example_cell(
            "Content action states",
            ActionGroup::new()
                .size(size)
                .action(ContentAction::label("Preview").on_press(Message::Noop))
                .action(ContentAction::label("Code"))
                .action(
                    ContentAction::icon(IconRole::EditDelete, "Delete")
                        .destructive()
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
                        ContentAction::icon(IconRole::GoPrevious, "Back")
                            .tooltip("Back")
                            .on_press(Message::Noop),
                    )
                    .action(
                        ContentAction::icon(IconRole::GoNext, "Forward")
                            .tooltip("Forward")
                            .on_press(Message::Noop),
                    )
                    .into(),
                nbutton::icon(IconRole::PreferencesSystem)
                    .size(size)
                    .tooltip("Settings")
                    .on_press(Message::Noop)
                    .into(),
            ]),
        ),
        example_cell(
            "Narrow wrapping",
            container(
                ActionGroup::new()
                    .size(size)
                    .fill_width()
                    .wrap()
                    .action(ContentAction::label("Inspect").on_press(Message::Noop))
                    .separator()
                    .action(
                        ContentAction::icon_label(IconRole::ViewRefresh, "Refresh")
                            .loading(false)
                            .on_press(Message::Noop),
                    )
                    .action(
                        ContentAction::label("Oversized complete action label")
                            .on_press(Message::Noop),
                    ),
            )
            .width(180),
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
                            ToolbarAction::icon(IconRole::GoPrevious)
                                .tooltip("Back")
                                .on_press(Message::Noop),
                        )
                        .action(
                            ToolbarAction::icon(IconRole::GoNext)
                                .tooltip("Forward")
                                .on_press(Message::Noop),
                        ),
                )
                .separator()
                .group(
                    ToolbarGroup::new().action(
                        ToolbarAction::icon_label(IconRole::ViewRefresh, "Refresh").loading(true),
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
                            ToolbarAction::icon(IconRole::EditDelete)
                                .destructive()
                                .disabled(true)
                                .tooltip("Delete"),
                        ),
                )
                .fill_width(),
            container(ntext::body_small("Selected project activity")).padding(16)
        ]
        .spacing(0),
    )
    .role(SurfaceRole::Panel)
    .body_padding(0)
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
                .icon(IconRole::EditModify)
                .trailing("Enter")
                .on_press(Message::Noop),
        )
        .item(
            DropdownMenuItem::new("Copy link")
                .icon(IconRole::EditCopy)
                .trailing("Cmd+C")
                .selected(true)
                .on_press(Message::Noop),
        )
        .separator()
        .item(DropdownMenuItem::new("Disabled command").disabled(true))
        .item(
            DropdownMenuItem::new("Delete")
                .icon(IconRole::EditDelete)
                .destructive()
                .on_press(Message::Noop),
        )
        .width(260)
        .into()
}

fn card_family() -> Element<'static, Message> {
    let cells = [
        canvas_cell("Card · filled", Card::new(card_content("Filled", "Default passive frame"))),
        canvas_cell(
            "Card · outlined",
            Card::new(card_content("Outlined", "Transparent with one perimeter")).outlined(),
        ),
        canvas_cell(
            "Card · elevated",
            Card::new(card_content("Elevated", "Semantic elevation and shadow")).elevated(),
        ),
        canvas_cell(
            "Card · ghost",
            Card::new(card_content("Ghost", "Surface-free local grouping")).ghost(),
        ),
        canvas_cell(
            "ActionCard · filled",
            ActionCard::new(card_content("Run import", "One whole-surface action"))
                .on_press(Message::Noop),
        ),
        canvas_cell(
            "ActionCard · outlined",
            ActionCard::new(card_content("Inspect", "Keyboard and pointer capable"))
                .outlined()
                .on_press(Message::Noop),
        ),
        canvas_cell(
            "ActionCard · elevated",
            ActionCard::new(card_content("New project", "Preserves elevated identity"))
                .elevated()
                .on_press(Message::Noop),
        ),
        canvas_cell(
            "ActionCard · absent callback",
            ActionCard::new(card_content("Unavailable capability", "Enabled idle presentation"))
                .ghost(),
        ),
        canvas_cell(
            "SelectableCard · selected",
            SelectableCard::new(card_content("Compact", "Controlled persistent selection"))
                .selected(true)
                .selection_indicator(true)
                .on_press(Message::Noop),
        ),
        canvas_cell(
            "SelectableCard · outlined",
            SelectableCard::new(card_content("Detailed", "Unselected controlled object"))
                .outlined()
                .selection_indicator(true)
                .on_press(Message::Noop),
        ),
        canvas_cell(
            "SelectableCard · elevated disabled",
            SelectableCard::new(card_content("Pinned", "Selected and explicitly disabled"))
                .elevated()
                .selected(true)
                .selection_indicator(true)
                .disabled(true)
                .on_press(Message::Noop),
        ),
        canvas_cell(
            "Long arbitrary content",
            ActionCard::new(
                row![
                    Icon::role(IconRole::DialogInformation).md(),
                    card_content(
                        "A deliberately long title for constrained review",
                        "The description remains ordinary 14 px content and the trailing icon is display-only.",
                    ),
                    Icon::role(IconRole::GoNext).sm(),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .ghost()
            .on_press(Message::Noop),
        ),
    ];

    Panel::new(variant_grid(cells))
        .role(SurfaceRole::Canvas)
        .body_padding(14)
        .fill_width()
        .into()
}

fn card_content(title: &'static str, description: &'static str) -> Element<'static, Message> {
    column![ntext::body_strong(title), ntext::body(description)]
        .spacing(4)
        .into()
}

fn canvas_cell(
    label: &'static str,
    content: impl Into<Element<'static, Message>>,
) -> Element<'static, Message> {
    Panel::new(column![ntext::caption(label), content.into()].spacing(10))
        .role(SurfaceRole::Canvas)
        .body_padding(14)
        .width(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_family_and_content_action_stress_views_build() {
        let _: Element<'static, Message> = card_family();
        let _: Element<'static, Message> = action_groups(ControlSize::Xs);
        let _: Element<'static, Message> = action_groups(ControlSize::Lg);
    }
}
