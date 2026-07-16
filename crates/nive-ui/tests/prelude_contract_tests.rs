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
    let scale = theme::typography::scale();
    let _: theme::TextStyle = scale.badge_label;
    let _: theme::TextStyle = scale.metadata_tag;
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
    let _: Element<'_, ()> = Field::new(text_input("Name", "")).label("Name").into();
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
fn widget_taxonomy_exposes_category_facades() {
    use nive_ui::widgets::{containers, controls, display, navigation, overlays, primitives};

    let _: controls::ButtonIntent = controls::ButtonIntent::Suggested;
    let _: controls::ButtonVariant = controls::ButtonVariant::Solid;
    let _: Element<'_, ()> = controls::Checkbox::new("Enabled", true).into();
    let _: Element<'_, ()> = controls::Field::new(text_input("Name", "")).into();
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
