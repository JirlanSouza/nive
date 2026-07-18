#[test]
fn nive_ui_has_no_legacy_outer_focusable_implementation() {
    let sources = [
        include_str!("../advanced/pressable.rs"),
        include_str!("../widgets/controls/single_choice.rs"),
        include_str!("../widgets/controls/input/adapter.rs"),
        include_str!("../widgets/controls/color_input/state.rs"),
        include_str!("../widgets/controls/color_picker/controls/control_state.rs"),
        include_str!("../widgets/controls/radio_group.rs"),
        include_str!("../widgets/controls/segmented_control/typed.rs"),
        include_str!("../widgets/containers/split_pane/state.rs"),
        include_str!("../widgets/navigation/tabs.rs"),
        include_str!("../widgets/display/tree/focus.rs"),
    ];
    let legacy_owners = [
        "Focusable for PressableState",
        "Focusable for SingleChoiceState",
        "Focusable for ColorInputState",
        "Focusable for ControlState",
        "Focusable for RadioGroupState",
        "Focusable for SegmentedState",
        "Focusable for SplitPaneState",
        "Focusable for TabBarState",
        "Focusable for TreeFocusState",
    ];

    for owner in legacy_owners {
        assert!(
            sources.iter().all(|source| !source.contains(owner)),
            "runtime enablement is blocked by legacy {owner}"
        );
    }
}
