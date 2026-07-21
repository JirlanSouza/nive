use std::{borrow::Cow, marker::PhantomData};

use iced::{
    widget::{container, Space},
    Alignment, Background, Border, Length, Padding, Shadow,
};

use crate::{
    theme::{TextRole, ToneRole, TypographyRole},
    widgets::text,
    Element,
};

use super::{
    measured_text::{EllipsisStrategy, MeasuredText},
    min_width::MinWidth,
};

const HEIGHT: f32 = 20.0;
const MINIMUM_WIDTH: f32 = 20.0;
const HORIZONTAL_PADDING: f32 = 6.0;
const STATUS_MAXIMUM_WIDTH: f32 = 96.0;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BadgeContent<'a> {
    /// Numeric content formatted exactly through 99 and compacted afterward.
    Count(u64),
    /// Compact semantic status text retained exactly as owned or borrowed data.
    Status(Cow<'a, str>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BadgeKind {
    /// Numeric Count content.
    Count,
    /// Semantic Status text.
    Status,
}

impl BadgeContent<'_> {
    pub const fn kind(&self) -> BadgeKind {
        match self {
            Self::Count(_) => BadgeKind::Count,
            Self::Status(_) => BadgeKind::Status,
        }
    }
}

impl<'a> From<Cow<'a, str>> for BadgeContent<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self::Status(value)
    }
}

impl<'a> From<&'a str> for BadgeContent<'a> {
    fn from(value: &'a str) -> Self {
        Self::Status(Cow::Borrowed(value))
    }
}

impl From<String> for BadgeContent<'static> {
    fn from(value: String) -> Self {
        Self::Status(Cow::Owned(value))
    }
}

/// Compact, non-interactive Count or Status display content.
///
/// Count is neutral by default and formats values above 99 as `99+`. Status
/// requires complete visible wording; its conditional tooltip only discloses
/// the exact retained text after renderer-measured truncation. Hosts should not
/// combine a Status badge with a second compact semantic-status channel.
/// Numeric tabular-glyph (`tnum`) support is renderer/font dependent and is not
/// claimed by this widget.
pub struct Badge<'a, Message> {
    content: BadgeContent<'a>,
    tone: ToneRole,
    disabled: bool,
    _marker: PhantomData<Message>,
}

impl<'a, Message> Badge<'a, Message>
where
    Message: 'a,
{
    /// Creates compact semantic Status content.
    pub fn status(label: impl Into<Cow<'a, str>>) -> Self {
        Self::from_content(BadgeContent::Status(label.into()))
    }

    /// Creates neutral numeric Count content.
    pub fn count(value: u64) -> Self {
        Self::from_content(BadgeContent::Count(value))
    }

    /// Renders retained typed content without parsing its visible label.
    pub fn from_content(content: BadgeContent<'a>) -> Self {
        Self {
            content,
            tone: ToneRole::Neutral,
            disabled: false,
            _marker: PhantomData,
        }
    }

    pub fn tone(mut self, tone: ToneRole) -> Self {
        self.tone = tone;
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let content: Element<'a, Message> = match self.content {
            BadgeContent::Count(value) => text::badge_label(format_count(value)).into(),
            BadgeContent::Status(label) if label.trim().is_empty() => {
                return Space::new()
                    .width(Length::Fixed(0.0))
                    .height(Length::Fixed(0.0))
                    .into();
            }
            BadgeContent::Status(label) => MeasuredText::new(
                label,
                EllipsisStrategy::End,
                TypographyRole::BadgeLabel,
                TextRole::Primary,
            )
            .max_width(STATUS_MAXIMUM_WIDTH)
            .into(),
        };

        let frame = container(content)
            .style(style(self.tone, self.disabled))
            .padding(Padding::ZERO.horizontal(HORIZONTAL_PADDING))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .height(Length::Fixed(HEIGHT));

        MinWidth::new(frame, MINIMUM_WIDTH).into()
    }
}

impl<'a, Message> From<Badge<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(badge: Badge<'a, Message>) -> Self {
        badge.into_element()
    }
}

pub(crate) fn format_count(value: u64) -> String {
    if value <= 99 {
        value.to_string()
    } else {
        "99+".to_string()
    }
}

fn style(tone: ToneRole, disabled: bool) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let tone = theme.tone(tone);
        let alpha = if disabled { 0.55 } else { 1.0 };

        container::Style {
            text_color: Some(tone.color.scale_alpha(alpha)),
            background: Some(Background::Color(tone.container.scale_alpha(alpha))),
            border: Border {
                radius: (HEIGHT / 2.0).into(),
                ..Border::default()
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

#[cfg(test)]
mod badge_tests {
    use super::*;
    use crate::theme::Theme;
    use iced::Size;

    #[test]
    fn content_kind_and_count_formatting_are_deterministic() {
        assert_eq!(BadgeContent::Count(3).kind(), BadgeKind::Count);
        assert_eq!(
            BadgeContent::Status(Cow::Borrowed("Ready")).kind(),
            BadgeKind::Status
        );
        for (value, expected) in [(0, "0"), (9, "9"), (10, "10"), (99, "99"), (100, "99+")] {
            assert_eq!(format_count(value), expected);
        }
        assert_eq!(format_count(u64::MAX), "99+");
    }

    #[test]
    fn style_is_borderless_shadowless_and_preserves_disabled_tone() {
        let theme = Theme::Dark;
        let enabled = style(ToneRole::Success, false)(&theme);
        let disabled = style(ToneRole::Success, true)(&theme);

        assert_eq!(enabled.border.width, 0.0);
        assert_eq!(enabled.shadow, Shadow::default());
        assert_ne!(enabled.text_color, disabled.text_color);
        assert!(disabled.text_color.is_some());
    }

    #[test]
    fn status_ownership_is_preserved() {
        let borrowed = BadgeContent::from("Ready");
        let owned = BadgeContent::from(String::from("Ready"));

        assert!(matches!(
            borrowed,
            BadgeContent::Status(Cow::Borrowed("Ready"))
        ));
        assert!(matches!(owned, BadgeContent::Status(Cow::Owned(value)) if value == "Ready"));
    }

    #[test]
    fn real_layout_keeps_single_digit_square_bounds_status_and_empty_no_op() {
        let count =
            crate::test_support::layout(Badge::<()>::count(9).into(), Size::new(400.0, 100.0));
        let long = crate::test_support::layout(
            Badge::<()>::status("a deliberately long compact status value").into(),
            Size::new(400.0, 100.0),
        );
        let empty =
            crate::test_support::layout(Badge::<()>::status("  ").into(), Size::new(400.0, 100.0));

        assert_eq!(count.size(), iced::Size::new(20.0, 20.0));
        assert_eq!(long.size().height, 20.0);
        assert!(long.size().width <= STATUS_MAXIMUM_WIDTH + HORIZONTAL_PADDING * 2.0);
        assert_eq!(empty.size(), iced::Size::ZERO);
    }

    #[test]
    fn real_layout_clamps_to_sub_minimum_hosts_without_overflow() {
        for width in [8.0, 12.0] {
            let node = crate::test_support::layout(
                Badge::<()>::status("Status").into(),
                Size::new(width, HEIGHT),
            );
            assert_eq!(node.size(), Size::new(width, HEIGHT));
            assert!(node.children().iter().all(|child| child.bounds().x >= 0.0));
        }
    }
}
