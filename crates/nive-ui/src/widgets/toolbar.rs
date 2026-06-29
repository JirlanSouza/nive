mod action;
mod action_group;
mod group;
mod separator;
mod style;

use iced::{
    widget::{container, Row},
    Alignment, Length, Padding,
};

use crate::theme::{ControlSize, SurfaceRole};
use crate::Element;

use self::style as theme_toolbar;

pub use action::ToolbarAction;
pub use action_group::ActionGroup;
pub use group::ToolbarGroup;

pub struct Toolbar<'a, Message> {
    groups: Vec<ToolbarGroup<'a, Message>>,
    size: ControlSize,
    role: SurfaceRole,
    width: Option<Length>,
}

impl<'a, Message> Toolbar<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            size: ControlSize::Sm,
            role: SurfaceRole::Chrome,
            width: None,
        }
    }

    pub fn push(mut self, group: ToolbarGroup<'a, Message>) -> Self {
        self.groups.push(group);
        self
    }

    pub fn group(self, group: ToolbarGroup<'a, Message>) -> Self {
        self.push(group)
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

    pub fn role(mut self, role: SurfaceRole) -> Self {
        self.role = role;
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn fill(self) -> Self {
        self.width(Length::Fill)
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_toolbar::metrics(self.size);
        let mut groups = Row::new()
            .spacing(metrics.group_gap)
            .align_y(Alignment::Center)
            .height(Length::Fixed(
                metrics.action_height + metrics.group_padding * 2.0,
            ));

        for group in self.groups {
            groups = groups.push(group.into_element(metrics));
        }

        let mut toolbar = container(groups)
            .style(theme_toolbar::toolbar_style(self.role))
            .padding(
                Padding::ZERO
                    .vertical(metrics.toolbar_padding_v)
                    .horizontal(metrics.toolbar_padding_h),
            )
            .height(Length::Fixed(metrics.height));

        if let Some(width) = self.width {
            toolbar = toolbar.width(width);
        }

        toolbar.into()
    }
}

impl<'a, Message> Default for Toolbar<'a, Message>
where
    Message: Clone + 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> From<Toolbar<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(toolbar: Toolbar<'a, Message>) -> Self {
        toolbar.into_element()
    }
}
