use iced::{widget::container, Background, Border, Shadow};

use crate::theme::SurfaceRole;

pub(super) fn rail_container_style(
    role: SurfaceRole,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let surface = theme.surface(role);

        container::Style {
            text_color: Some(surface.foreground),
            background: Some(Background::Color(surface.background)),
            border: Border {
                color: surface.border.color,
                width: surface.border.width,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}
