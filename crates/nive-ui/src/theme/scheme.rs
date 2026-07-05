use std::sync::LazyLock;

use iced::Padding;

use crate::icons::{self, IconCatalog, IconGlyph, IconRole};

use super::color_scheme::{BorderSpec, ColorScheme, ControlSpec, SurfaceSpec, TextSpec, ToneSpec};
use super::component::{self, ControlMetrics, ControlMetricsScale, ControlSize};
use super::shape::{self, ShapeRole, ShapeScale, ShapeSpec};
use super::spacing::{self, GapRole, PaddingRole, SpaceStep, SpacingScale};
use super::typography::{self, TextStyle, TypographyRole, TypographyScale};
use super::{BorderRole, ControlRole, ControlState, SurfaceRole, TextRole, ThemeMode, ToneRole};

#[derive(Debug, Clone, Copy)]
pub enum Theme {
    Light,
    Dark,
    Custom(&'static ThemeData),
}

impl PartialEq for Theme {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Self::Light, Self::Light) | (Self::Dark, Self::Dark) => true,
            (Self::Custom(left), Self::Custom(right)) => std::ptr::eq(left, right),
            _ => false,
        }
    }
}

impl Eq for Theme {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeId {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeCatalog {
    light: Theme,
    dark: Theme,
}

impl ThemeCatalog {
    pub const NIVE: Self = Self::new(Theme::Light, Theme::Dark);

    pub const fn new(light: Theme, dark: Theme) -> Self {
        Self { light, dark }
    }

    pub fn theme(self, id: ThemeId) -> Theme {
        match id {
            ThemeId::Light => self.light,
            ThemeId::Dark => self.dark,
        }
    }

    pub fn get(self, id: ThemeId) -> &'static ThemeData {
        self.theme(id).data()
    }

    pub fn resolve(self, mode: ThemeMode) -> Theme {
        match mode {
            ThemeMode::Light => self.light,
            ThemeMode::Dark => self.dark,
        }
    }
}

impl Default for ThemeCatalog {
    fn default() -> Self {
        Self::NIVE
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeData {
    pub name: &'static str,
    pub mode: ThemeMode,
    pub color_scheme: ColorScheme,
    pub typography: TypographyScale,
    pub shapes: ShapeScale,
    pub spacing: SpacingScale,
    pub controls: ControlMetricsScale,
    pub icons: IconCatalog,
}

static LIGHT_THEME_DATA: LazyLock<ThemeData> = LazyLock::new(|| ThemeData::new(ThemeMode::Light));
static DARK_THEME_DATA: LazyLock<ThemeData> = LazyLock::new(|| ThemeData::new(ThemeMode::Dark));

impl Theme {
    pub fn builder(name: &'static str, mode: ThemeMode) -> super::ThemeBuilder {
        super::ThemeBuilder::new(name, mode)
    }

    pub fn custom(data: ThemeData) -> Self {
        Self::Custom(Box::leak(Box::new(data)))
    }

    pub const fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self::Light,
            ThemeMode::Dark => Self::Dark,
        }
    }

    pub const fn from_system(mode: iced::theme::Mode) -> Self {
        Self::from_mode(ThemeMode::from_system(mode))
    }

    pub const fn mode(self) -> ThemeMode {
        match self {
            Self::Light => ThemeMode::Light,
            Self::Dark => ThemeMode::Dark,
            Self::Custom(data) => data.mode,
        }
    }

    pub fn data(self) -> &'static ThemeData {
        match self {
            Self::Light => &LIGHT_THEME_DATA,
            Self::Dark => &DARK_THEME_DATA,
            Self::Custom(data) => data,
        }
    }

    pub fn name(self) -> &'static str {
        self.data().name
    }

    pub fn color_scheme(self) -> &'static ColorScheme {
        &self.data().color_scheme
    }

    pub fn is_dark(self) -> bool {
        self.color_scheme().is_dark()
    }

    pub fn surface(self, role: SurfaceRole) -> SurfaceSpec {
        self.color_scheme().surface(role)
    }

    pub fn text(self, role: TextRole) -> TextSpec {
        self.color_scheme().text(role)
    }

    pub fn border(self, role: BorderRole) -> BorderSpec {
        self.color_scheme().border(role)
    }

    pub fn control(self, role: ControlRole, state: ControlState) -> ControlSpec {
        self.color_scheme().control(role, state)
    }

    pub fn tone(self, role: ToneRole) -> ToneSpec {
        self.color_scheme().tone(role)
    }

    pub fn typography_scale(self) -> TypographyScale {
        self.data().typography
    }

    pub fn typography(self, role: TypographyRole) -> TextStyle {
        self.typography_scale().get(role)
    }

    pub fn shapes(self) -> ShapeScale {
        self.data().shapes
    }

    pub fn shape(self, role: ShapeRole) -> ShapeSpec {
        self.shapes().get(role)
    }

    pub fn spacing(self) -> SpacingScale {
        self.data().spacing
    }

    pub fn controls(self) -> ControlMetricsScale {
        self.data().controls
    }

    pub fn icons(self) -> IconCatalog {
        self.data().icons
    }

    pub fn icon(self, role: IconRole) -> IconGlyph {
        self.icons()
            .glyph(role)
            .unwrap_or_else(|| icons::lucide::glyph_for(role))
    }

    pub fn control_metrics(self, size: ControlSize) -> ControlMetrics {
        self.controls().get(size)
    }

    pub fn space(self, step: SpaceStep) -> f32 {
        self.spacing().step(step)
    }

    pub fn gap(self, role: GapRole) -> f32 {
        self.spacing().gap(role)
    }

    pub fn padding(self, role: PaddingRole) -> Padding {
        self.spacing().padding(role)
    }
}

impl ThemeData {
    fn new(mode: ThemeMode) -> Self {
        let typography = typography::scale();
        let shapes = shape::scale();
        let spacing = spacing::scale();
        let controls = component::scale(shapes, typography, spacing);

        Self {
            name: mode.name(),
            mode,
            color_scheme: ColorScheme::from_mode(mode),
            typography,
            shapes,
            spacing,
            controls,
            icons: icons::lucide::default_catalog(),
        }
    }
}

impl iced::theme::Base for Theme {
    fn default(preference: iced::theme::Mode) -> Self {
        Self::from_system(preference)
    }

    fn mode(&self) -> iced::theme::Mode {
        match Theme::mode(*self) {
            ThemeMode::Light => iced::theme::Mode::Light,
            ThemeMode::Dark => iced::theme::Mode::Dark,
        }
    }

    fn base(&self) -> iced::theme::Style {
        iced::theme::Style {
            background_color: self.surface(SurfaceRole::App).background,
            text_color: self.text(TextRole::Primary).color,
        }
    }

    fn palette(&self) -> Option<iced::theme::Palette> {
        Some(iced::theme::Palette {
            background: self.surface(SurfaceRole::App).background,
            text: self.text(TextRole::Primary).color,
            primary: self.tone(ToneRole::Primary).color,
            success: self.tone(ToneRole::Success).color,
            warning: self.tone(ToneRole::Warning).color,
            danger: self.tone(ToneRole::Danger).color,
        })
    }

    fn name(&self) -> &str {
        self.data().name
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_mode(ThemeMode::default())
    }
}

#[cfg(test)]
mod theme_tests {
    use super::*;
    use iced::theme::Base as _;

    #[test]
    fn active_theme_helpers_follow_guarded_theme() {
        use super::super::active::{active, gap, padding, ThemeTestGuard};

        let _guard = ThemeTestGuard::activate(Theme::Light);
        assert_eq!(active(), Theme::Light);
        assert_eq!(active().mode(), ThemeMode::Light);
        assert_eq!(gap(GapRole::Related), Theme::Light.gap(GapRole::Related));
        assert_eq!(
            padding(PaddingRole::Panel),
            Theme::Light.padding(PaddingRole::Panel)
        );
    }

    #[test]
    fn maps_system_mode_to_effective_theme() {
        assert_eq!(Theme::from_system(iced::theme::Mode::Light), Theme::Light);
        assert_eq!(Theme::from_system(iced::theme::Mode::Dark), Theme::Dark);
        assert_eq!(
            Theme::from_system(iced::theme::Mode::None),
            <Theme as Default>::default()
        );
    }

    #[test]
    fn base_default_respects_app_preference() {
        assert_eq!(
            <Theme as iced::theme::Base>::default(iced::theme::Mode::Light),
            Theme::Light
        );
        assert_eq!(
            <Theme as iced::theme::Base>::default(iced::theme::Mode::Dark),
            Theme::Dark
        );
        assert_eq!(
            <Theme as iced::theme::Base>::default(iced::theme::Mode::None),
            <Theme as Default>::default()
        );
    }

    #[test]
    fn base_style_uses_semantic_app_surface() {
        let theme = Theme::Dark;
        let style = theme.base();

        assert_eq!(
            style.background_color,
            theme.surface(SurfaceRole::App).background
        );
        assert_eq!(style.text_color, theme.text(TextRole::Primary).color);
    }

    #[test]
    fn static_theme_data_matches_effective_mode() {
        assert_eq!(Theme::Light.data().mode, ThemeMode::Light);
        assert_eq!(Theme::Dark.data().mode, ThemeMode::Dark);
        assert_eq!(Theme::Light.data().name, ThemeMode::Light.name());
        assert_eq!(Theme::Dark.data().name, ThemeMode::Dark.name());
    }

    #[test]
    fn control_metrics_resolve_from_theme_data() {
        assert_eq!(
            Theme::Dark.control_metrics(ControlSize::Md),
            Theme::Dark.data().controls.get(ControlSize::Md)
        );
    }

    #[test]
    fn built_in_themes_expose_default_icon_catalog() {
        for role in IconRole::ALL {
            assert_eq!(Theme::Light.icon(*role), icons::lucide::glyph_for(*role));
            assert_eq!(Theme::Dark.icon(*role), icons::lucide::glyph_for(*role));
        }
    }

    #[test]
    fn custom_theme_icon_catalog_overrides_framework_role() {
        const GLYPH: IconGlyph = IconGlyph::new(
            br#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#,
            "custom:close",
        );
        const CATALOG: IconCatalog = IconCatalog::new(&[crate::icons::IconCatalogEntry::new(
            IconRole::WindowClose,
            GLYPH,
        )]);

        let theme = Theme::builder("Acme", ThemeMode::Light)
            .icons(CATALOG)
            .build();

        assert_eq!(theme.icon(IconRole::WindowClose), GLYPH);
        assert_ne!(
            theme.icon(IconRole::WindowClose),
            icons::lucide::glyph_for(IconRole::WindowClose)
        );
    }

    #[test]
    fn base_palette_and_name_follow_mode() {
        assert_eq!(
            <Theme as iced::theme::Base>::mode(&Theme::Light),
            iced::theme::Mode::Light
        );
        assert_eq!(
            <Theme as iced::theme::Base>::mode(&Theme::Dark),
            iced::theme::Mode::Dark
        );
        assert_eq!(Theme::Light.name(), ThemeMode::Light.name());
        assert_eq!(Theme::Dark.name(), ThemeMode::Dark.name());
        assert_eq!(
            Theme::Light.palette(),
            Some(palette_from_theme(Theme::Light))
        );
        assert_eq!(Theme::Dark.palette(), Some(palette_from_theme(Theme::Dark)));
    }

    #[test]
    fn catalog_resolves_custom_themes_by_mode() {
        let light = Theme::builder("Acme Light", ThemeMode::Light).build();
        let dark = Theme::builder("Acme Dark", ThemeMode::Dark).build();
        let catalog = ThemeCatalog::new(light, dark);

        assert_eq!(catalog.resolve(ThemeMode::Light).name(), "Acme Light");
        assert_eq!(catalog.resolve(ThemeMode::Dark).name(), "Acme Dark");
        assert_eq!(catalog.get(ThemeId::Light).name, "Acme Light");
    }

    fn palette_from_theme(theme: Theme) -> iced::theme::Palette {
        iced::theme::Palette {
            background: theme.surface(SurfaceRole::App).background,
            text: theme.text(TextRole::Primary).color,
            primary: theme.tone(ToneRole::Primary).color,
            success: theme.tone(ToneRole::Success).color,
            warning: theme.tone(ToneRole::Warning).color,
            danger: theme.tone(ToneRole::Danger).color,
        }
    }
}
