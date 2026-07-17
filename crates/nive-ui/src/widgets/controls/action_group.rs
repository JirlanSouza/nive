mod content_action;
mod flow;

use iced::{
    widget::{container, Space},
    Background, Length, Shadow,
};

use crate::{
    advanced::control_style::transparent_border,
    theme::{self, BorderRole, ControlSize, Theme},
    Element,
};

pub use content_action::ContentAction;
use content_action::ContentActionMetrics;
use flow::{Flow, FlowItem};

/// Chrome-free group of compact peer actions embedded in content.
///
/// The group owns shared [`ControlSize`] geometry; individual
/// [`ContentAction`] values remain intrinsic-width and non-selectable.
///
/// `ActionGroup` is content-owned, not navigation-owned:
///
/// ```compile_fail
/// use nive_ui::widgets::navigation::ActionGroup;
/// ```
///
/// Toolbar actions cannot be inserted into a content group:
///
/// ```compile_fail
/// use nive_ui::widgets::{ActionGroup, ToolbarAction};
///
/// let _ = ActionGroup::<()>::new().action(ToolbarAction::label("Refresh"));
/// ```
pub struct ActionGroup<'a, Message> {
    items: Vec<ActionGroupItem<'a, Message>>,
    size: ControlSize,
    width: Length,
    wrap: bool,
}

enum ActionGroupItem<'a, Message> {
    Action(ContentAction<'a, Message>),
    Separator,
}

impl<'a, Message> ActionGroup<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            size: ControlSize::Sm,
            width: Length::Shrink,
            wrap: false,
        }
    }

    pub fn push(mut self, action: ContentAction<'a, Message>) -> Self {
        self.items.push(ActionGroupItem::Action(action));
        self
    }

    pub fn action(self, action: ContentAction<'a, Message>) -> Self {
        self.push(action)
    }

    pub fn separator(mut self) -> Self {
        self.items.push(ActionGroupItem::Separator);
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

    /// Enables constrained wrapping between complete actions.
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }

    crate::impl_layout_builders!(fill_width_direct, shrink_width_direct);

    fn into_element(self) -> Element<'a, Message> {
        let metrics = ContentActionMetrics::resolve(theme::active(), self.size);
        let items = normalize_items(self.items)
            .into_iter()
            .map(|item| match item {
                ActionGroupItem::Action(action) => FlowItem::action(action.into_element(metrics)),
                ActionGroupItem::Separator => FlowItem::separator(separator(metrics)),
            })
            .collect();

        Flow::new(
            items,
            self.width,
            metrics.item_gap,
            metrics.row_gap,
            self.wrap,
        )
        .into()
    }
}

fn normalize_items<'a, Message>(
    items: Vec<ActionGroupItem<'a, Message>>,
) -> Vec<ActionGroupItem<'a, Message>> {
    let mut normalized = Vec::with_capacity(items.len());
    let mut pending_separator = false;

    for item in items {
        match item {
            ActionGroupItem::Separator => pending_separator = !normalized.is_empty(),
            ActionGroupItem::Action(action) => {
                if pending_separator {
                    normalized.push(ActionGroupItem::Separator);
                }
                normalized.push(ActionGroupItem::Action(action));
                pending_separator = false;
            }
        }
    }

    normalized
}

fn separator<'a, Message>(metrics: ContentActionMetrics) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(Space::new().width(Length::Fixed(metrics.separator_width)))
        .style(separator_style())
        .width(Length::Fixed(metrics.separator_width))
        .height(Length::Fixed(metrics.separator_height))
        .into()
}

fn separator_style() -> impl Fn(&Theme) -> container::Style {
    |theme| container::Style {
        text_color: None,
        background: Some(Background::Color(theme.border(BorderRole::Subtle).color)),
        border: transparent_border(),
        shadow: Shadow::default(),
        ..container::Style::default()
    }
}

impl<'a, Message> Default for ActionGroup<'a, Message>
where
    Message: Clone + 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message> From<ActionGroup<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(group: ActionGroup<'a, Message>) -> Self {
        group.into_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::containers::card_test_support::{CardHarness, Message};
    use crate::widgets::primitives::IconRole;
    use iced::{advanced::layout::Node, keyboard::key::Named, mouse, Point, Rectangle, Size};

    #[test]
    fn defaults_to_small_intrinsic_single_line_group() {
        let group = ActionGroup::<()>::new();

        assert_eq!(group.size, ControlSize::Sm);
        assert_eq!(group.width, Length::Shrink);
        assert!(!group.wrap);
    }

    #[test]
    fn invalid_separator_sequences_are_normalized() {
        let items = normalize_items(vec![
            ActionGroupItem::Separator,
            ActionGroupItem::Action(ContentAction::<()>::label("One")),
            ActionGroupItem::Separator,
            ActionGroupItem::Separator,
            ActionGroupItem::Action(ContentAction::<()>::label("Two")),
            ActionGroupItem::Separator,
        ]);

        assert_eq!(items.len(), 3);
        assert!(matches!(items[0], ActionGroupItem::Action(_)));
        assert!(matches!(items[1], ActionGroupItem::Separator));
        assert!(matches!(items[2], ActionGroupItem::Action(_)));
    }

    #[test]
    fn fill_and_wrap_are_orthogonal_to_intrinsic_action_widths() {
        let metrics = ContentActionMetrics::resolve(theme::active(), ControlSize::Sm);
        let single_line = CardHarness::new(
            ActionGroup::new()
                .fill_width()
                .action(ContentAction::label("Inspect service").on_press(Message::Activated))
                .action(ContentAction::label("Run health check").on_press(Message::Activated))
                .into(),
            Size::new(160.0, 200.0),
        );
        let wrapped = CardHarness::new(
            ActionGroup::new()
                .fill_width()
                .wrap()
                .action(ContentAction::label("Inspect service").on_press(Message::Activated))
                .action(ContentAction::label("Run health check").on_press(Message::Activated))
                .into(),
            Size::new(160.0, 200.0),
        );

        assert_eq!(single_line.size().width, 160.0);
        assert_eq!(single_line.size().height, metrics.height);
        assert_eq!(wrapped.size().width, 160.0);
        assert!(wrapped.size().height > metrics.height);
    }

    #[test]
    fn loading_slot_keeps_group_geometry_stable() {
        let idle = CardHarness::new(
            ActionGroup::new()
                .action(
                    ContentAction::icon_label(IconRole::ViewRefresh, "Refresh")
                        .loading(false)
                        .on_press(Message::Activated),
                )
                .into(),
            Size::new(300.0, 100.0),
        );
        let loading = CardHarness::new(
            ActionGroup::new()
                .action(
                    ContentAction::icon_label(IconRole::ViewRefresh, "Refresh")
                        .loading(true)
                        .on_press(Message::Activated),
                )
                .into(),
            Size::new(300.0, 100.0),
        );

        assert_eq!(idle.size(), loading.size());
    }

    #[test]
    fn icon_only_content_centers_icon_vertically() {
        let metrics = ContentActionMetrics::resolve(theme::active(), ControlSize::Sm);
        let node = crate::test_support::layout(
            ContentAction::<Message>::icon(IconRole::GoNext, "Next")
                .on_press(Message::Activated)
                .into_element(metrics),
            Size::new(100.0, 100.0),
        );
        let icon = leaf_with_size(&node, Point::ORIGIN, metrics.icon_size)
            .expect("icon leaf with the resolved control size");

        assert_eq!(node.size().height, metrics.height);
        assert!((icon.center_x() - node.bounds().center_x()).abs() < 0.01);
        assert!((icon.center_y() - node.bounds().center_y()).abs() < 0.01);
    }

    #[test]
    fn wrapped_boundary_suppresses_an_orphaned_separator() {
        let mut harness = CardHarness::new(
            ActionGroup::new()
                .fill_width()
                .wrap()
                .action(ContentAction::label("One"))
                .separator()
                .action(ContentAction::label("Two"))
                .into(),
            Size::new(72.0, 160.0),
        );
        let children = harness.child_bounds();

        assert_eq!(children.len(), 3);
        assert_eq!(children[1].size(), Size::ZERO);
        assert!(children[2].y > children[0].y);
        assert!(!harness.has_overlay());
    }

    #[test]
    fn host_narrower_than_one_action_clips_the_group_not_the_action() {
        let harness = CardHarness::new(
            ActionGroup::new()
                .fill_width()
                .wrap()
                .action(ContentAction::label(
                    "One deliberately oversized complete content action",
                ))
                .into(),
            Size::new(80.0, 120.0),
        );
        let action = harness.child_bounds()[0];

        assert_eq!(harness.size().width, 80.0);
        assert!(action.width > harness.size().width);
        assert_eq!(action.y, 0.0);
    }

    #[test]
    fn action_capability_loading_and_disabled_states_control_events_and_focus() {
        let mut enabled = CardHarness::new(
            ActionGroup::new()
                .action(ContentAction::label("Run").on_press(Message::Activated))
                .into(),
            Size::new(200.0, 80.0),
        );
        assert_eq!(enabled.click_center(), vec![Message::Activated]);
        assert_eq!(
            enabled.activate_key(Named::Enter, false),
            vec![Message::Activated]
        );

        let mut keyboard = CardHarness::new(
            ActionGroup::new()
                .action(ContentAction::label("Run").on_press(Message::Activated))
                .into(),
            Size::new(200.0, 80.0),
        );
        keyboard.focus_next();
        assert_eq!(
            keyboard.activate_key(Named::Enter, false),
            vec![Message::Activated]
        );

        for action in [
            ContentAction::label("Absent"),
            ContentAction::label("Loading")
                .loading(true)
                .on_press(Message::Activated),
            ContentAction::label("Disabled")
                .disabled(true)
                .on_press(Message::Activated),
        ] {
            let mut inert = CardHarness::new(
                ActionGroup::new().action(action).into(),
                Size::new(200.0, 80.0),
            );
            assert_ne!(inert.mouse_interaction(), mouse::Interaction::Pointer);
            assert!(inert.click_center().is_empty());
            inert.focus_next();
            assert!(inert.activate_key(Named::Space, false).is_empty());
        }
    }

    fn leaf_with_size(node: &Node, origin: Point, side: f32) -> Option<Rectangle> {
        let bounds = node.bounds();
        let absolute = Rectangle {
            x: origin.x + bounds.x,
            y: origin.y + bounds.y,
            ..bounds
        };

        if node.children().is_empty()
            && (bounds.width - side).abs() < 0.01
            && (bounds.height - side).abs() < 0.01
        {
            return Some(absolute);
        }

        node.children()
            .iter()
            .find_map(|child| leaf_with_size(child, absolute.position(), side))
    }
}
