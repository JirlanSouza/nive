use iced::{Rectangle, Size};

use super::{
    BorderRole, ControlRole, ControlSize, ControlState, FieldValidation, FormControlMetrics,
    InteractionState, SpaceStep, TextRole, TextStyle, Theme, TypographyRole,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ChoiceMetrics {
    pub(crate) form: FormControlMetrics,
    pub(crate) indicator_size: f32,
    pub(crate) checkbox_radius: f32,
    pub(crate) switch_track: Size,
    pub(crate) switch_thumb_size: f32,
    pub(crate) switch_thumb_inset: f32,
    pub(crate) support_text_style: TextStyle,
    pub(crate) support_gap: f32,
    pub(crate) option_gap: f32,
    pub(crate) group_gap: f32,
    pub(crate) perimeter_width: f32,
    pub(crate) focus_stroke_width: f32,
}

impl ChoiceMetrics {
    pub(crate) fn for_theme(theme: Theme, size: ControlSize) -> Self {
        let form = theme.form_control_metrics(size);
        let (indicator_size, switch_track) = match size {
            ControlSize::Xs => (14.0, Size::new(28.0, 16.0)),
            ControlSize::Sm => (16.0, Size::new(32.0, 18.0)),
            ControlSize::Md => (18.0, Size::new(36.0, 20.0)),
            ControlSize::Lg => (20.0, Size::new(40.0, 22.0)),
        };

        Self {
            form,
            indicator_size,
            checkbox_radius: 4.0,
            switch_track,
            switch_thumb_size: switch_track.height - 4.0,
            switch_thumb_inset: 2.0,
            support_text_style: theme.typography(TypographyRole::BodySmall),
            support_gap: theme.space(SpaceStep::Xs),
            option_gap: theme.space(SpaceStep::Md),
            group_gap: theme.space(SpaceStep::Lg),
            perimeter_width: form.field_border_width,
            focus_stroke_width: form.focus_stroke_width,
        }
    }

    pub(crate) fn indicator_focus_bounds(self, indicator: Rectangle) -> Rectangle {
        inflate(indicator, self.focus_stroke_width)
    }

    pub(crate) fn track_focus_bounds(self, track: Rectangle) -> Rectangle {
        inflate(track, self.focus_stroke_width)
    }

    pub(crate) fn segment_focus_bounds(self, item: Rectangle) -> Rectangle {
        inset(item, self.form.focus_inset)
    }

    pub(crate) fn checkbox_focus_radius(self) -> f32 {
        self.checkbox_radius + self.focus_stroke_width
    }

    pub(crate) fn radio_focus_radius(self) -> f32 {
        (self.indicator_size + self.focus_stroke_width * 2.0) / 2.0
    }

    pub(crate) fn radio_dot_size(self) -> f32 {
        ((self.indicator_size / 2.0) / 2.0).floor() * 2.0
    }

    pub(crate) fn switch_focus_radius(self) -> f32 {
        (self.switch_track.height + self.focus_stroke_width * 2.0) / 2.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChoicePersistentState {
    Unselected,
    Selected,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ChoiceStateInput {
    pub(crate) persistent: ChoicePersistentState,
    pub(crate) validation: FieldValidation,
    pub(crate) callback_present: bool,
    pub(crate) disabled: bool,
    pub(crate) hovered: bool,
    pub(crate) pressed: bool,
    pub(crate) focused: bool,
}

impl ChoiceStateInput {
    #[cfg(test)]
    pub(crate) const fn enabled(persistent: ChoicePersistentState) -> Self {
        Self {
            persistent,
            validation: FieldValidation::Valid,
            callback_present: true,
            disabled: false,
            hovered: false,
            pressed: false,
            focused: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResolvedChoiceState {
    pub(crate) control: ControlState,
    pub(crate) validation: FieldValidation,
    pub(crate) mixed: bool,
    pub(crate) interactive: bool,
}

pub(crate) fn resolve_state(input: ChoiceStateInput) -> ResolvedChoiceState {
    let interactive = input.callback_present && !input.disabled;
    let interaction = if interactive {
        InteractionState {
            hovered: input.hovered,
            pressed: input.pressed,
            focused: input.focused,
            dragged: false,
        }
    } else {
        InteractionState::NONE
    };
    let selected = !matches!(input.persistent, ChoicePersistentState::Unselected);
    let mut control = ControlState::new().interaction(interaction);

    if selected {
        control = control.selected();
    }
    if input.disabled {
        control = control.disabled();
    }

    ResolvedChoiceState {
        control,
        validation: input.validation,
        mixed: matches!(input.persistent, ChoicePersistentState::Mixed),
        interactive,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ChoicePalette {
    pub(crate) background: iced::Color,
    pub(crate) foreground: iced::Color,
    pub(crate) perimeter: iced::Color,
    pub(crate) mark: iced::Color,
    pub(crate) focus: iced::Color,
}

pub(crate) fn palette(theme: Theme, state: ResolvedChoiceState) -> ChoicePalette {
    palette_with_strength(theme, state, true)
}

pub(crate) fn segment_palette(theme: Theme, state: ResolvedChoiceState) -> ChoicePalette {
    palette_with_strength(theme, state, false)
}

fn palette_with_strength(
    theme: Theme,
    state: ResolvedChoiceState,
    strong_selection: bool,
) -> ChoicePalette {
    let control = theme.control(ControlRole::Selectable, state.control);
    let accent = theme.tone(super::ToneRole::Accent);
    let perimeter = if matches!(state.validation, FieldValidation::Invalid) && state.control.enabled
    {
        theme.border(BorderRole::Danger).color
    } else if state.control.selected && state.control.enabled {
        accent.border.color
    } else {
        control.border.color
    };
    let mark = if state.control.selected && state.control.enabled {
        accent.on_color
    } else if !state.control.enabled {
        control.foreground
    } else {
        theme.text(TextRole::Secondary).color
    };
    ChoicePalette {
        background: if strong_selection && state.control.selected && state.control.enabled {
            accent.color
        } else {
            control.background
        },
        foreground: if state.control.selected && state.control.enabled {
            theme.text(TextRole::Primary).color
        } else {
            control.foreground
        },
        perimeter,
        mark,
        focus: theme.border(BorderRole::Focus).color,
    }
}

fn inflate(bounds: Rectangle, amount: f32) -> Rectangle {
    let amount = finite_nonnegative(amount);

    Rectangle {
        x: bounds.x - amount,
        y: bounds.y - amount,
        width: bounds.width + amount * 2.0,
        height: bounds.height + amount * 2.0,
    }
}

fn inset(bounds: Rectangle, amount: f32) -> Rectangle {
    let amount = finite_nonnegative(amount);

    Rectangle {
        x: bounds.x + amount,
        y: bounds.y + amount,
        width: (bounds.width - amount * 2.0).max(0.0),
        height: (bounds.height - amount * 2.0).max(0.0),
    }
}

fn finite_nonnegative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ThemeDensity, ThemeMode};

    #[test]
    fn metrics_follow_every_density_and_size() {
        let expected_heights = [
            (ThemeDensity::Compact, [20.0, 24.0, 28.0, 32.0]),
            (ThemeDensity::Standard, [24.0, 28.0, 32.0, 36.0]),
            (ThemeDensity::Comfortable, [28.0, 32.0, 36.0, 40.0]),
        ];
        let sizes = [
            ControlSize::Xs,
            ControlSize::Sm,
            ControlSize::Md,
            ControlSize::Lg,
        ];

        for (density, heights) in expected_heights {
            let theme = Theme::builder("choice metrics", ThemeMode::Light)
                .density(density)
                .build();

            for (size, expected_height) in sizes.into_iter().zip(heights) {
                let metrics = ChoiceMetrics::for_theme(theme, size);

                assert_eq!(metrics.form.height, expected_height);
                assert_eq!(metrics.form.text_style.size, 14.0);
                assert_eq!(metrics.form.strong_text_style.size, 14.0);
                assert_eq!(metrics.support_text_style.size, 12.0);
                assert_eq!(metrics.perimeter_width, 1.0);
                assert_eq!(metrics.focus_stroke_width, 2.0);
            }
        }
    }

    #[test]
    fn radio_dot_sizes_are_even_and_near_half_the_indicator() {
        let expected = [6.0, 8.0, 8.0, 10.0];

        for (size, dot) in [
            ControlSize::Xs,
            ControlSize::Sm,
            ControlSize::Md,
            ControlSize::Lg,
        ]
        .into_iter()
        .zip(expected)
        {
            let metrics = ChoiceMetrics::for_theme(Theme::Light, size);

            assert_eq!(metrics.radio_dot_size(), dot);
            assert_eq!(metrics.radio_dot_size() % 2.0, 0.0);
            assert!((metrics.radio_dot_size() - metrics.indicator_size / 2.0).abs() <= 1.0);
        }
    }

    #[test]
    fn category_geometry_is_exact_and_focus_is_layout_neutral() {
        let theme = Theme::Dark;
        let sizes = [
            (ControlSize::Xs, 14.0, Size::new(28.0, 16.0), 12.0),
            (ControlSize::Sm, 16.0, Size::new(32.0, 18.0), 14.0),
            (ControlSize::Md, 18.0, Size::new(36.0, 20.0), 16.0),
            (ControlSize::Lg, 20.0, Size::new(40.0, 22.0), 18.0),
        ];

        for (size, indicator, track, thumb) in sizes {
            let metrics = ChoiceMetrics::for_theme(theme, size);
            let bounds = Rectangle::new(iced::Point::new(4.0, 6.0), track);

            assert_eq!(metrics.indicator_size, indicator);
            assert_eq!(metrics.switch_track, track);
            assert_eq!(metrics.switch_thumb_size, thumb);
            assert_eq!(metrics.switch_thumb_inset, 2.0);
            assert_eq!(
                metrics.track_focus_bounds(bounds),
                Rectangle::new(
                    iced::Point::new(2.0, 4.0),
                    Size::new(track.width + 4.0, track.height + 4.0),
                )
            );
            assert_eq!(metrics.checkbox_focus_radius(), 6.0);
        }
    }

    #[test]
    fn state_projection_preserves_persistence_and_applies_disabled_once() {
        let mixed = resolve_state(ChoiceStateInput {
            persistent: ChoicePersistentState::Mixed,
            validation: FieldValidation::Invalid,
            callback_present: true,
            disabled: false,
            hovered: true,
            pressed: true,
            focused: true,
        });

        assert!(mixed.control.selected);
        assert!(mixed.mixed);
        assert!(mixed.interactive);
        assert!(mixed.control.interaction.hovered);
        assert!(mixed.control.interaction.pressed);
        assert!(mixed.control.interaction.focused);

        let disabled = resolve_state(ChoiceStateInput {
            disabled: true,
            ..ChoiceStateInput::enabled(ChoicePersistentState::Selected)
        });

        assert!(!disabled.control.enabled);
        assert!(disabled.control.selected);
        assert_eq!(disabled.control.interaction, InteractionState::NONE);
        assert!(!disabled.interactive);
    }

    #[test]
    fn callback_absence_is_display_only_not_disabled() {
        let state = resolve_state(ChoiceStateInput {
            callback_present: false,
            hovered: true,
            pressed: true,
            focused: true,
            ..ChoiceStateInput::enabled(ChoicePersistentState::Selected)
        });

        assert!(state.control.enabled);
        assert!(state.control.selected);
        assert_eq!(state.control.interaction, InteractionState::NONE);
        assert!(!state.interactive);
    }

    #[test]
    fn selected_text_uses_on_accent_contrast_and_keeps_interaction_fill() {
        for theme in [Theme::Light, Theme::Dark] {
            let selected = segment_palette(
                theme,
                resolve_state(ChoiceStateInput::enabled(ChoicePersistentState::Selected)),
            );
            let hovered = segment_palette(
                theme,
                resolve_state(ChoiceStateInput {
                    hovered: true,
                    ..ChoiceStateInput::enabled(ChoicePersistentState::Selected)
                }),
            );

            let track = theme
                .control(ControlRole::Standard, ControlState::ENABLED)
                .background;
            let background = iced::Color {
                r: selected.background.r * selected.background.a
                    + track.r * (1.0 - selected.background.a),
                g: selected.background.g * selected.background.a
                    + track.g * (1.0 - selected.background.a),
                b: selected.background.b * selected.background.a
                    + track.b * (1.0 - selected.background.a),
                a: 1.0,
            };

            assert_eq!(selected.foreground, theme.text(TextRole::Primary).color);
            assert!(crate::theme::color::contrast_ratio(selected.foreground, background) >= 4.5);
            assert_ne!(hovered.background, selected.background);
            assert_eq!(hovered.foreground, selected.foreground);

            let strong = palette(
                theme,
                resolve_state(ChoiceStateInput::enabled(ChoicePersistentState::Selected)),
            );
            assert_eq!(
                strong.background,
                theme.tone(super::super::ToneRole::Accent).color
            );
            assert_eq!(
                strong.mark,
                theme.tone(super::super::ToneRole::Accent).on_color
            );
        }
    }
}
