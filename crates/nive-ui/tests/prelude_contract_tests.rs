use nive_ui::prelude::*;

#[test]
fn prelude_exposes_common_ui_contracts() {
    let _: Element<'_, ()> = text("Nive").into();
    let _: Theme = theme::active();
    let _: ThemePreference = ThemePreference::System;
    let _: Color = Color::TRANSPARENT;
    let _: Background = Background::Color(Color::TRANSPARENT);
    let _: Border = Border::default();
    let _: Shadow = Shadow::default();
}

#[test]
fn prelude_exposes_common_widget_contracts() {
    let _: ButtonVariant = ButtonVariant::Primary;
    let _: Element<'_, ()> = Card::new(text("Card")).into();
    let _: Element<'_, ()> = Field::new(text_input("Name", "")).label("Name").into();
    let _: Element<'_, ()> = Dialog::new(text("Dialog")).into();
    let _: Element<'_, ()> = EmptyState::new("No results").into();
    let _: Element<'_, ()> = Separator::horizontal().into();
}

#[test]
fn command_palette_exposes_filter_view_and_row_types() {
    let save = CommandPaletteRow::new("file.save", "Save", ()).description("Persist the buffer");
    let open = CommandPaletteRow::new("file.open", "Open", ());
    let rows = [save, open];

    assert_eq!(command_palette_filter("save", &rows), vec![0]);
    assert_eq!(command_palette_filter("", &rows), vec![0, 1]);

    let _: Element<'_, ()> =
        command_palette_view("Type a command", "", &rows, Some(0), |_| (), None);
}

#[test]
fn theme_facade_builds_product_catalogs() {
    let light = Theme::builder("Contract Light", theme::ThemeMode::Light)
        .primary(theme::hex(0x0EA5E9))
        .build();
    let dark = Theme::builder("Contract Dark", theme::ThemeMode::Dark)
        .primary(theme::hex(0x38BDF8))
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
