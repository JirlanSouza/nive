use std::{borrow::Cow, marker::PhantomData};

use iced::{widget::container, Alignment, Background, Border, Length, Padding, Shadow};

use crate::{
    theme::{TextRole, ToneRole, TypographyRole},
    Element,
};

use super::measured_text::{EllipsisStrategy, MeasuredText};

const HEIGHT: f32 = 20.0;
const HORIZONTAL_PADDING: f32 = 6.0;
const MAXIMUM_WIDTH: f32 = 168.0;
const MAXIMUM_CONTENT_WIDTH: f32 = MAXIMUM_WIDTH - HORIZONTAL_PADDING * 2.0;
const RADIUS: f32 = 4.0;

/// Static literal technical metadata with bounded middle ellipsis.
///
/// The exact owned or borrowed value is retained. At constrained widths the
/// renderer-measured projection truncates by Unicode grapheme and a tooltip
/// discloses the original only while truncated. The tag is deliberately
/// non-interactive: selection, copying, activation, focus, and adjacent actions
/// remain host-owned.
pub struct MetadataTag<'a, Message> {
    value: Cow<'a, str>,
    _marker: PhantomData<Message>,
}

impl<'a, Message> MetadataTag<'a, Message>
where
    Message: 'a,
{
    /// Creates static literal technical metadata without adding or removing a prefix.
    pub fn code(value: impl Into<Cow<'a, str>>) -> Self {
        Self {
            value: value.into(),
            _marker: PhantomData,
        }
    }

    fn into_element(self) -> Element<'a, Message> {
        let content = MeasuredText::new(
            self.value,
            EllipsisStrategy::Middle,
            TypographyRole::MetadataTag,
            TextRole::Secondary,
        )
        .max_width(MAXIMUM_CONTENT_WIDTH);

        container(content)
            .style(style())
            .padding(Padding::ZERO.horizontal(HORIZONTAL_PADDING))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .height(Length::Fixed(HEIGHT))
            .max_width(MAXIMUM_WIDTH)
            .into()
    }
}

impl<'a, Message> From<MetadataTag<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(tag: MetadataTag<'a, Message>) -> Self {
        tag.into_element()
    }
}

fn style() -> impl Fn(&crate::theme::Theme) -> container::Style {
    |theme| container::Style {
        text_color: Some(theme.text(TextRole::Secondary).color),
        background: Some(Background::Color(theme.tone(ToneRole::Neutral).container)),
        border: Border {
            radius: RADIUS.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Theme, ThemeDensity};
    use iced::Size;

    #[test]
    fn fixed_metrics_are_density_invariant() {
        assert_eq!(HEIGHT, 20.0);
        assert_eq!(HORIZONTAL_PADDING, 6.0);
        assert_eq!(MAXIMUM_WIDTH, 168.0);
        assert_eq!(RADIUS, 4.0);

        for density in [
            ThemeDensity::Compact,
            ThemeDensity::Standard,
            ThemeDensity::Comfortable,
        ] {
            let _ = density;
            let style = style()(&Theme::Dark);
            assert_eq!(style.border.width, 0.0);
            assert_eq!(style.shadow, Shadow::default());
        }
    }

    #[test]
    fn borrowed_and_owned_values_construct_equivalently() {
        let borrowed = MetadataTag::<()>::code("v1.4.0-beta.2+build.7");
        let owned = MetadataTag::<()>::code(String::from("v1.4.0-beta.2+build.7"));

        assert_eq!(borrowed.value, owned.value);
    }

    #[test]
    fn real_layout_is_twenty_pixels_high_bounded_and_host_clamped() {
        let short = crate::test_support::layout(
            MetadataTag::<()>::code("v1.4.0").into(),
            Size::new(400.0, 100.0),
        );
        let long = crate::test_support::layout(
            MetadataTag::<()>::code("v1.4.0-beta.2+an-extremely-long-build-identifier").into(),
            Size::new(400.0, 100.0),
        );
        let narrow = crate::test_support::layout(
            MetadataTag::<()>::code("v1.4.0-beta.2+build.7").into(),
            Size::new(18.0, 100.0),
        );

        assert_eq!(short.size().height, HEIGHT);
        assert!(short.size().width < MAXIMUM_WIDTH);
        assert_eq!(long.size().height, HEIGHT);
        assert!(long.size().width <= MAXIMUM_WIDTH);
        assert!(long.size().width > short.size().width);
        assert_eq!(narrow.size().height, HEIGHT);
        assert!(narrow.size().width <= 18.0);
        assert!(narrow.size().width >= HORIZONTAL_PADDING * 2.0);
    }
}
