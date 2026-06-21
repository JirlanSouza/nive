use iced::{
    border::Radius,
    widget::button::Status,
    widget::{button, container, text, Row},
    Alignment, Background, Border, Color, Length, Shadow,
};

use crate::theme::{self, control_metrics, ControlRole, ControlSize, ControlState, TextRole};
use crate::Element;

use super::{button::ButtonFocusRing, icon, pressable::Pressable, IconName};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectableItemVariant {
    Default,
    Selected,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SelectableItemMetrics {
    font_size: f32,
    padding_v: f32,
    padding_h: f32,
    radius: f32,
    leading_size: f32,
    icon_size: f32,
    gap: f32,
    height: f32,
}

pub struct SelectableItem<'a, Message> {
    label: &'a str,
    selected: bool,
    leading_icon: Option<IconName>,
    leading_color: Option<Color>,
    trailing_text: Option<&'a str>,
    trailing: Option<Element<'a, Message>>,
    size: ControlSize,
    fill: bool,
    on_press: Option<Message>,
    disabled: bool,
    tooltip_label: Option<&'a str>,
}

impl<'a, Message> SelectableItem<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            selected: false,
            leading_icon: None,
            leading_color: None,
            trailing_text: None,
            trailing: None,
            size: ControlSize::Sm,
            fill: true,
            on_press: None,
            disabled: false,
            tooltip_label: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn leading_color(mut self, color: Color) -> Self {
        self.leading_color = Some(color);
        self
    }

    pub fn trailing_text(mut self, trailing: &'a str) -> Self {
        self.trailing_text = Some(trailing);
        self
    }

    pub fn trailing(mut self, trailing: impl Into<Element<'a, Message>>) -> Self {
        self.trailing = Some(trailing.into());
        self
    }

    pub fn xs(mut self) -> Self {
        self.size = ControlSize::Xs;
        self
    }

    pub fn sm(mut self) -> Self {
        self.size = ControlSize::Sm;
        self
    }

    pub fn md(mut self) -> Self {
        self.size = ControlSize::Md;
        self
    }

    pub fn lg(mut self) -> Self {
        self.size = ControlSize::Lg;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn shrink(mut self) -> Self {
        self.fill = false;
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

    pub fn tooltip(mut self, tooltip: &'a str) -> Self {
        self.tooltip_label = Some(tooltip);
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = metrics(self.size);
        let variant = if self.selected {
            SelectableItemVariant::Selected
        } else {
            SelectableItemVariant::Default
        };

        let label = text(self.label).size(metrics.font_size).width(Length::Fill);

        let mut content = Row::new()
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        if let Some(color) = self.leading_color {
            content = content.push(color_square(color, metrics.leading_size));
        }

        if let Some(icon) = self.leading_icon {
            content = content.push(icon::new(icon).size(metrics.icon_size));
        }

        content = content.push(label);

        if let Some(trailing) = self.trailing_text {
            content = content.push(
                text(trailing)
                    .size(metrics.font_size)
                    .shaping(text::Shaping::Auto),
            );
        }

        if let Some(trailing) = self.trailing {
            content = content.push(trailing);
        }

        let mut item = button::Button::new(content)
            .style(style(variant, metrics.radius))
            .padding([metrics.padding_v, metrics.padding_h]);

        if self.fill {
            item = item.width(Length::Fill);
        }

        item = item.height(metrics.height);
        let activation = if self.disabled {
            None
        } else {
            self.on_press.clone()
        };
        let item = item.on_press_maybe(activation.clone());

        let item = Pressable::maybe(
            item,
            activation,
            metrics.radius.into(),
            ButtonFocusRing::Default,
        );

        if let Some(label) = self.tooltip_label {
            super::tooltip::bottom(item, label)
        } else {
            item
        }
    }
}

impl<'a, Message> From<SelectableItem<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(item: SelectableItem<'a, Message>) -> Self {
        item.into_element()
    }
}

fn color_square<'a, Message>(color: Color, size: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    container(text(""))
        .style(color_square_style(color, size))
        .width(size)
        .height(size)
        .into()
}

fn metrics(size: ControlSize) -> SelectableItemMetrics {
    let control = control_metrics(size);
    let spacing = theme::spacing();

    SelectableItemMetrics {
        font_size: control.font_size,
        padding_v: match size {
            ControlSize::Xs => spacing.xxs,
            ControlSize::Sm => spacing.xs,
            ControlSize::Md => spacing.xs + 1.0,
            ControlSize::Lg => spacing.md,
        },
        padding_h: match size {
            ControlSize::Xs => spacing.sm,
            ControlSize::Sm => spacing.md,
            ControlSize::Md => spacing.md + spacing.xxs,
            ControlSize::Lg => spacing.xl,
        },
        radius: control.radius,
        leading_size: match size {
            ControlSize::Xs => 8.0,
            ControlSize::Sm | ControlSize::Md => 10.0,
            ControlSize::Lg => 12.0,
        },
        icon_size: control.font_size,
        gap: match size {
            ControlSize::Lg => spacing.sm,
            _ => spacing.xs,
        },
        height: control.height,
    }
}

fn content_color(
    theme: &crate::theme::Theme,
    variant: SelectableItemVariant,
    status: Status,
) -> Color {
    let theme = *theme;

    match (variant, status) {
        (_, Status::Disabled) => disabled_alpha(theme.text(TextRole::Muted).color),
        (SelectableItemVariant::Selected, _) => {
            theme
                .control(ControlRole::Selectable, ControlState::SELECTED)
                .foreground
        }
        (SelectableItemVariant::Default, Status::Hovered | Status::Pressed) => {
            theme.text(TextRole::Primary).color
        }
        (SelectableItemVariant::Default, Status::Active) => theme.text(TextRole::Muted).color,
    }
}

fn style(
    variant: SelectableItemVariant,
    radius: f32,
) -> impl Fn(&crate::theme::Theme, Status) -> button::Style {
    move |theme: &crate::theme::Theme, status: Status| {
        let mut style = match variant {
            SelectableItemVariant::Default => default_style(theme, status),
            SelectableItemVariant::Selected => selected_style(theme, status),
        };
        style.border.radius = radius.into();
        style
    }
}

fn color_square_style(
    color: Color,
    size: f32,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |_theme: &crate::theme::Theme| container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::new(size / 2.6),
        },
        ..container::Style::default()
    }
}

fn default_style(theme: &crate::theme::Theme, status: Status) -> button::Style {
    let theme = *theme;
    let control = theme.control(ControlRole::Standard, button_control_state(status));
    let background = match status {
        Status::Active | Status::Disabled => Color::TRANSPARENT,
        Status::Hovered | Status::Pressed => control.background,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: content_color(&theme, SelectableItemVariant::Default, status),
        border: transparent_border(),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn selected_style(theme: &crate::theme::Theme, status: Status) -> button::Style {
    let theme = *theme;
    let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);
    let background = match status {
        Status::Active => selected.background,
        Status::Hovered => selected.background.scale_alpha(1.20),
        Status::Pressed => selected.background.scale_alpha(0.88),
        Status::Disabled => selected.background.scale_alpha(0.55),
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: content_color(&theme, SelectableItemVariant::Selected, status),
        border: transparent_border(),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn button_control_state(status: Status) -> ControlState {
    match status {
        Status::Active => ControlState::ENABLED,
        Status::Hovered => ControlState::HOVERED,
        Status::Pressed => ControlState::PRESSED,
        Status::Disabled => ControlState::DISABLED,
    }
}

fn disabled_alpha(color: Color) -> Color {
    color.scale_alpha(0.55)
}

fn transparent_border() -> Border {
    Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: 0.0.into(),
    }
}

#[cfg(test)]
mod selectable_item_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn selected_item_uses_app_selected_control_spec() {
        let theme = Theme::Dark;
        let style = style(SelectableItemVariant::Selected, 6.0)(&theme, Status::Active);
        let selected = theme.control(ControlRole::Selectable, ControlState::SELECTED);

        assert_eq!(background_color(&style), selected.background);
        assert_eq!(style.text_color, selected.foreground);
    }

    fn background_color(style: &button::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
