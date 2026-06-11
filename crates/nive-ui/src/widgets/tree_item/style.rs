use iced::{
    border::Radius,
    widget::{button, container},
    Background, Border, Color, Shadow,
};

use super::super::button::button_control_state;
use super::super::control_style::{
    border_with_radius, disabled_alpha, transparent_border_with_radius,
};

use crate::theme::{
    self, control_metrics, BorderSpec, ControlRole, ControlSize, ControlState, TextRole, ToneRole,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeItemVariant {
    Default,
    Selected,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeItemMetrics {
    pub height: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub expander_side: f32,
    pub indent: f32,
    pub gap: f32,
    pub padding_h: f32,
    pub radius: f32,
    pub tone_size: f32,
}

pub fn metrics(size: ControlSize) -> TreeItemMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();

    TreeItemMetrics {
        height: control.height,
        font_size: control.font_size,
        icon_size: control.icon_size,
        expander_side: control.height,
        indent: match size {
            ControlSize::Xs => spacing.lg,
            ControlSize::Sm => spacing.xl,
            ControlSize::Md => spacing.xl + spacing.xs,
            ControlSize::Lg => spacing.xxl,
        },
        gap: control.gap,
        padding_h: match size {
            ControlSize::Xs => spacing.xs,
            ControlSize::Sm => spacing.sm,
            ControlSize::Md => spacing.md,
            ControlSize::Lg => spacing.md + spacing.xxs,
        },
        radius: control.radius,
        tone_size: match size {
            ControlSize::Xs => 5.0,
            ControlSize::Sm => 6.0,
            ControlSize::Md | ControlSize::Lg => 7.0,
        },
    }
}

pub fn item_style(
    variant: TreeItemVariant,
    radius: f32,
) -> impl Fn(&crate::theme::Theme, button::Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: button::Status| {
        let theme = *theme;
        let control = theme.control(ControlRole::Standard, button_control_state(status));
        let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);
        let disabled = theme.control(ControlRole::Standard, ControlState::DISABLED);

        let background = match (variant, status) {
            (TreeItemVariant::Selected, button::Status::Active) => selected.background,
            (TreeItemVariant::Selected, button::Status::Hovered) => {
                selected.background.scale_alpha(1.20)
            }
            (TreeItemVariant::Selected, button::Status::Pressed) => {
                selected.background.scale_alpha(0.88)
            }
            (TreeItemVariant::Selected, button::Status::Disabled) => {
                selected.background.scale_alpha(0.55)
            }
            (TreeItemVariant::Default, button::Status::Hovered | button::Status::Pressed) => {
                control.background
            }
            (TreeItemVariant::Default, _) => Color::TRANSPARENT,
        };

        let text_color = match (variant, status) {
            (_, button::Status::Disabled) => disabled.foreground,
            (TreeItemVariant::Selected, _) => selected.foreground,
            (TreeItemVariant::Default, button::Status::Hovered | button::Status::Pressed) => {
                theme.text(TextRole::Primary).color
            }
            (TreeItemVariant::Default, _) => theme.text(TextRole::Secondary).color,
        };

        let border = match variant {
            TreeItemVariant::Selected => selected.border,
            TreeItemVariant::Default => BorderSpec::none(),
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

pub fn expander_style(
    radius: f32,
) -> impl Fn(&crate::theme::Theme, button::Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: button::Status| {
        let theme = *theme;
        let control = theme.control(ControlRole::Standard, button_control_state(status));

        button::Style {
            background: Some(Background::Color(match status {
                button::Status::Hovered | button::Status::Pressed => control.background,
                _ => Color::TRANSPARENT,
            })),
            text_color: match status {
                button::Status::Disabled => disabled_alpha(theme.text(TextRole::Muted).color),
                button::Status::Hovered | button::Status::Pressed => {
                    theme.text(TextRole::Primary).color
                }
                button::Status::Active => theme.text(TextRole::Muted).color,
            },
            border: transparent_border_with_radius(radius),
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    }
}

pub fn tone_indicator_style(
    tone: ToneRole,
    size: f32,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let tone = theme.tone(tone);

        container::Style {
            text_color: None,
            background: Some(Background::Color(tone.color)),
            border: Border {
                color: tone.border.color,
                width: 0.0,
                radius: Radius::new(size / 2.0),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

#[cfg(test)]
mod tree_item_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn selected_item_uses_app_selected_control_background() {
        let theme = Theme::Dark;
        let item = item_style(TreeItemVariant::Selected, 6.0)(&theme, button::Status::Active);

        assert_eq!(
            background_color(&item),
            theme
                .control(ControlRole::Selectable, ControlState::SELECTED)
                .background
        );
    }

    #[test]
    fn indentation_grows_with_size() {
        assert!(metrics(ControlSize::Xs).indent < metrics(ControlSize::Lg).indent);
    }

    fn background_color(style: &button::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
