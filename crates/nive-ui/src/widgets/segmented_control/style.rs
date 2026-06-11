use iced::{
    border::Radius,
    widget::{button, container},
    Background, Color, Shadow,
};

use super::super::button::button_control_state;
use super::super::control_style::border_with_radius;

use crate::theme::{
    self, control_metrics, BorderRole, BorderSpec, ControlRole, ControlSize, ControlState, TextRole,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SegmentPosition {
    Single,
    First,
    Middle,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentedControlMetrics {
    pub height: f32,
    pub radius: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub gap: f32,
    pub padding_h: f32,
    pub outer_padding: f32,
}

pub fn metrics(size: ControlSize) -> SegmentedControlMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();

    SegmentedControlMetrics {
        height: control.height,
        radius: control.radius,
        font_size: control.font_size,
        icon_size: control.icon_size,
        gap: control.gap,
        padding_h: match size {
            ControlSize::Xs => spacing.sm,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.md + spacing.xxs,
            ControlSize::Lg => spacing.lg,
        },
        outer_padding: spacing.xxs,
    }
}

pub fn container_style(radius: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let theme = *theme;
        let control = theme.control(ControlRole::Standard, ControlState::ENABLED);
        let border = theme.border(BorderRole::Default);

        container::Style {
            text_color: Some(control.foreground),
            background: Some(Background::Color(control.background)),
            border: border_with_radius(border, radius),
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

pub(crate) fn item_style(
    selected: bool,
    position: SegmentPosition,
    radius: f32,
) -> impl Fn(&crate::theme::Theme, button::Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: button::Status| {
        let theme = *theme;
        let control = theme.control(ControlRole::Standard, button_control_state(status));
        let selected_control = theme.control(ControlRole::Selectable, ControlState::SELECTED);
        let disabled_control = theme.control(ControlRole::Standard, ControlState::DISABLED);

        let background = match (selected, status) {
            (true, button::Status::Hovered) => selected_control.background.scale_alpha(1.20),
            (true, button::Status::Pressed) => selected_control.background.scale_alpha(0.88),
            (true, button::Status::Disabled) => disabled_control.background,
            (true, _) => selected_control.background,
            (false, button::Status::Hovered | button::Status::Pressed) => control.background,
            (false, _) => Color::TRANSPARENT,
        };
        let text_color = match (selected, status) {
            (_, button::Status::Disabled) => disabled_control.foreground,
            (true, _) => selected_control.foreground,
            (false, button::Status::Hovered | button::Status::Pressed) => {
                theme.text(TextRole::Primary).color
            }
            (false, _) => theme.text(TextRole::Secondary).color,
        };
        let selected_border = selected_control.border;
        let border = if selected {
            selected_border
        } else {
            BorderSpec::none()
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color,
            border: border_with_radius(border, item_radius(position, radius)),
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    }
}

pub(crate) fn position_for_index(index: usize, len: usize) -> SegmentPosition {
    match (index, len) {
        (_, 1) => SegmentPosition::Single,
        (0, _) => SegmentPosition::First,
        (index, len) if index + 1 == len => SegmentPosition::Last,
        _ => SegmentPosition::Middle,
    }
}

fn item_radius(position: SegmentPosition, radius: f32) -> Radius {
    match position {
        SegmentPosition::Single => Radius::new(radius),
        SegmentPosition::First => Radius::default().left(radius),
        SegmentPosition::Middle => Radius::default(),
        SegmentPosition::Last => Radius::default().right(radius),
    }
}

#[cfg(test)]
mod segmented_control_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn maps_segment_positions() {
        assert_eq!(position_for_index(0, 1), SegmentPosition::Single);
        assert_eq!(position_for_index(0, 3), SegmentPosition::First);
        assert_eq!(position_for_index(1, 3), SegmentPosition::Middle);
        assert_eq!(position_for_index(2, 3), SegmentPosition::Last);
    }

    #[test]
    fn selected_item_uses_app_selected_control_background() {
        let theme = Theme::Dark;
        let style = item_style(true, SegmentPosition::Single, 6.0)(&theme, button::Status::Active);

        assert_eq!(
            background_color(&style),
            theme
                .control(ControlRole::Selectable, ControlState::SELECTED)
                .background
        );
    }

    fn background_color(style: &button::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
