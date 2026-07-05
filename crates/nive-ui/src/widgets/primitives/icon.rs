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

use crate::icons::{IconGlyph, IconRole, IconSource};
use crate::Element;
use crate::Theme;

pub fn new<S>(icon: S) -> Icon<S>
where
    S: IconSource,
{
    Icon::new(icon)
}

pub fn role(role: IconRole) -> Icon {
    Icon::role(role)
}

pub fn symbol<S>(symbol: S) -> Icon<S>
where
    S: IconSource,
{
    Icon::symbol(symbol)
}

pub fn glyph(glyph: IconGlyph) -> Icon {
    Icon::glyph(glyph)
}

pub(crate) fn handle(glyph: IconGlyph) -> Handle {
    Handle::from_memory(glyph.svg_bytes())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IconKind<S> {
    Role(IconRole),
    Source(S),
    Glyph(IconGlyph),
}

pub struct Icon<S = IconGlyph> {
    icon: IconKind<S>,
    size: f32,
    color: Option<Color>,
    rotation: Radians,
}

impl<S> Icon<S>
where
    S: IconSource,
{
    pub fn new(icon: S) -> Self {
        Self::from_kind(IconKind::Source(icon))
    }

    pub fn symbol(symbol: S) -> Self {
        Self::new(symbol)
    }
}

impl Icon {
    pub fn role(role: IconRole) -> Self {
        Self::from_kind(IconKind::Role(role))
    }

    pub fn glyph(glyph: IconGlyph) -> Self {
        Self::from_kind(IconKind::Glyph(glyph))
    }
}

impl<S> Icon<S> {
    const DEFAULT_SIZE: f32 = 16.0;

    fn from_kind(icon: IconKind<S>) -> Self {
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

    fn resolved_glyph(&self, theme: Theme) -> IconGlyph
    where
        S: IconSource,
    {
        match self.icon {
            IconKind::Role(role) => theme.icon(role),
            IconKind::Source(source) => IconGlyph::from_source(source),
            IconKind::Glyph(glyph) => glyph,
        }
    }

    fn handle(&self, theme: Theme) -> Handle
    where
        S: IconSource,
    {
        handle(self.resolved_glyph(theme))
    }
}

impl<Message, S> Widget<Message, Theme, iced::Renderer> for Icon<S>
where
    S: IconSource,
{
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
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        renderer.draw_svg(
            svg::Svg {
                handle: self.handle(*theme),
                color: Some(self.color.unwrap_or(inherited_style.text_color)),
                rotation: self.rotation,
                opacity: 1.0,
            },
            layout.bounds(),
            *viewport,
        );
    }
}

impl<'a, Message, S> From<Icon<S>> for Element<'a, Message>
where
    Message: 'a,
    S: IconSource,
{
    fn from(icon: Icon<S>) -> Self {
        Element::new(icon)
    }
}
