use iced::Font;

use crate::tokens::typography as token_typography;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypographyRole {
    Body,
    BodySmall,
    Label,
    LabelStrong,
    SectionLabel,
    Heading,
    Title,
    Caption,
    Code,
    CodeSmall,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub font: Font,
    pub size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypographyScale {
    pub title: TextStyle,
    pub heading: TextStyle,
    pub body: TextStyle,
    pub body_small: TextStyle,
    pub label: TextStyle,
    pub label_strong: TextStyle,
    pub section_label: TextStyle,
    pub caption: TextStyle,
    pub code: TextStyle,
    pub code_small: TextStyle,
}

pub fn scale() -> TypographyScale {
    TypographyScale {
        title: typography(TypographyRole::Title),
        heading: typography(TypographyRole::Heading),
        body: typography(TypographyRole::Body),
        body_small: typography(TypographyRole::BodySmall),
        label: typography(TypographyRole::Label),
        label_strong: typography(TypographyRole::LabelStrong),
        section_label: typography(TypographyRole::SectionLabel),
        caption: typography(TypographyRole::Caption),
        code: typography(TypographyRole::Code),
        code_small: typography(TypographyRole::CodeSmall),
    }
}

pub fn typography(role: TypographyRole) -> TextStyle {
    match role {
        TypographyRole::Body => spec(
            token_typography::UI.normal(),
            token_typography::TEXT_BASE,
            token_typography::LEADING_NORMAL,
        ),
        TypographyRole::BodySmall => spec(
            token_typography::UI.normal(),
            token_typography::TEXT_XS,
            token_typography::LEADING_NORMAL,
        ),
        TypographyRole::Label => spec(
            token_typography::UI.normal(),
            token_typography::TEXT_XS,
            token_typography::LEADING_SNUG,
        ),
        TypographyRole::LabelStrong => spec(
            token_typography::UI.semibold(),
            token_typography::TEXT_XS,
            token_typography::LEADING_SNUG,
        ),
        TypographyRole::SectionLabel => spec(
            token_typography::UI.semibold(),
            10.0,
            token_typography::LEADING_NONE,
        ),
        TypographyRole::Heading => spec(
            token_typography::UI.semibold(),
            token_typography::TEXT_LG,
            token_typography::LEADING_SNUG,
        ),
        TypographyRole::Title => spec(
            token_typography::UI.semibold(),
            token_typography::TEXT_XL,
            token_typography::LEADING_TIGHT,
        ),
        TypographyRole::Caption => spec(
            token_typography::UI.normal(),
            11.0,
            token_typography::LEADING_NORMAL,
        ),
        TypographyRole::Code => spec(
            token_typography::MONO.normal(),
            token_typography::TEXT_XS,
            token_typography::LEADING_NORMAL,
        ),
        TypographyRole::CodeSmall => spec(
            token_typography::MONO.normal(),
            10.0,
            token_typography::LEADING_NORMAL,
        ),
    }
}

impl TypographyScale {
    pub fn get(self, role: TypographyRole) -> TextStyle {
        match role {
            TypographyRole::Title => self.title,
            TypographyRole::Heading => self.heading,
            TypographyRole::Body => self.body,
            TypographyRole::BodySmall => self.body_small,
            TypographyRole::Label => self.label,
            TypographyRole::LabelStrong => self.label_strong,
            TypographyRole::SectionLabel => self.section_label,
            TypographyRole::Caption => self.caption,
            TypographyRole::Code => self.code,
            TypographyRole::CodeSmall => self.code_small,
        }
    }
}

fn spec(font: Font, size: f32, line_height: f32) -> TextStyle {
    TextStyle {
        font,
        size,
        line_height,
        letter_spacing: 0.0,
    }
}

#[cfg(test)]
mod typography_tests {
    use super::*;

    #[test]
    fn code_styles_use_mono_font() {
        assert_eq!(
            typography(TypographyRole::Code).font,
            token_typography::MONO.normal()
        );
        assert_eq!(
            typography(TypographyRole::CodeSmall).font,
            token_typography::MONO.normal()
        );
    }

    #[test]
    fn section_label_is_smaller_than_body() {
        assert!(
            typography(TypographyRole::SectionLabel).size < typography(TypographyRole::Body).size
        );
    }

    #[test]
    fn letter_spacing_defaults_to_zero() {
        assert_eq!(typography(TypographyRole::Body).letter_spacing, 0.0);
    }

    #[test]
    fn typography_scale_preserves_letter_spacing() {
        let scale = scale();

        for role in [
            TypographyRole::Title,
            TypographyRole::Heading,
            TypographyRole::Body,
            TypographyRole::BodySmall,
            TypographyRole::Label,
            TypographyRole::LabelStrong,
            TypographyRole::SectionLabel,
            TypographyRole::Caption,
            TypographyRole::Code,
            TypographyRole::CodeSmall,
        ] {
            assert_eq!(scale.get(role).letter_spacing, 0.0);
        }
    }
}
