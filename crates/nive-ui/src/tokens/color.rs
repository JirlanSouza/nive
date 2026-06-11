use iced::Color;

pub const fn hex(hex: u32) -> Color {
    Color::from_rgb(
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
    )
}

pub fn parse_hex_color(value: &str) -> Option<Color> {
    let normalized = normalize_hex_color(value)?;
    let color = u32::from_str_radix(&normalized[1..7], 16).ok()?;
    let alpha = if normalized.len() == 9 {
        u8::from_str_radix(&normalized[7..], 16).ok()?
    } else {
        u8::MAX
    };

    let color = hex(color);

    Some(Color::from_rgba(
        color.r,
        color.g,
        color.b,
        f32::from(alpha) / 255.0,
    ))
}

pub fn normalize_hex_color(value: &str) -> Option<String> {
    let hex = value.trim().strip_prefix('#').unwrap_or(value.trim());
    if !matches!(hex.len(), 6 | 8) || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    Some(format!("#{}", hex.to_ascii_lowercase()))
}

pub fn format_hex_color(color: Color) -> String {
    let alpha = component_to_u8(color.a);

    if alpha == u8::MAX {
        return format_rgb_hex_color(color);
    }

    format!("{}{:02x}", format_rgb_hex_color(color), alpha)
}

pub fn format_rgb_hex_color(color: Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        component_to_u8(color.r),
        component_to_u8(color.g),
        component_to_u8(color.b),
    )
}

fn component_to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod color_tests {
    use super::*;

    #[test]
    fn normalizes_hex_colors() {
        assert_eq!(normalize_hex_color("ABCDEF"), Some("#abcdef".into()));
        assert_eq!(normalize_hex_color("#16A34A"), Some("#16a34a".into()));
        assert_eq!(normalize_hex_color("#33669980"), Some("#33669980".into()));
    }

    #[test]
    fn rejects_invalid_hex_colors() {
        assert_eq!(normalize_hex_color("#12345"), None);
        assert_eq!(normalize_hex_color("#12345g"), None);
    }

    #[test]
    fn formats_app_color_as_hex() {
        assert_eq!(format_hex_color(hex(0x16a34a)), "#16a34a");
    }

    #[test]
    fn parses_and_formats_alpha_hex_colors() {
        let color = parse_hex_color("#33669980").expect("valid alpha color");

        assert_eq!(format_hex_color(color), "#33669980");
        assert_eq!(format_rgb_hex_color(color), "#336699");
    }
}
