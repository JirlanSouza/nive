use iced::{widget::Id, Color};

use super::state::ColorPickerState;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum ColorPickerEvent {
    FocusControl(ColorPickerControl),
    SaturationValueChanged { saturation: f32, value: f32 },
    HueChanged(f32),
    AlphaChanged(f32),
    HexInput(String),
    AlphaInput(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ColorPickerControl {
    SaturationValue,
    Hue,
    Alpha,
}

impl ColorPickerControl {
    pub(super) fn id(self) -> Id {
        match self {
            Self::SaturationValue => Id::new("color-picker-saturation-value"),
            Self::Hue => Id::new("color-picker-hue"),
            Self::Alpha => Id::new("color-picker-alpha"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct ColorPickerTransition {
    changed: Option<Color>,
    focus: Option<ColorPickerControl>,
    redraw: bool,
}

impl ColorPickerEvent {
    pub(super) fn apply(self, state: &mut ColorPickerState) -> ColorPickerTransition {
        let (changed, focus) = match self {
            Self::FocusControl(control) => (None, Some(control)),
            Self::SaturationValueChanged { saturation, value } => {
                (Some(state.set_saturation_value(saturation, value)), None)
            }
            Self::HueChanged(hue) => (Some(state.set_hue(hue)), None),
            Self::AlphaChanged(alpha) => (Some(state.set_alpha(alpha)), None),
            Self::HexInput(value) => (state.input_hex(value), None),
            Self::AlphaInput(value) => (state.input_alpha(value), None),
        };

        ColorPickerTransition {
            changed,
            focus,
            redraw: true,
        }
    }
}

impl ColorPickerTransition {
    pub(super) fn changed(self) -> Option<Color> {
        self.changed
    }

    pub(super) fn focus(self) -> Option<ColorPickerControl> {
        self.focus
    }

    pub(super) fn redraw(self) -> bool {
        self.redraw
    }
}
