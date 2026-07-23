use iced::Color;

pub(super) fn rgb(hex: u32) -> Color {
    Color::from_rgb(
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
    )
}

pub(super) fn mix(from: Color, to: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color {
        r: from.r + (to.r - from.r) * amount,
        g: from.g + (to.g - from.g) * amount,
        b: from.b + (to.b - from.b) * amount,
        a: from.a + (to.a - from.a) * amount,
    }
}

pub(super) fn with_alpha(color: Color, alpha: f32) -> Color {
    Color {
        a: alpha.clamp(0.0, 1.0),
        ..color
    }
}

pub(super) fn focus_color(primary: Color, is_dark: bool) -> Color {
    let brightness = if is_dark { 0.16 } else { 0.08 };

    with_alpha(mix(primary, Color::WHITE, brightness), 0.92)
}

pub(super) fn tone_background(color: Color, is_dark: bool) -> Color {
    with_alpha(color, if is_dark { 0.16 } else { 0.10 })
}

pub(super) fn readable_on(background: Color, _is_dark: bool) -> Color {
    if crate::theme::color::contrast_ratio(Color::WHITE, background)
        >= crate::theme::color::contrast_ratio(Color::BLACK, background)
    {
        Color::WHITE
    } else {
        Color::BLACK
    }
}

pub(super) fn readable_on_container(color: Color, _is_dark: bool) -> Color {
    color
}

#[cfg(test)]
pub(super) fn luminance(color: Color) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
}
