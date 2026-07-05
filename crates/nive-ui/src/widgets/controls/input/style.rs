use iced::{
    border::Radius,
    widget::text_input::{self, Status},
};

use crate::theme::{self, control_metrics, ControlSize, FieldValidation, TextInputClass};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextInputMetrics {
    pub font_size: f32,
    pub padding_v: f32,
    pub padding_h: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextInputAppearance {
    Standard,
    Embedded,
}

pub fn style(
    appearance: TextInputAppearance,
    validation: FieldValidation,
    radius: Radius,
) -> impl Fn(&crate::theme::Theme, Status) -> text_input::Style {
    move |theme: &crate::theme::Theme, status: Status| {
        let class = text_input_class(appearance, validation);
        let mut style = <crate::theme::Theme as text_input::Catalog>::style(theme, &class, status);
        style.border.radius = radius;
        style
    }
}

pub(crate) fn text_input_class(
    appearance: TextInputAppearance,
    validation: FieldValidation,
) -> TextInputClass<'static> {
    match appearance {
        TextInputAppearance::Standard => TextInputClass::Standard { validation },
        TextInputAppearance::Embedded => TextInputClass::Embedded { validation },
    }
}

pub fn metrics(size: ControlSize) -> TextInputMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();

    TextInputMetrics {
        font_size: control.font_size,
        padding_v: match size {
            ControlSize::Xs => spacing.xxs,
            ControlSize::Sm => spacing.xs,
            ControlSize::Md => spacing.xs + 1.0,
            ControlSize::Lg => spacing.md,
        },
        padding_h: match size {
            ControlSize::Xs => spacing.sm,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.md + spacing.xxs,
            ControlSize::Lg => spacing.xl,
        },
        radius: control.radius,
    }
}

#[cfg(test)]
mod text_input_tests {
    use super::*;
    use iced::{Background, Color};

    use crate::theme::{BorderRole, Theme, ToneRole};

    #[test]
    fn standard_valid_uses_catalog_class() {
        let theme = Theme::Dark;
        let radius = Radius::new(6.0);
        let style = style(
            TextInputAppearance::Standard,
            FieldValidation::Valid,
            radius,
        )(&theme, Status::Active);
        let expected = <Theme as text_input::Catalog>::style(
            &theme,
            &TextInputClass::Standard {
                validation: FieldValidation::Valid,
            },
            Status::Active,
        );

        assert_eq!(background_color(&style), background_color(&expected));
        assert_eq!(style.border.color, expected.border.color);
        assert_eq!(style.border.width, expected.border.width);
        assert_eq!(style.border.radius, radius);
    }

    #[test]
    fn standard_invalid_uses_catalog_validation() {
        let theme = Theme::Dark;
        let radius = Radius::new(6.0);
        let style = style(
            TextInputAppearance::Standard,
            FieldValidation::Invalid,
            radius,
        )(&theme, Status::Focused { is_hovered: false });

        assert_eq!(style.border.color, theme.border(BorderRole::Danger).color);
        assert_eq!(style.border.width, theme.border(BorderRole::Danger).width);
        assert_eq!(
            style.selection,
            theme.tone(ToneRole::Danger).color.scale_alpha(0.2)
        );
        assert_eq!(style.border.radius, radius);
    }

    #[test]
    fn embedded_valid_keeps_transparent_border() {
        let style = style(
            TextInputAppearance::Embedded,
            FieldValidation::Valid,
            Radius::new(6.0),
        )(&Theme::Dark, Status::Focused { is_hovered: false });

        assert_eq!(style.border.color, Color::TRANSPARENT);
        assert_eq!(style.border.width, 0.0);
    }

    #[test]
    fn embedded_invalid_uses_catalog_validation() {
        let theme = Theme::Dark;
        let style = style(
            TextInputAppearance::Embedded,
            FieldValidation::Invalid,
            Radius::new(6.0),
        )(&theme, Status::Focused { is_hovered: false });

        assert_eq!(style.border.color, Color::TRANSPARENT);
        assert_eq!(style.border.width, 0.0);
        assert_eq!(
            style.selection,
            theme.tone(ToneRole::Danger).color.scale_alpha(0.2)
        );
    }

    fn background_color(style: &text_input::Style) -> Color {
        match style.background {
            Background::Color(color) => color,
            _ => panic!("Expected color background"),
        }
    }
}
