use iced::{
    border::Radius,
    widget::button::{self, Status},
    Background, Border, Color, Shadow,
};

use crate::theme::{
    self, control_metrics, BorderRole, BorderSpec, ButtonClass, ControlRole, ControlSize,
    ControlState, TextRole, ToneRole,
};

use crate::advanced::control_style::{border_with_radius, transparent_border_with_radius};

pub fn button_control_state(status: button::Status) -> ControlState {
    match status {
        button::Status::Active => ControlState::ENABLED,
        button::Status::Hovered => ControlState::HOVERED,
        button::Status::Pressed => ControlState::PRESSED,
        button::Status::Disabled => ControlState::DISABLED,
    }
}

/// Visual metrics for a button at a resolved control size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonMetrics {
    pub font_size: f32,
    pub height: f32,
    pub padding_h: f32,
    pub radius: f32,
    pub icon_size: f32,
    pub gap: f32,
}

/// Action semantics for a button.
///
/// `Suggested` is the desktop default-action intent. Destructive actions use
/// `Destructive`; status tone remains `ToneRole::Danger` on non-action widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonIntent {
    /// Ordinary action.
    Neutral,
    /// Suggested/default action.
    Suggested,
    /// Action that may delete or otherwise damage data.
    Destructive,
}

/// Button appearance variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Filled button chrome.
    Solid,
    /// Low-emphasis filled/subtle chrome.
    Subtle,
    /// Outlined button chrome.
    Outline,
    /// Minimal transparent chrome.
    Ghost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonFocusRing {
    Default,
    OnPrimary,
}

pub fn style(
    intent: ButtonIntent,
    variant: ButtonVariant,
    radius: Radius,
) -> impl Fn(&crate::theme::Theme, Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: Status| {
        let class = button_class(intent, variant);
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

pub(crate) fn selectable_style(
    selected: bool,
    destructive: bool,
    radius: Radius,
) -> impl Fn(&crate::theme::Theme, Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: Status| {
        let theme = *theme;
        let control = theme.control(ControlRole::Standard, button_control_state(status));
        let selected_control = theme.control(ControlRole::Selectable, ControlState::SELECTED);
        let disabled_control = theme.control(ControlRole::Standard, ControlState::DISABLED);
        let danger = theme.tone(ToneRole::Danger);

        if destructive {
            let background = match status {
                Status::Active | Status::Disabled => Color::TRANSPARENT,
                Status::Hovered => danger.container,
                Status::Pressed => danger.container.scale_alpha(1.18),
            };
            let text_color = if matches!(status, Status::Disabled) {
                danger.color.scale_alpha(0.55)
            } else {
                danger.color
            };

            return button::Style {
                background: Some(Background::Color(background)),
                text_color,
                border: transparent_border_with_radius(radius),
                shadow: Shadow::default(),
                ..button::Style::default()
            };
        }

        let background = match (selected, status) {
            (true, Status::Hovered) => selected_control.background.scale_alpha(1.20),
            (true, Status::Pressed) => selected_control.background.scale_alpha(0.88),
            (true, Status::Disabled) => selected_control.background.scale_alpha(0.60),
            (true, _) => selected_control.background,
            (false, Status::Hovered | Status::Pressed) => control.background,
            (false, _) => Color::TRANSPARENT,
        };
        let text_color = match (selected, status) {
            (true, Status::Disabled) => selected_control.foreground.scale_alpha(0.60),
            (true, _) => selected_control.foreground,
            (false, Status::Hovered | Status::Pressed) => theme.text(TextRole::Primary).color,
            (false, Status::Disabled) => disabled_control.foreground,
            (false, _) => theme.text(TextRole::Secondary).color,
        };
        let border = if selected {
            selected_control.border
        } else {
            BorderSpec::none()
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color,
            border: border_with_radius(border, radius),
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    }
}

pub(crate) fn button_class(intent: ButtonIntent, variant: ButtonVariant) -> ButtonClass<'static> {
    ButtonClass::Standard { intent, variant }
}

pub fn focus_ring(theme: &crate::theme::Theme, ring: ButtonFocusRing, radius: Radius) -> Border {
    let theme = *theme;
    let focus = theme.border(BorderRole::Focus);
    let color = match ring {
        ButtonFocusRing::Default => focus.color,
        ButtonFocusRing::OnPrimary => theme.tone(ToneRole::Accent).on_color,
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
    fn suggested_solid_uses_catalog_class() {
        let theme = Theme::Dark;
        let radius = Radius::new(4.0);
        let style =
            style(ButtonIntent::Suggested, ButtonVariant::Solid, radius)(&theme, Status::Active);
        let expected = <Theme as button::Catalog>::style(
            &theme,
            &ButtonClass::Standard {
                intent: ButtonIntent::Suggested,
                variant: ButtonVariant::Solid,
            },
            Status::Active,
        );

        assert_eq!(background_color(&style), theme.tone(ToneRole::Accent).color);
        assert_eq!(style.text_color, expected.text_color);
        assert_eq!(style.border.color, expected.border.color);
        assert_eq!(style.border.width, expected.border.width);
        assert_eq!(style.border.radius, radius);
    }

    #[test]
    fn destructive_uses_catalog_class() {
        let theme = Theme::Dark;
        let radius = Radius::new(6.0);
        let style =
            style(ButtonIntent::Destructive, ButtonVariant::Solid, radius)(&theme, Status::Active);
        let expected = <Theme as button::Catalog>::style(
            &theme,
            &ButtonClass::Standard {
                intent: ButtonIntent::Destructive,
                variant: ButtonVariant::Solid,
            },
            Status::Active,
        );

        assert_eq!(background_color(&style), background_color(&expected));
        assert_eq!(style.text_color, expected.text_color);
        assert_eq!(style.border.color, expected.border.color);
        assert_eq!(style.border.width, expected.border.width);
        assert_eq!(style.border.radius, radius);
    }

    #[test]
    fn selectable_selected_uses_selectable_role() {
        let theme = Theme::Dark;
        let style = selectable_style(true, false, Radius::new(6.0))(&theme, Status::Active);
        let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);

        assert_eq!(background_color(&style), selected.background);
        assert_eq!(style.text_color, selected.foreground);
        assert_eq!(style.border.color, selected.border.color);
        assert_eq!(style.border.width, selected.border.width);
    }

    #[test]
    fn selectable_unselected_active_is_transparent() {
        let theme = Theme::Dark;
        let style = selectable_style(false, false, Radius::new(6.0))(&theme, Status::Active);

        assert_eq!(background_color(&style), Color::TRANSPARENT);
        assert_eq!(style.text_color, theme.text(TextRole::Secondary).color);
        assert_eq!(style.border.color, Color::TRANSPARENT);
    }

    #[test]
    fn selectable_unselected_hover_uses_standard_control_background() {
        let theme = Theme::Dark;
        let style = selectable_style(false, false, Radius::new(6.0))(&theme, Status::Hovered);
        let control = theme.control(ControlRole::Standard, ControlState::HOVERED);

        assert_eq!(background_color(&style), control.background);
        assert_eq!(style.text_color, theme.text(TextRole::Primary).color);
    }

    #[test]
    fn selectable_selected_pressed_scales_selected_background() {
        let theme = Theme::Dark;
        let style = selectable_style(true, false, Radius::new(6.0))(&theme, Status::Pressed);
        let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);

        assert_eq!(
            background_color(&style),
            selected.background.scale_alpha(0.88)
        );
        assert_eq!(style.text_color, selected.foreground);
    }

    #[test]
    fn selectable_disabled_selected_uses_canonical_scaled_selected_color() {
        let theme = Theme::Dark;
        let style = selectable_style(true, false, Radius::new(6.0))(&theme, Status::Disabled);
        let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);

        assert_eq!(
            background_color(&style),
            selected.background.scale_alpha(0.60)
        );
        assert_eq!(style.text_color, selected.foreground.scale_alpha(0.60));
    }

    fn background_color(style: &button::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
