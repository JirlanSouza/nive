//! Controlled hierarchy widget and tree interaction contracts.
//!
//! [`Tree`] is the high-level widget for app-owned hierarchical data. Apps
//! rebuild [`TreeNode`] values on every view pass, keep [`TreeState`] in their
//! own state, and call [`TreeState::apply`] for every emitted [`TreeEvent`]
//! before handling app-specific effects such as loading children, opening
//! context menus, copying domain data, or committing drops.
//!
//! IDs are app-domain values. They must be stable and unique within one
//! rendered tree because expansion, selection, focus, clipboard feedback,
//! drag/drop feedback, and reveal helpers all refer to nodes by ID. If domain
//! data changes, call [`TreeState::retain_ids`] with the current ID set to drop
//! stale expansion, selection, focus, anchor, and transfer state.
//!
//! # Event Flow
//!
//! ```no_run
//! use nive_ui::{Element, Length};
//! use nive_ui::interaction::SelectionMode;
//! use nive_ui::widgets::{Tree, TreeEvent, TreeEventKind, TreeNode, TreeState};
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! enum NodeId {
//!     Root,
//!     Remote,
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     Tree(TreeEvent<NodeId>),
//! }
//!
//! fn view(state: &TreeState<NodeId>) -> Element<'_, Message> {
//!     let nodes = vec![TreeNode::branch(
//!         NodeId::Root,
//!         "src",
//!         [TreeNode::branch_deferred(NodeId::Remote, "remote")],
//!     )];
//!
//!     Tree::new(nodes)
//!         .state(state)
//!         .selection_mode(SelectionMode::Multiple)
//!         .height(Length::Fixed(240.0))
//!         .on_event(Message::Tree)
//!         .into()
//! }
//!
//! fn update(state: &mut TreeState<NodeId>, message: Message) {
//!     let Message::Tree(event) = message;
//!     state.apply(&event);
//!
//!     match event.kind {
//!         TreeEventKind::ExpandRequested { id } => {
//!             // Rebuild `NodeId::Remote` with loaded children when ready.
//!             let _ = id;
//!         }
//!         _ => {}
//!     }
//! }
//! ```
//!
//! # Deferred Loading
//!
//! Use [`TreeNode::branch_deferred`] when children are not available yet. The
//! tree toggles expansion state and emits [`TreeEventKind::ExpandRequested`]
//! every time that deferred branch is expanded. While the branch remains
//! deferred and expanded, traversal renders one loading placeholder row at the
//! child depth. Rebuild the node with [`TreeNode::branch`] when the app has
//! loaded children, or with [`TreeNode::branch_failed`] when the load failed.
//!
//! Apps map their own per-branch async state into [`TreeChildren`] on every
//! view pass — for example a runtime `Resource<Vec<Child>>` whose loading
//! state maps to `Deferred`, success to `Loaded`, and failure to `Failed`
//! (built with [`TreeNode::branch_failed`] from a value implementing the core
//! `ErrorPresentation` contract, such as `UserFacingError`; `nive-ui` never
//! names the runtime types itself). While expanded, a `Failed` branch renders
//! one canonical error row showing the error summary with a retry affordance
//! that re-emits `ExpandRequested { id }`; an expanded `Loaded` branch with no
//! children renders one canonical empty affordance row. All three canonical
//! rows — loading, failed, empty — are chrome: they are excluded from
//! selection, focus, navigation, type-ahead, clipboard, and drag/drop, while
//! still counting in [`visible_index_of`]/[`scroll_offset_to`] rendered order.
//!
//! # Keyboard And Pointer Semantics
//!
//! Selection follows [`crate::interaction::SelectionMode`]. Multiple selection
//! supports additive primary-modifier clicks and Shift ranges. Keyboard
//! navigation moves focus with Up/Down, expands or enters branches with Right,
//! collapses or moves to parents with Left, jumps with Home/End, pages by the
//! configured [`Tree::page_rows`], and skips disabled rows for selection,
//! type-ahead, and domain requests. Type-ahead matches visible non-disabled
//! labels and wraps through visible order; [`Tree::type_ahead`] disables it.
//! Activation and rename intent mapping use
//! [`crate::interaction::ActivationBehavior`] and
//! [`crate::interaction::RenameBehavior`] so platform defaults stay explicit.
//! `Activate { id, trigger }` carries the input source. Rename stays an
//! intent: Tree emits [`TreeEventKind::RenameRequested`] and marks the rename
//! target for styling; it hosts no inline editor. The application owns any
//! editor and commits the change by rebuilding nodes.
//!
//! # Viewport And Reveal
//!
//! By default, `Tree` owns a vertical viewport with `height = Length::Fill`.
//! Use [`Tree::id`] to identify that viewport and [`reveal`] to expand
//! ancestors, focus a node, and scroll to its uniform row offset. Use
//! [`Tree::height`] to constrain the viewport, or [`Tree::no_scroll`] with
//! [`row_height`], [`visible_index_of`], and [`scroll_offset_to`] when composing
//! rows inside an app-owned scroll container. Tree renders every
//! expanded-visible row; it does not virtualize the viewport. The uniform-row
//! geometry stays virtualization-ready, and windowed rendering for very large
//! trees is a dedicated later change.
//!
//! # Context, Clipboard, Paste, And Drag/Drop
//!
//! Context requests carry a [`crate::interaction::ContextRequest`] with the
//! target and visible-order [`crate::interaction::SelectionSnapshot`], and
//! honor [`crate::interaction::ContextSelectionBehavior`] (default
//! `SelectTargetIfUnselected`) for right-click selection. Tree owns no menu
//! widget: the application hosts the canonical
//! [`Menu`](crate::widgets::Menu) at the request position and performs the
//! chosen action. Clipboard events emit normalized
//! [`crate::interaction::CollectionTransferPayload`] values without touching
//! the system clipboard. Paste requests report a [`TreePasteTarget`].
//! Drag/drop is app-internal intent only: [`TreeDrag`] controls allowed
//! operations and target validation, while accepted releases emit
//! [`TreeEventKind::DropRequested`] with a [`TreeDrop`]. Apps perform all
//! model mutation.
//!
//! # Accessibility Mapping
//!
//! The internal visible traversal already records the data needed for an
//! AccessKit tree mapping: row depth/level, parentage, visible order,
//! expanded/collapsed state, selected state, disabled state, and placeholder
//! rows. `Tree` keeps this information in the widget layer so role,
//! `aria-level`, `aria-expanded`, `aria-selected`, `aria-disabled`,
//! position-in-set, and set-size plumbing can be attached when the Iced
//! AccessKit surface is wired here.
//!
//! Most public enums in this module are `#[non_exhaustive]`; application
//! matches should include a wildcard arm.
//!
//! # Public Surface
//!
//! The supported Tree family is [`Tree`], [`TreeItem`](super::TreeItem),
//! [`TreeNode`], [`TreeChildren`], [`TreeState`], [`TreeStateChange`],
//! [`TreeEvent`], [`TreeEventKind`], [`TreeExpandBehavior`], [`TreeDrag`],
//! [`TreeDrop`], [`TreeDropTarget`], [`TreePasteTarget`], and the
//! [`reveal`]/[`row_height`]/[`visible_index_of`]/[`scroll_offset_to`]
//! helpers. The internal widget, navigation, selection, focus, and drag/drop
//! machinery stays private:
//!
//! ```compile_fail
//! use nive_ui::widgets::display::tree::widget::selection::batch_or_single;
//! ```
//!
//! ```compile_fail
//! use nive_ui::widgets::display::tree::focus::TreeFocus;
//! ```
//!
//! ```compile_fail
//! use nive_ui::widgets::display::tree::visible::{visible_entries, VisibleTreeEntry};
//! ```
//!
//! ```compile_fail
//! use nive_ui::widgets::display::tree::keymap::key_action;
//! ```

mod event;
mod focus;
mod keymap;
mod node;
mod state;
mod transfer;
mod visible;
mod widget;

pub use event::{TreeEvent, TreeEventKind};
pub use node::{TreeChildren, TreeNode};
pub use state::{TreeState, TreeStateChange};
pub use transfer::{TreeDrag, TreeDrop, TreeDropTarget, TreePasteTarget};
pub use widget::{Tree, TreeExpandBehavior};

use crate::theme::ControlSize;
use iced::{
    widget::operation::{scroll_to, AbsoluteOffset},
    Task,
};

/// Returns the fixed row height for a tree rendered at `size`.
pub fn row_height(size: ControlSize) -> f32 {
    crate::theme::control_metrics(size).height
}

/// Returns the visible rendered row index for `id`.
///
/// Loading placeholders count in the rendered row order, while hidden or absent
/// node IDs return `None`.
pub fn visible_index_of<Id>(
    roots: &[TreeNode<'_, Id>],
    state: &TreeState<Id>,
    id: &Id,
) -> Option<usize>
where
    Id: Clone + Ord,
{
    visible::visible_entries(roots, state)
        .into_iter()
        .find_map(|entry| match entry {
            visible::VisibleTreeEntry::Row(row) if row.id == *id => Some(row.visible_index),
            _ => None,
        })
}

/// Returns the absolute scroll offset for `id` in a content-sized tree.
///
/// This is the low-level composition path for apps that opt out of the
/// tree-owned viewport.
pub fn scroll_offset_to<Id>(
    roots: &[TreeNode<'_, Id>],
    state: &TreeState<Id>,
    id: &Id,
    size: ControlSize,
) -> Option<f32>
where
    Id: Clone + Ord,
{
    visible_index_of(roots, state, id).map(|index| index as f32 * row_height(size))
}

/// Expands ancestors, focuses `id`, and scrolls the identified tree viewport to it.
pub fn reveal<Id, Message>(
    state: &mut TreeState<Id>,
    roots: &[TreeNode<'_, Id>],
    tree_id: impl Into<iced::widget::Id>,
    id: &Id,
) -> Task<Message>
where
    Id: Clone + Ord,
{
    if !state.reveal(roots, id) {
        return Task::none();
    }

    let Some(offset) = scroll_offset_to(roots, state, id, ControlSize::Sm) else {
        return Task::none();
    };

    scroll_to(tree_id, AbsoluteOffset { x: 0.0, y: offset })
}

#[cfg(test)]
mod tree_helper_tests {
    use super::*;

    #[derive(Debug, Clone)]
    enum Message {}

    fn helper_nodes() -> Vec<TreeNode<'static, &'static str>> {
        vec![
            TreeNode::branch(
                "root",
                "Root",
                [
                    TreeNode::leaf("child", "Child"),
                    TreeNode::branch_deferred("deferred", "Deferred"),
                    TreeNode::branch(
                        "nested",
                        "Nested",
                        [TreeNode::leaf("grandchild", "Grandchild")],
                    ),
                ],
            ),
            TreeNode::leaf("after", "After"),
        ]
    }

    #[test]
    fn row_height_matches_control_metrics() {
        for size in [
            ControlSize::Xs,
            ControlSize::Sm,
            ControlSize::Md,
            ControlSize::Lg,
        ] {
            assert_eq!(row_height(size), crate::theme::control_metrics(size).height);
        }
    }

    #[test]
    fn visible_index_returns_none_for_hidden_or_absent_ids() {
        let nodes = helper_nodes();
        let state = TreeState::default();

        assert_eq!(visible_index_of(&nodes, &state, &"root"), Some(0));
        assert_eq!(visible_index_of(&nodes, &state, &"child"), None);
        assert_eq!(visible_index_of(&nodes, &state, &"missing"), None);
    }

    #[test]
    fn visible_index_and_offset_count_deferred_placeholders() {
        let nodes = helper_nodes();
        let mut state = TreeState::default();
        state.expand("root");
        state.expand("deferred");

        assert_eq!(visible_index_of(&nodes, &state, &"nested"), Some(4));
        assert_eq!(
            scroll_offset_to(&nodes, &state, &"nested", ControlSize::Sm),
            Some(4.0 * row_height(ControlSize::Sm))
        );
    }

    #[test]
    fn reveal_expands_focuses_and_returns_task() {
        let nodes = helper_nodes();
        let mut state = TreeState::default();

        let _task: Task<Message> = reveal(&mut state, &nodes, "tree", &"grandchild");

        assert!(state.is_expanded(&"root"));
        assert!(state.is_expanded(&"nested"));
        assert_eq!(state.focused(), Some(&"grandchild"));
        assert_eq!(visible_index_of(&nodes, &state, &"grandchild"), Some(4));
    }

    #[test]
    fn reveal_absent_id_returns_without_mutating_state() {
        let nodes = helper_nodes();
        let mut state = TreeState::default();

        let _task: Task<Message> = reveal(&mut state, &nodes, "tree", &"missing");

        assert!(state.expanded.is_empty());
        assert_eq!(state.focused(), None);
    }
}
