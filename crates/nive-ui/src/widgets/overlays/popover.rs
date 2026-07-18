mod overlay;
mod placement;
mod widget;

use iced::{
    border::Radius,
    widget::{container, scrollable},
    Background, Border, Length, Padding,
};

use crate::{
    theme::{BorderRole, SurfaceRole},
    widgets::scrollable::overlay_scrollbar,
    Element,
};

use self::widget::PopoverWidget;

pub(crate) use overlay::PopoverOverlay;
pub(crate) use placement::translated_bounds;
pub use placement::{PopoverCollision, PopoverPlacement, PopoverWidth};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Padding owned by the canonical Popover surface.
pub enum PopoverInset {
    /// Twelve pixels on every side. This is the default.
    #[default]
    Standard,
    /// Eight pixels on every side for denser content.
    Compact,
    /// No surface padding, for content such as canonical menu rows.
    EdgeToEdge,
}

impl PopoverInset {
    const fn value(self) -> f32 {
        match self {
            Self::Standard => 12.0,
            Self::Compact => 8.0,
            Self::EdgeToEdge => 0.0,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// How keyboard focus behaves when a Popover opens.
pub enum PopoverFocusPolicy {
    /// Keep focus on the anchor. This is the default.
    #[default]
    RetainAnchor,
    /// Focus the first focusable descendant and allow ordinary Tab traversal to leave.
    FocusFirst,
    /// Focus the first focusable descendant and cycle Tab traversal inside the Popover.
    Trap,
}

/// A controlled floating surface anchored to one logical widget.
///
/// The Popover owns its border, eight-pixel radius, shadow, clipping, inset, and
/// vertical overflow viewport. Supply surface-free content instead of wrapping
/// it in another Panel or Scrollable.
pub struct Popover<'a, Message> {
    anchor: Element<'a, Message>,
    content: Element<'a, Message>,
    open: bool,
    placement: PopoverPlacement,
    width: PopoverWidth,
    collision: PopoverCollision,
    gap: f32,
    on_dismiss: Option<Message>,
    inset: PopoverInset,
    focus_policy: PopoverFocusPolicy,
}

impl<'a, Message> Popover<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(anchor: impl Into<Element<'a, Message>>) -> Self {
        Self {
            anchor: anchor.into(),
            content: iced::widget::Space::new().into(),
            open: false,
            placement: PopoverPlacement::default(),
            width: PopoverWidth::default(),
            collision: PopoverCollision::default(),
            gap: 4.0,
            on_dismiss: None,
            inset: PopoverInset::default(),
            focus_policy: PopoverFocusPolicy::default(),
        }
    }

    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content = content.into();
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn width(mut self, width: PopoverWidth) -> Self {
        self.width = width;
        self
    }

    pub fn width_px(self, width: f32) -> Self {
        self.width(PopoverWidth::Fixed(width))
    }

    pub fn match_anchor_width(self) -> Self {
        self.width(PopoverWidth::MatchAnchor)
    }

    pub fn at_least_anchor_width(self) -> Self {
        self.width(PopoverWidth::AtLeastAnchor)
    }

    pub fn content_width(self) -> Self {
        self.width(PopoverWidth::Content)
    }

    pub fn collision(mut self, collision: PopoverCollision) -> Self {
        self.collision = collision;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }

    pub fn on_dismiss_maybe(mut self, message: Option<Message>) -> Self {
        self.on_dismiss = message;
        self
    }

    pub fn inset(mut self, inset: PopoverInset) -> Self {
        self.inset = inset;
        self
    }

    pub fn focus_policy(mut self, focus_policy: PopoverFocusPolicy) -> Self {
        self.focus_policy = focus_policy;
        self
    }

    #[deprecated(note = "use focus_policy(PopoverFocusPolicy::Trap)")]
    pub fn trap_focus(self, trap_focus: bool) -> Self {
        self.focus_policy(if trap_focus {
            PopoverFocusPolicy::Trap
        } else {
            PopoverFocusPolicy::RetainAnchor
        })
    }

    fn into_element(self) -> Element<'a, Message> {
        let content = surface(self.content, self.inset);
        Element::new(PopoverWidget {
            anchor: self.anchor,
            content,
            open: self.open,
            placement: self.placement,
            width: self.width,
            collision: self.collision,
            gap: self.gap,
            on_dismiss: self.on_dismiss,
            focus_policy: self.focus_policy,
        })
    }
}

fn surface<'a, Message>(content: Element<'a, Message>, inset: PopoverInset) -> Element<'a, Message>
where
    Message: 'a,
{
    let content = container(content)
        .padding(Padding::from(inset.value()))
        .width(Length::Fill);
    let viewport = scrollable(content)
        .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
        .width(Length::Shrink)
        .height(Length::Shrink);

    container(viewport)
        .style(surface_style)
        .clip(true)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .into()
}

fn surface_style(theme: &crate::theme::Theme) -> container::Style {
    let theme = *theme;
    let surface = theme.surface(SurfaceRole::Popover);
    let perimeter = theme.border(BorderRole::Subtle);

    container::Style {
        text_color: Some(surface.foreground),
        background: Some(Background::Color(surface.background)),
        border: Border {
            color: perimeter.color,
            width: 1.0,
            radius: Radius::new(8.0),
        },
        shadow: surface.shadow,
        ..container::Style::default()
    }
}

impl<'a, Message> From<Popover<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(popover: Popover<'a, Message>) -> Self {
        popover.into_element()
    }
}

#[cfg(test)]
#[path = "popover/basic_tests.rs"]
mod basic_tests;

#[cfg(test)]
#[path = "popover/focus_tests.rs"]
mod focus_tests;
