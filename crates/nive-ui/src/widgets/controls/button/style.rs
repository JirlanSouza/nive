use iced::{
    border::Radius,
    widget::button::{self, Status},
    Background, Border, Shadow,
};

use crate::theme::{
    self, BorderRole, BorderSpec, ButtonClass, ControlRole, ControlSize, ControlState, TextRole,
    ToneRole,
};

use crate::advanced::control_style::{border_with_radius, transparent_border_with_radius};
use crate::widgets::controls::button::SelectionChrome;

pub fn button_control_state(status: button::Status) -> ControlState {
    match status {
        button::Status::Active => ControlState::ENABLED,
        button::Status::Hovered => ControlState::HOVERED,
        button::Status::Pressed => ControlState::PRESSED,
        button::Status::Disabled => ControlState::DISABLED,
    }
}

/// Visual metrics for a button at a resolved control size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonMetrics {
    pub font_size: f32,
    pub height: f32,
    pub padding_h: f32,
    pub radius: f32,
    pub icon_size: f32,
    pub gap: f32,
}

/// Action semantics for a button.
///
/// `Suggested` is the desktop default-action intent. Destructive actions use
/// `Destructive`; status tone remains `ToneRole::Danger` on non-action widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonIntent {
    /// Ordinary action.
    Neutral,
    /// Suggested/default action.
    Suggested,
    /// Action that may delete or otherwise damage data.
    Destructive,
}

/// Button appearance variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Filled button chrome.
    Solid,
    /// Low-emphasis filled/subtle chrome.
    Subtle,
    /// Outlined button chrome.
    Outline,
    /// Minimal transparent chrome.
    Ghost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonFocusRing {
    Default,
    OnPrimary,
    OnDanger,
    Danger,
}

pub fn style(
    intent: ButtonIntent,
    variant: ButtonVariant,
    radius: Radius,
) -> impl Fn(&crate::theme::Theme, Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: Status| {
        let class = button_class(intent, variant);
        let mut style = <crate::theme::Theme as button::Catalog>::style(theme, &class, status);
        style.border.radius = radius;
        style
    }
}

pub(crate) fn embedded_style(
    selected: bool,
    selection: SelectionChrome,
    radius: Radius,
) -> impl Fn(&crate::theme::Theme, Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: Status| {
        // Embedded/flat chrome keeps its own subtler unselected hover/pressed
        // treatment (category-owned visual identity); selected state routes
        // through the same shared `ControlRole::Selectable` resolver
        // `toolbar_style` uses, so a flat `SegmentedControl` doesn't lose its
        // selected item's active state the way it did when this branch was
        // missing entirely.
        if selected {
            let theme = *theme;
            let state = button_control_state(status).selected();
            let control = theme.control(ControlRole::Selectable, state);

            return button::Style {
                background: Some(Background::Color(control.background)),
                text_color: control.foreground,
                border: match selection {
                    SelectionChrome::Outlined => border_with_radius(control.border, radius),
                    SelectionChrome::Flat => transparent_border_with_radius(radius),
                },
                shadow: Shadow::default(),
                ..button::Style::default()
            };
        }

        let class = ButtonClass::Embedded;
        let mut style = <crate::theme::Theme as button::Catalog>::style(theme, &class, status);
        style.border.radius = radius;
        style
    }
}

pub(crate) fn toolbar_style(
    selected: bool,
    destructive: bool,
    radius: Radius,
) -> impl Fn(&crate::theme::Theme, Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: Status| {
        let theme = *theme;
        let interacting = matches!(status, Status::Hovered | Status::Pressed);

        if destructive && interacting {
            let danger = theme.tone(ToneRole::Danger);
            return button::Style {
                background: Some(Background::Color(match status {
                    Status::Pressed => danger.container.scale_alpha(1.18),
                    _ => danger.container,
                })),
                text_color: danger.color,
                border: transparent_border_with_radius(radius),
                shadow: Shadow::default(),
                ..button::Style::default()
            };
        }

        let mut state = button_control_state(status);
        if selected {
            state = state.selected();
        }
        // Embedded: toolbar chrome paints on the bar hosting it, so untouched
        // and disabled resolve transparent without a local guard. Selection is
        // resolved before the role and is therefore unchanged.
        let control = theme.control(ControlRole::Embedded, state);
        let text_color = match status {
            Status::Disabled => control.foreground,
            Status::Hovered | Status::Pressed => theme.text(TextRole::Primary).color,
            Status::Active if selected => control.foreground,
            Status::Active => theme.text(TextRole::Secondary).color,
        };

        button::Style {
            background: Some(Background::Color(control.background)),
            text_color,
            border: transparent_border_with_radius(radius),
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    }
}

pub(crate) fn button_class(intent: ButtonIntent, variant: ButtonVariant) -> ButtonClass<'static> {
    ButtonClass::Standard { intent, variant }
}

pub fn focus_ring(theme: &crate::theme::Theme, ring: ButtonFocusRing, radius: Radius) -> Border {
    let theme = *theme;
    let focus = theme.border(BorderRole::Focus);
    let color = match ring {
        ButtonFocusRing::Default => focus.color,
        ButtonFocusRing::OnPrimary => theme.tone(ToneRole::Accent).on_color,
        ButtonFocusRing::OnDanger => theme.tone(ToneRole::Danger).on_color,
        ButtonFocusRing::Danger => theme.tone(ToneRole::Danger).color,
    };

    border_with_radius(BorderSpec::new(color, focus.width), radius)
}

pub fn metrics(size: ControlSize) -> ButtonMetrics {
    let control = theme::form_control_metrics(size);

    ButtonMetrics {
        font_size: control.strong_text_style.size,
        height: control.height,
        padding_h: control.padding.left,
        radius: control.radius,
        icon_size: control.icon_size,
        gap: control.gap,
    }
}

pub fn icon_side(size: ControlSize) -> f32 {
    theme::form_control_metrics(size).height
}

#[cfg(test)]
mod button_tests {
    use super::*;
    use iced::{Background, Color};

    use crate::theme::{Theme, ToneRole};

    #[test]
    fn metrics_follow_control_size() {
        assert_eq!(
            metrics(ControlSize::Sm).height,
            theme::form_control_metrics(ControlSize::Sm).height
        );
        assert_eq!(
            metrics(ControlSize::Sm).font_size,
            theme::typography(crate::theme::TypographyRole::ControlStrong).size
        );
    }

    #[test]
    fn suggested_solid_uses_catalog_class() {
        let theme = Theme::Dark;
        let radius = Radius::new(4.0);
        let style =
            style(ButtonIntent::Suggested, ButtonVariant::Solid, radius)(&theme, Status::Active);
        let expected = <Theme as button::Catalog>::style(
            &theme,
            &ButtonClass::Standard {
                intent: ButtonIntent::Suggested,
                variant: ButtonVariant::Solid,
            },
            Status::Active,
        );

        assert_eq!(background_color(&style), theme.tone(ToneRole::Accent).color);
        assert_eq!(style.text_color, expected.text_color);
        assert_eq!(style.border.color, expected.border.color);
        assert_eq!(style.border.width, expected.border.width);
        assert_eq!(style.border.radius, radius);
    }

    #[test]
    fn destructive_uses_catalog_class() {
        let theme = Theme::Dark;
        let radius = Radius::new(6.0);
        let style =
            style(ButtonIntent::Destructive, ButtonVariant::Solid, radius)(&theme, Status::Active);
        let expected = <Theme as button::Catalog>::style(
            &theme,
            &ButtonClass::Standard {
                intent: ButtonIntent::Destructive,
                variant: ButtonVariant::Solid,
            },
            Status::Active,
        );

        assert_eq!(background_color(&style), background_color(&expected));
        assert_eq!(style.text_color, expected.text_color);
        assert_eq!(style.border.color, expected.border.color);
        assert_eq!(style.border.width, expected.border.width);
        assert_eq!(style.border.radius, radius);
    }

    #[test]
    fn on_danger_focus_ring_contrasts_with_the_solid_danger_fill() {
        for theme in [Theme::Light, Theme::Dark] {
            let ring = focus_ring(&theme, ButtonFocusRing::OnDanger, Radius::new(6.0));
            let danger = theme.tone(ToneRole::Danger);

            assert_eq!(ring.color, danger.on_color);
            assert_ne!(ring.color, danger.color);
        }
    }

    #[test]
    fn built_in_solid_action_foregrounds_meet_text_contrast() {
        for theme in [Theme::Light, Theme::Dark] {
            for intent in [ButtonIntent::Suggested, ButtonIntent::Destructive] {
                let style =
                    style(intent, ButtonVariant::Solid, Radius::new(4.0))(&theme, Status::Active);
                let background = background_color(&style);
                let contrast = crate::theme::color::contrast_ratio(style.text_color, background);

                assert!(contrast >= 4.5, "{theme:?} {intent:?}: {contrast}");
            }
        }
    }

    #[test]
    fn embedded_selected_keeps_the_active_selected_state() {
        let theme = Theme::Dark;
        let selected = embedded_style(true, SelectionChrome::Outlined, Radius::new(6.0))(
            &theme,
            Status::Active,
        );
        let unselected = embedded_style(false, SelectionChrome::Outlined, Radius::new(6.0))(
            &theme,
            Status::Active,
        );
        let resolved = theme.control(ControlRole::Selectable, ControlState::SELECTED);

        assert_eq!(background_color(&selected), resolved.background);
        assert_eq!(selected.text_color, resolved.foreground);
        assert_ne!(background_color(&selected), background_color(&unselected));
    }

    #[test]
    fn embedded_selected_hover_and_pressed_use_the_shared_resolver() {
        let theme = Theme::Dark;
        let hovered = embedded_style(true, SelectionChrome::Outlined, Radius::new(6.0))(
            &theme,
            Status::Hovered,
        );
        let pressed = embedded_style(true, SelectionChrome::Outlined, Radius::new(6.0))(
            &theme,
            Status::Pressed,
        );
        let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);

        assert_eq!(
            background_color(&hovered),
            selected.background.scale_alpha(1.20)
        );
        assert_eq!(
            background_color(&pressed),
            selected.background.scale_alpha(0.88)
        );
    }

    #[test]
    fn flat_selection_keeps_the_fill_and_drops_the_outline() {
        let theme = Theme::Dark;
        let outlined = embedded_style(true, SelectionChrome::Outlined, Radius::new(6.0))(
            &theme,
            Status::Active,
        );
        let flat =
            embedded_style(true, SelectionChrome::Flat, Radius::new(6.0))(&theme, Status::Active);

        assert_eq!(background_color(&flat), background_color(&outlined));
        assert_eq!(flat.border.color, Color::TRANSPARENT);
        assert_ne!(outlined.border.color, Color::TRANSPARENT);
        assert_eq!(flat.border.radius, outlined.border.radius);
    }

    #[test]
    fn toolbar_states_are_flat_and_semantic() {
        let theme = Theme::Dark;
        let radius = Radius::new(6.0);
        let idle = toolbar_style(false, false, radius)(&theme, Status::Active);
        let hovered = toolbar_style(false, false, radius)(&theme, Status::Hovered);
        let selected = toolbar_style(true, false, radius)(&theme, Status::Active);
        let destructive_idle = toolbar_style(false, true, radius)(&theme, Status::Active);
        let destructive_hover = toolbar_style(false, true, radius)(&theme, Status::Hovered);

        assert_eq!(background_color(&idle), Color::TRANSPARENT);
        assert_eq!(idle.text_color, theme.text(TextRole::Secondary).color);
        assert_eq!(idle.border.width, 0.0);
        assert_ne!(background_color(&hovered), Color::TRANSPARENT);
        assert_eq!(hovered.text_color, theme.text(TextRole::Primary).color);
        assert_ne!(background_color(&selected), Color::TRANSPARENT);
        assert_eq!(selected.border.width, 0.0);
        assert_eq!(
            destructive_idle.text_color,
            theme.text(TextRole::Secondary).color
        );
        assert_eq!(
            destructive_hover.text_color,
            theme.tone(ToneRole::Danger).color
        );
    }

    fn background_color(style: &button::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
