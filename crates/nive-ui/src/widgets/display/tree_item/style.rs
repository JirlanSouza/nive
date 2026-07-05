use iced::{
    border::Radius,
    widget::{button, container, text},
    Background, Border, Color, Shadow,
};

use crate::advanced::control_style::{
    border_with_radius, disabled_alpha, transparent_border, transparent_border_with_radius,
};
use crate::widgets::controls::button::button_control_state;

use crate::theme::{
    self, control_metrics, BorderSpec, ControlRole, ControlSize, ControlState, TextRole,
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
            ControlSize::Xs => spacing.md,
            ControlSize::Sm => spacing.lg,
            ControlSize::Md => spacing.xl,
            ControlSize::Lg => spacing.xl + spacing.xs,
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
        let disabled = theme.control(ControlRole::Standard, ControlState::DISABLED);

        let background = match (variant, status) {
            (TreeItemVariant::Selected, _) => Color::TRANSPARENT,
            (TreeItemVariant::Default, button::Status::Hovered | button::Status::Pressed) => {
                control.background
            }
            (TreeItemVariant::Default, _) => Color::TRANSPARENT,
        };

        let text_color = match (variant, status) {
            (_, button::Status::Disabled) => disabled.foreground,
            (TreeItemVariant::Selected, _) => theme.text(TextRole::Primary).color,
            (TreeItemVariant::Default, button::Status::Hovered | button::Status::Pressed) => {
                theme.text(TextRole::Primary).color
            }
            (TreeItemVariant::Default, _) => theme.text(TextRole::Primary).color,
        };

        let border_radius = match variant {
            TreeItemVariant::Selected => 0.0,
            TreeItemVariant::Default => radius,
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color,
            border: border_with_radius(BorderSpec::none(), border_radius),
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    }
}

pub fn expander_style(
    selected: bool,
    radius: f32,
) -> impl Fn(&crate::theme::Theme, button::Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: button::Status| {
        let theme = *theme;
        let control = theme.control(ControlRole::Standard, button_control_state(status));

        let background = match (selected, status) {
            (true, _) => Color::TRANSPARENT,
            (false, button::Status::Hovered | button::Status::Pressed) => control.background,
            (false, _) => Color::TRANSPARENT,
        };

        let text_color = match (selected, status) {
            (_, button::Status::Disabled) => disabled_alpha(theme.text(TextRole::Muted).color),
            (true, _) => theme.text(TextRole::Primary).color,
            (false, button::Status::Hovered | button::Status::Pressed) => {
                theme.text(TextRole::Primary).color
            }
            (false, button::Status::Active) => theme.text(TextRole::Secondary).color,
        };

        let border_radius = if selected { 0.0 } else { radius };

        button::Style {
            background: Some(Background::Color(background)),
            text_color,
            border: transparent_border_with_radius(border_radius),
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    }
}

pub fn indent_style(
    selected: bool,
) -> impl Fn(&crate::theme::Theme, button::Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: button::Status| {
        let theme = *theme;
        let control = theme.control(ControlRole::Standard, button_control_state(status));

        let background = if selected {
            Color::TRANSPARENT
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => control.background,
                _ => Color::TRANSPARENT,
            }
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: Color::TRANSPARENT,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: Radius::new(0.0),
            },
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    }
}

pub fn row_style(
    selected: bool,
    disabled: bool,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let theme = *theme;
        let selected_control = theme.control(ControlRole::Selectable, ControlState::SELECTED);

        let background = if selected {
            if disabled {
                selected_control.background.scale_alpha(0.55)
            } else {
                selected_control.background
            }
        } else {
            Color::TRANSPARENT
        };

        container::Style {
            text_color: None,
            background: Some(Background::Color(background)),
            border: transparent_border(),
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

pub fn trailing_text_style(disabled: bool) -> impl Fn(&crate::theme::Theme) -> text::Style {
    move |theme: &crate::theme::Theme| {
        let theme = *theme;
        let disabled_control = theme.control(ControlRole::Standard, ControlState::DISABLED);

        let color = if disabled {
            disabled_control.foreground
        } else {
            theme.text(TextRole::Secondary).color
        };

        text::Style { color: Some(color) }
    }
}

#[cfg(test)]
mod tree_item_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn selected_item_uses_app_selected_control_background() {
        let theme = Theme::Dark;
        let item = row_style(true, false)(&theme);

        assert_eq!(
            background_color(&item),
            theme
                .control(ControlRole::Selectable, ControlState::SELECTED)
                .background
        );
        assert_eq!(item.border.width, 0.0);
        assert_eq!(item.border.radius, Radius::default());
    }

    #[test]
    fn selected_item_button_keeps_row_background_visible() {
        let theme = Theme::Dark;
        let item = item_style(TreeItemVariant::Selected, 6.0)(&theme, button::Status::Active);

        assert_eq!(button_background_color(&item), Color::TRANSPARENT);
        assert_eq!(item.text_color, theme.text(TextRole::Primary).color);
        assert_eq!(item.border.width, 0.0);
        assert_eq!(item.border.radius, Radius::default());
    }

    #[test]
    fn default_item_uses_primary_text() {
        let theme = Theme::Dark;
        let item = item_style(TreeItemVariant::Default, 6.0)(&theme, button::Status::Active);

        assert_eq!(item.text_color, theme.text(TextRole::Primary).color);
    }

    #[test]
    fn indentation_grows_with_size() {
        assert!(metrics(ControlSize::Xs).indent < metrics(ControlSize::Lg).indent);
    }

    #[test]
    fn sm_indentation_uses_compact_theme_spacing() {
        assert_eq!(metrics(ControlSize::Sm).indent, theme::spacing().lg);
    }

    #[test]
    fn indent_hover_shows_control_background() {
        let theme = Theme::Dark;
        let style = indent_style(false)(&theme, button::Status::Hovered);
        let expected = theme
            .control(ControlRole::Standard, ControlState::HOVERED)
            .background;

        assert_eq!(button_background_color(&style), expected);
    }

    #[test]
    fn indent_selected_hover_is_transparent() {
        let theme = Theme::Dark;
        let style = indent_style(true)(&theme, button::Status::Hovered);

        assert_eq!(button_background_color(&style), Color::TRANSPARENT);
    }

    #[test]
    fn indent_active_is_transparent() {
        let theme = Theme::Dark;
        let style = indent_style(false)(&theme, button::Status::Active);

        assert_eq!(button_background_color(&style), Color::TRANSPARENT);
    }

    fn background_color(style: &container::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }

    fn button_background_color(style: &button::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
