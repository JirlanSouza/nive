mod style;

use iced::{
    time::Duration,
    widget::{container, row, text, Row},
    Alignment, Length, Padding, Radians,
};

use crate::animation::{AnimatedVisual, Animation};
use crate::theme::{self, ControlSize, TextRole, ToneRole};
use crate::widgets::controls::button;
use crate::widgets::feedback::presentation::{resource_status_phase, ResourceStatusPhase};
use crate::widgets::feedback::ResourceStatusPresentation;
use crate::widgets::primitives::{icon, IconRole};
use crate::Element;

use self::style as theme_section_header;

pub struct SectionHeader<'a, Message> {
    title: &'a str,
    size: ControlSize,
    status: Option<SectionHeaderStatus<'a>>,
    actions: Vec<SectionHeaderAction<'a, Message>>,
}

pub struct SectionHeaderAction<'a, Message> {
    icon: IconRole,
    tooltip: Option<&'a str>,
    disabled: bool,
    on_press: Option<Message>,
}

pub struct SectionHeaderStatus<'a> {
    kind: SectionHeaderStatusKind<'a>,
}

enum SectionHeaderStatusKind<'a> {
    Refreshing {
        label: &'a str,
        tone: ToneRole,
    },
    IconLabel {
        icon: IconRole,
        label: &'a str,
        tone: ToneRole,
    },
}

impl<'a, Message> SectionHeader<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            size: ControlSize::Sm,
            status: None,
            actions: Vec::new(),
        }
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
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

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_section_header::metrics(self.size);
        let title = text(self.title)
            .size(metrics.title_size)
            .line_height(metrics.title_line_height)
            .style(theme::text::style(TextRole::Muted));
        let mut header = Row::new()
            .push(title.width(Length::Fill))
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
            icon,
            tooltip: None,
            disabled: false,
            on_press: None,
        }
    }

    pub fn tooltip(mut self, tooltip: &'a str) -> Self {
        self.tooltip = Some(tooltip);
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
        button::icon(self.icon)
            .size(size)
            .padding(Padding::ZERO)
            .width(Length::Fixed(metrics.icon_button_side))
            .height(Length::Fixed(metrics.icon_button_side))
            .disabled(self.disabled)
            .tooltip_maybe(self.tooltip)
            .on_press_maybe(self.on_press)
            .into()
    }
}

impl<'a> SectionHeaderStatus<'a> {
    pub fn refreshing(label: &'a str) -> Self {
        Self {
            kind: SectionHeaderStatusKind::Refreshing {
                label,
                tone: ToneRole::Neutral,
            },
        }
    }

    pub fn icon_label(icon: IconRole, label: &'a str, tone: ToneRole) -> Self {
        Self {
            kind: SectionHeaderStatusKind::IconLabel { icon, label, tone },
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
                            .size(icon_size)
                            .color(color)
                            .rotation(Radians(frame.turns() * std::f32::consts::TAU))
                            .into()
                    })
                    .animation(Animation::linear(Duration::from_millis(1500)).repeat()),
                    text(label)
                        .size(metrics.status_size)
                        .line_height(metrics.status_line_height)
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
                    icon::role(icon).size(metrics.icon_size).color(color),
                    text(label)
                        .size(metrics.status_size)
                        .line_height(metrics.status_line_height)
                        .style(theme::text::tone(tone)),
                ]
                .spacing(metrics.status_gap)
                .align_y(Alignment::Center)
                .height(Length::Fixed(metrics.status_height))
                .into()
            }
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
            SectionHeaderStatusKind::Refreshing {
                label: "Refreshing",
                ..
            }
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
                label: "Failed",
                tone: ToneRole::Danger,
                ..
            }
        ));
    }
}
