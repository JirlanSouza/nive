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
mod popover_tests {
    use super::*;
    use crate::test_support::WidgetHarness;
    use iced::{
        keyboard::{self, key},
        mouse, touch, Event, Point, Size,
    };

    #[test]
    fn width_shortcuts_map_to_popover_width_variants() {
        assert_eq!(
            empty_popover().width_px(240.0).width,
            PopoverWidth::Fixed(240.0)
        );
        assert_eq!(
            empty_popover().match_anchor_width().width,
            PopoverWidth::MatchAnchor
        );
        assert_eq!(
            empty_popover().at_least_anchor_width().width,
            PopoverWidth::AtLeastAnchor
        );
        assert_eq!(empty_popover().content_width().width, PopoverWidth::Content);
    }

    #[test]
    fn defaults_own_standard_surface_geometry_and_retain_anchor_focus() {
        let popover = empty_popover();

        assert_eq!(popover.gap, 4.0);
        assert_eq!(popover.inset, PopoverInset::Standard);
        assert_eq!(popover.focus_policy, PopoverFocusPolicy::RetainAnchor);
    }

    #[test]
    fn inset_values_are_exact() {
        assert_eq!(PopoverInset::Standard.value(), 12.0);
        assert_eq!(PopoverInset::Compact.value(), 8.0);
        assert_eq!(PopoverInset::EdgeToEdge.value(), 0.0);
    }

    #[test]
    fn surface_style_owns_one_pixel_eight_pixel_radius_frame() {
        let style = surface_style(&crate::theme::Theme::Dark);

        assert_eq!(style.border.width, 1.0);
        assert_eq!(style.border.radius, Radius::new(8.0));
    }

    #[test]
    fn escape_requests_one_controlled_dismissal() {
        let mut harness = popover_harness(Some("dismiss"));
        let result = harness
            .update_overlay(key_pressed(key::Named::Escape, key::Code::Escape))
            .expect("open Popover overlay");

        assert_eq!(result.messages, vec!["dismiss"]);
        assert!(result.captured);

        let repeated = harness
            .update_overlay(key_pressed(key::Named::Escape, key::Code::Escape))
            .expect("open Popover overlay");
        assert!(repeated.messages.is_empty());
        assert!(repeated.captured);
    }

    #[test]
    fn outside_mouse_and_touch_request_one_dismissal() {
        let mut mouse_harness = popover_harness(Some("mouse"));
        mouse_harness.set_cursor(Point::new(300.0, 180.0));
        let mouse = mouse_harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )))
            .expect("open Popover overlay");
        assert_eq!(mouse.messages, vec!["mouse"]);
        assert!(mouse.captured);

        let mut touch_harness = popover_harness(Some("touch"));
        let touch = touch_harness
            .update_overlay(Event::Touch(touch::Event::FingerPressed {
                id: touch::Finger(1),
                position: Point::new(300.0, 180.0),
            }))
            .expect("open Popover overlay");
        assert_eq!(touch.messages, vec!["touch"]);
        assert!(touch.captured);
    }

    #[test]
    fn callback_absence_does_not_capture_escape_or_outside_press() {
        let mut harness = popover_harness(None);
        let escape = harness
            .update_overlay(key_pressed(key::Named::Escape, key::Code::Escape))
            .expect("open Popover overlay");
        assert!(escape.messages.is_empty());
        assert!(!escape.captured);

        harness.set_cursor(Point::new(300.0, 180.0));
        let outside = harness
            .update_overlay(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )))
            .expect("open Popover overlay");
        assert!(outside.messages.is_empty());
        assert!(!outside.captured);
    }

    fn popover_harness(message: Option<&'static str>) -> WidgetHarness<'static, &'static str> {
        let anchor = iced::widget::Space::new().width(40).height(24);
        let content = iced::widget::Space::new().width(120).height(60);
        let popover = Popover::new(anchor)
            .content(content)
            .open(true)
            .on_dismiss_maybe(message);
        WidgetHarness::new(popover.into(), Size::new(320.0, 200.0))
    }

    fn key_pressed(named: key::Named, code: key::Code) -> Event {
        let key = keyboard::Key::Named(named);
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: key::Physical::Code(code),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::NONE,
            text: None,
            repeat: false,
        })
    }

    fn empty_popover() -> Popover<'static, ()> {
        Popover::new(iced::widget::Space::new())
    }
}
