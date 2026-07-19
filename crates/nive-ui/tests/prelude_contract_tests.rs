use nive_ui::prelude::*;

#[test]
fn prelude_exposes_common_ui_contracts() {
    let _: Element<'_, ()> = text("Nive").into();
    let _: Theme = theme::active();
    let _: ThemePreference = ThemePreference::System;
    let _: ThemeDensity = ThemeDensity::Standard;
    let _: Color = Color::TRANSPARENT;
    let _: Background = Background::Color(Color::TRANSPARENT);
    let _: Border = Border::default();
    let _: Shadow = Shadow::default();
}

#[test]
fn prelude_exposes_common_widget_contracts() {
    let _: ButtonIntent = ButtonIntent::Suggested;
    let _: ButtonVariant = ButtonVariant::Solid;
    let _: Element<'_, ()> = Card::new(text("Card")).into();
    let _: CardVariant = CardVariant::Outlined;
    let _: Element<'_, ()> = Card::new(text("Card")).outlined().into();
    let _: Element<'_, ()> = ActionCard::new(text("Open")).elevated().into();
    let _: Element<'_, ()> = SelectableCard::new(text("Object"))
        .ghost()
        .selection_indicator(true)
        .into();
    let _: theme::TypographyRole = theme::TypographyRole::BodyStrong;
    let _: theme::TypographyRole = theme::TypographyRole::BadgeLabel;
    let _: theme::TypographyRole = theme::TypographyRole::MetadataTag;
    let _: theme::TypographyRole = theme::TypographyRole::Control;
    let _: theme::TypographyRole = theme::TypographyRole::ControlStrong;
    let scale = theme::typography::scale();
    let _: theme::TextStyle = scale.badge_label;
    let _: theme::TextStyle = scale.metadata_tag;
    let _: theme::TextStyle = scale.control;
    let _: theme::TextStyle = scale.control_strong;
    let _: Element<'_, ()> = nive_ui::widgets::text::body_strong("Card title").into();
    let _: Element<'_, ()> = nive_ui::widgets::text::badge_label("3").into();
    let _: Element<'_, ()> = nive_ui::widgets::text::metadata_tag("1.0.0").into();
    let _: Element<'_, ()> = MetricCard::new("Latency", "18.4")
        .unit("ms")
        .status(text("healthy"))
        .trend(text("-2.1%"))
        .into();
    let _: Element<'_, ()> = ActionGroup::new()
        .sm()
        .wrap()
        .action(ContentAction::label("Refresh"))
        .into();
    let _: Element<'_, ()> = Field::new("Name", Input::new("Name", "")).into();
    let _: Element<'_, ()> = Dialog::new(text("Dialog")).into();
    let _: Element<'_, ()> = EmptyState::new("No results").into();
    let _: Element<'_, ()> = Separator::horizontal().into();
    let _: Element<'_, ()> = Separator::horizontal()
        .strength(SeparatorStrength::Section)
        .extent(SeparatorExtent::Inset {
            leading: 12.0,
            trailing: 4.0,
        })
        .into();
    let _: Element<'_, ()> = SectionHeader::new("A long section")
        .title_tooltip("A long section")
        .into();
    let _: Element<'_, ()> = ToneDot::new(theme::roles::ToneRole::Success).sm().into();
}

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

#[test]
#[allow(deprecated)]
fn segmented_migration_bridge_remains_separate() {
    let _: Element<'_, ()> = LegacySegmentedControl::new()
        .item(SegmentedItem::new("Status").status_text(theme::roles::ToneRole::Success, "Healthy"))
        .into();
}

#[test]
fn widget_taxonomy_exposes_category_facades() {
    use nive_ui::widgets::{containers, controls, display, navigation, overlays, primitives};

    let _: controls::ButtonIntent = controls::ButtonIntent::Suggested;
    let _: controls::ButtonVariant = controls::ButtonVariant::Solid;
    let _: Element<'_, ()> = controls::Checkbox::new("Enabled", true).into();
    let _: Element<'_, ()> = controls::Field::new("Name", controls::Input::new("Name", "")).into();
    let _: Element<'_, ()> = containers::Panel::new(text("Panel")).into();
    let _: Element<'_, ()> = containers::SectionHeader::new("Title").into();
    let _: Element<'_, ()> = display::Badge::status("Ready").success().into();
    let _: Element<'_, ()> = display::Badge::count(3).into();
    let _: Element<'_, ()> = display::MetadataTag::code("1.4.0-beta.2").into();
    let _: Element<'_, ()> = display::InitialAvatar::new("Ada Lovelace").person().into();
    let _: Element<'_, ()> = navigation::Toolbar::new().into();
    let _: Element<'_, ()> = navigation::VerticalRail::new(navigation::RailSide::Left)
        .on_select(|_: &str| ())
        .item(
            navigation::VerticalRailItem::new("explorer", "Explorer")
                .badge(navigation::VerticalRailBadge::count(3).description("3 open explorers")),
        )
        .into();
    let _: Element<'_, ()> = navigation::TabBar::new("overview")
        .tab(navigation::TabItem::new("overview", "Overview").closable(true))
        .active_role(theme::SurfaceRole::Canvas)
        .on_select(|_: &str| ())
        .into();
    let _: Element<'_, ()> = controls::SelectableItem::new("Result")
        .selected(true)
        .trailing_text("12")
        .into();
    let _: Element<'_, ()> = controls::ActionGroup::new()
        .action(controls::ContentAction::icon(
            primitives::IconRole::ViewRefresh,
            "Refresh",
        ))
        .into();
    let _: Element<'_, ()> = overlays::Dialog::new(text("Dialog")).into();
    let _: Element<'_, ()> = primitives::Separator::horizontal().into();
    let _: Element<'_, ()> = primitives::ToneDot::new(theme::roles::ToneRole::Accent)
        .xs()
        .into();
    let _: Element<'_, ()> =
        primitives::StatusIndicator::new(theme::roles::ToneRole::Success, "Healthy").into();
}

#[test]
fn top_level_ui_facades_expose_layout_graphics_and_accessibility() {
    let _: Element<'_, ()> = nive_ui::layout::Panel::new(text("Panel")).into();
    let _: Element<'_, ()> =
        nive_ui::graphics::Icon::role(nive_ui::graphics::IconRole::EditFind).into();
    let _: nive_ui::graphics::IconGlyph =
        Theme::Light.icon(nive_ui::graphics::IconRole::WindowClose);
    let _: fn(&iced::Event) -> Option<nive_ui::accessibility::FocusDirection> =
        nive_ui::accessibility::direction_from_event;
}

#[test]
fn command_palette_exposes_filter_view_and_row_types() {
    let save = CommandPaletteRow::new("file.save", "Save", ()).description("Persist the buffer");
    let open = CommandPaletteRow::new("file.open", "Open", ());
    let rows = [save, open];

    assert_eq!(command_palette_filter("save", &rows), vec![0]);
    assert_eq!(command_palette_filter("", &rows), vec![0, 1]);

    let _: Element<'_, ()> =
        command_palette_view("Type a command", "", rows, Some(0), |_| (), None);
}

#[test]
fn canonical_menu_projects_core_actions_from_the_ui_prelude() {
    let action = nive_core::Action::new("file.save", "Save", ())
        .shortcut(nive_core::ShortcutBinding::primary_character('s'));
    let child: Menu<'_, ()> =
        Menu::new(button("More")).command(MenuCommand::new("Child").on_press(()));
    let _: Element<'_, ()> = Menu::new(button("Actions"))
        .open(false)
        .on_dismiss(())
        .command(MenuCommand::from_action(&action).icon(IconRole::ActionConfirm))
        .checkbox(
            MenuCheckbox::new("Pinned", CheckboxState::Unchecked)
                .on_toggle(|_| ())
                .dismiss_policy(MenuDismissPolicy::KeepOpen),
        )
        .radio_group(
            MenuRadioGroup::new(Some(1))
                .option(MenuRadioOption::new(1, "One"))
                .option(MenuRadioOption::new(2, "Two").annotation("Secondary"))
                .on_select(|_| ()),
        )
        .separator()
        .submenu(MenuSubmenu::new("More", child))
        .into();
}

#[test]
fn feedback_presentation_contracts_are_reexported_from_nive_core() {
    use nive_ui::widgets::overlays::{ToastPresentation, ToastTone};
    use nive_ui::widgets::{
        ErrorPresentation, OperationStatusPresentation, ResourceStatusPresentation,
    };

    struct NoError;

    impl nive_core::ErrorPresentation for NoError {
        fn summary(&self) -> &str {
            "summary"
        }

        fn detail(&self) -> &str {
            "detail"
        }
    }

    struct NoResource;

    impl nive_core::ResourceStatusPresentation for NoResource {
        fn is_refreshing(&self) -> bool {
            false
        }

        fn has_value(&self) -> bool {
            false
        }

        fn error(&self) -> Option<&dyn nive_core::ErrorPresentation> {
            None
        }
    }

    struct NoOperation;

    impl nive_core::OperationStatusPresentation for NoOperation {
        fn is_running(&self) -> bool {
            false
        }

        fn error(&self) -> Option<&dyn nive_core::ErrorPresentation> {
            None
        }
    }

    struct NoToast;

    impl nive_core::ToastPresentation for NoToast {
        type Id = u64;

        fn id(&self) -> u64 {
            0
        }

        fn title(&self) -> &str {
            "title"
        }

        fn body(&self) -> Option<&str> {
            None
        }

        fn tone(&self) -> nive_core::ToastTone {
            nive_core::ToastTone::Info
        }
    }

    // Each assertion only compiles if `nive_ui`'s facade name is the exact
    // same trait/type as `nive_core`'s, not a duplicate local definition.
    fn assert_error<T: ErrorPresentation>() {}
    fn assert_resource_status<T: ResourceStatusPresentation>() {}
    fn assert_operation_status<T: OperationStatusPresentation>() {}
    fn assert_toast<T: ToastPresentation>() {}

    assert_error::<NoError>();
    assert_resource_status::<NoResource>();
    assert_operation_status::<NoOperation>();
    assert_toast::<NoToast>();
    let _: ToastTone = nive_core::ToastTone::Success;
}

#[test]
fn theme_facade_builds_product_catalogs() {
    let light = Theme::builder("Contract Light", theme::ThemeMode::Light)
        .accent(theme::hex(0x0EA5E9))
        .build();
    let dark = Theme::builder("Contract Dark", theme::ThemeMode::Dark)
        .accent(theme::hex(0x38BDF8))
        .build();
    let catalog = ThemeCatalog::new(light, dark);

    assert_eq!(
        catalog.resolve(theme::ThemeMode::Light).name(),
        "Contract Light"
    );
    assert_eq!(
        catalog.resolve(theme::ThemeMode::Dark).name(),
        "Contract Dark"
    );
}

#[test]
fn density_aware_theme_builder_compiles() {
    let theme = ThemeBuilder::new("Density Contract", theme::ThemeMode::Light)
        .density(ThemeDensity::Compact)
        .build();

    assert_eq!(theme.density(), ThemeDensity::Compact);
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DialogMessage {
    Close,
    Save,
    Delete,
}

#[test]
fn prelude_exposes_the_complete_canonical_dialog_family_contract() {
    let owned_title = String::from("Owned title");

    // Every DialogSize, Cow-owned/borrowed header copy, an icon, and a safe
    // named close affordance.
    for size in [DialogSize::Sm, DialogSize::Md, DialogSize::Lg] {
        let header = DialogHeader::new(owned_title.clone())
            .description("Borrowed description")
            .icon(IconRole::DialogWarning)
            .close("Close dialog", DialogMessage::Close);

        let _: Element<'_, DialogMessage> =
            Dialog::new(text("Body")).size(size).header(header).into();
    }

    // Plain DialogFooter for non-action footer content.
    let _: Element<'_, DialogMessage> = Dialog::new(text("Body"))
        .footer(DialogFooter::new(text("Custom footer content")))
        .into();

    // Zero, one, and two preceding actions plus one required terminal
    // action, through every DialogActionRole and every ergonomic static
    // constructor.
    let _: Element<'_, DialogMessage> = Dialog::new(text("Body"))
        .footer(DialogActionFooter::new(DialogTerminalAction::primary(
            "Save",
            DialogMessage::Save,
        )))
        .into();

    let _: Element<'_, DialogMessage> = Dialog::new(text("Body"))
        .footer(DialogActionFooter::with_one(
            DialogAction::cancel("Cancel", DialogMessage::Close),
            DialogTerminalAction::primary("Save", DialogMessage::Save),
        ))
        .into();

    let _: Element<'_, DialogMessage> = Dialog::new(text("Body"))
        .footer(
            DialogActionFooter::with_two(
                [
                    DialogAction::cancel("Cancel", DialogMessage::Close),
                    DialogAction::secondary("More", DialogMessage::Close).disabled(true),
                ],
                DialogTerminalAction::destructive("Delete", DialogMessage::Delete).disabled(false),
            )
            .status(text("Status/help content")),
        )
        .into();

    // Fallible dynamic construction and its typed error.
    let dynamic: Result<DialogActionFooter<'_, DialogMessage>, DialogActionFooterError> =
        DialogActionFooter::try_from_parts(
            vec![DialogAction::cancel("Cancel", DialogMessage::Close)],
            DialogTerminalAction::primary("Save", DialogMessage::Save),
        );
    assert!(dynamic.is_ok());

    // DialogInitialFocus and a stable action Id.
    let _: DialogInitialFocus = DialogInitialFocus::First;
    let _: DialogInitialFocus = DialogInitialFocus::Target(iced::widget::Id::new("field"));
    let _ =
        DialogAction::cancel("Cancel", DialogMessage::Close).id(iced::widget::Id::new("cancel"));

    // Canonical UI-only hosting: DialogHost with initial focus and a stable
    // declarative session id, entirely through `nive_ui::prelude::*`.
    let dialog: Element<'_, DialogMessage> = Dialog::new(text("Body"))
        .footer(DialogActionFooter::new(DialogTerminalAction::primary(
            "Save",
            DialogMessage::Save,
        )))
        .into();
    let _: Element<'_, DialogMessage> = DialogHost::new(text("Base content"))
        .dialog(
            dialog,
            Some(DialogMessage::Close),
            Some(DialogMessage::Close),
            DialogInitialFocus::Target(iced::widget::Id::new("field")),
        )
        .dialog_id(iced::widget::Id::new("workflow-step"))
        .into();

    // ErrorDetailsDialog reuses the same canonical composition.
    let _: Element<'_, DialogMessage> = ErrorDetailsDialog::new("stack trace")
        .on_close(DialogMessage::Close)
        .into();
}
