use iced::{border::Radius, widget::container, Background, Border};

use crate::theme::{BorderSpec, ShapeSize, SurfaceRole, SurfaceSpec};

pub fn style(role: SurfaceRole) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let spec = theme.surface(role);
        container_style(spec, default_radius(*theme, role))
    }
}

pub fn style_with_radius(
    role: SurfaceRole,
    radius: Radius,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let spec = theme.surface(role);
        container_style(spec, radius)
    }
}

fn container_style(spec: SurfaceSpec, radius: Radius) -> container::Style {
    container::Style {
        text_color: Some(spec.foreground),
        background: Some(Background::Color(spec.background)),
        border: border(spec.border, radius),
        shadow: spec.shadow,
        ..container::Style::default()
    }
}

fn border(spec: BorderSpec, radius: Radius) -> Border {
    Border {
        color: spec.color,
        width: spec.width,
        radius,
    }
}

fn default_radius(theme: crate::theme::Theme, role: SurfaceRole) -> Radius {
    match role {
        SurfaceRole::Dialog | SurfaceRole::Popover => theme.shape(ShapeSize::Lg).radius(),
        _ => 0.0.into(),
    }
}

#[cfg(test)]
mod surface_tests {
    use iced::Background;

    use super::*;
    use crate::theme::Theme;

    #[test]
    fn sidebar_style_uses_app_semantic_sidebar_surface() {
        let theme = Theme::Dark;
        let style = style(SurfaceRole::Sidebar)(&theme);

        assert_eq!(
            background_color(&style),
            theme.surface(SurfaceRole::Sidebar).background
        );
    }

    #[test]
    fn popover_has_default_radius() {
        let theme = Theme::Dark;
        let style = style(SurfaceRole::Popover)(&theme);

        assert_eq!(style.border.radius, theme.shape(ShapeSize::Lg).radius());
    }

    fn background_color(style: &container::Style) -> iced::Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
