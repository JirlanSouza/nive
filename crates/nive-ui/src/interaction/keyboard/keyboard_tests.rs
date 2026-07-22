use iced::keyboard::{
    key::{Code, Named, Physical},
    Event, Key, Location, Modifiers,
};

use super::*;
use crate::interaction::Orientation;

fn key_pressed(key: Key) -> Event {
    Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: Physical::Code(Code::Enter),
        location: Location::Standard,
        modifiers: Modifiers::NONE,
        text: None,
        repeat: false,
    }
}

fn key_pressed_with_modifiers(key: Key, modifiers: Modifiers) -> Event {
    Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: Physical::Code(Code::Enter),
        location: Location::Standard,
        modifiers,
        text: None,
        repeat: false,
    }
}

#[test]
fn platform_activation_resolves_for_target_os() {
    let resolved = ActivationBehavior::Platform.resolve();

    if cfg!(target_os = "macos") {
        assert_eq!(resolved, ActivationBehavior::SpaceCommandOpenAndDoubleClick);
    } else {
        assert_eq!(resolved, ActivationBehavior::EnterAndDoubleClick);
    }
}

#[test]
fn platform_rename_resolves_for_target_os() {
    let resolved = RenameBehavior::Platform.resolve();

    if cfg!(target_os = "macos") {
        assert_eq!(resolved, RenameBehavior::Return);
    } else {
        assert_eq!(resolved, RenameBehavior::F2);
    }
}

#[test]
fn explicit_activation_behaviors_resolve_to_themselves() {
    assert_eq!(
        ActivationBehavior::DoubleClick.resolve(),
        ActivationBehavior::DoubleClick
    );
    assert_eq!(
        ActivationBehavior::Enter.resolve(),
        ActivationBehavior::Enter
    );
    assert_eq!(
        ActivationBehavior::Space.resolve(),
        ActivationBehavior::Space
    );
    assert_eq!(
        ActivationBehavior::EnterAndDoubleClick.resolve(),
        ActivationBehavior::EnterAndDoubleClick
    );
    assert_eq!(
        ActivationBehavior::SpaceAndDoubleClick.resolve(),
        ActivationBehavior::SpaceAndDoubleClick
    );
    assert_eq!(
        ActivationBehavior::EnterSpaceAndDoubleClick.resolve(),
        ActivationBehavior::EnterSpaceAndDoubleClick
    );
    assert_eq!(
        ActivationBehavior::CommandOpenAndDoubleClick.resolve(),
        ActivationBehavior::CommandOpenAndDoubleClick
    );
    assert_eq!(
        ActivationBehavior::SpaceCommandOpenAndDoubleClick.resolve(),
        ActivationBehavior::SpaceCommandOpenAndDoubleClick
    );
}

#[test]
fn click_trigger_is_not_included_in_existing_activation_presets() {
    for behavior in [
        ActivationBehavior::DoubleClick,
        ActivationBehavior::Enter,
        ActivationBehavior::Space,
        ActivationBehavior::EnterAndDoubleClick,
        ActivationBehavior::SpaceAndDoubleClick,
        ActivationBehavior::EnterSpaceAndDoubleClick,
        ActivationBehavior::CommandOpenAndDoubleClick,
        ActivationBehavior::SpaceCommandOpenAndDoubleClick,
    ] {
        assert!(!behavior.includes(ActivationTrigger::Click));
    }
}

#[test]
fn explicit_rename_behaviors_resolve_to_themselves() {
    assert_eq!(RenameBehavior::Disabled.resolve(), RenameBehavior::Disabled);
    assert_eq!(RenameBehavior::F2.resolve(), RenameBehavior::F2);
    assert_eq!(RenameBehavior::Return.resolve(), RenameBehavior::Return);
    assert_eq!(
        RenameBehavior::F2OrReturn.resolve(),
        RenameBehavior::F2OrReturn
    );
}

#[test]
fn double_click_behavior_includes_only_double_click() {
    let behavior = ActivationBehavior::DoubleClick;
    assert!(behavior.should_activate(ActivationTrigger::DoubleClick));
    assert!(!behavior.should_activate(ActivationTrigger::Enter));
    assert!(!behavior.should_activate(ActivationTrigger::Space));
    assert!(!behavior.should_activate(ActivationTrigger::CommandO));
    assert!(!behavior.should_activate(ActivationTrigger::CommandDown));
}

#[test]
fn enter_behavior_includes_only_enter() {
    let behavior = ActivationBehavior::Enter;
    assert!(behavior.should_activate(ActivationTrigger::Enter));
    assert!(!behavior.should_activate(ActivationTrigger::DoubleClick));
    assert!(!behavior.should_activate(ActivationTrigger::Space));
    assert!(!behavior.should_activate(ActivationTrigger::CommandO));
    assert!(!behavior.should_activate(ActivationTrigger::CommandDown));
}

#[test]
fn space_behavior_includes_only_space() {
    let behavior = ActivationBehavior::Space;
    assert!(behavior.should_activate(ActivationTrigger::Space));
    assert!(!behavior.should_activate(ActivationTrigger::Enter));
    assert!(!behavior.should_activate(ActivationTrigger::DoubleClick));
    assert!(!behavior.should_activate(ActivationTrigger::CommandO));
    assert!(!behavior.should_activate(ActivationTrigger::CommandDown));
}

#[test]
fn enter_and_double_click_behavior() {
    let behavior = ActivationBehavior::EnterAndDoubleClick;
    assert!(behavior.should_activate(ActivationTrigger::Enter));
    assert!(behavior.should_activate(ActivationTrigger::DoubleClick));
    assert!(!behavior.should_activate(ActivationTrigger::Space));
    assert!(!behavior.should_activate(ActivationTrigger::CommandO));
    assert!(!behavior.should_activate(ActivationTrigger::CommandDown));
}

#[test]
fn space_and_double_click_behavior() {
    let behavior = ActivationBehavior::SpaceAndDoubleClick;
    assert!(behavior.should_activate(ActivationTrigger::Space));
    assert!(behavior.should_activate(ActivationTrigger::DoubleClick));
    assert!(!behavior.should_activate(ActivationTrigger::Enter));
    assert!(!behavior.should_activate(ActivationTrigger::CommandO));
    assert!(!behavior.should_activate(ActivationTrigger::CommandDown));
}

#[test]
fn enter_space_and_double_click_behavior() {
    let behavior = ActivationBehavior::EnterSpaceAndDoubleClick;
    assert!(behavior.should_activate(ActivationTrigger::Enter));
    assert!(behavior.should_activate(ActivationTrigger::Space));
    assert!(behavior.should_activate(ActivationTrigger::DoubleClick));
    assert!(!behavior.should_activate(ActivationTrigger::CommandO));
    assert!(!behavior.should_activate(ActivationTrigger::CommandDown));
}

#[test]
fn command_open_and_double_click_behavior() {
    let behavior = ActivationBehavior::CommandOpenAndDoubleClick;
    assert!(behavior.should_activate(ActivationTrigger::CommandO));
    assert!(behavior.should_activate(ActivationTrigger::CommandDown));
    assert!(behavior.should_activate(ActivationTrigger::DoubleClick));
    assert!(!behavior.should_activate(ActivationTrigger::Enter));
    assert!(!behavior.should_activate(ActivationTrigger::Space));
}

#[test]
fn space_command_open_and_double_click_behavior() {
    let behavior = ActivationBehavior::SpaceCommandOpenAndDoubleClick;
    assert!(behavior.should_activate(ActivationTrigger::Space));
    assert!(behavior.should_activate(ActivationTrigger::CommandO));
    assert!(behavior.should_activate(ActivationTrigger::CommandDown));
    assert!(behavior.should_activate(ActivationTrigger::DoubleClick));
    assert!(!behavior.should_activate(ActivationTrigger::Enter));
}

#[test]
fn platform_activation_should_activate_matches_resolved() {
    let behavior = ActivationBehavior::Platform;
    let resolved = behavior.resolve();

    for trigger in [
        ActivationTrigger::Enter,
        ActivationTrigger::Space,
        ActivationTrigger::DoubleClick,
        ActivationTrigger::CommandO,
        ActivationTrigger::CommandDown,
    ] {
        assert_eq!(
            behavior.should_activate(trigger),
            resolved.should_activate(trigger),
            "trigger {:?} mismatch",
            trigger
        );
    }
}

#[test]
fn disabled_rename_never_matches() {
    let behavior = RenameBehavior::Disabled;
    assert!(!behavior.should_rename(Named::F2));
    assert!(!behavior.should_rename(Named::Enter));
    assert!(!behavior.should_rename(Named::Space));
    assert!(!behavior.should_rename(Named::Escape));
}

#[test]
fn f2_rename_matches_only_f2() {
    let behavior = RenameBehavior::F2;
    assert!(behavior.should_rename(Named::F2));
    assert!(!behavior.should_rename(Named::Enter));
    assert!(!behavior.should_rename(Named::Space));
}

#[test]
fn return_rename_matches_only_enter() {
    let behavior = RenameBehavior::Return;
    assert!(behavior.should_rename(Named::Enter));
    assert!(!behavior.should_rename(Named::F2));
    assert!(!behavior.should_rename(Named::Space));
}

#[test]
fn f2_or_return_rename_matches_both() {
    let behavior = RenameBehavior::F2OrReturn;
    assert!(behavior.should_rename(Named::F2));
    assert!(behavior.should_rename(Named::Enter));
    assert!(!behavior.should_rename(Named::Space));
    assert!(!behavior.should_rename(Named::Escape));
}

#[test]
fn platform_rename_should_rename_matches_resolved() {
    let behavior = RenameBehavior::Platform;
    let resolved = behavior.resolve();

    for key in [Named::F2, Named::Enter, Named::Space, Named::Escape] {
        assert_eq!(
            behavior.should_rename(key),
            resolved.should_rename(key),
            "key {:?} mismatch",
            key
        );
    }
}

#[test]
fn trigger_from_key_event_enter() {
    let behavior = ActivationBehavior::Enter;
    let event = key_pressed(Key::Named(Named::Enter));
    assert_eq!(
        behavior.trigger_from_key_event(&event, Modifiers::NONE),
        Some(ActivationTrigger::Enter)
    );
}

#[test]
fn trigger_from_key_event_space() {
    let behavior = ActivationBehavior::Space;
    let event = key_pressed(Key::Named(Named::Space));
    assert_eq!(
        behavior.trigger_from_key_event(&event, Modifiers::NONE),
        Some(ActivationTrigger::Space)
    );
}

#[test]
fn trigger_from_key_event_command_o() {
    let behavior = ActivationBehavior::CommandOpenAndDoubleClick;
    let event = key_pressed_with_modifiers(Key::Character("o".into()), Modifiers::COMMAND);
    assert_eq!(
        behavior.trigger_from_key_event(&event, Modifiers::COMMAND),
        Some(ActivationTrigger::CommandO)
    );
}

#[test]
fn trigger_from_key_event_command_down() {
    let behavior = ActivationBehavior::CommandOpenAndDoubleClick;
    let event = key_pressed_with_modifiers(Key::Named(Named::ArrowDown), Modifiers::COMMAND);
    assert_eq!(
        behavior.trigger_from_key_event(&event, Modifiers::COMMAND),
        Some(ActivationTrigger::CommandDown)
    );
}

#[test]
fn trigger_from_key_event_returns_none_for_unmatched() {
    let behavior = ActivationBehavior::Enter;
    let event = key_pressed(Key::Named(Named::Space));
    assert_eq!(
        behavior.trigger_from_key_event(&event, Modifiers::NONE),
        None
    );
}

#[test]
fn trigger_from_key_event_ignores_repeated() {
    let behavior = ActivationBehavior::Enter;
    let event = Event::KeyPressed {
        key: Key::Named(Named::Enter),
        modified_key: Key::Named(Named::Enter),
        physical_key: Physical::Code(Code::Enter),
        location: Location::Standard,
        modifiers: Modifiers::NONE,
        text: None,
        repeat: true,
    };
    assert_eq!(
        behavior.trigger_from_key_event(&event, Modifiers::NONE),
        None
    );
}

#[test]
fn is_rename_key_event_f2() {
    let behavior = RenameBehavior::F2;
    let event = key_pressed(Key::Named(Named::F2));
    assert!(behavior.is_rename_key_event(&event));
}

#[test]
fn is_rename_key_event_return() {
    let behavior = RenameBehavior::Return;
    let event = key_pressed(Key::Named(Named::Enter));
    assert!(behavior.is_rename_key_event(&event));
}

#[test]
fn is_rename_key_event_returns_false_for_unmatched() {
    let behavior = RenameBehavior::F2;
    let event = key_pressed(Key::Named(Named::Enter));
    assert!(!behavior.is_rename_key_event(&event));
}

#[test]
fn is_rename_key_event_ignores_repeated() {
    let behavior = RenameBehavior::F2;
    let event = Event::KeyPressed {
        key: Key::Named(Named::F2),
        modified_key: Key::Named(Named::F2),
        physical_key: Physical::Code(Code::F2),
        location: Location::Standard,
        modifiers: Modifiers::NONE,
        text: None,
        repeat: true,
    };
    assert!(!behavior.is_rename_key_event(&event));
}

#[test]
fn step_adjustment_horizontal_keys() {
    let step = StepAdjustment::new(0.01, 0.1);

    assert_eq!(
        step.delta(
            &Key::Named(Named::ArrowRight),
            Modifiers::NONE,
            Orientation::Horizontal
        ),
        Some(0.01)
    );
    assert_eq!(
        step.delta(
            &Key::Named(Named::ArrowLeft),
            Modifiers::NONE,
            Orientation::Horizontal
        ),
        Some(-0.01)
    );
    assert_eq!(
        step.delta(
            &Key::Named(Named::ArrowUp),
            Modifiers::NONE,
            Orientation::Horizontal
        ),
        None
    );
    assert_eq!(
        step.delta(
            &Key::Named(Named::ArrowDown),
            Modifiers::NONE,
            Orientation::Horizontal
        ),
        None
    );
}

#[test]
fn step_adjustment_vertical_keys() {
    let step = StepAdjustment::new(0.01, 0.1);

    assert_eq!(
        step.delta(
            &Key::Named(Named::ArrowDown),
            Modifiers::NONE,
            Orientation::Vertical
        ),
        Some(0.01)
    );
    assert_eq!(
        step.delta(
            &Key::Named(Named::ArrowUp),
            Modifiers::NONE,
            Orientation::Vertical
        ),
        Some(-0.01)
    );
    assert_eq!(
        step.delta(
            &Key::Named(Named::ArrowLeft),
            Modifiers::NONE,
            Orientation::Vertical
        ),
        None
    );
    assert_eq!(
        step.delta(
            &Key::Named(Named::ArrowRight),
            Modifiers::NONE,
            Orientation::Vertical
        ),
        None
    );
}

#[test]
fn step_adjustment_shift_increases_step() {
    let step = StepAdjustment::new(0.01, 0.1);

    assert_eq!(
        step.delta(
            &Key::Named(Named::ArrowRight),
            Modifiers::SHIFT,
            Orientation::Horizontal
        ),
        Some(0.1)
    );
}

#[test]
fn step_adjustment_command_doubles_large_step() {
    let step = StepAdjustment::new(0.01, 0.1);

    assert_eq!(
        step.delta(
            &Key::Named(Named::ArrowRight),
            Modifiers::COMMAND,
            Orientation::Horizontal
        ),
        Some(0.2)
    );
}

#[test]
fn step_adjustment_command_uses_explicit_modifier_step() {
    let step = StepAdjustment::new(0.01, 0.1).with_modifier_step(0.5);

    assert_eq!(
        step.delta(
            &Key::Named(Named::ArrowRight),
            Modifiers::COMMAND,
            Orientation::Horizontal
        ),
        Some(0.5)
    );
}
