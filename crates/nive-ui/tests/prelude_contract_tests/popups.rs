use nive_ui::prelude::*;

#[test]
fn prelude_exposes_complete_anchored_popup_control_chain() {
    use std::{borrow::Cow, fmt};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Target(String);

    impl fmt::Display for Target {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(&self.0)
        }
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    enum Message {
        Command,
        Toggle(CheckboxState),
        Choose(Target),
        Query(String),
        Opened,
        Closed,
        Clear,
        Submit,
        Blur,
        Dismiss,
    }

    for (index, placement) in [
        TooltipPlacement::Top,
        TooltipPlacement::Right,
        TooltipPlacement::Bottom,
        TooltipPlacement::Left,
    ]
    .into_iter()
    .enumerate()
    {
        let label = if index == 0 {
            Cow::Owned(String::from("Owned tooltip"))
        } else {
            Cow::Borrowed("Borrowed tooltip")
        };
        let tooltip: Element<'_, Message> = Tooltip::new(text("Anchor"), label)
            .placement(placement)
            .into();
        let _: Element<'_, Message> = TooltipScope::new(tooltip).into();
    }

    for placement in [
        PopoverPlacement::TopStart,
        PopoverPlacement::TopCenter,
        PopoverPlacement::TopEnd,
        PopoverPlacement::RightStart,
        PopoverPlacement::RightCenter,
        PopoverPlacement::RightEnd,
        PopoverPlacement::BottomStart,
        PopoverPlacement::BottomCenter,
        PopoverPlacement::BottomEnd,
        PopoverPlacement::LeftStart,
        PopoverPlacement::LeftCenter,
        PopoverPlacement::LeftEnd,
    ] {
        let _: Element<'_, Message> = Popover::new(text("Anchor"))
            .content(text(String::from("Owned content")))
            .open(true)
            .placement(placement)
            .on_dismiss(Message::Dismiss)
            .into();
    }

    for collision in [
        PopoverCollision::FlipAndShift,
        PopoverCollision::Flip,
        PopoverCollision::Shift,
        PopoverCollision::None,
    ] {
        let _: Element<'_, Message> = Popover::new(text("Anchor"))
            .content(text("Borrowed content"))
            .collision(collision)
            .into();
    }

    for width in [
        PopoverWidth::Content,
        PopoverWidth::MatchAnchor,
        PopoverWidth::AtLeastAnchor,
        PopoverWidth::Fixed(240.0),
    ] {
        let _: Element<'_, Message> = Popover::new(text("Anchor"))
            .content(text("Content"))
            .width(width)
            .into();
    }

    for inset in [
        PopoverInset::Standard,
        PopoverInset::Compact,
        PopoverInset::EdgeToEdge,
    ] {
        let _: Element<'_, Message> = Popover::new(text("Anchor"))
            .content(text("Content"))
            .inset(inset)
            .into();
    }

    for focus_policy in [
        PopoverFocusPolicy::RetainAnchor,
        PopoverFocusPolicy::FocusFirst,
        PopoverFocusPolicy::Trap,
    ] {
        let _: Element<'_, Message> = Popover::new(text("Anchor"))
            .content(text("Content"))
            .focus_policy(focus_policy)
            .into();
    }

    let _: Element<'_, Message> = Popover::new(text("Anchor"))
        .content(text("Content"))
        .width_px(280.0)
        .match_anchor_width()
        .at_least_anchor_width()
        .content_width()
        .gap(8.0)
        .on_dismiss(Message::Dismiss)
        .on_dismiss_maybe(None)
        .into();

    let shared_action = nive_core::Action::new(
        "contract.open",
        String::from("Owned action label"),
        Message::Command,
    )
    .description("Borrowed action description")
    .shortcut(nive_core::ShortcutBinding::primary_character('o'));
    let projected = MenuCommand::from_action(&shared_action);
    let standalone = MenuCommand::new("Borrowed standalone command")
        .icon(IconRole::ActionConfirm)
        .shortcut(nive_core::ShortcutBinding::primary_character('s'))
        .destructive()
        .disabled(false)
        .on_press(Message::Command)
        .on_press_maybe(Some(Message::Command))
        .dismiss_policy(MenuDismissPolicy::KeepOpen);
    let absent_command = MenuCommand::new(String::from("Owned absent command"))
        .on_press_maybe(None)
        .dismiss_policy(MenuDismissPolicy::DismissAll);
    let child = Menu::new(text("Child trigger")).command(
        MenuCommand::new("Child command")
            .on_press(Message::Command)
            .dismiss_policy(MenuDismissPolicy::DismissAll),
    );
    let menu = Menu::new(text("Menu trigger"))
        .open(true)
        .on_dismiss(Message::Dismiss)
        .on_dismiss_maybe(Some(Message::Dismiss))
        .placement(PopoverPlacement::BottomEnd)
        .collision(PopoverCollision::FlipAndShift)
        .match_anchor_width()
        .command(projected)
        .command(standalone)
        .command(absent_command)
        .separator()
        .checkbox(
            MenuCheckbox::new(String::from("Owned checkbox"), CheckboxState::Mixed)
                .shortcut(nive_core::ShortcutBinding::primary_character('c'))
                .disabled(false)
                .on_toggle(Message::Toggle)
                .on_toggle_maybe(Some(Message::Toggle))
                .dismiss_policy(MenuDismissPolicy::KeepOpen),
        )
        .checkbox(
            MenuCheckbox::new("Borrowed callback-free checkbox", CheckboxState::Unchecked)
                .on_toggle_maybe(None::<fn(CheckboxState) -> Message>)
                .dismiss_policy(MenuDismissPolicy::DismissAll),
        )
        .radio_group(
            MenuRadioGroup::new(Some(Target("one".into())))
                .option(
                    MenuRadioOption::new(Target("one".into()), "Borrowed radio")
                        .icon(IconRole::ActionConfirm)
                        .annotation(String::from("Owned annotation")),
                )
                .option(
                    MenuRadioOption::new(Target("two".into()), String::from("Owned radio"))
                        .disabled(false),
                )
                .on_select(Message::Choose)
                .on_select_maybe(Some(Message::Choose))
                .dismiss_policy(MenuDismissPolicy::DismissAll),
        )
        .submenu(
            MenuSubmenu::new(String::from("Owned submenu"), child)
                .icon(IconRole::NiveDisclosureRight)
                .disabled(false),
        );
    // Menu intentionally exposes no focus-policy builder: it owns FocusFirst internally.
    let _: Element<'_, Message> = menu.into();

    let callback_free_radio = MenuRadioGroup::<Target, Message>::new(None)
        .option(MenuRadioOption::new(
            Target("none".into()),
            "Callback-free radio",
        ))
        .on_select_maybe(None::<fn(Target) -> Message>)
        .dismiss_policy(MenuDismissPolicy::KeepOpen);
    let _: Element<'_, Message> = Menu::new(text("Callback-free menu"))
        .on_dismiss_maybe(None)
        .radio_group(callback_free_radio)
        .into();

    let select = Select::new(
        vec![
            SelectOption::new(Target("one".into()), "Borrowed option"),
            SelectOption::new(Target("two".into()), String::from("Owned option")).disabled(false),
        ],
        Some(Target("one".into())),
    )
    .placeholder(String::from("Owned placeholder"))
    .semantic_name("Borrowed semantic name")
    .size(theme::ControlSize::Xs)
    .xs()
    .sm()
    .md()
    .lg()
    .width(320.0)
    .fill_width()
    .shrink_width()
    .validation(FieldValidation::Invalid)
    .invalid(false)
    .id("contract-select")
    .disabled(false)
    .on_select(Message::Choose)
    .on_open(Message::Opened)
    .on_close(Message::Closed);
    let select_control: FieldControl<'_, Message> = select.into();
    let select_field = Field::new("Select field", select_control);

    let callback_free_select = Select::from_values(vec![Target("three".into())], None::<Target>)
        .on_select_maybe(None::<fn(Target) -> Message>)
        .on_open_maybe(None)
        .on_close_maybe(None);
    let callback_free_select_field = Field::new("Callback-free select", callback_free_select);

    let suggestions = AutocompleteResults::suggestions(vec![
        AutocompleteSuggestion::new(Target("one".into()), "Borrowed suggestion")
            .leading(IconRole::EditFind)
            .trailing(String::from("Owned trailing text")),
        AutocompleteSuggestion::new(Target("two".into()), String::from("Owned suggestion"))
            .disabled(false),
    ]);
    let autocomplete = Autocomplete::new(
        String::from("owned query"),
        Some(Target("one".into())),
        suggestions,
    )
    .placeholder("Borrowed placeholder")
    .semantic_name(String::from("Owned semantic name"))
    .size(theme::ControlSize::Xs)
    .xs()
    .sm()
    .md()
    .lg()
    .width(360.0)
    .fill_width()
    .shrink_width()
    .validation(FieldValidation::Invalid)
    .invalid(false)
    .disabled(false)
    .id("contract-autocomplete")
    .open(true)
    .highlight(AutocompleteHighlight::First)
    .on_change(Message::Query)
    .on_select(Message::Choose)
    .on_clear(Message::Clear)
    .on_submit(Message::Submit)
    .on_blur(Message::Blur)
    .on_dismiss(Message::Dismiss);
    let autocomplete_control: FieldControl<'_, Message> = autocomplete.into();
    let autocomplete_field = Field::new("Autocomplete field", autocomplete_control);

    let callback_free_autocomplete = Autocomplete::new(
        "borrowed query",
        None::<Target>,
        AutocompleteResults::Loading,
    )
    .highlight(AutocompleteHighlight::None)
    .on_change_maybe(None::<fn(String) -> Message>)
    .on_select_maybe(None::<fn(Target) -> Message>)
    .on_clear_maybe(None)
    .on_submit_maybe(None)
    .on_blur_maybe(None)
    .on_dismiss_maybe(None);
    let callback_free_autocomplete_field =
        Field::new("Callback-free autocomplete", callback_free_autocomplete);

    let _: Element<'_, Message> = FieldGroup::new(
        "Popup controls",
        [
            select_field,
            callback_free_select_field,
            autocomplete_field,
            callback_free_autocomplete_field,
        ],
    )
    .into();

    let _: Element<'_, Message> = Autocomplete::new(
        "",
        None::<Target>,
        AutocompleteResults::empty("Borrowed empty state"),
    )
    .into();
    let _: Element<'_, Message> = Autocomplete::new(
        "",
        None::<Target>,
        AutocompleteResults::error(String::from("Owned error state")),
    )
    .into();
}
