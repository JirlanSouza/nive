use iced::theme::Palette;

use crate::tokens::color::hex;

use super::color;
use super::ThemeMode;

pub fn palette(mode: ThemeMode) -> Palette {
    match mode {
        ThemeMode::Light => light_palette(),
        ThemeMode::Dark => dark_palette(),
    }
}

fn dark_palette() -> Palette {
    Palette {
        background: color::DARK_BACKGROUND,
        text: color::DARK_FOREGROUND,
        primary: color::PRIMARY,
        success: color::STATUS_READY,
        warning: hex(0xD97706),
        danger: color::DESTRUCTIVE,
    }
}

fn light_palette() -> Palette {
    Palette {
        background: hex(0xF6F7FB),
        text: hex(0x1D1D24),
        primary: color::PRIMARY,
        success: hex(0x15803D),
        warning: hex(0xD97706),
        danger: hex(0xDC2626),
    }
}

#[cfg(test)]
mod palette_tests {
    use super::*;
    use crate::theme::Theme;
    use iced::theme::Base as _;

    #[test]
    fn palette_supports_named_light_and_dark_themes() {
        assert_eq!(Theme::Light.name(), "RAG Studio Light");
        assert_eq!(Theme::Dark.name(), "RAG Studio Dark");
    }
}
