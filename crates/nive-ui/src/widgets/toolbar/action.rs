use iced::{
    widget::{button, container, row, text, Space},
    Alignment, Length, Padding,
};

use crate::Element;

use super::super::button::ButtonFocusRing;
use super::super::{
    feedback::LoadingIndicator, icon as icon_widget, pressable::Pressable,
    tooltip as tooltip_widget, IconName,
};
use super::style as theme_toolbar;

pub struct ToolbarAction<'a, Message> {
    label: Option<&'a str>,
    icon: Option<IconName>,
    selected: bool,
    disabled: bool,
    loading: bool,
    reserve_loading_indicator: bool,
    on_press: Option<Message>,
    tooltip: Option<&'a str>,
}

impl<'a, Message: Clone + 'a> ToolbarAction<'a, Message> {
    pub fn icon(icon: IconName) -> Self {
        Self {
            label: None,
            icon: Some(icon),
            selected: false,
            disabled: false,
            loading: false,
            reserve_loading_indicator: false,
            on_press: None,
            tooltip: None,
        }
    }

    pub fn label(label: &'a str) -> Self {
        Self {
            label: Some(label),
            icon: None,
            selected: false,
            disabled: false,
            loading: false,
            reserve_loading_indicator: false,
            on_press: None,
            tooltip: None,
        }
    }

    pub fn icon_label(icon: IconName, label: &'a str) -> Self {
        Self {
            label: Some(label),
            icon: Some(icon),
            selected: false,
            disabled: false,
            loading: false,
            reserve_loading_indicator: false,
            on_press: None,
            tooltip: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self.reserve_loading_indicator = true;
        self
    }

    pub fn tooltip(mut self, tooltip: &'a str) -> Self {
        self.tooltip = Some(tooltip);
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

    pub(super) fn into_element(
        self,
        metrics: theme_toolbar::ToolbarMetrics,
    ) -> Element<'a, Message> {
        let is_icon_only = self.label.is_none();
        let disabled = self.disabled || self.loading;
        let activation = if disabled {
            None
        } else {
            self.on_press.clone()
        };
        let button = button::Button::new(self.content(metrics))
            .style(theme_toolbar::action_style(self.selected, metrics.radius))
            .padding(if is_icon_only {
                Padding::ZERO
            } else {
                Padding::ZERO.horizontal(metrics.action_padding_h)
            })
            .width(if is_icon_only {
                Length::Fixed(metrics.action_height)
            } else {
                Length::Shrink
            })
            .height(Length::Fixed(metrics.action_height))
            .on_press_maybe(activation.clone());
        let button = Pressable::maybe(
            button,
            activation,
            metrics.radius.into(),
            ButtonFocusRing::Default,
        );

        match self.tooltip {
            Some(label) => tooltip_widget::bottom(button, label),
            None => button,
        }
    }

    fn content(&self, metrics: theme_toolbar::ToolbarMetrics) -> Element<'a, Message> {
        match (self.icon, self.label) {
            (Some(icon), Some(label)) => self.icon_label_content(icon, label, metrics),
            (Some(icon), None) => self.icon_content(icon, metrics),
            (None, Some(label)) => self.label_content(label, metrics),
            (None, None) => {
                if self.loading {
                    loading_indicator_slot(metrics.icon_size)
                } else {
                    text("")
                        .size(metrics.font_size)
                        .shaping(text::Shaping::Auto)
                        .into()
                }
            }
        }
    }

    fn icon_label_content(
        &self,
        icon: IconName,
        label: &'a str,
        metrics: theme_toolbar::ToolbarMetrics,
    ) -> Element<'a, Message> {
        row![
            self.leading_slot(Some(icon), metrics.icon_size),
            text(label)
                .size(metrics.font_size)
                .shaping(text::Shaping::Auto)
        ]
        .spacing(metrics.gap)
        .align_y(Alignment::Center)
        .into()
    }

    fn icon_content(
        &self,
        icon: IconName,
        metrics: theme_toolbar::ToolbarMetrics,
    ) -> Element<'a, Message> {
        if self.loading {
            loading_indicator_slot(metrics.icon_size)
        } else {
            icon_widget::new(icon).size(metrics.icon_size).into()
        }
    }

    fn label_content(
        &self,
        label: &'a str,
        metrics: theme_toolbar::ToolbarMetrics,
    ) -> Element<'a, Message> {
        let label = text(label)
            .size(metrics.font_size)
            .shaping(text::Shaping::Auto);

        if self.reserve_loading_indicator {
            row![self.leading_slot(None, metrics.icon_size), label]
                .spacing(metrics.gap)
                .align_y(Alignment::Center)
                .into()
        } else {
            label.into()
        }
    }

    fn leading_slot(&self, icon: Option<IconName>, size: f32) -> Element<'a, Message> {
        if self.loading {
            loading_indicator_slot(size)
        } else if let Some(icon) = icon {
            icon_widget::new(icon).size(size).into()
        } else {
            Space::new().width(Length::Fixed(size)).into()
        }
    }
}

fn loading_indicator_slot<'a, Message>(size: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    container(LoadingIndicator::new().neutral().xs())
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}
