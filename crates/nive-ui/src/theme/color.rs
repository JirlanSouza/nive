use iced::Color;

use crate::tokens::color::hex;

pub(crate) const DARK_BACKGROUND: Color = hex(0x0D0D0F);
pub(crate) const DARK_FOREGROUND: Color = hex(0xCBCBD0);
pub(crate) const PRIMARY: Color = hex(0x6E4DAF);
pub(crate) const STATUS_READY: Color = hex(0x4ADE80);
pub(crate) const DESTRUCTIVE: Color = hex(0xF87171);

/// WCAG relative luminance (sRGB, gamma-corrected).
///
/// Used as the basis for [`contrast_ratio`] and [`perceptual_lightness`].
pub fn relative_luminance(color: Color) -> f32 {
    0.2126 * linear_channel(color.r)
        + 0.7152 * linear_channel(color.g)
        + 0.0722 * linear_channel(color.b)
}

fn linear_channel(value: f32) -> f32 {
    if value <= 0.03928 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG contrast ratio between two colors, in `[1.0, 21.0]`.
pub fn contrast_ratio(first: Color, second: Color) -> f32 {
    let first_luminance = relative_luminance(first);
    let second_luminance = relative_luminance(second);
    let lighter = first_luminance.max(second_luminance);
    let darker = first_luminance.min(second_luminance);

    (lighter + 0.05) / (darker + 0.05)
}

/// CIE L* perceptual lightness (`[0.0, 100.0]`) derived from relative luminance.
///
/// This is what makes adjacent surface fills *look* evenly spaced, unlike raw
/// linear luminance which compresses differences at the dark end.
pub fn perceptual_lightness(color: Color) -> f32 {
    const EPSILON: f32 = 216.0 / 24389.0;
    const KAPPA: f32 = 24389.0 / 27.0;

    let y = relative_luminance(color);

    if y > EPSILON {
        116.0 * y.cbrt() - 16.0
    } else {
        KAPPA * y
    }
}

/// Absolute perceptual lightness difference (ΔL\*) between two colors.
pub fn lightness_difference(first: Color, second: Color) -> f32 {
    (perceptual_lightness(first) - perceptual_lightness(second)).abs()
}

#[cfg(test)]
mod color_math_tests {
    use super::*;

    #[test]
    fn black_and_white_have_maximum_contrast() {
        let contrast = contrast_ratio(Color::BLACK, Color::WHITE);

        assert!((contrast - 21.0).abs() < 0.01);
    }

    #[test]
    fn identical_colors_have_unit_contrast() {
        assert!((contrast_ratio(Color::WHITE, Color::WHITE) - 1.0).abs() < 0.001);
        assert!((contrast_ratio(Color::BLACK, Color::BLACK) - 1.0).abs() < 0.001);
    }

    #[test]
    fn contrast_ratio_is_symmetric() {
        let a = hex(0x6E4DAF);
        let b = hex(0xCBCBD0);

        assert!((contrast_ratio(a, b) - contrast_ratio(b, a)).abs() < 0.0001);
    }

    #[test]
    fn white_has_maximum_perceptual_lightness() {
        assert!((perceptual_lightness(Color::WHITE) - 100.0).abs() < 0.01);
    }

    #[test]
    fn black_has_zero_perceptual_lightness() {
        assert!(perceptual_lightness(Color::BLACK).abs() < 0.01);
    }

    #[test]
    fn perceptual_lightness_increases_monotonically_with_luminance() {
        let dark = hex(0x0D0D0F);
        let mid = hex(0x6E4DAF);
        let light = hex(0xCBCBD0);

        assert!(perceptual_lightness(dark) < perceptual_lightness(mid));
        assert!(perceptual_lightness(mid) < perceptual_lightness(light));
    }

    #[test]
    fn lightness_difference_is_symmetric_and_zero_for_identical_colors() {
        let a = hex(0x0D0D0F);
        let b = hex(0x6E4DAF);

        assert_eq!(lightness_difference(a, a), 0.0);
        assert!((lightness_difference(a, b) - lightness_difference(b, a)).abs() < 0.0001);
    }
}
