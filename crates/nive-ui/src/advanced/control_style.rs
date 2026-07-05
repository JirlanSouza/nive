use iced::{border::Radius, Border, Color};

use crate::theme::BorderSpec;

const DISABLED_ALPHA: f32 = 0.5;

pub fn border(spec: BorderSpec) -> Border {
    border_with_radius(spec, Radius::default())
}

pub fn border_with_radius(spec: BorderSpec, radius: impl Into<Radius>) -> Border {
    Border {
        color: spec.color,
        width: spec.width,
        radius: radius.into(),
    }
}

pub fn transparent_border() -> Border {
    border(BorderSpec::none())
}

pub fn transparent_border_with_radius(radius: impl Into<Radius>) -> Border {
    border_with_radius(BorderSpec::none(), radius)
}

pub fn disabled_alpha(color: Color) -> Color {
    color.scale_alpha(DISABLED_ALPHA)
}

pub fn alpha_when_disabled(color: Color, disabled: bool) -> Color {
    if disabled {
        disabled_alpha(color)
    } else {
        color
    }
}

#[cfg(test)]
mod common_tests {
    use super::*;

    #[test]
    fn border_preserves_spec_with_default_radius() {
        let border = border(BorderSpec::new(Color::from_rgb(0.2, 0.4, 0.6), 2.0));

        assert_eq!(border.color, Color::from_rgb(0.2, 0.4, 0.6));
        assert_eq!(border.width, 2.0);
        assert_eq!(border.radius, Radius::default());
    }

    #[test]
    fn transparent_border_has_no_visible_chrome() {
        let border = transparent_border();

        assert_eq!(border.color, Color::TRANSPARENT);
        assert_eq!(border.width, 0.0);
        assert_eq!(border.radius, Radius::default());
    }

    #[test]
    fn disabled_alpha_scales_existing_alpha() {
        let color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.8,
        };

        assert!((disabled_alpha(color).a - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn alpha_when_disabled_preserves_enabled_color() {
        let color = Color::from_rgb(0.2, 0.4, 0.6);

        assert_eq!(alpha_when_disabled(color, false), color);
    }
}
