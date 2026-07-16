use iced::{
    widget::{column, container, rule, stack},
    Length, Padding,
};

use crate::theme::{self, surface as theme_surface};
use crate::theme::{BorderRole, ShapeSize, SurfaceRole};
use crate::Element;

/// Panel surface with optional header and configurable shape.
pub struct Panel<'a, Message> {
    header: Option<Element<'a, Message>>,
    content: Element<'a, Message>,
    role: SurfaceRole,
    radius: f32,
    body_padding: Option<Padding>,
    width: Option<iced::Length>,
    height: Option<iced::Length>,
    center: Option<iced::Length>,
    border: bool,
}

impl<'a, Message> Panel<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            header: None,
            content: content.into(),
            role: SurfaceRole::Panel,
            radius: theme::active().shape(ShapeSize::None).radius_value(),
            body_padding: None,
            width: None,
            height: None,
            center: None,
            border: false,
        }
    }

    /// Opts into an explicit border. Surfaces render fill + shadow only by
    /// default; a panel asks for a border explicitly when it needs one.
    pub fn bordered(mut self) -> Self {
        self.border = true;
        self
    }

    pub fn header(mut self, header: impl Into<Element<'a, Message>>) -> Self {
        self.header = Some(header.into());
        self
    }

    pub fn role(mut self, role: SurfaceRole) -> Self {
        self.role = role;
        self
    }

    /// Sets the panel shape from the theme scale.
    pub fn shape(mut self, shape: ShapeSize) -> Self {
        self.radius = theme::active().shape(shape).radius_value();
        self
    }

    pub fn shape_xs(self) -> Self {
        self.shape(ShapeSize::Xs)
    }

    pub fn shape_sm(self) -> Self {
        self.shape(ShapeSize::Sm)
    }

    pub fn shape_md(self) -> Self {
        self.shape(ShapeSize::Md)
    }

    pub fn shape_lg(self) -> Self {
        self.shape(ShapeSize::Lg)
    }

    pub fn shape_xl(self) -> Self {
        self.shape(ShapeSize::Xl)
    }

    pub fn shape_xxl(self) -> Self {
        self.shape(ShapeSize::Xxl)
    }

    /// Sets square corners, equivalent to `shape(ShapeSize::None)`.
    pub fn square(self) -> Self {
        self.shape(ShapeSize::None)
    }

    /// Sets a raw radius in pixels.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Insets only the panel body, leaving the header and seam unchanged.
    pub fn body_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.body_padding = Some(padding.into());
        self
    }

    /// Insets only the panel body.
    ///
    /// This previously behaved as outer panel padding. Use
    /// [`Panel::body_padding`] and let headers own their own inset.
    #[deprecated(since = "0.1.0", note = "use Panel::body_padding")]
    pub fn padding(self, padding: impl Into<Padding>) -> Self {
        self.body_padding(padding)
    }

    crate::impl_layout_builders!(
        width_opt,
        height_opt,
        fill_width_opt,
        fill_height_opt,
        fill_opt,
        shrink_width_opt
    );

    pub fn center(mut self, length: impl Into<iced::Length>) -> Self {
        self.center = Some(length.into());
        self
    }

    fn into_container(self) -> container::Container<'a, Message, crate::theme::Theme> {
        let body_height = match self.height {
            Some(Length::Fixed(_) | Length::Fill | Length::FillPortion(_)) => Length::Fill,
            Some(Length::Shrink) | None => Length::Shrink,
        };
        let mut content = container(self.content)
            .width(Length::Fill)
            .height(body_height)
            .clip(true);
        if let Some(padding) = self.body_padding {
            content = content.padding(padding);
        }

        let body: Element<'a, Message> = match self.header {
            Some(header) => {
                let seam = rule::horizontal(1).style(header_seam_style);
                let body = stack![content, seam]
                    .width(Length::Fill)
                    .height(body_height);
                column![header, body]
                    .spacing(0.0)
                    .width(Length::Fill)
                    .height(body_height)
                    .into()
            }
            None => content.into(),
        };

        let style: Box<dyn Fn(&crate::theme::Theme) -> container::Style> = if self.border {
            Box::new(theme_surface::style_with_border(
                self.role,
                self.radius.into(),
                BorderRole::Default,
            ))
        } else {
            Box::new(theme_surface::style_with_radius(
                self.role,
                self.radius.into(),
            ))
        };
        let mut panel = container(body).style(style).clip(true);

        if let Some(width) = self.width {
            panel = panel.width(width);
        }

        if let Some(height) = self.height {
            panel = panel.height(height);
        }

        if let Some(length) = self.center {
            panel = panel.center(length);
        }

        panel
    }
}

fn header_seam_style(theme: &crate::theme::Theme) -> rule::Style {
    rule::Style {
        color: theme.border(BorderRole::Subtle).color,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

impl<'a, Message> From<Panel<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(panel: Panel<'a, Message>) -> Self {
        panel.into_container().into()
    }
}

#[cfg(test)]
mod panel_tests {
    use super::*;
    use crate::tokens::radius as token_radius;

    #[test]
    fn border_is_opt_in() {
        let default = Panel::<()>::new(iced::widget::Space::new());
        let bordered = Panel::<()>::new(iced::widget::Space::new()).bordered();

        assert!(!default.border);
        assert!(bordered.border);
    }

    #[test]
    fn shape_builders_resolve_panel_radius() {
        let default = Panel::<()>::new(iced::widget::Space::new());
        let square = Panel::<()>::new(iced::widget::Space::new()).square();
        let none = Panel::<()>::new(iced::widget::Space::new()).shape(ShapeSize::None);
        let full = Panel::<()>::new(iced::widget::Space::new()).shape(ShapeSize::Full);

        assert_eq!(default.radius, 0.0);
        assert_eq!(square.radius, none.radius);
        assert_eq!(full.radius, token_radius::FULL);
    }

    #[test]
    fn body_padding_is_independent_from_header() {
        let panel = Panel::<()>::new(iced::widget::Space::new())
            .header(iced::widget::Space::new())
            .body_padding(12);

        assert_eq!(panel.body_padding, Some(Padding::new(12.0)));
        assert!(panel.header.is_some());
    }

    #[test]
    fn shrink_height_panel_preserves_intrinsic_body_height() {
        let node = crate::test_support::layout(
            Panel::<()>::new(iced::widget::text("Visible body"))
                .body_padding(14)
                .into(),
            iced::Size::new(300.0, 300.0),
        );

        assert!(node.size().height > 28.0);
        assert!(node.size().height < 300.0);
    }

    #[test]
    fn explicitly_shrink_height_panel_preserves_intrinsic_body_height() {
        let node = crate::test_support::layout(
            Panel::<()>::new(iced::widget::text("Visible body"))
                .body_padding(14)
                .height(Length::Shrink)
                .into(),
            iced::Size::new(300.0, 300.0),
        );

        assert!(node.size().height > 28.0);
        assert!(node.size().height < 300.0);
    }
}
