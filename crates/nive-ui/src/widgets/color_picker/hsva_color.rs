use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct HsvaColor {
    hue: f32,
    saturation: f32,
    value: f32,
    alpha: f32,
}

impl HsvaColor {
    pub(super) fn from_color(color: Color) -> Self {
        let max = color.r.max(color.g).max(color.b);
        let min = color.r.min(color.g).min(color.b);
        let delta = max - min;

        let hue = if delta == 0.0 {
            0.0
        } else if max == color.r {
            60.0 * ((color.g - color.b) / delta).rem_euclid(6.0)
        } else if max == color.g {
            60.0 * (((color.b - color.r) / delta) + 2.0)
        } else {
            60.0 * (((color.r - color.g) / delta) + 4.0)
        };

        let saturation = if max == 0.0 { 0.0 } else { delta / max };

        Self {
            hue: normalize_hue(hue),
            saturation: saturation.clamp(0.0, 1.0),
            value: max.clamp(0.0, 1.0),
            alpha: color.a.clamp(0.0, 1.0),
        }
    }

    pub(super) fn from_color_preserving_hue(color: Color, fallback_hue: f32) -> Self {
        let mut hsva = Self::from_color(color);

        if !hsva.has_defined_hue() {
            hsva.hue = normalize_hue(fallback_hue);
        }

        hsva
    }

    pub(super) fn hue_color(hue: f32) -> Color {
        Self {
            hue: normalize_hue(hue),
            saturation: 1.0,
            value: 1.0,
            alpha: 1.0,
        }
        .to_color()
    }

    pub(super) fn to_color(self) -> Color {
        let chroma = self.value * self.saturation;
        let hue = normalize_hue(self.hue);
        let x = chroma * (1.0 - ((hue / 60.0).rem_euclid(2.0) - 1.0).abs());
        let m = self.value - chroma;

        let (r, g, b) = match hue {
            h if h < 60.0 => (chroma, x, 0.0),
            h if h < 120.0 => (x, chroma, 0.0),
            h if h < 180.0 => (0.0, chroma, x),
            h if h < 240.0 => (0.0, x, chroma),
            h if h < 300.0 => (x, 0.0, chroma),
            _ => (chroma, 0.0, x),
        };

        Color::from_rgba(
            (r + m).clamp(0.0, 1.0),
            (g + m).clamp(0.0, 1.0),
            (b + m).clamp(0.0, 1.0),
            self.alpha.clamp(0.0, 1.0),
        )
    }

    pub(super) fn hue(self) -> f32 {
        self.hue
    }

    pub(super) fn saturation(self) -> f32 {
        self.saturation
    }

    pub(super) fn value(self) -> f32 {
        self.value
    }

    pub(super) fn alpha(self) -> f32 {
        self.alpha
    }

    pub(super) fn with_hue(self, hue: f32) -> Self {
        Self {
            hue: normalize_hue(hue),
            ..self
        }
    }

    pub(super) fn with_saturation_value(self, saturation: f32, value: f32) -> Self {
        Self {
            saturation: saturation.clamp(0.0, 1.0),
            value: value.clamp(0.0, 1.0),
            ..self
        }
    }

    pub(super) fn with_alpha(self, alpha: f32) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0),
            ..self
        }
    }

    fn has_defined_hue(self) -> bool {
        self.saturation > 0.0 && self.value > 0.0
    }
}

fn normalize_hue(hue: f32) -> f32 {
    if hue.is_finite() {
        hue.rem_euclid(360.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod hsva_color_tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.01, "{actual} != {expected}");
    }

    #[test]
    fn converts_primary_colors_from_rgb() {
        assert_close(
            HsvaColor::from_color(Color::from_rgb(1.0, 0.0, 0.0)).hue(),
            0.0,
        );
        assert_close(
            HsvaColor::from_color(Color::from_rgb(0.0, 1.0, 0.0)).hue(),
            120.0,
        );
        assert_close(
            HsvaColor::from_color(Color::from_rgb(0.0, 0.0, 1.0)).hue(),
            240.0,
        );
    }

    #[test]
    fn converts_hsva_to_rgb() {
        let color = HsvaColor {
            hue: 300.0,
            saturation: 1.0,
            value: 1.0,
            alpha: 0.5,
        }
        .to_color();

        assert_close(color.r, 1.0);
        assert_close(color.g, 0.0);
        assert_close(color.b, 1.0);
        assert_close(color.a, 0.5);
    }

    #[test]
    fn round_trips_representative_colors() {
        let color = Color::from_rgba(0.2, 0.58, 0.72, 0.8);
        let actual = HsvaColor::from_color(color).to_color();

        assert_close(actual.r, color.r);
        assert_close(actual.g, color.g);
        assert_close(actual.b, color.b);
        assert_close(actual.a, color.a);
    }

    #[test]
    fn setters_clamp_values() {
        let color = HsvaColor::from_color(Color::WHITE)
            .with_hue(725.0)
            .with_saturation_value(2.0, -1.0)
            .with_alpha(2.0);

        assert_close(color.hue(), 5.0);
        assert_close(color.saturation(), 1.0);
        assert_close(color.value(), 0.0);
        assert_close(color.alpha(), 1.0);
    }

    #[test]
    fn preserves_fallback_hue_for_achromatic_colors() {
        assert_close(
            HsvaColor::from_color_preserving_hue(Color::from_rgb(0.5, 0.5, 0.5), 275.0).hue(),
            275.0,
        );
        assert_close(
            HsvaColor::from_color_preserving_hue(Color::BLACK, 125.0).hue(),
            125.0,
        );
    }
}
