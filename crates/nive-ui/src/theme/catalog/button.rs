use iced::{widget::button, Background, Color, Shadow};

use super::shared::{border_with_radius, transparent_border_with_radius};
use crate::theme::{ControlRole, ControlState, ShapeRole, TextRole, Theme, ToneRole};

pub(super) fn button_control_state(status: button::Status) -> ControlState {
    match status {
        button::Status::Active => ControlState::ENABLED,
        button::Status::Hovered => ControlState::HOVERED,
        button::Status::Pressed => ControlState::PRESSED,
        button::Status::Disabled => ControlState::DISABLED,
    }
}

pub(super) fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let tone = theme.tone(ToneRole::Primary);
    let alpha = match status {
        button::Status::Active => 1.0,
        button::Status::Hovered => 0.90,
        button::Status::Pressed => 0.82,
        button::Status::Disabled => 0.45,
    };

    button::Style {
        background: Some(Background::Color(tone.color.scale_alpha(alpha))),
        text_color: theme.tone(ToneRole::Primary).on_color.scale_alpha(
            if matches!(status, button::Status::Disabled) {
                0.65
            } else {
                1.0
            },
        ),
        border: transparent_border_with_radius(theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

pub(super) fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);
    let disabled = theme.control(ControlRole::Standard, ControlState::DISABLED);
    let (background, text_color, border) = match status {
        button::Status::Active => (selected.background, selected.foreground, selected.border),
        button::Status::Hovered => (
            selected.background.scale_alpha(1.25),
            selected.foreground,
            selected.border,
        ),
        button::Status::Pressed => (
            selected.background.scale_alpha(0.88),
            selected.foreground,
            selected.border,
        ),
        button::Status::Disabled => (disabled.background, disabled.foreground, disabled.border),
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: border_with_radius(border, theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

pub(super) fn outline_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let control = theme.control(ControlRole::Standard, button_control_state(status));
    let background = match status {
        button::Status::Active | button::Status::Disabled => Color::TRANSPARENT,
        button::Status::Hovered | button::Status::Pressed => control.background,
    };
    let text_color = if matches!(status, button::Status::Disabled) {
        control.foreground
    } else {
        theme.text(TextRole::Primary).color
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: border_with_radius(control.border, theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

pub(super) fn ghost_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let control = theme.control(ControlRole::Standard, button_control_state(status));
    let background = match status {
        button::Status::Active | button::Status::Disabled => Color::TRANSPARENT,
        button::Status::Hovered | button::Status::Pressed => control.background,
    };
    let text_color = match status {
        button::Status::Hovered | button::Status::Pressed => theme.text(TextRole::Primary).color,
        button::Status::Disabled => theme.text(TextRole::Muted).color.scale_alpha(0.55),
        button::Status::Active => theme.text(TextRole::Secondary).color,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: transparent_border_with_radius(theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

pub(super) fn destructive_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let tone = theme.tone(ToneRole::Danger);
    let background = match status {
        button::Status::Active => tone.container,
        button::Status::Hovered => tone.container.scale_alpha(1.35),
        button::Status::Pressed => tone.container.scale_alpha(1.12),
        button::Status::Disabled => tone.container.scale_alpha(0.55),
    };
    let text_color = if matches!(status, button::Status::Disabled) {
        tone.color.scale_alpha(0.55)
    } else {
        tone.color
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: border_with_radius(tone.border, theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

pub(super) fn link_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let tone = theme.tone(ToneRole::Primary);
    let alpha = match status {
        button::Status::Active => 1.0,
        button::Status::Hovered => 0.88,
        button::Status::Pressed => 0.76,
        button::Status::Disabled => 0.50,
    };

    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: tone.color.scale_alpha(alpha),
        border: transparent_border_with_radius(theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

pub(super) fn embedded_button(theme: &Theme, status: button::Status) -> button::Style {
    let theme = *theme;
    let control = theme.control(ControlRole::Embedded, button_control_state(status));
    let foreground = match status {
        button::Status::Hovered | button::Status::Pressed => theme.text(TextRole::Primary).color,
        button::Status::Disabled => theme.text(TextRole::Muted).color.scale_alpha(0.55),
        button::Status::Active => theme.text(TextRole::Secondary).color,
    };
    let background = match status {
        button::Status::Active | button::Status::Disabled => Color::TRANSPARENT,
        button::Status::Hovered => control.background.scale_alpha(0.70),
        button::Status::Pressed => control.background.scale_alpha(0.86),
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: foreground,
        border: transparent_border_with_radius(theme.shape(ShapeRole::Medium).radius()),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}
