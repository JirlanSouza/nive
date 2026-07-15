use iced::{
    border::Radius,
    widget::{button, container},
    Background, Border, Color, Padding, Shadow,
};

use crate::theme::{
    BorderRole, BorderSpec, ControlRole, ControlState, InteractionState, PaddingRole, ShapeSize,
    SurfaceRole, TextRole, Theme,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CardVariant {
    /// Panel fill without perimeter or shadow.
    #[default]
    Filled,
    /// Transparent fill with one semantic perimeter.
    Outlined,
    /// Elevated semantic fill and shadow without perimeter.
    Elevated,
    /// Transparent frame without perimeter or shadow.
    Ghost,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct CardFrameMetrics {
    pub(super) radius: f32,
    pub(super) padding: Padding,
}

pub(super) fn metrics(theme: Theme) -> CardFrameMetrics {
    CardFrameMetrics {
        radius: theme.shape(ShapeSize::Md).radius_value(),
        padding: theme.padding(PaddingRole::Content),
    }
}

pub(super) fn base_style(variant: CardVariant, radius: f32) -> impl Fn(&Theme) -> container::Style {
    move |theme| resolve_base_style(*theme, variant, radius)
}

pub(super) fn resolve_base_style(
    theme: Theme,
    variant: CardVariant,
    radius: f32,
) -> container::Style {
    let panel = theme.surface(SurfaceRole::Panel);
    let (background, border, shadow) = match variant {
        CardVariant::Filled => (panel.background, BorderSpec::none(), Shadow::default()),
        CardVariant::Outlined => (
            Color::TRANSPARENT,
            theme.border(BorderRole::Default),
            Shadow::default(),
        ),
        CardVariant::Elevated => {
            let elevated = theme.surface(SurfaceRole::Elevated);
            (elevated.background, BorderSpec::none(), elevated.shadow)
        }
        CardVariant::Ghost => (Color::TRANSPARENT, BorderSpec::none(), Shadow::default()),
    };

    container::Style {
        text_color: Some(panel.foreground),
        background: Some(Background::Color(background)),
        border: border_with_radius(border, radius),
        shadow,
        snap: true,
    }
}

pub(super) fn interaction_style(
    selected: bool,
    explicitly_disabled: bool,
    capable: bool,
    radius: f32,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        resolve_interaction_style(
            *theme,
            selected,
            explicitly_disabled,
            capable,
            radius,
            status,
        )
    }
}

pub(super) fn resolve_interaction_style(
    theme: Theme,
    selected: bool,
    explicitly_disabled: bool,
    capable: bool,
    radius: f32,
    status: button::Status,
) -> button::Style {
    let status = if !explicitly_disabled && !capable {
        button::Status::Active
    } else {
        status
    };
    let interaction = match status {
        button::Status::Hovered => InteractionState::HOVERED,
        button::Status::Pressed => InteractionState::PRESSED,
        button::Status::Active | button::Status::Disabled => InteractionState::NONE,
    };
    let mut state = ControlState::new().interaction(interaction);
    if selected {
        state = state.selected();
    }
    if explicitly_disabled {
        state = state.disabled();
    }

    let control = theme.control(
        if selected {
            ControlRole::Selectable
        } else {
            ControlRole::Embedded
        },
        state,
    );
    let interacting = capable
        && !explicitly_disabled
        && matches!(status, button::Status::Hovered | button::Status::Pressed);
    let background = if selected || interacting {
        control.background
    } else {
        Color::TRANSPARENT
    };
    let foreground = if explicitly_disabled {
        theme.text(TextRole::Disabled).color
    } else if selected {
        control.foreground
    } else if interacting {
        theme.text(TextRole::Primary).color
    } else {
        theme.surface(SurfaceRole::Panel).foreground
    };
    let border = if selected {
        control.border
    } else {
        BorderSpec::none()
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: foreground,
        border: border_with_radius(border, radius),
        shadow: Shadow::default(),
        snap: true,
    }
}

fn border_with_radius(border: BorderSpec, radius: f32) -> Border {
    Border {
        color: border.color,
        width: border.width,
        radius: Radius::new(radius),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ThemeBuilder, ThemeDensity, ThemeMode};

    #[test]
    fn shared_metrics_follow_md_shape_and_content_padding_for_every_density() {
        for (density, expected_padding) in [
            (ThemeDensity::Compact, 8.0),
            (ThemeDensity::Standard, 12.0),
            (ThemeDensity::Comfortable, 14.0),
        ] {
            let theme = ThemeBuilder::new("Card metrics", ThemeMode::Dark)
                .density(density)
                .build();
            let metrics = metrics(theme);

            assert_eq!(metrics.radius, theme.shape(ShapeSize::Md).radius_value());
            assert_eq!(metrics.padding, Padding::new(expected_padding));
        }
    }

    #[test]
    fn variants_keep_one_owner_for_fill_border_and_shadow_in_both_modes() {
        for (mode, density) in [ThemeMode::Light, ThemeMode::Dark]
            .into_iter()
            .flat_map(|mode| ThemeDensity::ALL.map(|density| (mode, density)))
        {
            let theme = ThemeBuilder::new("Card variants", mode)
                .density(density)
                .build();
            let radius = metrics(theme).radius;
            let filled = resolve_base_style(theme, CardVariant::Filled, radius);
            let outlined = resolve_base_style(theme, CardVariant::Outlined, radius);
            let elevated = resolve_base_style(theme, CardVariant::Elevated, radius);
            let ghost = resolve_base_style(theme, CardVariant::Ghost, radius);

            assert_eq!(filled.border.width, 0.0);
            assert_eq!(filled.shadow, Shadow::default());
            assert_eq!(
                outlined.border,
                border_with_radius(theme.border(BorderRole::Default), radius)
            );
            assert_eq!(outlined.shadow, Shadow::default());
            assert_eq!(elevated.border.width, 0.0);
            assert_eq!(elevated.shadow, theme.surface(SurfaceRole::Elevated).shadow);
            assert_eq!(ghost.border.width, 0.0);
            assert_eq!(ghost.shadow, Shadow::default());
            assert_eq!(background(&outlined), Color::TRANSPARENT);
            assert_eq!(background(&ghost), Color::TRANSPARENT);
        }
    }

    #[test]
    fn interaction_matrix_preserves_capability_disabled_and_selection_semantics() {
        for theme in [Theme::Light, Theme::Dark] {
            for variant in [
                CardVariant::Filled,
                CardVariant::Outlined,
                CardVariant::Elevated,
                CardVariant::Ghost,
            ] {
                let radius = metrics(theme).radius;
                let base = resolve_base_style(theme, variant, radius);
                let idle = resolve_interaction_style(
                    theme,
                    false,
                    false,
                    true,
                    radius,
                    button::Status::Active,
                );
                let absent = resolve_interaction_style(
                    theme,
                    false,
                    false,
                    false,
                    radius,
                    button::Status::Disabled,
                );
                let disabled = resolve_interaction_style(
                    theme,
                    false,
                    true,
                    true,
                    radius,
                    button::Status::Disabled,
                );
                let selected = resolve_interaction_style(
                    theme,
                    true,
                    false,
                    true,
                    radius,
                    button::Status::Active,
                );

                assert_eq!(background_button(&idle), Color::TRANSPARENT);
                assert_eq!(absent, idle);
                assert_eq!(disabled.text_color, theme.text(TextRole::Disabled).color);
                assert_eq!(selected.border.width, 1.0);
                assert_eq!(
                    selected.border.color,
                    theme.border(BorderRole::Accent).color
                );
                assert_eq!(
                    base.shadow,
                    resolve_base_style(theme, variant, radius).shadow
                );
            }
        }
    }

    fn background(style: &container::Style) -> Color {
        match style.background {
            Some(Background::Color(color)) => color,
            _ => panic!("card frame must use a color background"),
        }
    }

    fn background_button(style: &button::Style) -> Color {
        match style.background {
            Some(Background::Color(color)) => color,
            _ => panic!("card interaction layer must use a color background"),
        }
    }
}
