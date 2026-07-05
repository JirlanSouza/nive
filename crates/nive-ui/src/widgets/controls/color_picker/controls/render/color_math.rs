use iced::Color;

use super::super::super::hsva_color::HsvaColor;

const CHECKER_CELL_SIZE: f32 = 4.0;
const CHECKER_COLOR: Color = Color::from_rgb(214.0 / 255.0, 218.0 / 255.0, 224.0 / 255.0);

pub(super) fn saturation_value_color(hue: f32, saturation: f32, vertical_ratio: f32) -> Color {
    let hue_color = HsvaColor::hue_color(hue);
    let saturation = saturation.clamp(0.0, 1.0);
    let value = (1.0 - vertical_ratio).clamp(0.0, 1.0);
    let white_mix = 1.0 - saturation;

    Color::from_rgb(
        value * (white_mix + saturation * hue_color.r),
        value * (white_mix + saturation * hue_color.g),
        value * (white_mix + saturation * hue_color.b),
    )
}

pub(super) fn hue_surface_color(vertical_ratio: f32) -> Color {
    HsvaColor::hue_color((vertical_ratio * 359.999).clamp(0.0, 359.999))
}

pub(super) fn alpha_surface_color(color: Color, alpha: f32, x: f32, y: f32) -> Color {
    let alpha = alpha_visual_opacity(alpha);
    let foreground = Color::from_rgba(
        color.r.clamp(0.0, 1.0),
        color.g.clamp(0.0, 1.0),
        color.b.clamp(0.0, 1.0),
        alpha,
    );

    composite_over(foreground, checker_color(x, y))
}

fn alpha_visual_opacity(alpha: f32) -> f32 {
    alpha.clamp(0.0, 1.0)
}

fn checker_color(x: f32, y: f32) -> Color {
    let col = (x / CHECKER_CELL_SIZE).floor() as usize;
    let row = (y / CHECKER_CELL_SIZE).floor() as usize;

    if (row + col).is_multiple_of(2) {
        Color::WHITE
    } else {
        CHECKER_COLOR
    }
}

fn composite_over(foreground: Color, background: Color) -> Color {
    let alpha = foreground.a + background.a * (1.0 - foreground.a);

    if alpha <= 0.0 {
        return Color::TRANSPARENT;
    }

    Color::from_rgba(
        (foreground.r * foreground.a + background.r * background.a * (1.0 - foreground.a)) / alpha,
        (foreground.g * foreground.a + background.g * background.a * (1.0 - foreground.a)) / alpha,
        (foreground.b * foreground.a + background.b * background.a * (1.0 - foreground.a)) / alpha,
        alpha,
    )
}

#[cfg(test)]
mod color_math_tests {
    use super::*;

    #[test]
    fn alpha_surface_color_blends_color_over_checkerboard() {
        let color = alpha_surface_color(Color::from_rgb(0.0, 0.0, 1.0), 0.5, 0.0, 0.0);

        assert!(color.b > color.r);
        assert!(color.b > color.g);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn alpha_surface_zero_alpha_keeps_checkerboard_color() {
        let color = alpha_surface_color(Color::from_rgb(0.0, 0.0, 1.0), 0.0, 0.0, 0.0);

        assert_eq!(color, Color::WHITE);
    }

    #[test]
    fn alpha_visual_curve_stays_linear() {
        let visual_opacity = alpha_visual_opacity(0.5);

        assert_eq!(visual_opacity, 0.5);
        assert_eq!(alpha_visual_opacity(0.0), 0.0);
        assert_eq!(alpha_visual_opacity(1.0), 1.0);
    }
}
