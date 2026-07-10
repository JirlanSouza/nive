mod overlay;
mod placement;
mod widget;

use crate::Element;

use self::widget::PopoverWidget;

pub use overlay::PopoverOverlay;
pub use placement::translated_bounds;
pub use placement::{PopoverCollision, PopoverPlacement, PopoverWidth};

pub struct Popover<'a, Message> {
    anchor: Element<'a, Message>,
    content: Element<'a, Message>,
    open: bool,
    placement: PopoverPlacement,
    width: PopoverWidth,
    collision: PopoverCollision,
    gap: f32,
    on_dismiss: Option<Message>,
    trap_focus: bool,
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
            gap: 0.0,
            on_dismiss: None,
            trap_focus: false,
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

    pub fn trap_focus(mut self, trap_focus: bool) -> Self {
        self.trap_focus = trap_focus;
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        Element::new(PopoverWidget {
            anchor: self.anchor,
            content: self.content,
            open: self.open,
            placement: self.placement,
            width: self.width,
            collision: self.collision,
            gap: self.gap,
            on_dismiss: self.on_dismiss,
            trap_focus: self.trap_focus,
        })
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

    fn empty_popover() -> Popover<'static, ()> {
        Popover::new(iced::widget::Space::new())
    }
}
