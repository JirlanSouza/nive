use iced::{
    widget::{checkbox, text_input, toggler},
    Background, Color,
};

use super::classes::FieldValidation;
use super::shared::{alpha_when_disabled, border_with_radius, transparent_border_with_radius};
use crate::theme::{
    choice::{self, ChoicePersistentState, ChoiceStateInput},
    BorderRole, BorderSpec, ControlRole, ControlState, InteractionState, ShapeSize, TextRole,
    Theme, ToneRole,
};

pub(super) fn default_checkbox(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let theme = *theme;
    let is_checked = match status {
        checkbox::Status::Active { is_checked }
        | checkbox::Status::Hovered { is_checked }
        | checkbox::Status::Disabled { is_checked } => is_checked,
    };
    let disabled = matches!(status, checkbox::Status::Disabled { .. });
    let resolved = choice::resolve_state(ChoiceStateInput {
        persistent: if is_checked {
            ChoicePersistentState::Selected
        } else {
            ChoicePersistentState::Unselected
        },
        validation: FieldValidation::Valid,
        callback_present: !disabled,
        disabled,
        hovered: matches!(status, checkbox::Status::Hovered { .. }),
        pressed: false,
        focused: false,
    });
    let palette = choice::palette(theme, resolved);

    checkbox::Style {
        background: Background::Color(palette.background),
        icon_color: if is_checked {
            palette.mark
        } else {
            Color::TRANSPARENT
        },
        border: border_with_radius(
            BorderSpec::new(palette.perimeter, 1.0),
            theme.shape(ShapeSize::Sm).radius(),
        ),
        text_color: Some(palette.foreground),
    }
}

pub(super) fn standard_text_input(
    theme: &Theme,
    validation: FieldValidation,
    status: text_input::Status,
) -> text_input::Style {
    let theme = *theme;
    let control = theme.control(ControlRole::Standard, text_input_control_state(status));
    let muted = theme.text(TextRole::Muted).color;
    let disabled = matches!(status, text_input::Status::Disabled);

    let mut style = text_input::Style {
        background: Background::Color(control.background),
        border: border_with_radius(control.border, theme.shape(ShapeSize::Md).radius()),
        icon: alpha_when_disabled(muted, disabled),
        placeholder: alpha_when_disabled(muted, disabled),
        value: alpha_when_disabled(control.foreground, disabled),
        selection: theme.tone(ToneRole::Accent).color.scale_alpha(if disabled {
            0.15
        } else {
            0.30
        }),
    };

    apply_standard_text_input_validation(&mut style, theme, validation, disabled);

    style
}

pub(super) fn embedded_text_input(
    theme: &Theme,
    _validation: FieldValidation,
    status: text_input::Status,
) -> text_input::Style {
    let theme = *theme;
    let disabled = matches!(status, text_input::Status::Disabled);
    let placeholder = theme
        .text(if disabled {
            TextRole::Disabled
        } else {
            TextRole::Muted
        })
        .color;
    let value = theme
        .text(if disabled {
            TextRole::Disabled
        } else {
            TextRole::Primary
        })
        .color;

    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: transparent_border_with_radius(theme.shape(ShapeSize::Md).radius()),
        icon: placeholder,
        placeholder,
        value,
        selection: theme.tone(ToneRole::Accent).color.scale_alpha(if disabled {
            0.15
        } else {
            0.30
        }),
    }
}

fn apply_standard_text_input_validation(
    style: &mut text_input::Style,
    theme: Theme,
    validation: FieldValidation,
    disabled: bool,
) {
    if matches!(validation, FieldValidation::Invalid) {
        let danger = theme.border(BorderRole::Danger);

        style.border.color = alpha_when_disabled(danger.color, disabled);
        style.border.width = danger.width;
        style.selection =
            theme
                .tone(ToneRole::Danger)
                .color
                .scale_alpha(if disabled { 0.1 } else { 0.2 });
    }
}

pub(super) fn default_toggler(theme: &Theme, status: toggler::Status) -> toggler::Style {
    let theme = *theme;
    let is_toggled = match status {
        toggler::Status::Active { is_toggled }
        | toggler::Status::Hovered { is_toggled }
        | toggler::Status::Disabled { is_toggled } => is_toggled,
    };
    let disabled = matches!(status, toggler::Status::Disabled { .. });
    let resolved = choice::resolve_state(ChoiceStateInput {
        persistent: if is_toggled {
            ChoicePersistentState::Selected
        } else {
            ChoicePersistentState::Unselected
        },
        validation: FieldValidation::Valid,
        callback_present: !disabled,
        disabled,
        hovered: matches!(status, toggler::Status::Hovered { .. }),
        pressed: false,
        focused: false,
    });
    let palette = choice::palette(theme, resolved);

    toggler::Style {
        background: Background::Color(palette.background),
        foreground: Background::Color(palette.mark),
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        background_border_width: 1.0,
        background_border_color: palette.perimeter,
        text_color: None,
        border_radius: None,
        padding_ratio: 0.1,
    }
}

fn text_input_control_state(status: text_input::Status) -> ControlState {
    match status {
        text_input::Status::Active => ControlState::ENABLED,
        text_input::Status::Hovered => ControlState::HOVERED,
        text_input::Status::Focused { is_hovered } => {
            ControlState::new().interaction(if is_hovered {
                InteractionState::FOCUSED.hovered()
            } else {
                InteractionState::FOCUSED
            })
        }
        text_input::Status::Disabled => ControlState::DISABLED,
    }
}
