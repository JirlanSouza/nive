#[cfg(test)]
mod placement;
mod widget;

use iced::{
    border::Radius,
    widget::{container, scrollable},
    Background, Border, Length, Padding,
};

use crate::widgets::overlays::anchored_overlay::scroll::EnsureVisibleHandle;
use crate::{
    theme::{BorderRole, SurfaceRole},
    widgets::scrollable::overlay_scrollbar,
    Element,
};

use self::widget::PopoverWidget;
pub use super::anchored_overlay::{
    PopoverCollision, PopoverFocusPolicy, PopoverPlacement, PopoverWidth,
};

pub(crate) fn take_dismissal_request(tree: &mut iced::advanced::widget::Tree) -> bool {
    widget::take_dismissal_request(tree)
}

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

/// A controlled floating surface anchored to one logical widget.
///
/// The Popover owns its border, eight-pixel radius, shadow, clipping, inset, and
/// vertical overflow viewport. Supply surface-free content instead of wrapping
/// it in another Panel or Scrollable. Iced currently clips arbitrary
/// descendants to the rectangular outer bounds; Nive-owned edge-to-edge lists
/// add their own inset to remain visually contained by the rounded surface.
///
/// Geometry defaults to [`PopoverPlacement::BottomStart`],
/// [`PopoverCollision::FlipAndShift`], a four-pixel anchor gap, an eight-pixel
/// safe viewport, and [`PopoverWidth::Content`]. Placement `Start`/`End` values
/// use physical LTR semantics. Invalid supplied geometry is normalized and the
/// final frame is bounded to finite nonnegative space on the selected side.
///
/// `open` is application-controlled. Escape, an outside primary press, or a
/// FocusFirst traversal exit publishes `on_dismiss` exactly once only when
/// that capability is configured. Without it, the widget manufactures no
/// close state and does not capture an outside press merely to simulate one.
/// Programmatic closure is silent. Focus entry and conditional restoration are
/// governed by [`PopoverFocusPolicy`] through the shared logical-focus root.
///
/// Open, close, placement, and focus visuals move directly to their terminal
/// state; this API does not implement local opacity or translation animation.
/// Retained open/name/focus metadata is preparatory and does not claim native
/// accessibility roles, names, expanded state, relations, or announcements.
///
/// Low-level overlay, geometry, focus, and rendering kernels are intentionally
/// unavailable to application code:
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::popover::PopoverOverlay;
/// ```
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::anchored_overlay::{
///     translated_bounds, AnchoredOverlay, PopoverDismissalCause,
/// };
/// ```
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::popover::widget::{PopoverState, PopoverWidget};
/// ```
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
    ensure_visible: Option<EnsureVisibleHandle>,
    anchor_width_cap: Option<f32>,
    max_height: Option<f32>,
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
            ensure_visible: None,
            anchor_width_cap: None,
            max_height: None,
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

    pub(crate) fn ensure_visible(mut self, handle: EnsureVisibleHandle) -> Self {
        self.ensure_visible = Some(handle);
        self
    }

    pub(crate) fn capped_anchor_width(mut self, cap: f32) -> Self {
        self.width = PopoverWidth::MatchAnchor;
        self.anchor_width_cap = Some(finite_nonnegative(cap));
        self
    }

    pub(crate) fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(finite_nonnegative(max_height));
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let content = surface_with_constraints(
            self.content,
            self.inset,
            self.ensure_visible.as_ref(),
            self.max_height,
        );
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
            ensure_visible: self.ensure_visible,
            anchor_width_cap: self.anchor_width_cap,
        })
    }
}

pub(crate) fn surface_with_ensure_visible<'a, Message>(
    content: Element<'a, Message>,
    inset: PopoverInset,
    ensure_visible: Option<&EnsureVisibleHandle>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    surface_with_constraints(content, inset, ensure_visible, None)
}

fn surface_with_constraints<'a, Message>(
    content: Element<'a, Message>,
    inset: PopoverInset,
    ensure_visible: Option<&EnsureVisibleHandle>,
    max_height: Option<f32>,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let content = container(content)
        .padding(Padding::from(inset.value()))
        .width(Length::Fill);
    let mut viewport = scrollable(content)
        .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
        .width(Length::Shrink)
        .height(Length::Shrink);
    if let Some(handle) = ensure_visible {
        viewport = viewport.id(handle.scrollable());
    }

    let mut surface = container(viewport)
        .style(surface_style)
        .clip(true)
        .width(Length::Shrink)
        .height(Length::Shrink);
    if let Some(max_height) = max_height {
        surface = surface.max_height(max_height);
    }
    surface.into()
}

fn finite_nonnegative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
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

#[cfg(test)]
#[path = "popover/nested_tests.rs"]
mod nested_tests;
