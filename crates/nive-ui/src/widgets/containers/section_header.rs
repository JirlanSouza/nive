mod style;

use std::borrow::Cow;

use iced::{
    time::Duration,
    widget::{container, row, text, Row, Space},
    Alignment, Length, Padding, Radians,
};

use crate::animation::{AnimatedVisual, Animation};
use crate::theme::{self, ControlSize, TextRole, ToneRole};
use crate::widgets::controls::button;
use crate::widgets::feedback::presentation::{resource_status_phase, ResourceStatusPhase};
use crate::widgets::feedback::ResourceStatusPresentation;
use crate::widgets::primitives::{icon, IconRole};
use crate::widgets::Badge;
use crate::Element;

use self::style as theme_section_header;

/// Compact panel or content-section heading.
///
/// Titles resolve to [`crate::theme::TypographyRole::SectionLabel`] (12 px
/// semibold) with [`TextRole::Primary`]. Compose a principal document title
/// separately with [`crate::theme::TypographyRole::Heading`].
pub struct SectionHeader<'a, Message> {
    title: Cow<'a, str>,
    icon: Option<IconRole>,
    badge: Option<Cow<'a, str>>,
    title_tooltip: Option<Cow<'a, str>>,
    size: ControlSize,
    status: Option<SectionHeaderStatus<'a>>,
    actions: Vec<SectionHeaderAction<'a, Message>>,
    trailing: Option<Element<'a, Message>>,
}

pub struct SectionHeaderAction<'a, Message> {
    icon: Option<IconRole>,
    label: Option<Cow<'a, str>>,
    tooltip: Option<Cow<'a, str>>,
    disabled: bool,
    on_press: Option<Message>,
    separator: bool,
}

pub struct SectionHeaderStatus<'a> {
    kind: SectionHeaderStatusKind<'a>,
}

enum SectionHeaderStatusKind<'a> {
    Refreshing {
        label: Cow<'a, str>,
        tone: ToneRole,
    },
    IconLabel {
        icon: IconRole,
        label: Cow<'a, str>,
        tone: ToneRole,
    },
    Icon {
        icon: IconRole,
        tooltip: Cow<'a, str>,
        tone: ToneRole,
    },
}

impl<'a, Message> SectionHeader<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(title: impl Into<Cow<'a, str>>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            badge: None,
            title_tooltip: None,
            size: ControlSize::Sm,
            status: None,
            actions: Vec::new(),
            trailing: None,
        }
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn badge(mut self, badge: impl Into<Cow<'a, str>>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// Exposes the full title when the visible single-line title is clipped.
    pub fn title_tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.title_tooltip = Some(tooltip.into());
        self
    }

    /// Configures the full-title tooltip when one is available.
    pub fn title_tooltip_maybe(mut self, tooltip: Option<impl Into<Cow<'a, str>>>) -> Self {
        self.title_tooltip = tooltip.map(Into::into);
        self
    }

    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    pub fn status(mut self, status: SectionHeaderStatus<'a>) -> Self {
        self.status = Some(status);
        self
    }

    pub fn status_from(
        mut self,
        presentation: &impl ResourceStatusPresentation,
        refreshing_label: &'a str,
        error_label: &'a str,
    ) -> Self {
        self.status =
            SectionHeaderStatus::from_presentation(presentation, refreshing_label, error_label);
        self
    }

    pub fn action(mut self, action: SectionHeaderAction<'a, Message>) -> Self {
        self.actions.push(action);
        self
    }

    pub fn actions(
        mut self,
        actions: impl IntoIterator<Item = SectionHeaderAction<'a, Message>>,
    ) -> Self {
        self.actions.extend(actions);
        self
    }

    /// Adds protected custom trailing content after status and header actions.
    pub fn trailing(mut self, trailing: impl Into<Element<'a, Message>>) -> Self {
        self.trailing = Some(trailing.into());
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_section_header::metrics(self.size);
        let title: Element<'a, Message> = text(self.title)
            .font(metrics.title_style.font)
            .size(metrics.title_style.size)
            .line_height(metrics.title_style.line_height)
            .wrapping(text::Wrapping::None)
            .style(theme::text::style(TextRole::Primary))
            .into();
        let title: Element<'a, Message> = container(title).width(Length::Fill).clip(true).into();
        let title = match self.title_tooltip {
            Some(tooltip) => crate::widgets::overlays::tooltip::bottom(title, tooltip),
            None => title,
        };
        let mut leading = row![]
            .spacing(metrics.status_gap)
            .align_y(Alignment::Center)
            .width(Length::Fill);
        if let Some(icon) = self.icon {
            leading = leading.push(
                icon::role(icon)
                    .custom_size(metrics.icon_size)
                    .color(theme::active().text(TextRole::Secondary).color),
            );
        }
        leading = leading.push(title);
        if let Some(badge) = self.badge {
            leading = leading.push(Badge::new(badge).neutral().xs());
        }
        let mut header = Row::new()
            .push(leading)
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        if let Some(status) = self.status {
            header = header.push(status.into_element(metrics));
        }

        if !self.actions.is_empty() {
            let actions = row(self
                .actions
                .into_iter()
                .map(|action| action.into_element(metrics, self.size))
                .collect::<Vec<_>>())
            .spacing(metrics.action_gap)
            .align_y(Alignment::Center);
            header = header.push(actions);
        }
        if let Some(trailing) = self.trailing {
            header = header.push(trailing);
        }

        container(header)
            .height(Length::Fixed(metrics.height))
            .align_y(Alignment::Center)
            .into()
    }
}

impl<'a, Message> From<SectionHeader<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(header: SectionHeader<'a, Message>) -> Self {
        header.into_element()
    }
}

impl<'a, Message> SectionHeaderAction<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn icon(icon: IconRole) -> Self {
        Self {
            icon: Some(icon),
            label: None,
            tooltip: None,
            disabled: false,
            on_press: None,
            separator: false,
        }
    }

    pub fn text(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            icon: None,
            label: Some(label.into()),
            tooltip: None,
            disabled: false,
            on_press: None,
            separator: false,
        }
    }

    pub fn icon_text(icon: IconRole, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            icon: Some(icon),
            label: Some(label.into()),
            tooltip: None,
            disabled: false,
            on_press: None,
            separator: false,
        }
    }

    /// Adds a quiet vertical boundary inside a protected action lane.
    pub fn separator() -> Self {
        Self {
            icon: None,
            label: None,
            tooltip: None,
            disabled: true,
            on_press: None,
            separator: true,
        }
    }

    /// Composes actions as a protected, control-size-derived lane.
    pub fn group(
        actions: impl IntoIterator<Item = Self>,
        size: ControlSize,
    ) -> Element<'a, Message> {
        let metrics = theme_section_header::metrics(size);
        row(actions
            .into_iter()
            .map(|action| action.into_element(metrics, size))
            .collect::<Vec<_>>())
        .spacing(metrics.action_gap)
        .align_y(Alignment::Center)
        .into()
    }

    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message;
        self
    }

    fn into_element(
        self,
        metrics: theme_section_header::SectionHeaderMetrics,
        size: ControlSize,
    ) -> Element<'a, Message> {
        if self.separator {
            let color = theme::active()
                .border(crate::theme::BorderRole::Subtle)
                .color;
            return container(Space::new())
                .width(Length::Fixed(1.0))
                .height(Length::Fixed(metrics.status_height))
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(color)),
                    ..container::Style::default()
                })
                .into();
        }
        let mut action = match (self.icon, self.label) {
            (Some(icon), None) => button::icon(icon)
                .padding(Padding::ZERO)
                .width(Length::Fixed(metrics.icon_button_side)),
            (None, Some(label)) => button::Button::custom(text(label).into())
                .ghost()
                .padding(Padding::ZERO.horizontal(metrics.action_gap)),
            (Some(icon), Some(label)) => {
                let content = row![icon::role(icon).custom_size(metrics.icon_size), text(label)]
                    .spacing(metrics.status_gap)
                    .align_y(Alignment::Center);
                button::Button::custom(content.into())
                    .ghost()
                    .padding(Padding::ZERO.horizontal(metrics.action_gap))
            }
            (None, None) => button::Button::custom(Space::new().into()).ghost(),
        };

        action = action
            .size(size)
            .height(Length::Fixed(metrics.icon_button_side))
            .disabled(self.disabled)
            .tooltip_maybe(self.tooltip)
            .on_press_maybe(self.on_press);

        action.into()
    }
}

impl<'a> SectionHeaderStatus<'a> {
    pub fn refreshing(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            kind: SectionHeaderStatusKind::Refreshing {
                label: label.into(),
                tone: ToneRole::Neutral,
            },
        }
    }

    pub fn icon_label(icon: IconRole, label: impl Into<Cow<'a, str>>, tone: ToneRole) -> Self {
        Self {
            kind: SectionHeaderStatusKind::IconLabel {
                icon,
                label: label.into(),
                tone,
            },
        }
    }

    /// Creates compact icon-only semantic status with a discoverable description.
    pub fn icon(icon: IconRole, tone: ToneRole, tooltip: impl Into<Cow<'a, str>>) -> Self {
        Self {
            kind: SectionHeaderStatusKind::Icon {
                icon,
                tooltip: tooltip.into(),
                tone,
            },
        }
    }

    fn from_presentation(
        presentation: &impl ResourceStatusPresentation,
        refreshing_label: &'a str,
        error_label: &'a str,
    ) -> Option<Self> {
        match resource_status_phase(presentation) {
            ResourceStatusPhase::Idle => None,
            ResourceStatusPhase::Refreshing => Some(Self::refreshing(refreshing_label)),
            ResourceStatusPhase::StaleError => Some(Self::icon_label(
                IconRole::DialogError,
                error_label,
                ToneRole::Danger,
            )),
        }
    }

    pub fn tone(mut self, tone: ToneRole) -> Self {
        match &mut self.kind {
            SectionHeaderStatusKind::Refreshing {
                tone: status_tone, ..
            }
            | SectionHeaderStatusKind::IconLabel {
                tone: status_tone, ..
            }
            | SectionHeaderStatusKind::Icon {
                tone: status_tone, ..
            } => {
                *status_tone = tone;
            }
        }

        self
    }

    pub fn neutral(self) -> Self {
        self.tone(ToneRole::Neutral)
    }

    pub fn accent(self) -> Self {
        self.tone(ToneRole::Accent)
    }

    pub fn info(self) -> Self {
        self.tone(ToneRole::Info)
    }

    pub fn success(self) -> Self {
        self.tone(ToneRole::Success)
    }

    pub fn warning(self) -> Self {
        self.tone(ToneRole::Warning)
    }

    pub fn danger(self) -> Self {
        self.tone(ToneRole::Danger)
    }

    fn into_element<Message>(
        self,
        metrics: theme_section_header::SectionHeaderMetrics,
    ) -> Element<'a, Message>
    where
        Message: 'a,
    {
        match self.kind {
            SectionHeaderStatusKind::Refreshing { label, tone } => {
                let color = theme::active().tone(tone).color;
                let icon_size = metrics.icon_size;

                row![
                    AnimatedVisual::new(move |frame| -> Element<'a, Message> {
                        icon::role(IconRole::ViewRefresh)
                            .custom_size(icon_size)
                            .color(color)
                            .animated_rotation(Radians(frame.turns() * std::f32::consts::TAU))
                            .into()
                    })
                    .animation(Animation::linear(Duration::from_millis(1500)).repeat()),
                    text(label)
                        .font(metrics.status_style.font)
                        .size(metrics.status_style.size)
                        .line_height(metrics.status_style.line_height)
                        .style(theme::text::style(TextRole::Muted)),
                ]
                .spacing(metrics.status_gap)
                .align_y(Alignment::Center)
                .height(Length::Fixed(metrics.status_height))
                .into()
            }
            SectionHeaderStatusKind::IconLabel { icon, label, tone } => {
                let color = theme::active().tone(tone).color;

                row![
                    icon::role(icon).custom_size(metrics.icon_size).color(color),
                    text(label)
                        .font(metrics.status_style.font)
                        .size(metrics.status_style.size)
                        .line_height(metrics.status_style.line_height)
                        .style(theme::text::tone(tone)),
                ]
                .spacing(metrics.status_gap)
                .align_y(Alignment::Center)
                .height(Length::Fixed(metrics.status_height))
                .into()
            }
            SectionHeaderStatusKind::Icon {
                icon,
                tooltip,
                tone,
            } => crate::widgets::overlays::tooltip::bottom(
                icon::role(icon)
                    .custom_size(metrics.icon_size)
                    .color(theme::active().tone(tone).color),
                tooltip,
            ),
        }
    }
}

#[cfg(test)]
mod section_header_status_tests {
    use super::*;
    use crate::widgets::feedback::ErrorPresentation;

    struct Presentation {
        refreshing: bool,
        has_value: bool,
        has_error: bool,
    }

    impl ResourceStatusPresentation for Presentation {
        fn is_refreshing(&self) -> bool {
            self.refreshing
        }

        fn has_value(&self) -> bool {
            self.has_value
        }

        fn error(&self) -> Option<&dyn ErrorPresentation> {
            self.has_error.then_some(&TestError)
        }
    }

    struct TestError;

    impl ErrorPresentation for TestError {
        fn summary(&self) -> &str {
            "summary"
        }

        fn detail(&self) -> &str {
            "detail"
        }
    }

    #[test]
    fn idle_presentation_yields_no_status() {
        let status = SectionHeaderStatus::from_presentation(
            &Presentation {
                refreshing: false,
                has_value: false,
                has_error: false,
            },
            "Refreshing",
            "Failed",
        );

        assert!(status.is_none());
    }

    #[test]
    fn refreshing_with_cached_value_yields_refreshing_status() {
        let status = SectionHeaderStatus::from_presentation(
            &Presentation {
                refreshing: true,
                has_value: true,
                has_error: false,
            },
            "Refreshing",
            "Failed",
        )
        .expect("expected a status");

        assert!(matches!(
            status.kind,
            SectionHeaderStatusKind::Refreshing { label, .. }
                if label == "Refreshing"
        ));
    }

    #[test]
    fn stale_error_yields_danger_icon_label_status() {
        let status = SectionHeaderStatus::from_presentation(
            &Presentation {
                refreshing: false,
                has_value: true,
                has_error: true,
            },
            "Refreshing",
            "Failed",
        )
        .expect("expected a status");

        assert!(matches!(
            status.kind,
            SectionHeaderStatusKind::IconLabel {
                label,
                tone: ToneRole::Danger,
                ..
            } if label == "Failed"
        ));
    }
}
