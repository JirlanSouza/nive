use iced::widget::rule;

use crate::theme::BorderRole;
use crate::Element;

/// Neutral visual strength for a decorative separator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SeparatorStrength {
    /// Quiet structural seam.
    #[default]
    Subtle,
    /// Stronger neutral boundary between semantic sections.
    Section,
}

/// Visible extent along a separator's host axis.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SeparatorExtent {
    /// Fill the complete host axis.
    #[default]
    Full,
    /// Leave independent logical inset at the leading and trailing ends.
    ///
    /// Until logical-direction plumbing lands, leading/trailing map to
    /// left/right for horizontal rules and top/bottom for vertical rules.
    Inset {
        /// Leading logical-pixel inset.
        leading: f32,
        /// Trailing logical-pixel inset.
        trailing: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    Horizontal,
    Vertical,
}

/// One-pixel square, snapped, noninteractive decorative rule.
pub struct Separator {
    strength: SeparatorStrength,
    extent: SeparatorExtent,
    orientation: Orientation,
}

impl Separator {
    pub fn horizontal() -> Self {
        Self {
            strength: SeparatorStrength::Subtle,
            extent: SeparatorExtent::Full,
            orientation: Orientation::Horizontal,
        }
    }

    pub fn vertical() -> Self {
        Self {
            orientation: Orientation::Vertical,
            ..Self::horizontal()
        }
    }

    pub fn strength(mut self, strength: SeparatorStrength) -> Self {
        self.strength = strength;
        self
    }

    pub fn extent(mut self, extent: SeparatorExtent) -> Self {
        self.extent = extent;
        self
    }

    pub fn subtle(self) -> Self {
        self.strength(SeparatorStrength::Subtle)
    }

    pub fn section(self) -> Self {
        self.strength(SeparatorStrength::Section)
    }

    pub fn inset(self, leading: f32, trailing: f32) -> Self {
        self.extent(SeparatorExtent::Inset { leading, trailing })
    }

    /// Aligns a horizontal rule with a text column after leading icon/avatar content.
    pub fn text_column(self, leading: f32) -> Self {
        self.inset(leading, 0.0)
    }

    fn into_rule(self) -> iced::widget::Rule<'static, crate::theme::Theme> {
        let fill_mode = fill_mode(self.extent);
        let rule = match self.orientation {
            Orientation::Horizontal => iced::widget::rule::horizontal(1),
            Orientation::Vertical => iced::widget::rule::vertical(1),
        };
        rule.style(style(self.strength, fill_mode))
    }
}

impl<'a, Message> From<Separator> for Element<'a, Message>
where
    Message: 'a + 'static,
{
    fn from(separator: Separator) -> Self {
        separator.into_rule().into()
    }
}

fn fill_mode(extent: SeparatorExtent) -> rule::FillMode {
    match extent {
        SeparatorExtent::Full => rule::FillMode::Full,
        SeparatorExtent::Inset { leading, trailing } => {
            rule::FillMode::AsymmetricPadding(inset_pixels(leading), inset_pixels(trailing))
        }
    }
}

fn inset_pixels(value: f32) -> u16 {
    if value.is_finite() {
        value.max(0.0).round().min(u16::MAX as f32) as u16
    } else {
        0
    }
}

fn style(
    strength: SeparatorStrength,
    fill_mode: rule::FillMode,
) -> impl Fn(&crate::theme::Theme) -> rule::Style {
    move |theme| {
        let role = match strength {
            SeparatorStrength::Subtle => BorderRole::Subtle,
            SeparatorStrength::Section => BorderRole::Default,
        };
        let border = theme.border(role);

        rule::Style {
            color: border.color,
            radius: 0.0.into(),
            fill_mode,
            snap: true,
        }
    }
}

#[cfg(test)]
mod separator_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn strengths_map_only_to_neutral_border_roles() {
        let theme = Theme::Dark;
        let subtle = style(SeparatorStrength::Subtle, rule::FillMode::Full)(&theme);
        let section = style(SeparatorStrength::Section, rule::FillMode::Full)(&theme);

        assert_eq!(subtle.color, theme.border(BorderRole::Subtle).color);
        assert_eq!(section.color, theme.border(BorderRole::Default).color);
        assert_eq!(subtle.radius, 0.0.into());
        assert!(subtle.snap);
    }

    #[test]
    fn extents_map_and_clamp_deterministically() {
        assert_eq!(fill_mode(SeparatorExtent::Full), rule::FillMode::Full);
        assert_eq!(
            fill_mode(SeparatorExtent::Inset {
                leading: 12.4,
                trailing: 8.6,
            }),
            rule::FillMode::AsymmetricPadding(12, 9)
        );
        assert_eq!(inset_pixels(-1.0), 0);
        assert_eq!(inset_pixels(f32::NAN), 0);
        assert_eq!(rule::FillMode::AsymmetricPadding(8, 8).fill(10.0).1, 0.0);
    }

    #[test]
    fn text_column_is_a_leading_only_inset() {
        let separator = Separator::horizontal().text_column(24.0);

        assert_eq!(
            separator.extent,
            SeparatorExtent::Inset {
                leading: 24.0,
                trailing: 0.0,
            }
        );
    }
}
