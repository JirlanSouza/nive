use iced::widget::rule;

use crate::theme::BorderRole;
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orientation {
    Horizontal,
    Vertical,
}

pub struct Separator {
    role: BorderRole,
    orientation: Orientation,
}

impl Separator {
    pub fn horizontal() -> Self {
        Self {
            role: BorderRole::Default,
            orientation: Orientation::Horizontal,
        }
    }

    pub fn vertical() -> Self {
        Self {
            role: BorderRole::Default,
            orientation: Orientation::Vertical,
        }
    }

    pub fn role(mut self, role: BorderRole) -> Self {
        self.role = role;
        self
    }

    pub fn subtle(mut self) -> Self {
        self.role = BorderRole::Subtle;
        self
    }

    fn into_rule(self) -> iced::widget::Rule<'static, crate::theme::Theme> {
        let rule = match self.orientation {
            Orientation::Horizontal => iced::widget::rule::horizontal(1),
            Orientation::Vertical => iced::widget::rule::vertical(1),
        };
        rule.style(style(self.role))
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

fn style(role: BorderRole) -> impl Fn(&crate::theme::Theme) -> rule::Style {
    move |theme| {
        let border = theme.border(role);

        rule::Style {
            color: border.color,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        }
    }
}

#[cfg(test)]
mod separator_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn style_uses_app_border_role() {
        let theme = Theme::Dark;
        let style = style(BorderRole::Strong)(&theme);

        assert_eq!(style.color, theme.border(BorderRole::Strong).color);
    }
}
