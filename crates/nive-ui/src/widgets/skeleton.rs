use iced::{
    widget::{container, Row, Space},
    Alignment, Background, Border, Color, Length, Padding, Shadow,
};

use crate::theme::{
    self, control_metrics, ControlSize, PaddingRole, ShapeRole, SurfaceRole, TextRole,
};
use crate::{Element, Renderer};

#[derive(Debug, Clone, Copy, PartialEq)]
struct SkeletonMetrics {
    width: Length,
    height: f32,
    radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SkeletonKind {
    Block,
    Rounded,
    TextRow,
}

pub struct Skeleton {
    width: Option<Length>,
    height: Option<Length>,
    radius: Option<f32>,
    size: ControlSize,
    kind: SkeletonKind,
}

pub struct SkeletonControl<'a, Message> {
    content: Row<'a, Message, crate::theme::Theme, Renderer>,
    width: Length,
    size: ControlSize,
}

pub struct SkeletonCard<'a, Message> {
    content: Element<'a, Message>,
    role: SurfaceRole,
    width: Length,
    padding: Padding,
    radius: f32,
}

pub fn block() -> Skeleton {
    Skeleton::new()
}

pub fn rounded() -> Skeleton {
    Skeleton::rounded()
}

pub fn text_row() -> Skeleton {
    Skeleton::text_row()
}

pub fn control<'a, Message>(
    content: Row<'a, Message, crate::theme::Theme, Renderer>,
) -> SkeletonControl<'a, Message>
where
    Message: 'a,
{
    SkeletonControl::new(content)
}

pub fn card<'a, Message>(content: impl Into<Element<'a, Message>>) -> SkeletonCard<'a, Message>
where
    Message: 'a,
{
    SkeletonCard::new(content)
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
            radius: None,
            size: ControlSize::Sm,
            kind: SkeletonKind::Block,
        }
    }

    pub fn rounded() -> Self {
        Self::new().kind(SkeletonKind::Rounded)
    }

    pub fn text_row() -> Self {
        Self::new().kind(SkeletonKind::TextRow)
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }

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

    fn kind(mut self, kind: SkeletonKind) -> Self {
        self.kind = kind;
        self
    }

    fn into_element<'a, Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let metrics = metrics(self.kind, self.size);
        let width = self.width.unwrap_or(metrics.width);
        let height = self.height.unwrap_or(Length::Fixed(metrics.height));
        let radius = self.radius.unwrap_or(metrics.radius);

        container(Space::new().width(width).height(height))
            .style(style(radius))
            .width(width)
            .height(height)
            .into()
    }
}

impl<'a, Message> SkeletonControl<'a, Message>
where
    Message: 'a,
{
    pub fn new(content: Row<'a, Message, crate::theme::Theme, Renderer>) -> Self {
        Self {
            content,
            width: Length::Fill,
            size: ControlSize::Sm,
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

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

    fn into_element(self) -> Element<'a, Message> {
        let metrics = control_metrics(self.size);
        let content = self
            .content
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        container(content)
            .padding(metrics.padding)
            .width(self.width)
            .height(Length::Fixed(metrics.height))
            .align_y(Alignment::Center)
            .into()
    }
}

impl<'a, Message> SkeletonCard<'a, Message>
where
    Message: 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            role: SurfaceRole::Panel,
            width: Length::Fill,
            padding: theme::padding(PaddingRole::Compact),
            radius: theme::active().shape(ShapeRole::Large).radius_value(),
        }
    }

    pub fn role(mut self, role: SurfaceRole) -> Self {
        self.role = role;
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        container(self.content)
            .style(card_style(self.role, self.radius))
            .padding(self.padding)
            .width(self.width)
            .into()
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> From<SkeletonCard<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(card: SkeletonCard<'a, Message>) -> Self {
        card.into_element()
    }
}

impl<'a, Message> From<Skeleton> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(skeleton: Skeleton) -> Self {
        skeleton.into_element()
    }
}

impl<'a, Message> From<SkeletonControl<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(control: SkeletonControl<'a, Message>) -> Self {
        control.into_element()
    }
}

fn metrics(kind: SkeletonKind, size: ControlSize) -> SkeletonMetrics {
    let control = control_metrics(size);

    match kind {
        SkeletonKind::Block => SkeletonMetrics {
            width: Length::Fill,
            height: control.height,
            radius: control.radius,
        },
        SkeletonKind::Rounded => {
            let side = leading_size(size);

            SkeletonMetrics {
                width: Length::Fixed(side),
                height: side,
                radius: side / 2.0,
            }
        }
        SkeletonKind::TextRow => SkeletonMetrics {
            width: Length::Fill,
            height: control.font_size,
            radius: control_metrics(ControlSize::Xs).radius,
        },
    }
}

fn leading_size(size: ControlSize) -> f32 {
    match size {
        ControlSize::Xs => 8.0,
        ControlSize::Sm | ControlSize::Md => 10.0,
        ControlSize::Lg => 12.0,
    }
}

fn style(radius: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| container::Style {
        background: Some(Background::Color(skeleton_color(*theme))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius.into(),
        },
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

fn card_style(role: SurfaceRole, radius: f32) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let surface = theme.surface(role);

        container::Style {
            text_color: Some(surface.foreground),
            background: Some(Background::Color(surface.background)),
            border: Border {
                color: surface.border.color,
                width: surface.border.width,
                radius: radius.into(),
            },
            shadow: surface.shadow,
            ..container::Style::default()
        }
    }
}

fn skeleton_color(theme: crate::theme::Theme) -> Color {
    let alpha = if theme.is_dark() { 0.18 } else { 0.12 };

    theme.text(TextRole::Muted).color.scale_alpha(alpha)
}

#[cfg(test)]
mod skeleton_tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn block_metrics_use_control_size() {
        let metrics = metrics(SkeletonKind::Block, ControlSize::Sm);
        let control = control_metrics(ControlSize::Sm);

        assert_eq!(metrics.width, Length::Fill);
        assert_eq!(metrics.height, control.height);
        assert_eq!(metrics.radius, control.radius);
    }

    #[test]
    fn rounded_metrics_use_leading_size() {
        let metrics = metrics(SkeletonKind::Rounded, ControlSize::Sm);

        assert_eq!(metrics.width, Length::Fixed(10.0));
        assert_eq!(metrics.height, 10.0);
        assert_eq!(metrics.radius, 5.0);
    }

    #[test]
    fn text_row_metrics_use_control_text_height() {
        let metrics = metrics(SkeletonKind::TextRow, ControlSize::Sm);
        let control = control_metrics(ControlSize::Sm);

        assert_eq!(metrics.width, Length::Fill);
        assert_eq!(metrics.height, control.font_size);
    }

    #[test]
    fn style_uses_skeleton_color() {
        let theme = Theme::Dark;
        let style = style(4.0)(&theme);

        assert_eq!(
            style.background,
            Some(Background::Color(skeleton_color(theme)))
        );
        assert_eq!(style.border.color, Color::TRANSPARENT);
    }

    #[test]
    fn card_style_uses_surface_role() {
        let theme = Theme::Dark;
        let style = card_style(SurfaceRole::Panel, 8.0)(&theme);
        let surface = theme.surface(SurfaceRole::Panel);

        assert_eq!(
            style.background,
            Some(Background::Color(surface.background))
        );
        assert_eq!(style.border.color, surface.border.color);
    }
}
