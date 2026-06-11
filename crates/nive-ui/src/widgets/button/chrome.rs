use iced::{border::Radius, widget::button as iced_button};

use super::style as theme_button;
use super::{content, Button};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum ButtonChrome {
    Standalone,
    Embedded { radius: Radius },
}

impl ButtonChrome {
    pub(super) fn radius(self, size: crate::theme::ControlSize) -> Radius {
        match self {
            Self::Standalone => theme_button::metrics(size).radius.into(),
            Self::Embedded { radius } => radius,
        }
    }
}

pub(super) fn into_app_button<'a, Message>(
    button: Button<'a, Message>,
    chrome: ButtonChrome,
) -> iced_button::Button<'a, Message, crate::theme::Theme>
where
    Message: Clone + 'a,
{
    let metrics = theme_button::metrics(button.size);
    let disabled = button.disabled || button.loading;
    let width = button
        .width
        .unwrap_or_else(|| button.content.default_width(button.size));

    let content = content::element(
        content::ContentSpec {
            content: button.content,
            leading_icon: button.leading_icon,
            trailing_icon: button.trailing_icon,
            size: button.size,
            text_align: button.text_align,
            loading: button.loading,
            reserve_loading_indicator: button.reserve_loading_indicator,
        },
        width,
    );

    let button_widget = iced_button::Button::new(content)
        .padding(
            button
                .padding
                .unwrap_or_else(|| button.content.default_padding(button.size)),
        )
        .width(width)
        .height(
            button
                .height
                .unwrap_or_else(|| button.content.default_height(button.size)),
        );

    let button_widget = match chrome {
        ButtonChrome::Standalone => {
            button_widget.style(theme_button::style(button.variant, metrics.radius.into()))
        }
        ButtonChrome::Embedded { radius } => {
            button_widget.style(theme_button::embedded_style(radius))
        }
    };

    button_widget.on_press_maybe(if disabled { None } else { button.on_press })
}
