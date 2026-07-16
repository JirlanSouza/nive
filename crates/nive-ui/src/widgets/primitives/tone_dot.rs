use std::borrow::Cow;

use iced::{
    border::Radius,
    widget::{container, Space},
    Background, Border, Length, Shadow,
};

use crate::theme::{ControlSize, TextRole, ToneRole, TypographyRole};
use crate::Element;

/// Complete visible semantic status supplied to a compact host.
///
/// A host may render a Count badge beside this model, but a nonempty Status
/// badge takes precedence over it. The label, rather than color or a tooltip,
/// carries the accessible visible meaning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusIndicator<'a> {
    tone: ToneRole,
    label: Cow<'a, str>,
}

impl<'a> StatusIndicator<'a> {
    /// Creates a stable semantic tone paired with complete visible text.
    pub fn new(tone: ToneRole, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            tone,
            label: label.into(),
        }
    }

    /// Returns the semantic tone.
    pub const fn tone(&self) -> ToneRole {
        self.tone
    }

    /// Returns the complete visible label.
    pub fn label(&self) -> &str {
        self.label.as_ref()
    }

    /// Returns whether the label carries no visible meaning.
    pub fn is_empty(&self) -> bool {
        self.label.trim().is_empty()
    }

    /// Consumes the model into its typed parts.
    pub fn into_parts(self) -> (ToneRole, Cow<'a, str>) {
        (self.tone, self.label)
    }
}

impl<'a, Message> From<StatusIndicator<'a>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(status: StatusIndicator<'a>) -> Self {
        if status.is_empty() {
            return Space::new()
                .width(Length::Fixed(0.0))
                .height(Length::Fixed(0.0))
                .into();
        }
        let (tone, label) = status.into_parts();
        iced::widget::row![
            ToneDot::new(tone),
            crate::widgets::text::with_role(label, TypographyRole::Body, TextRole::Secondary),
        ]
        .spacing(crate::theme::spacing().xs)
        .align_y(iced::Alignment::Center)
        .into()
    }
}

/// Compact filled dot for stable semantic status.
///
/// Use [`StatusIndicator`] in public compositions so the dot is not the only
/// status carrier. Xs/Sm resolve to 6 px and Md/Lg to 8 px. Enabled theme tones
/// are projected for at least 3:1 contrast against supported opaque surfaces;
/// the owning host passes disabled context once. Loading/activity belongs to
/// `Spinner`, not this primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToneDot {
    tone: ToneRole,
    size: ControlSize,
    disabled: bool,
}

impl ToneDot {
    /// Creates a static stable-status dot.
    pub fn new(tone: ToneRole) -> Self {
        Self {
            tone,
            size: ControlSize::Sm,
            disabled: false,
        }
    }

    /// Resolves Xs/Sm to 6 px and Md/Lg to 8 px.
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

    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    pub fn lg(self) -> Self {
        self.size(ControlSize::Lg)
    }

    /// Applies the owning host's disabled context exactly once.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn into_element<'a, Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        tone_dot(self.tone, self.size, self.disabled)
    }
}

impl<'a, Message> From<ToneDot> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(dot: ToneDot) -> Self {
        dot.into_element()
    }
}

pub(crate) fn tone_dot<'a, Message>(
    tone: ToneRole,
    size: ControlSize,
    disabled: bool,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let diameter = dot_size(size);
    container(Space::new().width(Length::Fixed(diameter)))
        .style(tone_dot_style(tone, diameter, disabled))
        .width(Length::Fixed(diameter))
        .height(Length::Fixed(diameter))
        .into()
}

fn tone_dot_style(
    tone: ToneRole,
    diameter: f32,
    disabled: bool,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let tone = theme.tone(tone);
        let alpha = if disabled { 0.55 } else { 1.0 };

        container::Style {
            text_color: None,
            background: Some(Background::Color(tone.color.scale_alpha(alpha))),
            border: Border {
                width: 0.0,
                radius: Radius::new(diameter / 2.0),
                ..Border::default()
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

pub(crate) const fn dot_size(size: ControlSize) -> f32 {
    match size {
        ControlSize::Xs | ControlSize::Sm => 6.0,
        ControlSize::Md | ControlSize::Lg => 8.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{SurfaceRole, Theme};

    const TONES: [ToneRole; 6] = [
        ToneRole::Neutral,
        ToneRole::Accent,
        ToneRole::Info,
        ToneRole::Success,
        ToneRole::Warning,
        ToneRole::Danger,
    ];
    const OPAQUE_SURFACES: [SurfaceRole; 8] = [
        SurfaceRole::App,
        SurfaceRole::Chrome,
        SurfaceRole::Sidebar,
        SurfaceRole::Panel,
        SurfaceRole::Elevated,
        SurfaceRole::Canvas,
        SurfaceRole::Dialog,
        SurfaceRole::Popover,
    ];

    #[test]
    fn control_sizes_collapse_to_two_semantic_diameters() {
        assert_eq!(dot_size(ControlSize::Xs), 6.0);
        assert_eq!(dot_size(ControlSize::Sm), 6.0);
        assert_eq!(dot_size(ControlSize::Md), 8.0);
        assert_eq!(dot_size(ControlSize::Lg), 8.0);
    }

    #[test]
    fn status_indicator_preserves_owned_borrowed_and_empty_labels() {
        let borrowed = StatusIndicator::new(ToneRole::Success, "Healthy");
        let owned = StatusIndicator::new(ToneRole::Success, String::from("Healthy"));
        let empty = StatusIndicator::new(ToneRole::Warning, "  ");

        assert_eq!(borrowed, owned);
        assert_eq!(borrowed.label(), "Healthy");
        assert!(empty.is_empty());
    }

    #[test]
    fn enabled_tones_clear_three_to_one_on_supported_opaque_surfaces() {
        let themes = [
            Theme::Light,
            Theme::Dark,
            Theme::builder("Custom Light", crate::theme::ThemeMode::Light).build(),
            Theme::builder("Custom Dark", crate::theme::ThemeMode::Dark).build(),
        ];
        for theme in themes {
            for tone in TONES {
                for surface in OPAQUE_SURFACES {
                    let foreground = theme.tone(tone).color;
                    let background = theme.surface(surface).background;
                    let contrast = crate::theme::color::contrast_ratio(foreground, background);
                    assert!(
                        contrast >= 3.0,
                        "{theme:?} {tone:?} over {surface:?}: {contrast:.2}"
                    );
                }
            }
        }
    }

    #[test]
    fn enabled_tones_use_distinct_colors() {
        for theme in [
            Theme::Light,
            Theme::Dark,
            Theme::builder("Custom Light", crate::theme::ThemeMode::Light).build(),
            Theme::builder("Custom Dark", crate::theme::ThemeMode::Dark).build(),
        ] {
            let colors = TONES.map(|tone| theme.tone(tone).color);
            for (index, color) in colors.iter().enumerate() {
                assert!(
                    !colors[index + 1..].contains(color),
                    "duplicate tone color in {theme:?}"
                );
            }
        }
    }
}
