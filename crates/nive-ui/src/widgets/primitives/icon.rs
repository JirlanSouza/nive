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

use crate::icons::{IconGlyph, IconRef, IconRole, IconSource};
use crate::Element;
use crate::Theme;

/// Semantic size scale for standalone icons.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum IconSize {
    /// 12 px.
    Xs,
    /// 14 px.
    Sm,
    /// 16 px, the default.
    #[default]
    Md,
    /// 20 px.
    Lg,
    /// 24 px.
    Xl,
}

impl IconSize {
    /// Resolves the semantic size to physical pixels.
    pub const fn pixels(self) -> f32 {
        match self {
            Self::Xs => 12.0,
            Self::Sm => 14.0,
            Self::Md => 16.0,
            Self::Lg => 20.0,
            Self::Xl => 24.0,
        }
    }
}

/// Static icon rotation restricted to quarter turns.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    /// 0°.
    #[default]
    None,
    /// 90° clockwise.
    Quarter,
    /// 180°.
    Half,
    /// 270° clockwise.
    ThreeQuarter,
}

impl Rotation {
    const fn radians(self) -> Radians {
        match self {
            Self::None => Radians(0.0),
            Self::Quarter => Radians(std::f32::consts::FRAC_PI_2),
            Self::Half => Radians(std::f32::consts::PI),
            Self::ThreeQuarter => Radians(std::f32::consts::PI * 1.5),
        }
    }
}

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

pub fn reference(icon: IconRef) -> Icon {
    Icon::reference(icon)
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

/// Decorative, monochrome icon primitive.
///
/// Icons inherit the host text color by default and are not independent focus
/// or accessibility targets. The hosting control owns the accessible name and
/// semantic meaning. Use a dedicated SVG/brand path for multicolor marks.
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

    pub fn reference(icon: IconRef) -> Self {
        match icon {
            IconRef::Role(role) => Self::role(role),
            IconRef::Glyph(glyph) => Self::glyph(glyph),
        }
    }
}

impl<S> Icon<S> {
    fn from_kind(icon: IconKind<S>) -> Self {
        Self {
            icon,
            size: IconSize::default().pixels(),
            color: None,
            rotation: Radians(0.0),
        }
    }

    pub fn xs(self) -> Self {
        self.size(IconSize::Xs)
    }

    pub fn sm(self) -> Self {
        self.size(IconSize::Sm)
    }

    pub fn md(self) -> Self {
        self.size(IconSize::Md)
    }

    pub fn lg(self) -> Self {
        self.size(IconSize::Lg)
    }

    pub fn xl(self) -> Self {
        self.size(IconSize::Xl)
    }

    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size.pixels();
        self
    }

    /// Sets a non-semantic pixel size.
    ///
    /// Prefer [`Self::size`] for standalone icons. Control-owned icons use
    /// this escape hatch with their resolved `control.icon_size` metric.
    pub fn custom_size(mut self, size: f32) -> Self {
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

    /// Applies a static quarter-turn rotation.
    pub fn rotation(mut self, rotation: Rotation) -> Self {
        self.rotation = rotation.radians();
        self
    }

    /// Applies a continuous rotation produced by an animation.
    pub fn animated_rotation(mut self, rotation: Radians) -> Self {
        self.rotation = rotation;
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

#[cfg(test)]
mod icon_tests {
    use super::*;
    use crate::icons::IconRole;

    #[test]
    fn semantic_sizes_and_builders_map_to_the_public_scale() {
        assert_eq!(IconSize::default(), IconSize::Md);
        assert_eq!(IconSize::Xl.pixels(), 24.0);
        assert_eq!(Icon::role(IconRole::EditFind).size, 16.0);
        assert_eq!(Icon::role(IconRole::EditFind).xs().size, 12.0);
        assert_eq!(Icon::role(IconRole::EditFind).sm().size, 14.0);
        assert_eq!(Icon::role(IconRole::EditFind).md().size, 16.0);
        assert_eq!(Icon::role(IconRole::EditFind).lg().size, 20.0);
        assert_eq!(Icon::role(IconRole::EditFind).xl().size, 24.0);
    }

    #[test]
    fn custom_size_is_the_raw_pixel_escape_hatch() {
        assert_eq!(Icon::role(IconRole::EditFind).custom_size(18.0).size, 18.0);
    }

    #[test]
    fn static_rotations_map_to_quarter_turns() {
        assert_eq!(Rotation::None.radians(), Radians(0.0));
        assert_eq!(
            Rotation::Quarter.radians(),
            Radians(std::f32::consts::FRAC_PI_2)
        );
        assert_eq!(Rotation::Half.radians(), Radians(std::f32::consts::PI));
        assert_eq!(
            Rotation::ThreeQuarter.radians(),
            Radians(std::f32::consts::PI * 1.5)
        );
    }
}
