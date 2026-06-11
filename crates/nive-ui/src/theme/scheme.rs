use std::sync::{atomic::AtomicU8, LazyLock};

use iced::Padding;

use super::color_scheme::{BorderSpec, ColorScheme, ControlSpec, SurfaceSpec, TextSpec, ToneSpec};
use super::component::{self, ControlMetrics, ControlMetricsScale, ControlSize};
use super::shape::{self, ShapeRole, ShapeScale, ShapeSpec};
use super::spacing::{self, GapRole, PaddingRole, SpaceStep, SpacingScale};
use super::typography::{self, TextStyle, TypographyRole, TypographyScale};
use super::{BorderRole, ControlRole, ControlState, SurfaceRole, TextRole, ThemeMode, ToneRole};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light = 0,
    Dark = 1,
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
}

pub(super) static ACTIVE_THEME: AtomicU8 = AtomicU8::new(Theme::Dark as u8);

static LIGHT_THEME_DATA: LazyLock<ThemeData> = LazyLock::new(|| ThemeData::new(ThemeMode::Light));
static DARK_THEME_DATA: LazyLock<ThemeData> = LazyLock::new(|| ThemeData::new(ThemeMode::Dark));

impl Theme {
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
        }
    }

    pub fn data(self) -> &'static ThemeData {
        match self {
            Self::Light => &LIGHT_THEME_DATA,
            Self::Dark => &DARK_THEME_DATA,
        }
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

    pub(super) const fn from_active_value(value: u8) -> Self {
        match value {
            0 => Self::Light,
            1 => Self::Dark,
            _ => Self::Dark,
        }
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
        Some(super::palette::palette(Theme::mode(*self)))
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
    fn active_theme_updates_without_locking() {
        use super::super::active::{active, active_mode, gap, padding, set_active};

        set_active(Theme::Light);
        assert_eq!(active(), Theme::Light);
        assert_eq!(active_mode(), ThemeMode::Light);
        assert_eq!(gap(GapRole::Related), Theme::Light.gap(GapRole::Related));
        assert_eq!(
            padding(PaddingRole::Panel),
            Theme::Light.padding(PaddingRole::Panel)
        );

        set_active(Theme::Dark);
        assert_eq!(active(), Theme::Dark);
        assert_eq!(active_mode(), ThemeMode::Dark);
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
            Some(super::super::palette::palette(ThemeMode::Light))
        );
        assert_eq!(
            Theme::Dark.palette(),
            Some(super::super::palette::palette(ThemeMode::Dark))
        );
    }
}
