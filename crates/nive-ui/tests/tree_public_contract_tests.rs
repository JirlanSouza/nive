//! Positive compile contract: the Tree family and its geometry helpers
//! resolve through the supported `nive_ui::widgets` surface, including the
//! `branch_failed`/`TreeChildren::Failed` addition. Negative contracts (the
//! internal widget/navigation/selection/focus/drag-drop machinery staying
//! private) live as `compile_fail` doctests on `Tree`'s module rustdoc.

use nive_ui::interaction::SelectionMode;
use nive_ui::theme::ControlSize;
use nive_ui::widgets::{
    reveal, row_height, scroll_offset_to, visible_index_of, Tree, TreeChildren, TreeDrag, TreeDrop,
    TreeDropTarget, TreeEvent, TreeEventKind, TreeExpandBehavior, TreeItem, TreeItemDropEdge,
    TreeNode, TreePasteTarget, TreeState, TreeStateChange,
};
use nive_ui::{Element, Length};

struct TestError;

impl nive_ui::widgets::ErrorPresentation for TestError {
    fn summary(&self) -> &str {
        "Load failed"
    }

    fn detail(&self) -> &str {
        "Load failed: connection reset"
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    Tree(TreeEvent<u32>),
}

#[test]
fn tree_family_resolves_through_supported_paths() {
    let error = TestError;
    let nodes: Vec<TreeNode<'_, u32>> = vec![
        TreeNode::branch(1, "src", [TreeNode::leaf(2, "main.rs")]),
        TreeNode::branch_deferred(3, "remote"),
        TreeNode::branch_failed(4, "broken", &error),
        TreeNode::branch(5, "empty", Vec::<TreeNode<'_, u32>>::new()),
    ];
    assert!(matches!(
        nodes[2].children(),
        Some(TreeChildren::Failed { .. })
    ));

    let state = TreeState::<u32>::default();
    let mut mutable_state = state.clone();
    mutable_state.apply_change(&TreeStateChange::SetExpanded {
        id: 1,
        expanded: true,
    });

    let _height = row_height(ControlSize::Sm);
    let _index = visible_index_of(&nodes, &state, &1);
    let _offset = scroll_offset_to(&nodes, &state, &1, ControlSize::Sm);
    let _task: iced::Task<Message> = reveal(&mut mutable_state, &nodes, "tree", &2);

    let tree: Tree<'_, u32, Message> = Tree::new(nodes)
        .state(&state)
        .selection_mode(SelectionMode::Multiple)
        .expand_behavior(TreeExpandBehavior::SingleClick)
        .drag(TreeDrag::disabled())
        .height(Length::Fixed(240.0))
        .on_event(Message::Tree);
    let _element: Element<'_, Message> = tree.into();

    let item: TreeItem<'_, Message> = TreeItem::new("Row")
        .depth(1)
        .expanded(true)
        .selected(false)
        .dragging(false)
        .drop_indicator(Some(TreeItemDropEdge::Before));
    let _element: Element<'_, Message> = item.into();

    let _paste_target: TreePasteTarget<u32> = TreePasteTarget::Root;
    let _drop_target: TreeDropTarget<u32> = TreeDropTarget::After(1);
    let _kind: TreeEventKind<u32> = TreeEventKind::ExpandRequested { id: 3 };

    // `TreeDrop` is `#[non_exhaustive]`; it resolves through the supported
    // path and is only constructed by the widget itself.
    let _drop_type_resolves: fn() -> Option<TreeDrop<u32>> = || None;
}
