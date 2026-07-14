use iced::{border::Radius, widget::container, Background, Border};

use crate::theme::{BorderRole, BorderSpec, ShapeSize, SurfaceRole, SurfaceSpec};

/// Resolves a surface fill and shadow without emitting an automatic border.
///
/// Composing regions own structural seams. Use [`style_with_border`] when a
/// card or panel explicitly opts into an outline.
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

/// Explicit, widget-owned opt-in for a surface border (e.g. a `Card`/`Panel`
/// that asks for one). Surfaces never emit a border on their own — see
/// [`style`]/[`style_with_radius`].
pub fn style_with_border(
    role: SurfaceRole,
    radius: Radius,
    border_role: BorderRole,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let spec = theme.surface(role);
        let mut style = container_style(spec, radius);
        style.border = border(theme.border(border_role), radius);
        style
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

    #[test]
    fn structural_surfaces_render_without_an_automatic_border() {
        let theme = Theme::Dark;

        for role in [
            SurfaceRole::App,
            SurfaceRole::Chrome,
            SurfaceRole::Sidebar,
            SurfaceRole::Canvas,
            SurfaceRole::Panel,
            SurfaceRole::Elevated,
        ] {
            let style = style(role)(&theme);

            assert_eq!(style.border.width, 0.0, "{role:?} should have no border");
        }
    }

    #[test]
    fn style_with_border_opts_into_an_explicit_border() {
        let theme = Theme::Dark;
        let style = style_with_border(SurfaceRole::Panel, 0.0.into(), BorderRole::Default)(&theme);

        assert_eq!(style.border.color, theme.border(BorderRole::Default).color);
        assert!(style.border.width > 0.0);
    }

    fn background_color(style: &container::Style) -> iced::Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
