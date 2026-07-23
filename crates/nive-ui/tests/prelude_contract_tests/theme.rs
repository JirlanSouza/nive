use nive_ui::prelude::*;

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
