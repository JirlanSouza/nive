mod style;

use iced::{
    time::Duration,
    widget::{container, row, text, Row},
    Alignment, Length, Padding, Radians,
};

use super::{button, icon, AnimatedVisual, Animation, AppIcon};
use crate::theme::{self, ControlSize, TextRole, ToneRole};
use crate::Element;

use self::style as theme_section_header;

pub struct SectionHeader<'a, Message> {
    title: &'a str,
    size: ControlSize,
    status: Option<SectionHeaderStatus<'a>>,
    actions: Vec<SectionHeaderAction<'a, Message>>,
}

pub struct SectionHeaderAction<'a, Message> {
    icon: AppIcon,
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
        icon: AppIcon,
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

    pub fn status_maybe(mut self, status: Option<SectionHeaderStatus<'a>>) -> Self {
        self.status = status;
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
    pub fn icon(icon: AppIcon) -> Self {
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

    pub fn icon_label(icon: AppIcon, label: &'a str, tone: ToneRole) -> Self {
        Self {
            kind: SectionHeaderStatusKind::IconLabel { icon, label, tone },
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
                        icon::new(AppIcon::RefreshCw)
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
                    icon::new(icon).size(metrics.icon_size).color(color),
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
