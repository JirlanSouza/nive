use iced::{widget::container, Background, Border, Color, Shadow};

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

pub(super) fn selected_item_background(theme: &crate::theme::Theme, disabled: bool) -> Color {
    let state = if disabled {
        ControlState::DISABLED.selected()
    } else {
        ControlState::SELECTED
    };
    theme.control(ControlRole::Selectable, state).background
}

pub(super) fn selected_indicator_color(theme: &crate::theme::Theme) -> Color {
    theme
        .control(ControlRole::Selectable, ControlState::SELECTED)
        .foreground
}
