use iced::{border::Radius, widget::container, Background, Color, Shadow};

use crate::theme::{
    self, control_metrics, BorderRole, BorderSpec, ControlRole, ControlSize, ControlState,
    FieldValidation, TextRole,
};

use super::super::control_style::{
    alpha_when_disabled, border_with_radius, disabled_alpha, transparent_border_with_radius,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InputGroupMetrics {
    pub height: f32,
    pub radius: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub input_padding_v: f32,
    pub input_padding_h: f32,
    pub slot_padding_h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputGroupVariant {
    Default,
    Ghost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputGroupStatus {
    Active,
    Hovered,
    Focused,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GroupSlotPosition {
    Single,
    First,
    Middle,
    Last,
}

pub fn metrics(size: ControlSize) -> InputGroupMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();

    InputGroupMetrics {
        height: control.height,
        radius: control.radius,
        font_size: control.font_size,
        icon_size: control.icon_size,
        input_padding_v: match size {
            ControlSize::Xs => spacing.xxs,
            ControlSize::Sm => spacing.xs,
            ControlSize::Md => spacing.xs + 1.0,
            ControlSize::Lg => spacing.md,
        },
        input_padding_h: match size {
            ControlSize::Xs => spacing.sm,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.md + spacing.xxs,
            ControlSize::Lg => spacing.xl,
        },
        slot_padding_h: match size {
            ControlSize::Xs => spacing.sm,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.md + spacing.xxs,
            ControlSize::Lg => spacing.xl,
        },
    }
}

pub(crate) fn slot_radius(position: GroupSlotPosition, radius: f32) -> Radius {
    match position {
        GroupSlotPosition::Single => Radius::new(radius),
        GroupSlotPosition::First => Radius::default().left(radius),
        GroupSlotPosition::Middle => Radius::default(),
        GroupSlotPosition::Last => Radius::default().right(radius),
    }
}

pub(crate) fn slot_style(
    radius: Radius,
    disabled: bool,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| container::Style {
        text_color: Some(slot_text_color(theme, disabled)),
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: transparent_border_with_radius(radius),
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

fn slot_text_color(theme: &crate::theme::Theme, disabled: bool) -> Color {
    let color = theme.text(TextRole::Muted).color;

    alpha_when_disabled(color, disabled)
}

pub fn style(
    variant: InputGroupVariant,
    input_validation: FieldValidation,
    radius: f32,
    status: InputGroupStatus,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let theme = *theme;
        let control = theme.control(ControlRole::Standard, control_state(status));
        let (background, base_border) = match variant {
            InputGroupVariant::Default => (control.background, control.border),
            InputGroupVariant::Ghost => (Color::TRANSPARENT, BorderSpec::none()),
        };

        let border = match (input_validation, status) {
            (FieldValidation::Invalid, InputGroupStatus::Disabled) => BorderSpec::new(
                disabled_alpha(theme.border(BorderRole::Danger).color),
                theme.border(BorderRole::Danger).width,
            ),
            (FieldValidation::Invalid, _) => theme.border(BorderRole::Danger),
            (_, InputGroupStatus::Focused) => theme.border(BorderRole::Focus),
            _ => base_border,
        };

        container::Style {
            text_color: Some(control.foreground),
            background: Some(Background::Color(background)),
            border: border_with_radius(border, radius),
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

fn control_state(status: InputGroupStatus) -> ControlState {
    match status {
        InputGroupStatus::Active => ControlState::ENABLED,
        InputGroupStatus::Hovered => ControlState::HOVERED,
        InputGroupStatus::Focused => ControlState::FOCUSED,
        InputGroupStatus::Disabled => ControlState::DISABLED,
    }
}

#[cfg(test)]
mod input_group_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn disabled_ghost_keeps_app_transparent_chrome() {
        let theme = Theme::Dark;
        let style = style(
            InputGroupVariant::Ghost,
            FieldValidation::Valid,
            6.0,
            InputGroupStatus::Disabled,
        )(&theme);

        assert_eq!(background_color(&style), Color::TRANSPARENT);
        assert_eq!(style.border.color, Color::TRANSPARENT);
        assert_eq!(
            style.text_color,
            Some(
                theme
                    .control(ControlRole::Standard, ControlState::DISABLED)
                    .foreground
            )
        );
    }

    #[test]
    fn disabled_default_uses_app_muted_chrome() {
        let theme = Theme::Dark;
        let control = theme.control(ControlRole::Standard, ControlState::DISABLED);
        let style = style(
            InputGroupVariant::Default,
            FieldValidation::Valid,
            6.0,
            InputGroupStatus::Disabled,
        )(&theme);

        assert_eq!(background_color(&style), control.background);
        assert_eq!(style.border.color, control.border.color);
        assert_eq!(style.text_color, Some(control.foreground));
    }

    #[test]
    fn focused_default_uses_app_focus_chrome() {
        let theme = Theme::Dark;
        let style = style(
            InputGroupVariant::Default,
            FieldValidation::Valid,
            6.0,
            InputGroupStatus::Focused,
        )(&theme);

        assert_eq!(style.border.color, theme.border(BorderRole::Focus).color);
    }

    #[test]
    fn invalid_focus_keeps_app_danger_chrome() {
        let theme = Theme::Dark;
        let style = style(
            InputGroupVariant::Default,
            FieldValidation::Invalid,
            6.0,
            InputGroupStatus::Focused,
        )(&theme);

        assert_eq!(style.border.color, theme.border(BorderRole::Danger).color);
    }

    fn background_color(style: &container::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
