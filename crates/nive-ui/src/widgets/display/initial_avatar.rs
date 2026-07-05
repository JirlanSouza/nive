use iced::{
    widget::{container, text},
    Alignment, Background, Border, Shadow,
};

use crate::{
    theme::{
        self, BorderRole, BorderSpec, ShapeRole, SurfaceRole, TextRole, ToneRole, TypographyRole,
    },
    Element,
};

const AVATAR_SIZE_SM: f32 = 32.0;
const AVATAR_SIZE_LG: f32 = 56.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarClass {
    Surface,
    Accent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarSize {
    Sm,
    Lg,
}

pub struct InitialAvatar<'a, Message> {
    label: &'a str,
    class: AvatarClass,
    size: AvatarSize,
    _marker: std::marker::PhantomData<Message>,
}

impl<'a, Message> InitialAvatar<'a, Message>
where
    Message: 'a,
{
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            class: AvatarClass::Surface,
            size: AvatarSize::Sm,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn accent(mut self) -> Self {
        self.class = AvatarClass::Accent;
        self
    }

    pub fn sm(mut self) -> Self {
        self.size = AvatarSize::Sm;
        self
    }

    pub fn lg(mut self) -> Self {
        self.size = AvatarSize::Lg;
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = metrics(self.size);
        let initial = self.label.chars().next().unwrap_or('?').to_string();

        container(text(initial).center().size(metrics.font_size))
            .style(style(self.class, metrics.radius))
            .width(metrics.dimension)
            .height(metrics.dimension)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}

impl<'a, Message> From<InitialAvatar<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(avatar: InitialAvatar<'a, Message>) -> Self {
        avatar.into_element()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Metrics {
    dimension: f32,
    font_size: f32,
    radius: f32,
}

fn metrics(size: AvatarSize) -> Metrics {
    match size {
        AvatarSize::Sm => Metrics {
            dimension: AVATAR_SIZE_SM,
            font_size: theme::typography(TypographyRole::Body).size,
            radius: theme::active().shape(ShapeRole::Medium).radius_value(),
        },
        AvatarSize::Lg => Metrics {
            dimension: AVATAR_SIZE_LG,
            font_size: 24.0,
            radius: theme::active().shape(ShapeRole::ExtraLarge).radius_value(),
        },
    }
}

fn style(class: AvatarClass, radius: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let theme = *theme;
        let (background, text_color, border) = match class {
            AvatarClass::Surface => {
                let surface = theme.surface(SurfaceRole::Elevated);
                (
                    surface.background,
                    theme.text(TextRole::Secondary).color,
                    theme.border(BorderRole::Subtle),
                )
            }
            AvatarClass::Accent => {
                let tone = theme.tone(ToneRole::Primary);
                (
                    tone.color,
                    theme.tone(ToneRole::Primary).on_color,
                    BorderSpec::none(),
                )
            }
        };

        container::Style {
            text_color: Some(text_color),
            background: Some(Background::Color(background)),
            border: Border {
                color: border.color,
                width: border.width,
                radius: radius.into(),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

#[cfg(test)]
mod initial_avatar_tests {
    use iced::Color;

    use super::*;
    use crate::theme::Theme;

    #[test]
    fn accent_avatar_uses_primary_tone() {
        let theme = Theme::Dark;
        let style = style(
            AvatarClass::Accent,
            theme.shape(ShapeRole::Medium).radius_value(),
        )(&theme);
        let tone = theme.tone(ToneRole::Primary);

        assert_eq!(background_color(&style), tone.color);
        assert_eq!(style.text_color, Some(tone.on_color));
    }

    fn background_color(style: &container::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
