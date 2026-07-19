use nive::{
    prelude::*,
    ui::{
        theme::{ControlSize, SurfaceRole},
        widgets::controls::button as nbutton,
        widgets::primitives::text as ntext,
    },
    widget::column,
};

use crate::app::{MenuKind, Message, WidgetGallery};
use crate::catalog::PageId;
use crate::layout::{example_cell, section, variant_grid, variant_row};

pub fn view(app: &WidgetGallery) -> Element<'_, Message> {
    crate::app::page_shell(
        PageId::Actions,
        column![
            section("Button hierarchy", button_hierarchy(app.control_size)),
            section("Advanced button axes", advanced_buttons(app.control_size)),
            section("Action states", button_states()),
            section(
                "SegmentedControl",
                segmented_controls(app.control_size, app.form.segment)
            ),
            section("ActionGroup", action_groups(app.control_size)),
            section("Toolbar", toolbar(app.control_size)),
            section("Menu", menu(app)),
            section("Card family", card_family()),
        ]
        .spacing(18),
    )
}

fn button_hierarchy(size: ControlSize) -> Element<'static, Message> {
    variant_grid([
        example_cell(
            "Primary",
            column![
                nbutton::primary("Create project")
                    .size(size)
                    .on_press(Message::Noop),
                ntext::caption("Use at most one primary action per local action group"),
            ]
            .spacing(6),
        ),
        example_cell(
            "Secondary",
            nbutton::secondary("Save draft")
                .size(size)
                .on_press(Message::Noop),
        ),
        example_cell(
            "Tertiary",
            nbutton::tertiary("Preview")
                .size(size)
                .on_press(Message::Noop),
        ),
        example_cell(
            "Destructive",
            nbutton::destructive("Delete project")
                .size(size)
                .on_press(Message::Noop),
        ),
    ])
}

fn advanced_buttons(size: ControlSize) -> Element<'static, Message> {
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
                nbutton::icon(IconRole::EditFind, "Search")
                    .tooltip("Search")
                    .on_press(Message::Noop)
                    .into(),
                nbutton::icon(IconRole::PreferencesSystem, "Settings")
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
            "Intrinsic / fill",
            column![
                nbutton::secondary("Intrinsic").on_press(Message::Noop),
                nbutton::secondary("Fill width")
                    .fill_width()
                    .on_press(Message::Noop),
            ]
            .spacing(8)
            .width(Length::Fill),
        ),
        example_cell(
            "All form sizes",
            variant_row([
                nbutton::secondary("XS")
                    .xs()
                    .on_press(Message::Noop)
                    .into(),
                nbutton::secondary("SM")
                    .sm()
                    .on_press(Message::Noop)
                    .into(),
                nbutton::secondary("MD")
                    .md()
                    .on_press(Message::Noop)
                    .into(),
                nbutton::secondary("LG")
                    .lg()
                    .on_press(Message::Noop)
                    .into(),
            ]),
        ),
        example_cell(
            "Manual padding escape",
            nbutton::secondary("Outer height remains fixed")
                .padding(30)
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
            SegmentedControl::new(
                "Editor mode",
                selected,
                [
                    segment("Preview"),
                    segment("Code"),
                    segment("Tests").icon(IconRole::ActionConfirm),
                ],
            )
                .size(size)
                .on_select(Message::SelectSegment)
                .fill_width(),
        ),
        example_cell(
            "Flat",
            SegmentedControl::new(
                "Linked editor mode",
                selected,
                [
                    segment("Preview"),
                    segment("Code"),
                    segment("Tests").icon(IconRole::ActionConfirm),
                ],
            )
                .linked()
                .size(size)
                .on_select(Message::SelectSegment)
                .fill_width(),
        ),
        example_cell(
            "Inline",
            variant_row([
                nbutton::secondary("Run")
                    .size(size)
                    .on_press(Message::Noop)
                    .into(),
                SegmentedControl::new(
                    "Inline editor mode",
                    selected,
                    [segment("Preview"), segment("Code"), segment("Tests")],
                )
                    .size(size)
                    .on_select(Message::SelectSegment)
                    .into(),
                nbutton::icon(IconRole::PreferencesSystem, "Settings")
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
                nbutton::icon(IconRole::PreferencesSystem, "Settings")
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

fn segment(label: &'static str) -> SegmentedOption<'static, &'static str> {
    SegmentedOption::new(label, label)
}

fn menu(app: &WidgetGallery) -> Element<'_, Message> {
    variant_grid([
        example_cell("Typed entries", view_menu_only(app)),
        example_cell("Persistent checkbox and radio", persistent_menu(app)),
        example_cell("Callback-absent leaves", callback_absent_menu(app)),
        example_cell("Nested submenu", nested_menu(app)),
        example_cell("Long list, typeahead, Home / End", long_list_menu(app)),
    ])
}

pub fn view_menu_only(app: &WidgetGallery) -> Element<'_, Message> {
    Menu::new(
        nive::widgets::button::secondary("Open typed menu")
            .on_press(Message::ToggleMenu(MenuKind::Typed)),
    )
        .open(app.overlays.active_menu == Some(MenuKind::Typed))
        .on_dismiss(Message::CloseMenu)
        .command(
            MenuCommand::new("Rename")
                .icon(IconRole::EditModify)
                .shortcut(ShortcutBinding::named(
                    NamedShortcutKey::Enter,
                    ShortcutModifiers::NONE,
                ))
                .on_press(Message::Noop),
        )
        .checkbox(
            MenuCheckbox::new("Copy link", CheckboxState::Checked)
                .shortcut(ShortcutBinding::primary_character('c'))
                .on_toggle(|_| Message::Noop),
        )
        .separator()
        .command(MenuCommand::new("Disabled command").disabled(true))
        .command(MenuCommand::new("Callback absent"))
        .command(
            MenuCommand::new("Delete")
                .icon(IconRole::EditDelete)
                .destructive()
                .on_press(Message::Noop),
        )
        .into()
}

fn persistent_menu(app: &WidgetGallery) -> Element<'_, Message> {
    Menu::new(
        nive::widgets::button::secondary("Open persistent choices")
            .on_press(Message::ToggleMenu(MenuKind::Persistent)),
    )
    .open(app.overlays.active_menu == Some(MenuKind::Persistent))
    .on_dismiss(Message::CloseMenu)
    .checkbox(
        MenuCheckbox::new("Keep project pinned", app.overlays.menu_pinned)
            .on_toggle(Message::MenuPinnedChanged)
            .dismiss_policy(MenuDismissPolicy::KeepOpen),
    )
    .separator()
    .radio_group(
        MenuRadioGroup::new(app.overlays.menu_mode)
            .option(MenuRadioOption::new("compact", "Compact").annotation("Dense"))
            .option(MenuRadioOption::new("standard", "Standard").annotation("Default"))
            .option(
                MenuRadioOption::new("comfortable", "Comfortable")
                    .annotation("Roomy")
                    .icon(IconRole::ViewMore),
            )
            .option(MenuRadioOption::new("unavailable", "Unavailable").disabled(true))
            .on_select(Message::MenuModeChanged)
            .dismiss_policy(MenuDismissPolicy::KeepOpen),
    )
    .into()
}

fn callback_absent_menu(app: &WidgetGallery) -> Element<'_, Message> {
    Menu::new(
        nive::widgets::button::secondary("Open display-only entries")
            .on_press(Message::ToggleMenu(MenuKind::CallbackAbsent)),
    )
    .open(app.overlays.active_menu == Some(MenuKind::CallbackAbsent))
    .on_dismiss(Message::CloseMenu)
    .command(MenuCommand::new("Command without activation"))
    .checkbox(MenuCheckbox::new("Checkbox without toggle", CheckboxState::Mixed))
    .radio_group(
        MenuRadioGroup::<_, Message>::new(Some("selected"))
            .option(MenuRadioOption::new("selected", "Selected display value"))
            .option(MenuRadioOption::new("other", "Other display value")),
    )
    .separator()
    .command(MenuCommand::new("Explicitly disabled").disabled(true))
    .into()
}

fn nested_menu(app: &WidgetGallery) -> Element<'_, Message> {
    let grandchild = Menu::new(nive::widget::Space::new())
        .command(MenuCommand::new("Reset zoom").on_press(Message::Noop));
    let child = Menu::new(nive::widget::Space::new())
        .command(MenuCommand::new("Zoom in").on_press(Message::Noop))
        .command(MenuCommand::new("Zoom out").on_press(Message::Noop))
        .submenu(MenuSubmenu::new("More zoom", grandchild));

    Menu::new(
        nive::widgets::button::secondary("Open nested menu")
            .on_press(Message::ToggleMenu(MenuKind::Nested)),
    )
    .open(app.overlays.active_menu == Some(MenuKind::Nested))
    .on_dismiss(Message::CloseMenu)
    .command(MenuCommand::new("New window").on_press(Message::Noop))
    .submenu(MenuSubmenu::new("View", child).icon(IconRole::ViewReveal))
    .submenu(
        MenuSubmenu::new("Disabled branch", Menu::new(nive::widget::Space::new())).disabled(true),
    )
    .into()
}

fn long_list_menu(app: &WidgetGallery) -> Element<'_, Message> {
    let mut menu = Menu::new(
        nive::widgets::button::secondary("Open long menu")
            .on_press(Message::ToggleMenu(MenuKind::LongList)),
    )
    .open(app.overlays.active_menu == Some(MenuKind::LongList))
    .on_dismiss(Message::CloseMenu)
    .command(MenuCommand::new("Callback-absent first row"))
    .command(MenuCommand::new("Disabled second row").disabled(true))
    .command(
        MenuCommand::new("A very long eligible command label that exercises stable columns")
            .shortcut(ShortcutBinding::primary_character('l'))
            .on_press(Message::Noop),
    );

    for index in 1..=18 {
        menu = menu.command(
            MenuCommand::new(format!("Project command {index:02}"))
                .on_press(Message::Noop),
        );
    }

    menu.into()
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

    #[test]
    fn professional_button_hierarchy_and_state_matrices_build() {
        for size in [
            ControlSize::Xs,
            ControlSize::Sm,
            ControlSize::Md,
            ControlSize::Lg,
        ] {
            let _: Element<'static, Message> = button_hierarchy(size);
            let _: Element<'static, Message> = advanced_buttons(size);
        }
        let _: Element<'static, Message> = button_states();
    }

    #[test]
    fn canonical_menu_matrix_builds_every_controlled_state() {
        let mut app = WidgetGallery::test_fixture();

        for kind in [
            MenuKind::Typed,
            MenuKind::Persistent,
            MenuKind::CallbackAbsent,
            MenuKind::Nested,
            MenuKind::LongList,
        ] {
            app.overlays.active_menu = Some(kind);
            app.overlays.menu_pinned = CheckboxState::Checked;
            app.overlays.menu_mode = Some("comfortable");
            let _: Element<'_, Message> = menu(&app);
        }
    }
}
