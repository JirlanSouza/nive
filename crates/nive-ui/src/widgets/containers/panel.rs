use iced::{
    widget::{column, container},
    Length, Padding,
};

use crate::theme::{self, surface as theme_surface};
use crate::theme::{gap, GapRole, ShapeSize, SurfaceRole};
use crate::Element;

/// Panel surface with optional header and configurable shape.
pub struct Panel<'a, Message> {
    header: Option<Element<'a, Message>>,
    content: Element<'a, Message>,
    role: SurfaceRole,
    radius: f32,
    padding: Option<Padding>,
    width: Option<iced::Length>,
    height: Option<iced::Length>,
    center: Option<iced::Length>,
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
            padding: None,
            width: None,
            height: None,
            center: None,
        }
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

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
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
        let body = match self.header {
            Some(header) => column![header, self.content]
                .spacing(gap(GapRole::Content))
                .width(Length::Fill)
                .into(),
            None => self.content,
        };

        let mut panel = container(body).style(theme_surface::style_with_radius(
            self.role,
            self.radius.into(),
        ));

        if let Some(padding) = self.padding {
            panel = panel.padding(padding);
        }

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
    fn shape_builders_resolve_panel_radius() {
        let default = Panel::<()>::new(iced::widget::Space::new());
        let square = Panel::<()>::new(iced::widget::Space::new()).square();
        let none = Panel::<()>::new(iced::widget::Space::new()).shape(ShapeSize::None);
        let full = Panel::<()>::new(iced::widget::Space::new()).shape(ShapeSize::Full);

        assert_eq!(default.radius, 0.0);
        assert_eq!(square.radius, none.radius);
        assert_eq!(full.radius, token_radius::FULL);
    }
}
