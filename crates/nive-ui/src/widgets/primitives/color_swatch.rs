use iced::{
    widget::{button as iced_button, container, text},
    Background, Border, Color, Length, Padding, Shadow,
};

use crate::{
    theme::{self, BorderRole, ShapeSize, TextRole, ToneRole},
    Element,
};

use crate::advanced::pressable::Pressable;
use crate::widgets::controls::button::ButtonFocusRing;
use crate::widgets::overlays::tooltip as tooltip_widget;

pub struct ColorSwatch<'a, Message> {
    color: Color,
    size: f32,
    radius: Option<f32>,
    selected: bool,
    disabled: bool,
    on_press: Option<Message>,
    tooltip: Option<&'a str>,
}

impl<'a, Message> ColorSwatch<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(color: Color) -> Self {
        Self {
            color,
            size: 14.0,
            radius: None,
            selected: false,
            disabled: false,
            on_press: None,
            tooltip: None,
        }
    }

    pub fn xs(mut self) -> Self {
        self.size = 10.0;
        self
    }

    pub fn sm(mut self) -> Self {
        self.size = 14.0;
        self
    }

    pub fn md(mut self) -> Self {
        self.size = 18.0;
        self
    }

    pub fn lg(mut self) -> Self {
        self.size = 22.0;
        self
    }

    pub fn size(mut self, size: impl Into<f32>) -> Self {
        self.size = size.into();
        self
    }

    pub fn radius(mut self, radius: impl Into<f32>) -> Self {
        self.radius = Some(radius.into());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

    fn into_element(self) -> Element<'a, Message> {
        let size = self.size;
        let default_radius = theme::active().shape(ShapeSize::Md).radius_value();
        let radius = self.radius.unwrap_or((size / 2.4).min(default_radius));
        let disabled = self.disabled;
        let selected = self.selected;
        let content = container(text(""))
            .style(swatch_style(self.color, radius, selected, disabled))
            .width(Length::Fixed(size))
            .height(Length::Fixed(size));

        let activation = if disabled {
            None
        } else {
            self.on_press.clone()
        };
        let element: Element<'a, Message> = if self.on_press.is_some() {
            let button = iced_button::Button::new(content)
                .style(button_style())
                .padding(Padding::ZERO)
                .width(Length::Fixed(size))
                .height(Length::Fixed(size))
                .on_press_maybe(activation.clone());

            Pressable::maybe(button, activation, radius.into(), ButtonFocusRing::Default)
        } else {
            content.into()
        };

        match self.tooltip {
            Some(label) => tooltip_widget::bottom(element, label),
            None => element,
        }
    }
}

impl<'a, Message> From<ColorSwatch<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(swatch: ColorSwatch<'a, Message>) -> Self {
        swatch.into_element()
    }
}

fn swatch_style(
    color: Color,
    radius: f32,
    selected: bool,
    disabled: bool,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        let theme = *theme;
        let background = if disabled {
            color.scale_alpha(0.45)
        } else {
            color
        };
        let border_color = if selected {
            theme.tone(ToneRole::Accent).color
        } else {
            theme.border(BorderRole::Default).color
        };
        let border_width = if selected { 2.0 } else { 1.0 };

        container::Style {
            background: Some(Background::Color(background)),
            border: Border {
                color: border_color,
                width: border_width,
                radius: radius.into(),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

fn button_style() -> impl Fn(&crate::theme::Theme, iced_button::Status) -> iced_button::Style {
    move |theme: &crate::theme::Theme, _status: iced_button::Status| iced_button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: theme.text(TextRole::Primary).color,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        shadow: Shadow::default(),
        ..iced_button::Style::default()
    }
}

#[cfg(test)]
mod color_swatch_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn selected_swatch_uses_app_primary_border() {
        let theme = Theme::Dark;
        let style = swatch_style(Color::WHITE, 4.0, true, false)(&theme);

        assert_eq!(style.border.color, theme.tone(ToneRole::Accent).color);
        assert_eq!(style.border.width, 2.0);
    }
}
