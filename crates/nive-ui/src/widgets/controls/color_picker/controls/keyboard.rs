use iced::{
    keyboard::{self, key::Named, Key, Modifiers},
    Event,
};

use crate::interaction::{Orientation, StepAdjustment};

const UNIT_STEP: f32 = 0.01;
const UNIT_LARGE_STEP: f32 = 0.10;
const HUE_STEP: f32 = 1.0;
const HUE_LARGE_STEP: f32 = 10.0;
const HUE_MAX: f32 = 359.999;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyboardAction {
    Adjust,
    Min,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct KeyboardAdjustment {
    action: KeyboardAction,
    delta: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum SaturationValueAction {
    Saturation(KeyboardAdjustment),
    Value(KeyboardAdjustment),
}

pub(super) fn unit_slider_action(event: &Event) -> Option<KeyboardAdjustment> {
    slider_action(event, StepAdjustment::new(UNIT_STEP, UNIT_LARGE_STEP))
}

pub(super) fn hue_slider_action(event: &Event) -> Option<KeyboardAdjustment> {
    slider_action(event, StepAdjustment::new(HUE_STEP, HUE_LARGE_STEP))
}

fn slider_action(event: &Event, adjustment: StepAdjustment) -> Option<KeyboardAdjustment> {
    let (key, modifiers) = key_press(event)?;
    let action = match named_key(key)? {
        Named::Home => KeyboardAdjustment::new(KeyboardAction::Min, 0.0),
        Named::End => KeyboardAdjustment::new(KeyboardAction::Max, 0.0),
        _ => {
            let delta = adjustment
                .delta(key, modifiers, Orientation::Vertical)
                .or_else(|| {
                    adjustment
                        .delta(key, modifiers, Orientation::Horizontal)
                        .map(|delta| -delta)
                })?;

            KeyboardAdjustment::new(KeyboardAction::Adjust, delta)
        }
    };

    Some(action)
}

pub(super) fn saturation_value_action(event: &Event) -> Option<SaturationValueAction> {
    let (key, modifiers) = key_press(event)?;
    match named_key(key)? {
        Named::Home => Some(SaturationValueAction::Saturation(KeyboardAdjustment::new(
            KeyboardAction::Min,
            0.0,
        ))),
        Named::End => Some(SaturationValueAction::Saturation(KeyboardAdjustment::new(
            KeyboardAction::Max,
            0.0,
        ))),
        _ => {
            let adjustment = StepAdjustment::new(UNIT_STEP, UNIT_LARGE_STEP);

            adjustment
                .delta(key, modifiers, Orientation::Horizontal)
                .map(|delta| {
                    SaturationValueAction::Saturation(KeyboardAdjustment::new(
                        KeyboardAction::Adjust,
                        delta,
                    ))
                })
                .or_else(|| {
                    adjustment
                        .delta(key, modifiers, Orientation::Vertical)
                        .map(|delta| {
                            SaturationValueAction::Value(KeyboardAdjustment::new(
                                KeyboardAction::Adjust,
                                -delta,
                            ))
                        })
                })
        }
    }
}

impl KeyboardAdjustment {
    const fn new(action: KeyboardAction, delta: f32) -> Self {
        Self { action, delta }
    }
}

pub(super) fn adjust_unit(value: f32, adjustment: KeyboardAdjustment) -> f32 {
    match adjustment.action {
        KeyboardAction::Adjust => value + adjustment.delta,
        KeyboardAction::Min => 0.0,
        KeyboardAction::Max => 1.0,
    }
    .clamp(0.0, 1.0)
}

pub(super) fn adjust_hue(value: f32, adjustment: KeyboardAdjustment) -> f32 {
    match adjustment.action {
        KeyboardAction::Adjust => value + adjustment.delta,
        KeyboardAction::Min => 0.0,
        KeyboardAction::Max => HUE_MAX,
    }
    .clamp(0.0, HUE_MAX)
}

fn key_press(event: &Event) -> Option<(&Key, Modifiers)> {
    let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event else {
        return None;
    };

    Some((key, *modifiers))
}

fn named_key(key: &Key) -> Option<Named> {
    let Key::Named(named) = key else {
        return None;
    };

    Some(*named)
}

#[cfg(test)]
mod keyboard_tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
    }

    #[test]
    fn unit_actions_clamp_to_range() {
        assert_eq!(
            adjust_unit(
                0.0,
                KeyboardAdjustment::new(KeyboardAction::Adjust, -UNIT_STEP)
            ),
            0.0
        );
        assert_eq!(
            adjust_unit(
                1.0,
                KeyboardAdjustment::new(KeyboardAction::Adjust, UNIT_STEP)
            ),
            1.0
        );
        assert_eq!(
            adjust_unit(0.5, KeyboardAdjustment::new(KeyboardAction::Min, 0.0)),
            0.0
        );
        assert_eq!(
            adjust_unit(0.5, KeyboardAdjustment::new(KeyboardAction::Max, 0.0)),
            1.0
        );
    }

    #[test]
    fn unit_actions_use_larger_shift_step() {
        assert_close(
            adjust_unit(
                0.5,
                KeyboardAdjustment::new(KeyboardAction::Adjust, UNIT_STEP),
            ),
            0.51,
        );
        assert_close(
            adjust_unit(
                0.5,
                KeyboardAdjustment::new(KeyboardAction::Adjust, UNIT_LARGE_STEP),
            ),
            0.6,
        );
    }

    #[test]
    fn hue_actions_clamp_to_hue_range() {
        assert_eq!(
            adjust_hue(
                0.0,
                KeyboardAdjustment::new(KeyboardAction::Adjust, -HUE_STEP)
            ),
            0.0
        );
        assert_eq!(
            adjust_hue(10.0, KeyboardAdjustment::new(KeyboardAction::Min, 0.0)),
            0.0
        );
        assert_eq!(
            adjust_hue(10.0, KeyboardAdjustment::new(KeyboardAction::Max, 0.0)),
            HUE_MAX
        );
    }

    #[test]
    fn hue_actions_use_degree_steps() {
        assert_eq!(
            adjust_hue(
                100.0,
                KeyboardAdjustment::new(KeyboardAction::Adjust, HUE_STEP)
            ),
            101.0
        );
        assert_eq!(
            adjust_hue(
                100.0,
                KeyboardAdjustment::new(KeyboardAction::Adjust, HUE_LARGE_STEP)
            ),
            110.0
        );
    }

    #[test]
    fn command_modifier_uses_large_tier() {
        let event = Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(Named::ArrowDown),
            modified_key: Key::Named(Named::ArrowDown),
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::ArrowDown),
            location: keyboard::Location::Standard,
            modifiers: Modifiers::COMMAND,
            text: None,
            repeat: false,
        });
        let action = unit_slider_action(&event).expect("command arrow maps to action");

        assert_close(adjust_unit(0.5, action), 0.7);
    }
}
