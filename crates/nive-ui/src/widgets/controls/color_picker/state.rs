use iced::Color;

use super::{
    alpha_percent::AlphaPercentDraft,
    external_sync::{ExternalColorSync, ExternalSync},
    hex_color::RgbHexDraft,
    hsva_color::HsvaColor,
};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ColorPickerState {
    color: HsvaColor,
    hex_input: RgbHexDraft,
    alpha_input: AlphaPercentDraft,
    external_sync: ExternalColorSync,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ColorPickerSnapshot {
    pub(super) color: Color,
    pub(super) hsva: HsvaColor,
    pub(super) hex_input: String,
    pub(super) alpha_input: String,
}

impl ColorPickerState {
    pub(super) fn new(value: Color) -> Self {
        let mut state = Self {
            color: HsvaColor::from_color(value),
            hex_input: RgbHexDraft::default(),
            alpha_input: AlphaPercentDraft::default(),
            external_sync: ExternalColorSync::new(value),
        };

        state.sync_drafts();
        state
    }

    pub(super) fn snapshot(&self) -> ColorPickerSnapshot {
        ColorPickerSnapshot {
            color: self.color(),
            hsva: self.color,
            hex_input: self.hex_input.as_str().to_owned(),
            alpha_input: self.alpha_input.as_str().to_owned(),
        }
    }

    pub(super) fn sync_external(&mut self, value: Color) {
        if let ExternalSync::Changed(value) = self.external_sync.sync(value) {
            self.color = HsvaColor::from_color_preserving_hue(value, self.color.hue());
            self.sync_drafts();
        }
    }

    pub(super) fn color(&self) -> Color {
        self.color.to_color()
    }

    pub(super) fn set_saturation_value(&mut self, saturation: f32, value: f32) -> Color {
        self.color = self.color.with_saturation_value(saturation, value);
        self.sync_drafts();
        self.accept_local_change()
    }

    pub(super) fn set_hue(&mut self, hue: f32) -> Color {
        self.color = self.color.with_hue(hue);
        self.sync_drafts();
        self.accept_local_change()
    }

    pub(super) fn set_alpha(&mut self, alpha: f32) -> Color {
        self.color = self.color.with_alpha(alpha);
        self.sync_drafts();
        self.accept_local_change()
    }

    pub(super) fn input_hex(&mut self, value: String) -> Option<Color> {
        let hex = self.hex_input.input(value)?;
        let color = hex.to_color();
        self.color = HsvaColor::from_color_preserving_hue(
            Color::from_rgba(color.r, color.g, color.b, self.color.alpha()),
            self.color.hue(),
        );
        self.alpha_input.sync_alpha(self.color.alpha());

        Some(self.accept_local_change())
    }

    pub(super) fn input_alpha(&mut self, value: String) -> Option<Color> {
        let alpha = self.alpha_input.input(value)?;
        self.color = self.color.with_alpha(alpha);

        Some(self.accept_local_change())
    }

    fn sync_drafts(&mut self) {
        self.hex_input.sync_color(self.color());
        self.alpha_input.sync_alpha(self.color.alpha());
    }

    fn accept_local_change(&mut self) -> Color {
        let color = self.color();
        self.external_sync.accept_local_change(color);

        color
    }
}

#[cfg(test)]
mod color_picker_state_tests {
    use super::*;

    use crate::theme;

    fn color(hex: &str) -> Color {
        theme::parse_hex_color(hex).expect("valid test color")
    }

    fn assert_color_close(actual: Color, expected: Color) {
        assert!((actual.r - expected.r).abs() < 0.01);
        assert!((actual.g - expected.g).abs() < 0.01);
        assert!((actual.b - expected.b).abs() < 0.01);
        assert!((actual.a - expected.a).abs() < 0.01);
    }

    #[test]
    fn starts_with_synced_hex_and_alpha_inputs() {
        let state = ColorPickerState::new(Color::from_rgba(0.2, 0.4, 0.6, 0.8));

        let snapshot = state.snapshot();

        assert_eq!(snapshot.hex_input, "#336699");
        assert_eq!(snapshot.alpha_input, "80");
    }

    #[test]
    fn valid_hex_input_returns_color_and_keeps_draft_text() {
        let mut state = ColorPickerState::new(Color::from_rgba(0.2, 0.4, 0.6, 0.8));

        let parsed = state.input_hex("#ABCDEF".into());

        assert_color_close(
            parsed.unwrap(),
            Color::from_rgba(171.0 / 255.0, 205.0 / 255.0, 239.0 / 255.0, 0.8),
        );
        let snapshot = state.snapshot();

        assert_eq!(snapshot.hex_input, "#ABCDEF");
        assert_eq!(snapshot.alpha_input, "80");
    }

    #[test]
    fn invalid_hex_input_returns_none_and_keeps_sanitized_draft_text() {
        let mut state = ColorPickerState::new(color("#abcdef"));

        let parsed = state.input_hex("#ff".into());

        assert_eq!(parsed, None);
        assert_eq!(state.snapshot().hex_input, "#ff");
    }

    #[test]
    fn alpha_input_changes_color_alpha_without_overwriting_hex() {
        let mut state = ColorPickerState::new(color("#abcdef"));

        let parsed = state.input_alpha("42".into()).unwrap();

        let snapshot = state.snapshot();

        assert_eq!(snapshot.hex_input, "#abcdef");
        assert_eq!(snapshot.alpha_input, "42");
        assert!((parsed.a - 0.42).abs() < 0.01);
    }

    #[test]
    fn hue_and_saturation_value_changes_sync_drafts() {
        let mut state = ColorPickerState::new(color("#abcdef"));

        state.set_hue(0.0);
        let color = state.set_saturation_value(1.0, 1.0);

        assert_eq!(theme::format_hex_color(color), "#ff0000");
        let snapshot = state.snapshot();

        assert_eq!(snapshot.hex_input, "#ff0000");
        assert_eq!(snapshot.alpha_input, "100");
    }

    #[test]
    fn local_saturation_value_change_preserves_hue_after_external_echo() {
        let mut state = ColorPickerState::new(color("#abcdef"));

        state.set_hue(275.0);
        let color = state.set_saturation_value(0.0, 0.8);
        state.sync_external(color);

        assert!((state.snapshot().hsva.hue() - 275.0).abs() < 0.01);
    }

    #[test]
    fn local_change_ignores_stale_external_value_until_parent_catches_up() {
        let external = color("#abcdef");
        let mut state = ColorPickerState::new(external);

        state.set_hue(275.0);
        let local = state.set_saturation_value(0.0, 0.8);
        state.sync_external(external);

        let snapshot = state.snapshot();

        assert_color_close(snapshot.color, local);
        assert!((snapshot.hsva.hue() - 275.0).abs() < 0.01);
    }

    #[test]
    fn local_change_accepts_quantized_parent_echo_without_losing_hue() {
        let mut state = ColorPickerState::new(color("#abcdef"));

        state.set_hue(275.0);
        state.set_saturation_value(0.0, 0.33);
        state.sync_external(Color::from_rgb8(84, 84, 84));

        assert!((state.snapshot().hsva.hue() - 275.0).abs() < 0.01);
    }

    #[test]
    fn stale_intermediate_parent_echo_does_not_rewind_current_drag_value() {
        let mut state = ColorPickerState::new(color("#abcdef"));

        state.set_hue(275.0);
        let stale = state.set_saturation_value(0.25, 0.7);
        let current = state.set_saturation_value(0.75, 0.7);
        state.sync_external(stale);

        let snapshot = state.snapshot();

        assert_color_close(snapshot.color, current);
        assert!((snapshot.hsva.hue() - 275.0).abs() < 0.01);
        assert!((snapshot.hsva.saturation() - 0.75).abs() < 0.01);
    }

    #[test]
    fn local_bottom_value_preserves_hue_after_parent_echo() {
        let mut state = ColorPickerState::new(color("#abcdef"));

        state.set_hue(275.0);
        let color = state.set_saturation_value(1.0, 0.0);
        state.sync_external(color);

        assert!((state.snapshot().hsva.hue() - 275.0).abs() < 0.01);
    }

    #[test]
    fn external_achromatic_value_preserves_current_hue() {
        let mut state = ColorPickerState::new(color("#abcdef"));
        let local = state.set_hue(275.0);

        state.sync_external(local);
        state.sync_external(Color::from_rgb8(84, 84, 84));

        let snapshot = state.snapshot();

        assert!((snapshot.hsva.hue() - 275.0).abs() < 0.01);
        assert_eq!(snapshot.hsva.saturation(), 0.0);
    }

    #[test]
    fn external_value_syncs_only_when_value_changes() {
        let mut state = ColorPickerState::new(color("#abcdef"));

        state.input_hex("#ff".into());
        state.sync_external(color("#abcdef"));

        assert_eq!(state.snapshot().hex_input, "#ff");

        state.sync_external(color("#123456"));

        assert_eq!(state.snapshot().hex_input, "#123456");
    }
}
