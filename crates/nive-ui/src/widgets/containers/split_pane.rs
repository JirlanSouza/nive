use iced::{
    widget::{column, container, row, Space},
    Background, Border, Color, Length, Shadow,
};

use crate::theme::{self, BorderRole, SpaceStep, SurfaceRole};
use crate::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitPaneDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SplitPaneMetrics {
    handle_size: f32,
    grip_length: f32,
    grip_thickness: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitPaneConstraints {
    pub leading_min: f32,
    pub trailing_min: f32,
}

impl SplitPaneConstraints {
    pub const fn new(leading_min: f32, trailing_min: f32) -> Self {
        Self {
            leading_min,
            trailing_min,
        }
    }

    pub fn clamp_ratio(self, ratio: f32, available: f32) -> f32 {
        if available <= 0.0 {
            return ratio.clamp(0.05, 0.95);
        }

        let min_total = self.leading_min + self.trailing_min;
        if min_total >= available {
            return (self.leading_min / min_total).clamp(0.05, 0.95);
        }

        let min_ratio = (self.leading_min / available).clamp(0.05, 0.95);
        let max_ratio = (1.0 - self.trailing_min / available).clamp(0.05, 0.95);

        ratio.clamp(min_ratio, max_ratio)
    }
}

pub struct SplitPane<'a, Message> {
    leading: Element<'a, Message>,
    trailing: Element<'a, Message>,
    direction: SplitPaneDirection,
    ratio: f32,
    constraints: SplitPaneConstraints,
    available_length: Option<f32>,
    handle_role: SurfaceRole,
    width: Length,
    height: Length,
}

impl<'a, Message> SplitPane<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(
        leading: impl Into<Element<'a, Message>>,
        trailing: impl Into<Element<'a, Message>>,
    ) -> Self {
        Self {
            leading: leading.into(),
            trailing: trailing.into(),
            direction: SplitPaneDirection::Horizontal,
            ratio: 0.5,
            constraints: SplitPaneConstraints::new(0.0, 0.0),
            available_length: None,
            handle_role: SurfaceRole::Canvas,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    pub fn direction(mut self, direction: SplitPaneDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn horizontal(self) -> Self {
        self.direction(SplitPaneDirection::Horizontal)
    }

    pub fn vertical(self) -> Self {
        self.direction(SplitPaneDirection::Vertical)
    }

    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio;
        self
    }

    pub fn min_sizes(mut self, leading_min: f32, trailing_min: f32) -> Self {
        self.constraints = SplitPaneConstraints::new(leading_min, trailing_min);
        self
    }

    pub fn constraints(mut self, constraints: SplitPaneConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn available_length(mut self, available_length: f32) -> Self {
        self.available_length = Some(available_length);
        self
    }

    pub fn handle_role(mut self, role: SurfaceRole) -> Self {
        self.handle_role = role;
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let ratio = match self.available_length {
            Some(available) => self.constraints.clamp_ratio(self.ratio, available),
            None => normalize_ratio(self.ratio),
        };
        let (leading_portion, trailing_portion) = ratio_to_portions(ratio);

        match self.direction {
            SplitPaneDirection::Horizontal => row![
                pane(self.leading)
                    .width(Length::FillPortion(leading_portion))
                    .height(Length::Fill),
                handle(self.direction, self.handle_role),
                pane(self.trailing)
                    .width(Length::FillPortion(trailing_portion))
                    .height(Length::Fill),
            ]
            .width(self.width)
            .height(self.height)
            .into(),
            SplitPaneDirection::Vertical => column![
                pane(self.leading)
                    .height(Length::FillPortion(leading_portion))
                    .width(Length::Fill),
                handle(self.direction, self.handle_role),
                pane(self.trailing)
                    .height(Length::FillPortion(trailing_portion))
                    .width(Length::Fill),
            ]
            .width(self.width)
            .height(self.height)
            .into(),
        }
    }
}

impl<'a, Message> From<SplitPane<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(split_pane: SplitPane<'a, Message>) -> Self {
        split_pane.into_element()
    }
}

fn pane<'a, Message>(
    content: Element<'a, Message>,
) -> container::Container<'a, Message, crate::theme::Theme>
where
    Message: Clone + 'a,
{
    container(content)
}

fn handle<'a, Message>(direction: SplitPaneDirection, role: SurfaceRole) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let metrics = metrics();
    let grip = match direction {
        SplitPaneDirection::Horizontal => {
            container(Space::new().width(Length::Fixed(metrics.grip_thickness)))
                .style(grip_style())
                .width(Length::Fixed(metrics.grip_thickness))
                .height(Length::Fixed(metrics.grip_length))
        }
        SplitPaneDirection::Vertical => {
            container(Space::new().height(Length::Fixed(metrics.grip_thickness)))
                .style(grip_style())
                .width(Length::Fixed(metrics.grip_length))
                .height(Length::Fixed(metrics.grip_thickness))
        }
    };

    match direction {
        SplitPaneDirection::Horizontal => container(grip)
            .style(handle_style(role))
            .center_x(Length::Fixed(metrics.handle_size))
            .center_y(Length::Fill)
            .into(),
        SplitPaneDirection::Vertical => container(grip)
            .style(handle_style(role))
            .center_x(Length::Fill)
            .center_y(Length::Fixed(metrics.handle_size))
            .into(),
    }
}

fn metrics() -> SplitPaneMetrics {
    SplitPaneMetrics {
        handle_size: theme::space(SpaceStep::Md),
        grip_length: theme::space(SpaceStep::Xxl),
        grip_thickness: 1.0,
    }
}

fn normalize_ratio(ratio: f32) -> f32 {
    ratio.clamp(0.05, 0.95)
}

fn ratio_to_portions(ratio: f32) -> (u16, u16) {
    let ratio = normalize_ratio(ratio);
    let leading = (ratio * 100.0).round().clamp(5.0, 95.0) as u16;
    let trailing = 100 - leading;

    (leading, trailing)
}

fn handle_style(role: SurfaceRole) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let surface = theme.surface(role);

        container::Style {
            text_color: Some(surface.foreground),
            background: Some(Background::Color(surface.background)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

fn grip_style() -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let border = theme.border(BorderRole::Strong);

        container::Style {
            text_color: None,
            background: Some(Background::Color(border.color)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 1.0.into(),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

#[cfg(test)]
mod split_pane_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn ratio_to_portions_clamps_extremes() {
        assert_eq!(ratio_to_portions(-1.0), (5, 95));
        assert_eq!(ratio_to_portions(2.0), (95, 5));
    }

    #[test]
    fn constraints_clamp_ratio_to_minimum_sizes() {
        let constraints = SplitPaneConstraints::new(200.0, 300.0);

        assert_eq!(constraints.clamp_ratio(0.1, 1000.0), 0.2);
        assert_eq!(constraints.clamp_ratio(0.9, 1000.0), 0.7);
    }

    #[test]
    fn handle_style_uses_app_surface_role() {
        let theme = Theme::Dark;
        let style = handle_style(SurfaceRole::Canvas)(&theme);

        assert_eq!(
            background_color(&style),
            theme.surface(SurfaceRole::Canvas).background
        );
    }

    fn background_color(style: &container::Style) -> Color {
        match style.background.as_ref() {
            Some(Background::Color(color)) => *color,
            _ => panic!("Expected color background"),
        }
    }
}
