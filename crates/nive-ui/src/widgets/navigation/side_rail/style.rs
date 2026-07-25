use iced::{
    widget::{container, rule},
    Background, Border, Color, Shadow,
};

use crate::theme::{ControlRole, ControlState, SurfaceRole};

pub(super) fn rail_container_style(
    role: SurfaceRole,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let surface = theme.surface(role);

        container::Style {
            text_color: Some(surface.foreground),
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border::default(),
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

pub(super) fn rail_background(theme: &crate::theme::Theme) -> Color {
    theme.surface(SurfaceRole::Chrome).background
}

pub(super) fn seam_color(theme: &crate::theme::Theme) -> Color {
    theme.surface(SurfaceRole::Chrome).border.color
}

pub(super) fn selected_accent_style() -> impl Fn(&crate::theme::Theme) -> rule::Style {
    move |theme| rule::Style {
        color: theme
            .control(ControlRole::Selectable, ControlState::SELECTED)
            .foreground,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}
