use std::fmt;

use iced::Color;

use crate::theme;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbHexColor(String);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct RgbHexDraft {
    value: String,
}

impl RgbHexColor {
    pub fn parse(value: &str) -> Option<Self> {
        normalize_rgb_hex_color(value).map(Self)
    }

    pub fn from_color(color: Color) -> Self {
        Self(theme::format_rgb_hex_color(color))
    }

    pub(super) fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn to_color(&self) -> Color {
        theme::parse_hex_color(self.as_str()).expect("RgbHexColor stores a valid color")
    }
}

impl RgbHexDraft {
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }

    pub(super) fn sync_color(&mut self, color: Color) {
        self.value = RgbHexColor::from_color(color).to_string();
    }

    pub(super) fn input(&mut self, value: String) -> Option<RgbHexColor> {
        self.value = sanitize_hex_draft(&value);
        self.hex_color()
    }

    pub(super) fn hex_color(&self) -> Option<RgbHexColor> {
        RgbHexColor::parse(self.as_str())
    }
}

impl fmt::Display for RgbHexColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

fn normalize_rgb_hex_color(value: &str) -> Option<String> {
    let hex = value.trim().strip_prefix('#').unwrap_or(value.trim());
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    Some(format!("#{}", hex.to_ascii_lowercase()))
}

fn sanitize_hex_draft(value: &str) -> String {
    let mut chars = value.trim().chars();
    let prefixed = matches!(chars.clone().next(), Some('#'));
    let mut sanitized = String::with_capacity(if prefixed { 7 } else { 6 });

    if prefixed {
        sanitized.push('#');
        chars.next();
    }

    sanitized.extend(chars.filter(|c| c.is_ascii_hexdigit()).take(6));
    sanitized
}

#[cfg(test)]
mod hex_color_tests {
    use super::*;

    use crate::theme;

    #[test]
    fn parses_and_normalizes_valid_hex_colors() {
        assert_eq!(RgbHexColor::parse("ABCDEF").unwrap().as_str(), "#abcdef");
        assert_eq!(RgbHexColor::parse("#16A34A").unwrap().as_str(), "#16a34a");
        assert_eq!(
            RgbHexColor::parse("#ffffff").unwrap().to_color(),
            theme::parse_hex_color("#ffffff").unwrap()
        );
    }

    #[test]
    fn rejects_incomplete_invalid_and_oversized_hex_colors() {
        assert_eq!(RgbHexColor::parse("#12345"), None);
        assert_eq!(RgbHexColor::parse("#12345g"), None);
        assert_eq!(RgbHexColor::parse("#1000000"), None);
        assert_eq!(RgbHexColor::parse("#33669980"), None);
    }

    #[test]
    fn draft_allows_partial_input_but_limits_to_six_hex_digits() {
        let mut draft = RgbHexDraft::default();

        let parsed = draft.input("#1000000".into());

        assert_eq!(draft.as_str(), "#100000");
        assert_eq!(parsed.unwrap().as_str(), "#100000");
    }

    #[test]
    fn draft_removes_non_hex_characters() {
        let mut draft = RgbHexDraft::default();

        let parsed = draft.input("#ff-00-gg".into());

        assert_eq!(draft.as_str(), "#ff00");
        assert_eq!(parsed, None);
    }

    #[test]
    fn draft_preserves_user_case() {
        let mut draft = RgbHexDraft::default();

        let parsed = draft.input("#ABCDEF".into());

        assert_eq!(draft.as_str(), "#ABCDEF");
        assert_eq!(parsed.unwrap().as_str(), "#abcdef");
    }
}
