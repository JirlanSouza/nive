use iced::{
    border::Radius,
    widget::button::{self, Status},
    Border,
};

use crate::theme::{
    self, control_metrics, BorderRole, BorderSpec, ButtonClass, ControlSize, ControlState, ToneRole,
};

use super::super::control_style::border_with_radius;

pub fn button_control_state(status: button::Status) -> ControlState {
    match status {
        button::Status::Active => ControlState::ENABLED,
        button::Status::Hovered => ControlState::HOVERED,
        button::Status::Pressed => ControlState::PRESSED,
        button::Status::Disabled => ControlState::DISABLED,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonMetrics {
    pub font_size: f32,
    pub height: f32,
    pub padding_h: f32,
    pub radius: f32,
    pub icon_size: f32,
    pub gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonFocusRing {
    Default,
    OnPrimary,
}

pub fn style(
    variant: ButtonVariant,
    radius: Radius,
) -> impl Fn(&crate::theme::Theme, Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: Status| {
        let class = button_class(variant);
        let mut style = <crate::theme::Theme as button::Catalog>::style(theme, &class, status);
        style.border.radius = radius;
        style
    }
}

pub(crate) fn embedded_style(
    radius: Radius,
) -> impl Fn(&crate::theme::Theme, Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: Status| {
        let class = ButtonClass::Embedded;
        let mut style = <crate::theme::Theme as button::Catalog>::style(theme, &class, status);
        style.border.radius = radius;
        style
    }
}

pub(crate) fn button_class(variant: ButtonVariant) -> ButtonClass<'static> {
    match variant {
        ButtonVariant::Primary => ButtonClass::Primary,
        ButtonVariant::Secondary => ButtonClass::Secondary,
        ButtonVariant::Outline => ButtonClass::Outline,
        ButtonVariant::Ghost => ButtonClass::Ghost,
        ButtonVariant::Destructive => ButtonClass::Destructive,
        ButtonVariant::Link => ButtonClass::Link,
    }
}

pub fn focus_ring(theme: &crate::theme::Theme, ring: ButtonFocusRing, radius: Radius) -> Border {
    let theme = *theme;
    let focus = theme.border(BorderRole::Focus);
    let color = match ring {
        ButtonFocusRing::Default => focus.color,
        ButtonFocusRing::OnPrimary => theme.tone(ToneRole::Primary).on_color,
    };

    border_with_radius(BorderSpec::new(color, focus.width), radius)
}

pub fn metrics(size: ControlSize) -> ButtonMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();

    ButtonMetrics {
        font_size: control.font_size,
        height: control.height,
        padding_h: match size {
            ControlSize::Xs => spacing.sm,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.md + spacing.xxs,
            ControlSize::Lg => spacing.xl,
        },
        radius: control.radius,
        icon_size: control.icon_size,
        gap: control.gap,
    }
}

pub fn icon_side(size: ControlSize) -> f32 {
    control_metrics(size).height
}

#[cfg(test)]
mod button_tests {
    use super::*;
    use iced::{Background, Color};

    use crate::theme::{Theme, ToneRole};

    #[test]
    fn metrics_follow_control_size() {
        assert_eq!(
            metrics(ControlSize::Sm).height,
            control_metrics(ControlSize::Sm).height
        );
        assert_eq!(
            metrics(ControlSize::Sm).font_size,
            control_metrics(ControlSize::Sm).font_size
        );
    }

    #[test]
    fn primary_uses_catalog_class() {
        let theme = Theme::Dark;
        let radius = Radius::new(4.0);
        let style = style(ButtonVariant::Primary, radius)(&theme, Status::Active);
        let expected =
            <Theme as button::Catalog>::style(&theme, &ButtonClass::Primary, Status::Active);

        assert_eq!(
            background_color(&style),
            theme.tone(ToneRole::Primary).color
        );
        assert_eq!(style.text_color, expected.text_color);
        assert_eq!(style.border.color, expected.border.color);
        assert_eq!(style.border.width, expected.border.width);
        assert_eq!(style.border.radius, radius);
    }

    #[test]
    fn destructive_uses_catalog_class() {
        let theme = Theme::Dark;
        let radius = Radius::new(6.0);
        let style = style(ButtonVariant::Destructive, radius)(&theme, Status::Active);
        let expected =
            <Theme as button::Catalog>::style(&theme, &ButtonClass::Destructive, Status::Active);

        assert_eq!(background_color(&style), background_color(&expected));
        assert_eq!(style.text_color, expected.text_color);
        assert_eq!(style.border.color, expected.border.color);
        assert_eq!(style.border.width, expected.border.width);
        assert_eq!(style.border.radius, radius);
    }

    #[test]
    fn link_uses_catalog_class() {
        let theme = Theme::Dark;
        let radius = Radius::new(8.0);
        let style = style(ButtonVariant::Link, radius)(&theme, Status::Active);
        let expected =
            <Theme as button::Catalog>::style(&theme, &ButtonClass::Link, Status::Active);

        assert_eq!(background_color(&style), Color::TRANSPARENT);
        assert_eq!(style.text_color, expected.text_color);
        assert_eq!(style.border.color, expected.border.color);
        assert_eq!(style.border.width, expected.border.width);
        assert_eq!(style.border.radius, radius);
    }

    fn background_color(style: &button::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
