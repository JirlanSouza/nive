use iced::{
    keyboard::{self, key::Named, Key, Modifiers},
    Event,
};

const UNIT_STEP: f32 = 0.01;
const UNIT_LARGE_STEP: f32 = 0.10;
const HUE_STEP: f32 = 1.0;
const HUE_LARGE_STEP: f32 = 10.0;
const HUE_MAX: f32 = 359.999;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyboardAction {
    Decrease,
    Increase,
    Min,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SaturationValueAction {
    Saturation(KeyboardAction),
    Value(KeyboardAction),
}

pub(super) fn slider_action(event: &Event) -> Option<(KeyboardAction, bool)> {
    let (key, modifiers) = key_press(event)?;
    let action = match key {
        Named::ArrowDown | Named::ArrowLeft => KeyboardAction::Increase,
        Named::ArrowUp | Named::ArrowRight => KeyboardAction::Decrease,
        Named::Home => KeyboardAction::Min,
        Named::End => KeyboardAction::Max,
        _ => return None,
    };

    Some((action, modifiers.shift()))
}

pub(super) fn saturation_value_action(event: &Event) -> Option<(SaturationValueAction, bool)> {
    let (key, modifiers) = key_press(event)?;
    let action = match key {
        Named::ArrowLeft => SaturationValueAction::Saturation(KeyboardAction::Decrease),
        Named::ArrowRight => SaturationValueAction::Saturation(KeyboardAction::Increase),
        Named::ArrowDown => SaturationValueAction::Value(KeyboardAction::Decrease),
        Named::ArrowUp => SaturationValueAction::Value(KeyboardAction::Increase),
        Named::Home => SaturationValueAction::Saturation(KeyboardAction::Min),
        Named::End => SaturationValueAction::Saturation(KeyboardAction::Max),
        _ => return None,
    };

    Some((action, modifiers.shift()))
}

pub(super) fn adjust_unit(value: f32, action: KeyboardAction, large: bool) -> f32 {
    let step = if large { UNIT_LARGE_STEP } else { UNIT_STEP };

    match action {
        KeyboardAction::Decrease => value - step,
        KeyboardAction::Increase => value + step,
        KeyboardAction::Min => 0.0,
        KeyboardAction::Max => 1.0,
    }
    .clamp(0.0, 1.0)
}

pub(super) fn adjust_hue(value: f32, action: KeyboardAction, large: bool) -> f32 {
    let step = if large { HUE_LARGE_STEP } else { HUE_STEP };

    match action {
        KeyboardAction::Decrease => value - step,
        KeyboardAction::Increase => value + step,
        KeyboardAction::Min => 0.0,
        KeyboardAction::Max => HUE_MAX,
    }
    .clamp(0.0, HUE_MAX)
}

fn key_press(event: &Event) -> Option<(Named, Modifiers)> {
    let Event::Keyboard(keyboard::Event::KeyPressed {
        key: Key::Named(key),
        modifiers,
        ..
    }) = event
    else {
        return None;
    };

    Some((*key, *modifiers))
}

#[cfg(test)]
mod keyboard_tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
    }

    #[test]
    fn unit_actions_clamp_to_range() {
        assert_eq!(adjust_unit(0.0, KeyboardAction::Decrease, false), 0.0);
        assert_eq!(adjust_unit(1.0, KeyboardAction::Increase, false), 1.0);
        assert_eq!(adjust_unit(0.5, KeyboardAction::Min, false), 0.0);
        assert_eq!(adjust_unit(0.5, KeyboardAction::Max, false), 1.0);
    }

    #[test]
    fn unit_actions_use_larger_shift_step() {
        assert_close(adjust_unit(0.5, KeyboardAction::Increase, false), 0.51);
        assert_close(adjust_unit(0.5, KeyboardAction::Increase, true), 0.6);
    }

    #[test]
    fn hue_actions_clamp_to_hue_range() {
        assert_eq!(adjust_hue(0.0, KeyboardAction::Decrease, false), 0.0);
        assert_eq!(adjust_hue(10.0, KeyboardAction::Min, false), 0.0);
        assert_eq!(adjust_hue(10.0, KeyboardAction::Max, false), HUE_MAX);
    }

    #[test]
    fn hue_actions_use_degree_steps() {
        assert_eq!(adjust_hue(100.0, KeyboardAction::Increase, false), 101.0);
        assert_eq!(adjust_hue(100.0, KeyboardAction::Increase, true), 110.0);
    }
}
