use std::borrow::Cow;

use crate::{theme::ToneRole, widgets::IconRole};

/// Declarative tree node data consumed by the high-level `Tree` widget.
///
/// IDs are app-domain values and must be stable and unique within a rendered
/// tree. Expansion, selection, transfer state, and emitted events all refer to
/// nodes by ID, so duplicate or unstable IDs make interaction state ambiguous.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode<'a, Id> {
    id: Id,
    label: Cow<'a, str>,
    children: Option<TreeChildren<'a, Id>>,
    leading_icon: Option<IconRole>,
    tone: Option<ToneRole>,
    trailing_text: Option<Cow<'a, str>>,
    disabled: bool,
}

/// Children for a branch node.
///
/// `Loaded` represents children already supplied by the app, including an empty
/// vector for an intentionally empty branch. `Deferred` represents a branch
/// whose children are not loaded yet; expanding it is a loading intent for the
/// app, and the tree renders a placeholder until the app rebuilds the node with
/// loaded children.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeChildren<'a, Id> {
    /// Children that are available for traversal and rendering.
    Loaded(Vec<TreeNode<'a, Id>>),
    /// Children that should be requested from the app on expansion.
    Deferred,
}

impl<'a, Id> TreeNode<'a, Id> {
    /// Builds a leaf node with no branch expander.
    pub fn leaf(id: Id, label: impl Into<Cow<'a, str>>) -> Self {
        Self::new(id, label, None)
    }

    /// Builds a branch node with loaded children.
    pub fn branch(
        id: Id,
        label: impl Into<Cow<'a, str>>,
        children: impl Into<Vec<TreeNode<'a, Id>>>,
    ) -> Self {
        Self::new(id, label, Some(TreeChildren::Loaded(children.into())))
    }

    /// Builds a branch node whose children will be loaded by the app later.
    pub fn branch_deferred(id: Id, label: impl Into<Cow<'a, str>>) -> Self {
        Self::new(id, label, Some(TreeChildren::Deferred))
    }

    /// Adds a leading icon to the row rendered for this node.
    pub fn leading_icon(mut self, icon: IconRole) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    /// Adds a tone indicator to the row rendered for this node.
    pub fn tone(mut self, tone: ToneRole) -> Self {
        self.tone = Some(tone);
        self
    }

    /// Adds trailing secondary text to the row rendered for this node.
    pub fn trailing_text(mut self, trailing: impl Into<Cow<'a, str>>) -> Self {
        self.trailing_text = Some(trailing.into());
        self
    }

    /// Marks whether the node should render disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns this node's app-domain ID.
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Returns this node's display label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns this node's branch children, if it is a branch.
    pub fn children(&self) -> Option<&TreeChildren<'a, Id>> {
        self.children.as_ref()
    }

    /// Returns whether this node is a branch.
    pub fn is_branch(&self) -> bool {
        self.children.is_some()
    }

    /// Returns whether this node is a leaf.
    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    /// Returns this node's leading icon metadata.
    pub fn leading_icon_name(&self) -> Option<IconRole> {
        self.leading_icon
    }

    /// Returns this node's tone metadata.
    pub fn tone_role(&self) -> Option<ToneRole> {
        self.tone
    }

    /// Returns this node's trailing text metadata.
    pub fn trailing_text_value(&self) -> Option<&str> {
        self.trailing_text.as_deref()
    }

    /// Returns whether this node is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn new(id: Id, label: impl Into<Cow<'a, str>>, children: Option<TreeChildren<'a, Id>>) -> Self {
        Self {
            id,
            label: label.into(),
            children,
            leading_icon: None,
            tone: None,
            trailing_text: None,
            disabled: false,
        }
    }
}

#[cfg(test)]
mod tree_node_tests {
    use super::*;

    #[test]
    fn leaf_has_no_children() {
        let node = TreeNode::leaf(1, "Cargo.toml");

        assert_eq!(node.id(), &1);
        assert_eq!(node.label(), "Cargo.toml");
        assert!(node.is_leaf());
        assert!(node.children().is_none());
    }

    #[test]
    fn branch_stores_loaded_children_even_when_empty() {
        let node = TreeNode::branch(1, "src", Vec::<TreeNode<'_, i32>>::new());

        assert!(node.is_branch());
        assert!(matches!(
            node.children(),
            Some(TreeChildren::Loaded(children)) if children.is_empty()
        ));
    }

    #[test]
    fn branch_deferred_stores_deferred_children() {
        let node = TreeNode::branch_deferred(1, "target");

        assert!(node.is_branch());
        assert!(matches!(node.children(), Some(TreeChildren::Deferred)));
    }

    #[test]
    fn builders_store_row_metadata() {
        let node = TreeNode::leaf(1, "README.md")
            .leading_icon(IconRole::Folder)
            .tone(ToneRole::Info)
            .trailing_text("4 KB")
            .disabled(true);

        assert_eq!(node.leading_icon_name(), Some(IconRole::Folder));
        assert_eq!(node.tone_role(), Some(ToneRole::Info));
        assert_eq!(node.trailing_text_value(), Some("4 KB"));
        assert!(node.is_disabled());
    }
}
