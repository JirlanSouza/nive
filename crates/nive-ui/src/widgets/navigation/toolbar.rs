mod action;
mod group;
mod separator;
mod style;

use iced::{
    widget::{container, rule, scrollable, stack, Row},
    Alignment, Length, Padding,
};

use crate::theme::{BorderRole, ControlSize, SurfaceRole};
use crate::widgets::overlays::TooltipScope;
use crate::Element;

use self::style as theme_toolbar;

pub use action::ToolbarAction;
pub use group::ToolbarGroup;

pub struct Toolbar<'a, Message> {
    items: Vec<ToolbarItem<'a, Message>>,
    size: ControlSize,
    role: SurfaceRole,
    width: Option<Length>,
}

enum ToolbarItem<'a, Message> {
    Group(ToolbarGroup<'a, Message>),
    Separator,
}

impl<'a, Message> Toolbar<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            size: ControlSize::Sm,
            role: SurfaceRole::Chrome,
            width: None,
        }
    }

    pub fn push(mut self, group: ToolbarGroup<'a, Message>) -> Self {
        self.items.push(ToolbarItem::Group(group));
        self
    }

    pub fn group(self, group: ToolbarGroup<'a, Message>) -> Self {
        self.push(group)
    }

    /// Adds an explicit semantic boundary between toolbar groups.
    pub fn separator(mut self) -> Self {
        self.items.push(ToolbarItem::Separator);
        self
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

    crate::impl_layout_builders!(width_opt, fill_width_opt, shrink_width_opt);

    fn into_element(self) -> Element<'a, Message> {
        let metrics = theme_toolbar::metrics(self.size);
        let mut groups = Row::new()
            .spacing(metrics.group_gap)
            .align_y(Alignment::Center)
            .height(Length::Fixed(metrics.action_height));

        for item in self.items {
            groups = groups.push(match item {
                ToolbarItem::Group(group) => group.into_element(metrics),
                ToolbarItem::Separator => self::separator::separator(metrics),
            });
        }

        let groups: Element<'a, Message> = match self.width {
            Some(width) => scrollable(groups)
                .direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::hidden(),
                ))
                .width(width)
                .height(Length::Fixed(metrics.action_height))
                .into(),
            None => groups.into(),
        };

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

        let edge = rule::horizontal(1).style(bottom_edge_style);
        let edge = container(edge).height(Length::Fill).align_y(Alignment::End);
        let mut with_edge = stack![toolbar, edge].height(Length::Fixed(metrics.height));
        if let Some(width) = self.width {
            with_edge = with_edge.width(width);
        }

        TooltipScope::new(with_edge).into()
    }
}

fn bottom_edge_style(theme: &crate::theme::Theme) -> rule::Style {
    rule::Style {
        color: theme.border(BorderRole::Subtle).color,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use iced::{Event, Point, Size};

    use super::*;
    use crate::{test_support::WidgetHarness, widgets::IconRole};

    #[test]
    fn icon_action_uses_shared_tooltip_without_changing_toolbar_height() {
        let toolbar: Element<'_, ()> = Toolbar::new()
            .group(
                ToolbarGroup::new().action(
                    ToolbarAction::icon(IconRole::ViewRefresh)
                        .tooltip("Refresh")
                        .on_press(()),
                ),
            )
            .sm()
            .into();
        let mut harness = WidgetHarness::new(toolbar, Size::new(240.0, 80.0));
        let start = iced::time::Instant::now();

        assert_eq!(
            harness.bounds().height,
            theme_toolbar::metrics(ControlSize::Sm).height
        );
        harness.set_cursor(Point::new(12.0, harness.bounds().center_y()));
        harness.update(redraw(start));
        assert!(!harness.has_overlay());
        harness.update(redraw(start + Duration::from_millis(500)));
        assert!(harness.has_overlay());
    }

    #[test]
    fn complete_visible_label_does_not_duplicate_itself_as_tooltip() {
        let toolbar: Element<'_, ()> = Toolbar::new()
            .group(ToolbarGroup::new().action(ToolbarAction::label("Save").on_press(())))
            .into();
        let mut harness = WidgetHarness::new(toolbar, Size::new(240.0, 80.0));
        let start = iced::time::Instant::now();

        harness.set_cursor(Point::new(20.0, harness.bounds().center_y()));
        harness.update(redraw(start));
        harness.update(redraw(start + Duration::from_millis(500)));

        assert!(!harness.has_overlay());
    }

    fn redraw(now: iced::time::Instant) -> Event {
        Event::Window(iced::window::Event::RedrawRequested(now))
    }
}
