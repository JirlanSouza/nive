use iced::{widget::text, Color, Font};

use crate::theme::{typography, TextRole, ToneRole, TypographyRole};

pub fn style(role: TextRole) -> impl Fn(&crate::theme::Theme) -> text::Style {
    move |theme| text::Style {
        color: Some(theme.text(role).color),
    }
}

pub fn tone(role: ToneRole) -> impl Fn(&crate::theme::Theme) -> text::Style {
    move |theme| text::Style {
        color: Some(theme.tone(role).color),
    }
}

pub fn on_tone(role: ToneRole) -> impl Fn(&crate::theme::Theme) -> text::Style {
    move |theme| text::Style {
        color: Some(theme.tone(role).on_color),
    }
}

pub fn color(color: Color) -> impl Fn(&crate::theme::Theme) -> text::Style {
    move |_theme| text::Style { color: Some(color) }
}

pub fn font_for_role(role: TypographyRole) -> Font {
    typography(role).font
}

pub fn size_for_role(role: TypographyRole) -> f32 {
    typography(role).size
}

pub fn line_height_for_role(role: TypographyRole) -> text::LineHeight {
    typography(role).line_height.into()
}

#[cfg(test)]
mod text_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn muted_style_uses_app_semantic_text_role() {
        let theme = Theme::Dark;

        assert_eq!(
            style(TextRole::Muted)(&theme).color,
            Some(theme.text(TextRole::Muted).color)
        );
    }

    #[test]
    fn style_uses_app_theme_semantic_color() {
        let theme = Theme::Dark;
        let result = style(TextRole::Muted)(&theme);

        assert_eq!(result.color, Some(theme.text(TextRole::Muted).color));
    }

    #[test]
    fn font_for_role_returns_correct_font() {
        assert_ne!(
            font_for_role(TypographyRole::Code),
            font_for_role(TypographyRole::Body)
        );
    }

    #[test]
    fn size_for_role_monotonically_increases_for_heading_roles() {
        assert!(size_for_role(TypographyRole::Caption) < size_for_role(TypographyRole::BodySmall));
        assert!(size_for_role(TypographyRole::BodySmall) <= size_for_role(TypographyRole::Body));
        assert!(size_for_role(TypographyRole::Body) < size_for_role(TypographyRole::Heading));
        assert!(size_for_role(TypographyRole::Heading) < size_for_role(TypographyRole::Title));
    }

    #[test]
    fn line_height_for_role_tracks_typography_spec() {
        for role in [
            TypographyRole::Body,
            TypographyRole::BodySmall,
            TypographyRole::Control,
            TypographyRole::ControlStrong,
            TypographyRole::Label,
            TypographyRole::LabelStrong,
            TypographyRole::BadgeLabel,
            TypographyRole::SectionLabel,
            TypographyRole::Heading,
            TypographyRole::Title,
            TypographyRole::Caption,
            TypographyRole::Code,
            TypographyRole::CodeSmall,
            TypographyRole::MetadataTag,
        ] {
            assert_eq!(
                line_height_for_role(role),
                text::LineHeight::Relative(typography(role).line_height)
            );
        }
    }
}
