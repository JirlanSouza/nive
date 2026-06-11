use iced::{
    advanced::{
        layout, mouse, renderer,
        svg::{self, Renderer as _},
        widget::Tree,
        Layout, Widget,
    },
    widget::svg::Handle,
    Color, Length, Radians, Rectangle, Size,
};

use crate::Element;

include!("icon.generated.rs");

pub fn new(icon: AppIcon) -> Icon {
    Icon::new(icon)
}

pub(crate) fn handle(icon: AppIcon) -> Handle {
    Handle::from_memory(icon.svg_bytes())
}

pub struct Icon {
    icon: AppIcon,
    size: f32,
    color: Option<Color>,
    rotation: Radians,
}

impl Icon {
    const DEFAULT_SIZE: f32 = 16.0;

    pub fn new(icon: AppIcon) -> Self {
        Self {
            icon,
            size: Self::DEFAULT_SIZE,
            color: None,
            rotation: Radians(0.0),
        }
    }

    pub fn xs(self) -> Self {
        self.size(12.0)
    }

    pub fn sm(self) -> Self {
        self.size(14.0)
    }

    pub fn md(self) -> Self {
        self.size(16.0)
    }

    pub fn lg(self) -> Self {
        self.size(20.0)
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn color_maybe(mut self, color: Option<Color>) -> Self {
        self.color = color;
        self
    }

    pub fn rotation(mut self, rotation: impl Into<Radians>) -> Self {
        self.rotation = rotation.into();
        self
    }

    fn handle(&self) -> Handle {
        handle(self.icon)
    }
}

impl<Message, Theme> Widget<Message, Theme, iced::Renderer> for Icon {
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fixed(self.size),
            height: Length::Fixed(self.size),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.resolve(
            Length::Fixed(self.size),
            Length::Fixed(self.size),
            Size::ZERO,
        ))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        renderer.draw_svg(
            svg::Svg {
                handle: self.handle(),
                color: Some(self.color.unwrap_or(inherited_style.text_color)),
                rotation: self.rotation,
                opacity: 1.0,
            },
            layout.bounds(),
            *viewport,
        );
    }
}

impl<'a, Message> From<Icon> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(icon: Icon) -> Self {
        Element::new(icon)
    }
}
