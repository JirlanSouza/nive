use iced::theme::{Base as _, Palette};

use crate::icons::{self, IconCatalog};

use super::color_scheme::ColorScheme;
use super::component::{self, ControlMetricsScale};
use super::shape::{self, ShapeScale};
use super::spacing::{self, SpacingScale};
use super::typography::{self, TypographyScale};
use super::{Theme, ThemeData, ThemeMode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeBuilder {
    name: &'static str,
    mode: ThemeMode,
    palette: Palette,
    typography: TypographyScale,
    shapes: ShapeScale,
    spacing: SpacingScale,
    controls: Option<ControlMetricsScale>,
    icons: IconCatalog,
}

impl ThemeBuilder {
    pub fn new(name: &'static str, mode: ThemeMode) -> Self {
        Self {
            name,
            mode,
            palette: super::palette::palette(mode),
            typography: typography::scale(),
            shapes: shape::scale(),
            spacing: spacing::scale(),
            controls: None,
            icons: icons::lucide::default_catalog(),
        }
    }

    pub fn from_theme(theme: Theme) -> Self {
        let data = theme.data();

        Self {
            name: data.name,
            mode: data.mode,
            palette: theme
                .palette()
                .unwrap_or_else(|| super::palette::palette(data.mode)),
            typography: data.typography,
            shapes: data.shapes,
            spacing: data.spacing,
            controls: Some(data.controls),
            icons: data.icons,
        }
    }

    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub fn mode(mut self, mode: ThemeMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn palette(mut self, palette: Palette) -> Self {
        self.palette = palette;
        self
    }

    pub fn app_background(mut self, color: iced::Color) -> Self {
        self.palette.background = color;
        self
    }

    pub fn text(mut self, color: iced::Color) -> Self {
        self.palette.text = color;
        self
    }

    pub fn primary(mut self, color: iced::Color) -> Self {
        self.palette.primary = color;
        self
    }

    pub fn success(mut self, color: iced::Color) -> Self {
        self.palette.success = color;
        self
    }

    pub fn warning(mut self, color: iced::Color) -> Self {
        self.palette.warning = color;
        self
    }

    pub fn danger(mut self, color: iced::Color) -> Self {
        self.palette.danger = color;
        self
    }

    pub fn typography(mut self, typography: TypographyScale) -> Self {
        self.typography = typography;
        self.controls = None;
        self
    }

    pub fn shapes(mut self, shapes: ShapeScale) -> Self {
        self.shapes = shapes;
        self.controls = None;
        self
    }

    pub fn spacing(mut self, spacing: SpacingScale) -> Self {
        self.spacing = spacing;
        self.controls = None;
        self
    }

    pub fn controls(mut self, controls: ControlMetricsScale) -> Self {
        self.controls = Some(controls);
        self
    }

    pub fn icons(mut self, icons: IconCatalog) -> Self {
        self.icons = icons;
        self
    }

    pub fn build_data(self) -> ThemeData {
        let controls = self
            .controls
            .unwrap_or_else(|| component::scale(self.shapes, self.typography, self.spacing));

        ThemeData {
            name: self.name,
            mode: self.mode,
            color_scheme: ColorScheme::from_palette(
                self.palette,
                matches!(self.mode, ThemeMode::Dark),
            ),
            typography: self.typography,
            shapes: self.shapes,
            spacing: self.spacing,
            controls,
            icons: self.icons,
        }
    }

    pub fn build(self) -> Theme {
        Theme::custom(self.build_data())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{SurfaceRole, ToneRole};
    use crate::tokens::color::hex;

    #[test]
    fn builder_creates_named_custom_theme_from_palette_overrides() {
        let theme = ThemeBuilder::new("Acme Dark", ThemeMode::Dark)
            .app_background(hex(0x101820))
            .text(hex(0xF6F7FB))
            .primary(hex(0x0EA5E9))
            .warning(hex(0xF59E0B))
            .build();

        assert_eq!(theme.name(), "Acme Dark");
        assert_eq!(theme.mode(), ThemeMode::Dark);
        assert_eq!(theme.surface(SurfaceRole::App).background, hex(0x101820));
        assert_eq!(theme.tone(ToneRole::Primary).color, hex(0x0EA5E9));
        assert_eq!(theme.tone(ToneRole::Warning).color, hex(0xF59E0B));
    }

    #[test]
    fn scale_overrides_recompute_control_metrics() {
        let mut spacing = spacing::scale();
        spacing.lg = 20.0;

        let theme = ThemeBuilder::new("Spacious", ThemeMode::Light)
            .spacing(spacing)
            .build();

        assert_eq!(theme.spacing().lg, 20.0);
        assert_eq!(
            theme.control_metrics(super::super::ControlSize::Md).gap,
            spacing.sm
        );
    }
}
