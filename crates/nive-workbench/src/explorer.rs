use std::borrow::Cow;

use nive_ui::widgets::{Tree, TreeEvent, TreeNode, TreeState};
use nive_ui::IconRole;

use crate::panels::WorkbenchPanel;

/// Lightweight explorer-panel helper over `nive-ui::Tree`.
pub struct ExplorerPanel<'a, NodeId, PanelId, ActionId, Message> {
    panel_id: PanelId,
    title: Cow<'a, str>,
    nodes: Vec<TreeNode<'a, NodeId>>,
    state: Option<&'a TreeState<NodeId>>,
    on_tree_event: Option<Box<dyn Fn(TreeEvent<NodeId>) -> Message + 'a>>,
    _action: std::marker::PhantomData<ActionId>,
}

impl<'a, NodeId, PanelId, ActionId, Message> ExplorerPanel<'a, NodeId, PanelId, ActionId, Message>
where
    NodeId: Clone + Ord + 'a,
    Message: Clone + 'a,
{
    /// Builds an explorer helper from app-provided tree nodes.
    pub fn new(panel_id: PanelId, nodes: impl Into<Vec<TreeNode<'a, NodeId>>>) -> Self {
        Self {
            panel_id,
            title: Cow::Borrowed("Explorer"),
            nodes: nodes.into(),
            state: None,
            on_tree_event: None,
            _action: std::marker::PhantomData,
        }
    }

    /// Sets panel title.
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = title.into();
        self
    }

    /// Provides app-owned tree state.
    pub fn state(mut self, state: &'a TreeState<NodeId>) -> Self {
        self.state = Some(state);
        self
    }

    /// Maps tree events into app messages.
    pub fn on_tree_event(mut self, mapper: impl Fn(TreeEvent<NodeId>) -> Message + 'a) -> Self {
        self.on_tree_event = Some(Box::new(mapper));
        self
    }

    /// Converts this helper into a generic panel.
    pub fn into_panel(self) -> WorkbenchPanel<'a, PanelId, ActionId, Message> {
        let mut tree = Tree::new(self.nodes);
        if let Some(state) = self.state {
            tree = tree.state(state);
        }
        if let Some(mapper) = self.on_tree_event {
            tree = tree.on_event(mapper);
        }

        WorkbenchPanel::new(self.panel_id, self.title, tree)
            .icon(IconRole::Folder)
            .collapsible(true)
            .maximizable(true)
    }
}

/// Builds a generic explorer panel from app-owned tree data and state.
pub fn explorer_panel<'a, NodeId, PanelId, ActionId, Message>(
    panel_id: PanelId,
    nodes: impl Into<Vec<TreeNode<'a, NodeId>>>,
    state: &'a TreeState<NodeId>,
    on_tree_event: impl Fn(TreeEvent<NodeId>) -> Message + 'a,
) -> WorkbenchPanel<'a, PanelId, ActionId, Message>
where
    NodeId: Clone + Ord + 'a,
    Message: Clone + 'a,
{
    ExplorerPanel::new(panel_id, nodes)
        .state(state)
        .on_tree_event(on_tree_event)
        .into_panel()
}
