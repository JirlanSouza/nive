mod dnd;
mod nav;
mod row_events;
mod selection;

#[cfg(test)]
mod widget_tests;

use iced::{
    widget::{button, column, container, scrollable},
    Length, Padding,
};

use crate::interaction::{ActivationBehavior, ActivationTrigger, RenameBehavior, SelectionMode};
use crate::theme::ControlSize;
use crate::Element;

use super::event::TreeEvent;
use super::focus::TreeFocus;
use super::state::{TreeState, TreeStateChange};
use super::transfer::TreeDrag;
use super::visible::{visible_entries, VisibleTreeEntry};
use super::TreeNode;

use dnd::loading_row;

/// Row-click expansion behavior for branch rows.
///
/// Expander button clicks always toggle non-disabled branches. This setting
/// controls whether row clicks also toggle expansion, and whether double-click
/// expansion composes with activation requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TreeExpandBehavior {
    /// Only expander clicks toggle expansion.
    ExpanderOnly,
    /// A single row click toggles expansion.
    #[default]
    SingleClick,
    /// A double row click toggles expansion.
    DoubleClick,
}

/// Controlled hierarchy widget that renders visible [`TreeNode`] rows.
///
/// `Tree` is controlled: it reads app-owned [`TreeState`] and emits
/// [`TreeEvent`] values instead of mutating domain data. Apps normally call
/// [`TreeState::apply`] first, then match [`TreeEventKind`](super::event::TreeEventKind) with a wildcard arm
/// for app-specific side effects such as deferred loading, context menus,
/// clipboard adapters, or drop commits.
///
/// The widget owns its vertical viewport by default. Assign an [`id`](Self::id)
/// when the app wants to call [`super::reveal`]; use [`height`](Self::height)
/// to constrain the viewport, or [`no_scroll`](Self::no_scroll) to render
/// content-sized rows for an app-owned scroll container.
pub struct Tree<'a, Id, Message> {
    nodes: Vec<TreeNode<'a, Id>>,
    id: Option<iced::widget::Id>,
    state: Option<&'a TreeState<Id>>,
    selection_mode: SelectionMode,
    activation_behavior: ActivationBehavior,
    rename_behavior: RenameBehavior,
    expand_behavior: TreeExpandBehavior,
    context_selection_behavior: crate::interaction::ContextSelectionBehavior,
    drag: TreeDrag<Id>,
    type_ahead: bool,
    page_rows: usize,
    on_event: Option<Box<dyn Fn(TreeEvent<Id>) -> Message + 'a>>,
    width: Length,
    height: Length,
    scroll: bool,
    size: ControlSize,
}

impl<'a, Id, Message> Tree<'a, Id, Message>
where
    Id: Clone + Ord + 'a,
    Message: Clone + 'a,
{
    /// Builds a tree from root nodes.
    ///
    /// The nodes are declarative view data. IDs must be stable and unique for
    /// the rendered tree so controlled state and emitted events remain
    /// meaningful across view rebuilds.
    pub fn new(nodes: impl Into<Vec<TreeNode<'a, Id>>>) -> Self {
        Self {
            nodes: nodes.into(),
            id: None,
            state: None,
            selection_mode: SelectionMode::Single,
            activation_behavior: ActivationBehavior::Platform,
            rename_behavior: RenameBehavior::Platform,
            expand_behavior: TreeExpandBehavior::SingleClick,
            context_selection_behavior:
                crate::interaction::ContextSelectionBehavior::SelectTargetIfUnselected,
            drag: TreeDrag::disabled(),
            type_ahead: true,
            page_rows: 10,
            on_event: None,
            width: Length::Fill,
            height: Length::Fill,
            scroll: true,
            size: ControlSize::Sm,
        }
    }

    /// Identifies the tree-owned viewport for programmatic operations.
    ///
    /// Use the same ID with [`super::reveal`] to expand ancestors, focus the
    /// node, and scroll this viewport to the node's uniform row offset.
    pub fn id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the app-owned tree state read by this view.
    ///
    /// If omitted, the tree renders against an empty temporary state and still
    /// emits events. Interactive apps should usually store a [`TreeState`] and
    /// apply every event to it during update.
    pub fn state(mut self, state: &'a TreeState<Id>) -> Self {
        self.state = Some(state);
        self
    }

    /// Sets the selection policy for row interactions.
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// Sets activation behavior for keyboard and pointer activation intents.
    ///
    /// [`ActivationBehavior::Platform`] resolves to the desktop platform
    /// default when events are interpreted.
    pub fn activation_behavior(mut self, behavior: ActivationBehavior) -> Self {
        self.activation_behavior = behavior;
        self
    }

    /// Sets rename behavior for keyboard rename intents.
    ///
    /// [`RenameBehavior::Platform`] resolves to Return on macOS and F2 on
    /// other targets.
    pub fn rename_behavior(mut self, behavior: RenameBehavior) -> Self {
        self.rename_behavior = behavior;
        self
    }

    /// Sets how row clicks toggle branch expansion.
    pub fn expand_behavior(mut self, behavior: TreeExpandBehavior) -> Self {
        self.expand_behavior = behavior;
        self
    }

    /// Sets how context requests affect selection before the request is
    /// emitted.
    pub fn context_selection_behavior(
        mut self,
        behavior: crate::interaction::ContextSelectionBehavior,
    ) -> Self {
        self.context_selection_behavior = behavior;
        self
    }

    /// Sets drag/drop configuration.
    ///
    /// Drag/drop is disabled by default. The tree only emits intent and
    /// transfer feedback; applications still decide whether and how to mutate
    /// their domain model.
    pub fn drag(mut self, drag: TreeDrag<Id>) -> Self {
        self.drag = drag;
        self
    }

    /// Enables or disables type-ahead matching.
    ///
    /// Enabled type-ahead accumulates printable characters for a short timeout,
    /// searches visible non-disabled rows, and wraps from the focused row.
    pub fn type_ahead(mut self, enabled: bool) -> Self {
        self.type_ahead = enabled;
        self
    }

    /// Sets PageUp/PageDown movement rows for keyboard navigation.
    pub fn page_rows(mut self, rows: usize) -> Self {
        self.page_rows = rows.max(1);
        self
    }

    /// Maps tree events into app messages.
    ///
    /// Use a handler that forwards [`TreeEvent`] into the app update path.
    /// Event-kind matches should include `_ => {}` because the enum is
    /// non-exhaustive.
    pub fn on_event(mut self, f: impl Fn(TreeEvent<Id>) -> Message + 'a) -> Self {
        self.on_event = Some(Box::new(f));
        self
    }

    crate::impl_layout_builders!(
        width_direct,
        height_direct,
        fill_width_direct,
        fill_height_direct,
        fill_direct
    );

    /// Renders content-sized rows without an internal viewport.
    ///
    /// This is the advanced composition path for app-owned scroll containers.
    /// Pair it with [`super::row_height`], [`super::visible_index_of`], and
    /// [`super::scroll_offset_to`] for deterministic offset math.
    pub fn no_scroll(mut self) -> Self {
        self.scroll = false;
        self
    }

    /// Sets the row size.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Uses extra-small rows.
    pub fn xs(self) -> Self {
        self.size(ControlSize::Xs)
    }

    /// Uses small rows.
    pub fn sm(self) -> Self {
        self.size(ControlSize::Sm)
    }

    /// Uses medium rows.
    pub fn md(self) -> Self {
        self.size(ControlSize::Md)
    }

    /// Uses large rows.
    pub fn lg(self) -> Self {
        self.size(ControlSize::Lg)
    }

    // `has_internal_scroll`/`height_value` are viewport accessors for the
    // internal-viewport widget introduced in a later task (section 7); no
    // production call site exists yet.
    #[allow(dead_code)]
    pub(crate) fn has_internal_scroll(&self) -> bool {
        self.scroll
    }

    #[allow(dead_code)]
    pub(crate) fn height_value(&self) -> Length {
        self.height
    }

    fn into_element(self) -> Element<'a, Message> {
        let content = self.build_content();
        TreeFocus::new(self, content).into()
    }

    fn build_content(&self) -> Element<'a, Message> {
        let default_state;
        let state = if let Some(state) = self.state {
            state
        } else {
            default_state = TreeState::default();
            &default_state
        };

        let mut rows = column!().width(Length::Fill).spacing(0);

        for entry in visible_entries(&self.nodes, state) {
            rows = rows.push(match entry {
                VisibleTreeEntry::Row(row) => {
                    let selected = state.is_selected(&row.id);
                    let focused = state.is_focused(&row.id);
                    let press = self.row_event(state, &row.id, row.expanded, row.disabled);
                    let toggle = self.toggle_event(row.id.clone(), row.expanded, row.disabled);

                    let mut item = crate::widgets::TreeItem::new(row.label)
                        .depth(row.depth)
                        .selected(selected)
                        .disabled(row.disabled)
                        .focused(focused)
                        .size(self.size)
                        .on_press_maybe(press)
                        .on_toggle_maybe(toggle);

                    if let Some(expanded) = row.expanded {
                        item = item.expanded(expanded);
                    } else {
                        item = item.leaf();
                    }

                    if let Some(icon) = row.leading_icon {
                        item = item.leading_icon(icon);
                    }

                    if let Some(tone) = row.tone {
                        item = item.tone(tone);
                    }

                    if let Some(trailing) = row.trailing_text {
                        item = item.trailing_text(trailing);
                    }

                    item.into()
                }
                VisibleTreeEntry::Loading(row) => loading_row(row.depth, self.size),
            });
        }

        let clear_msg = self.empty_space_event(state);
        let mut bg_button = button(rows)
            .width(Length::Fill)
            .padding(Padding::ZERO)
            .style(|_theme, _status| button::Style::default());

        if let Some(msg) = clear_msg {
            bg_button = bg_button.on_press(msg);
        }

        let content: Element<'a, Message> = bg_button.into();

        if self.scroll {
            let mut scrollable = scrollable(content).width(self.width).height(self.height);
            if let Some(id) = self.id.clone() {
                scrollable = scrollable.id(id);
            }
            scrollable.into()
        } else {
            container(content).width(self.width).into()
        }
    }

    pub(crate) fn id_ref(&self) -> Option<&iced::widget::Id> {
        self.id.as_ref()
    }

    pub(crate) fn type_ahead_enabled(&self) -> bool {
        self.type_ahead
    }

    pub(crate) fn state_or_default(&self) -> TreeState<Id> {
        self.state.cloned().unwrap_or_default()
    }

    pub(crate) fn publish(&self, event: TreeEvent<Id>) -> Option<Message> {
        self.on_event.as_ref().map(|on_event| on_event(event))
    }

    /// Returns whether the given trigger should emit an activation event.
    ///
    /// Not yet wired to a real Enter/Space/Command+O keyboard event; that
    /// wiring is out of this task's scope, tracked separately.
    #[allow(dead_code)]
    pub(crate) fn should_activate(&self, trigger: ActivationTrigger) -> bool {
        self.activation_behavior.should_activate(trigger)
    }

    /// Returns whether the given key should emit a rename request.
    ///
    /// Not yet wired to a real F2/Return keyboard event; that wiring is out
    /// of this task's scope, tracked separately.
    #[allow(dead_code)]
    pub(crate) fn should_rename(&self, key: iced::keyboard::key::Named) -> bool {
        self.rename_behavior.should_rename(key)
    }
}

impl<'a, Id, Message> From<Tree<'a, Id, Message>> for Element<'a, Message>
where
    Id: Clone + Ord + 'a,
    Message: Clone + 'a,
{
    fn from(tree: Tree<'a, Id, Message>) -> Self {
        tree.into_element()
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn fill_sets_tree_width_and_height() {
        let tree = Tree::<u32, ()>::new(Vec::<TreeNode<'_, u32>>::new())
            .width(Length::Shrink)
            .height(Length::Fixed(120.0))
            .fill();

        assert_eq!(tree.width, Length::Fill);
        assert_eq!(tree.height, Length::Fill);
    }
}
