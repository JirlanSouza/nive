use iced::{Color, Shadow, Vector};

pub const NONE: Shadow = Shadow {
    color: Color::TRANSPARENT,
    offset: Vector::ZERO,
    blur_radius: 0.0,
};

/// Elevation shadow ramp: `ELEVATED < POPOVER < DIALOG` in prominence
/// (`color.a * blur_radius`), guarded by a unit test in this module.
pub const ELEVATED: Shadow = Shadow {
    color: Color {
        a: 0.16,
        ..Color::BLACK
    },
    offset: Vector { x: 0.0, y: 1.0 },
    blur_radius: 6.0,
};

pub const POPOVER: Shadow = Shadow {
    color: Color {
        a: 0.22,
        ..Color::BLACK
    },
    offset: Vector { x: 0.0, y: 1.0 },
    blur_radius: 10.0,
};

pub const DIALOG: Shadow = Shadow {
    color: Color {
        a: 0.32,
        ..Color::BLACK
    },
    offset: Vector { x: 0.0, y: 4.0 },
    blur_radius: 24.0,
};

#[cfg(test)]
mod shadow_tests {
    use super::*;

    fn prominence(shadow: Shadow) -> f32 {
        shadow.color.a * shadow.blur_radius
    }

    #[test]
    fn elevation_ramp_increases_in_prominence() {
        assert!(prominence(ELEVATED) < prominence(POPOVER));
        assert!(prominence(POPOVER) < prominence(DIALOG));
    }

    #[test]
    fn elevated_carries_a_non_empty_subtle_shadow() {
        const {
            assert!(ELEVATED.color.a > 0.0);
            assert!(ELEVATED.blur_radius > 0.0);
        }
    }
}
