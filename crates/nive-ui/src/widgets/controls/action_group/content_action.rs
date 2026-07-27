use std::borrow::Cow;

use iced::{
    border::Radius,
    widget::{button, container, row, text, Space},
    Alignment, Background, Border, Color, Length, Padding, Shadow,
};

use crate::IconRef;
use crate::{
    advanced::pressable::{FocusRingPlacement, Pressable},
    theme::{
        self, ControlRole, ControlSize, ControlState, InteractionState, TextRole, Theme, ToneRole,
        TypographyRole,
    },
    widgets::{
        controls::button::ButtonFocusRing, feedback::Spinner, overlays::tooltip, primitives::Icon,
    },
    Element,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentShape {
    Label,
    Icon,
    IconLabel,
}

/// One non-selectable action embedded in an [`ActionGroup`](super::ActionGroup).
pub struct ContentAction<'a, Message> {
    shape: ContentShape,
    label: Cow<'a, str>,
    icon: Option<IconRef>,
    destructive: bool,
    disabled: bool,
    loading: bool,
    reserve_loading_indicator: bool,
    on_press: Option<Message>,
    tooltip: Option<Cow<'a, str>>,
}

impl<'a, Message: Clone + 'a> ContentAction<'a, Message> {
    /// Creates a visible 14 px text action.
    pub fn label(label: impl Into<Cow<'a, str>>) -> Self {
        Self::new(ContentShape::Label, label.into(), None)
    }

    /// Creates an icon-only action with required meaningful label metadata.
    pub fn icon(icon: impl Into<IconRef>, label: impl Into<Cow<'a, str>>) -> Self {
        let label = label.into();
        let tooltip = Some(label.clone());
        Self {
            tooltip,
            ..Self::new(ContentShape::Icon, label, Some(icon.into()))
        }
    }

    /// Creates a visible icon plus 14 px label action.
    pub fn icon_label(icon: impl Into<IconRef>, label: impl Into<Cow<'a, str>>) -> Self {
        Self::new(ContentShape::IconLabel, label.into(), Some(icon.into()))
    }

    fn new(shape: ContentShape, label: Cow<'a, str>, icon: Option<IconRef>) -> Self {
        Self {
            shape,
            label,
            icon,
            destructive: false,
            disabled: false,
            loading: false,
            reserve_loading_indicator: false,
            on_press: None,
            tooltip: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Configures busy state and permanently reserves indicator geometry.
    ///
    /// A loading action is inert and non-focusable but retains enabled-idle
    /// presentation unless explicit disabled styling also applies.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self.reserve_loading_indicator = true;
        self
    }

    /// Marks this action as destructive without persistent idle danger emphasis.
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Sets an optional activation callback.
    ///
    /// `None` removes capability without applying disabled presentation.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    pub(super) fn into_element(self, metrics: ContentActionMetrics) -> Element<'a, Message> {
        let capable = self.on_press.is_some();
        let activation = if self.disabled || self.loading {
            None
        } else {
            self.on_press.clone()
        };
        let typography = theme::typography(TypographyRole::Body);
        let mut content = row![].spacing(metrics.gap).align_y(Alignment::Center);

        if self.reserve_loading_indicator {
            let indicator: Element<'a, Message> = if self.loading {
                container(Spinner::new().size(metrics.size))
                    .center(Length::Fixed(metrics.icon_size))
                    .into()
            } else {
                Space::new()
                    .width(Length::Fixed(metrics.icon_size))
                    .height(Length::Fixed(metrics.icon_size))
                    .into()
            };
            content = content.push(indicator);
        }
        if let Some(icon) = self.icon {
            content = content.push(Icon::reference(icon).custom_size(metrics.icon_size));
        }
        if self.shape != ContentShape::Icon {
            content = content.push(
                text(self.label.clone())
                    .font(typography.font)
                    .size(typography.size)
                    .line_height(typography.line_height)
                    .wrapping(text::Wrapping::None),
            );
        }

        let icon_only = self.shape == ContentShape::Icon && !self.reserve_loading_indicator;
        let content = if icon_only {
            container(content).center(Length::Fixed(metrics.height))
        } else {
            container(content).center_y(Length::Fixed(metrics.height))
        };
        let radius = Radius::new(metrics.radius);
        let mut action = button::Button::new(content)
            .height(Length::Fixed(metrics.height))
            .padding(if icon_only {
                Padding::ZERO
            } else {
                Padding::ZERO.horizontal(metrics.padding_h)
            })
            .style(content_action_style(
                self.disabled,
                self.loading,
                capable,
                self.destructive,
                radius,
            ))
            .clip(true);
        if icon_only {
            action = action.width(Length::Fixed(metrics.height));
        }
        let action = action.on_press_maybe(activation.clone());
        let ring = if self.destructive {
            ButtonFocusRing::Danger
        } else {
            ButtonFocusRing::Default
        };
        let action: Element<'a, Message> = match activation {
            Some(message) => Pressable::new(action, message, None, radius, ring)
                .focus_placement(FocusRingPlacement::Inset)
                .into(),
            None => action.into(),
        };

        match self.tooltip {
            Some(label) => tooltip::Tooltip::new(action, label).into(),
            None => action,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ContentActionMetrics {
    pub(super) size: ControlSize,
    pub(super) height: f32,
    pub(super) padding_h: f32,
    pub(super) icon_size: f32,
    pub(super) gap: f32,
    pub(super) radius: f32,
    pub(super) item_gap: f32,
    pub(super) row_gap: f32,
    pub(super) separator_height: f32,
    pub(super) separator_width: f32,
    pub(super) text_size: f32,
}

impl ContentActionMetrics {
    pub(super) fn resolve(theme: Theme, size: ControlSize) -> Self {
        let control = theme.control_metrics(size);
        let spacing = theme.spacing();
        Self {
            size,
            height: control.height,
            padding_h: match size {
                ControlSize::Xs => spacing.sm,
                ControlSize::Sm => spacing.md,
                ControlSize::Md => spacing.md + spacing.xxs,
                ControlSize::Lg => spacing.lg,
            },
            icon_size: control.icon_size,
            gap: control.gap,
            radius: control.radius,
            item_gap: spacing.xxs,
            row_gap: theme.gap(crate::theme::GapRole::Related),
            separator_height: control.height - spacing.md,
            separator_width: 1.0,
            text_size: theme.typography(TypographyRole::Body).size,
        }
    }
}

fn content_action_style(
    explicitly_disabled: bool,
    loading: bool,
    capable: bool,
    destructive: bool,
    radius: Radius,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let status = if (loading || !capable) && !explicitly_disabled {
            button::Status::Active
        } else {
            status
        };
        let interacting = capable
            && !loading
            && !explicitly_disabled
            && matches!(status, button::Status::Hovered | button::Status::Pressed);

        let mut state = ControlState::new().interaction(match status {
            button::Status::Hovered => InteractionState::HOVERED,
            button::Status::Pressed => InteractionState::PRESSED,
            button::Status::Active | button::Status::Disabled => InteractionState::NONE,
        });
        if explicitly_disabled {
            state = state.disabled();
        }
        // Embedded resolves the untouched and disabled fills to transparent, so
        // only the destructive tone needs its own projection here.
        let control = theme.control(ControlRole::Embedded, state);
        let danger = theme.tone(ToneRole::Danger);

        let background = if destructive && interacting {
            if status == button::Status::Pressed {
                danger.container.scale_alpha(1.18)
            } else {
                danger.container
            }
        } else {
            control.background
        };
        let foreground = if explicitly_disabled {
            theme.text(TextRole::Disabled).color
        } else if destructive && interacting {
            danger.color
        } else if interacting {
            theme.text(TextRole::Primary).color
        } else {
            theme.text(TextRole::Secondary).color
        };

        button::Style {
            background: Some(Background::Color(background)),
            text_color: foreground,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius,
            },
            shadow: Shadow::default(),
            snap: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ThemeDensity, ThemeMode};
    #[allow(unused_imports)]
    use crate::IconRole;

    #[test]
    fn icon_only_action_keeps_required_label_as_default_tooltip() {
        let action = ContentAction::<()>::icon(IconRole::EditDelete, "Delete");

        assert_eq!(action.label, "Delete");
        assert_eq!(action.tooltip.as_deref(), Some("Delete"));
        assert_eq!(action.shape, ContentShape::Icon);
    }

    #[test]
    fn metrics_follow_size_and_density_but_text_stays_14px() {
        for density in ThemeDensity::ALL {
            let theme = Theme::builder("Content action metrics", ThemeMode::Dark)
                .density(density)
                .build();
            for size in [
                ControlSize::Xs,
                ControlSize::Sm,
                ControlSize::Md,
                ControlSize::Lg,
            ] {
                let metrics = ContentActionMetrics::resolve(theme, size);
                assert_eq!(metrics.height, theme.control_metrics(size).height);
                assert_eq!(metrics.icon_size, theme.control_metrics(size).icon_size);
                assert_eq!(metrics.text_size, 14.0);
            }
        }
    }

    #[test]
    fn loading_and_absent_capability_keep_enabled_idle_style() {
        let theme = Theme::Dark;
        let radius = Radius::new(4.0);
        let idle =
            content_action_style(false, false, true, false, radius)(&theme, button::Status::Active);
        let loading = content_action_style(false, true, true, false, radius)(
            &theme,
            button::Status::Disabled,
        );
        let absent = content_action_style(false, false, false, false, radius)(
            &theme,
            button::Status::Disabled,
        );

        assert_eq!(idle, loading);
        assert_eq!(idle, absent);
    }

    #[test]
    fn state_projection_keeps_geometry_neutral_and_disabled_authoritative() {
        let theme = Theme::Light;
        let radius = Radius::new(6.0);
        let idle =
            content_action_style(false, false, true, false, radius)(&theme, button::Status::Active);
        let hovered = content_action_style(false, false, true, false, radius)(
            &theme,
            button::Status::Hovered,
        );
        let pressed = content_action_style(false, false, true, false, radius)(
            &theme,
            button::Status::Pressed,
        );
        let disabled = content_action_style(true, false, true, false, radius)(
            &theme,
            button::Status::Disabled,
        );
        let destructive =
            content_action_style(false, false, true, true, radius)(&theme, button::Status::Hovered);

        assert_eq!(idle.border, hovered.border);
        assert_eq!(hovered.border, pressed.border);
        assert_eq!(idle.shadow, hovered.shadow);
        assert_ne!(idle.background, hovered.background);
        assert_ne!(hovered.background, pressed.background);
        assert_eq!(disabled.text_color, theme.text(TextRole::Disabled).color);
        assert_eq!(destructive.text_color, theme.tone(ToneRole::Danger).color);
    }
}
