use iced::{Color, Shadow, Vector};

pub const NONE: Shadow = Shadow {
    color: Color::TRANSPARENT,
    offset: Vector::ZERO,
    blur_radius: 0.0,
};

pub const POPOVER: Shadow = Shadow {
    color: Color {
        a: 0.22,
        ..Color::BLACK
    },
    offset: Vector { x: 0.0, y: 1.0 },
    blur_radius: 10.0,
};
