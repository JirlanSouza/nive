use std::borrow::Cow;

use crate::widgets::{IconRole, StatusIndicator};

use super::{TreeChildren, TreeNode, TreeState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum VisibleTreeEntry<'a, Id> {
    Row(VisibleTreeRow<'a, Id>),
    Loading(VisibleLoadingRow<Id>),
    Failed(VisibleFailedRow<'a, Id>),
    Empty(VisibleEmptyRow<Id>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VisibleTreeRow<'a, Id> {
    pub(crate) id: Id,
    pub(crate) depth: usize,
    pub(crate) parent: Option<Id>,
    pub(crate) visible_index: usize,
    pub(crate) expanded: Option<bool>,
    pub(crate) disabled: bool,
    pub(crate) label: Cow<'a, str>,
    pub(crate) leading_icon: Option<IconRole>,
    pub(crate) status: Option<StatusIndicator<'a>>,
    pub(crate) trailing_text: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VisibleLoadingRow<Id> {
    pub(crate) parent: Id,
    pub(crate) depth: usize,
    pub(crate) visible_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VisibleFailedRow<'a, Id> {
    pub(crate) parent: Id,
    pub(crate) depth: usize,
    pub(crate) visible_index: usize,
    pub(crate) summary: Cow<'a, str>,
    pub(crate) detail: Cow<'a, str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VisibleEmptyRow<Id> {
    pub(crate) parent: Id,
    pub(crate) depth: usize,
    pub(crate) visible_index: usize,
}

pub(crate) fn visible_entries<'a, Id>(
    roots: &[TreeNode<'a, Id>],
    state: &TreeState<Id>,
) -> Vec<VisibleTreeEntry<'a, Id>>
where
    Id: Clone + Ord,
{
    let mut entries = Vec::new();
    push_visible_nodes(roots, state, None, 0, &mut entries);
    entries
}

fn push_visible_nodes<'a, Id>(
    nodes: &[TreeNode<'a, Id>],
    state: &TreeState<Id>,
    parent: Option<&Id>,
    depth: usize,
    entries: &mut Vec<VisibleTreeEntry<'a, Id>>,
) where
    Id: Clone + Ord,
{
    for node in nodes {
        let children = node.children();
        let expanded = children.map(|_| state.is_expanded(node.id()));
        let row = VisibleTreeRow {
            id: node.id().clone(),
            depth,
            parent: parent.cloned(),
            visible_index: entries.len(),
            expanded,
            disabled: node.is_disabled(),
            label: Cow::Owned(node.label().to_owned()),
            leading_icon: node.leading_icon_name(),
            status: node.status().cloned(),
            trailing_text: node
                .trailing_text_value()
                .map(|trailing| Cow::Owned(trailing.to_owned())),
        };

        entries.push(VisibleTreeEntry::Row(row));

        if expanded != Some(true) {
            continue;
        }

        match children {
            Some(TreeChildren::Loaded(children)) if children.is_empty() => {
                entries.push(VisibleTreeEntry::Empty(VisibleEmptyRow {
                    parent: node.id().clone(),
                    depth: depth + 1,
                    visible_index: entries.len(),
                }));
            }
            Some(TreeChildren::Loaded(children)) => {
                push_visible_nodes(children, state, Some(node.id()), depth + 1, entries);
            }
            Some(TreeChildren::Deferred) => {
                entries.push(VisibleTreeEntry::Loading(VisibleLoadingRow {
                    parent: node.id().clone(),
                    depth: depth + 1,
                    visible_index: entries.len(),
                }));
            }
            Some(TreeChildren::Failed { summary, detail }) => {
                entries.push(VisibleTreeEntry::Failed(VisibleFailedRow {
                    parent: node.id().clone(),
                    depth: depth + 1,
                    visible_index: entries.len(),
                    summary: summary.clone(),
                    detail: detail.clone(),
                }));
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod visible_tree_tests {
    use super::*;
    use crate::widgets::IconRole;

    type TestNode = TreeNode<'static, &'static str>;

    fn expanded_state(ids: impl IntoIterator<Item = &'static str>) -> TreeState<&'static str> {
        let mut state = TreeState::default();

        for id in ids {
            state.expand(id);
        }

        state
    }

    fn row_ids(entries: &[VisibleTreeEntry<'_, &'static str>]) -> Vec<&'static str> {
        entries
            .iter()
            .filter_map(|entry| match entry {
                VisibleTreeEntry::Row(row) => Some(row.id),
                VisibleTreeEntry::Loading(_)
                | VisibleTreeEntry::Failed(_)
                | VisibleTreeEntry::Empty(_) => None,
            })
            .collect()
    }

    fn assert_row(
        entry: &VisibleTreeEntry<'_, &'static str>,
        id: &'static str,
        depth: usize,
        parent: Option<&'static str>,
        visible_index: usize,
        expanded: Option<bool>,
    ) {
        let VisibleTreeEntry::Row(row) = entry else {
            panic!("expected row entry for {id}");
        };

        assert_eq!(row.id, id);
        assert_eq!(row.depth, depth);
        assert_eq!(row.parent, parent);
        assert_eq!(row.visible_index, visible_index);
        assert_eq!(row.expanded, expanded);
    }

    fn assert_loading(
        entry: &VisibleTreeEntry<'_, &'static str>,
        parent: &'static str,
        depth: usize,
        visible_index: usize,
    ) {
        let VisibleTreeEntry::Loading(row) = entry else {
            panic!("expected loading entry for {parent}");
        };

        assert_eq!(row.parent, parent);
        assert_eq!(row.depth, depth);
        assert_eq!(row.visible_index, visible_index);
    }

    fn assert_failed(
        entry: &VisibleTreeEntry<'_, &'static str>,
        parent: &'static str,
        depth: usize,
        visible_index: usize,
        summary: &str,
    ) {
        let VisibleTreeEntry::Failed(row) = entry else {
            panic!("expected failed entry for {parent}");
        };

        assert_eq!(row.parent, parent);
        assert_eq!(row.depth, depth);
        assert_eq!(row.visible_index, visible_index);
        assert_eq!(row.summary.as_ref(), summary);
    }

    fn assert_empty(
        entry: &VisibleTreeEntry<'_, &'static str>,
        parent: &'static str,
        depth: usize,
        visible_index: usize,
    ) {
        let VisibleTreeEntry::Empty(row) = entry else {
            panic!("expected empty entry for {parent}");
        };

        assert_eq!(row.parent, parent);
        assert_eq!(row.depth, depth);
        assert_eq!(row.visible_index, visible_index);
    }

    #[test]
    fn collapsed_branch_hides_descendants() {
        let nodes = vec![
            TreeNode::branch(
                "root",
                "Root",
                [
                    TreeNode::leaf("child", "Child"),
                    TreeNode::branch(
                        "nested",
                        "Nested",
                        [TreeNode::leaf("grandchild", "Grandchild")],
                    ),
                ],
            ),
            TreeNode::leaf("sibling", "Sibling"),
        ];
        let state = TreeState::default();

        let entries = visible_entries(&nodes, &state);

        assert_eq!(row_ids(&entries), vec!["root", "sibling"]);
        assert_row(&entries[0], "root", 0, None, 0, Some(false));
        assert_row(&entries[1], "sibling", 0, None, 1, None);
    }

    #[test]
    fn expanded_nested_branches_preserve_pre_order_depth_parent_and_index() {
        let nodes = vec![
            TreeNode::branch(
                "root",
                "Root",
                [
                    TreeNode::branch(
                        "src",
                        "src",
                        [
                            TreeNode::leaf("main", "main.rs"),
                            TreeNode::leaf("lib", "lib.rs"),
                        ],
                    ),
                    TreeNode::leaf("tests", "tests"),
                ],
            ),
            TreeNode::leaf("target", "target"),
        ];
        let state = expanded_state(["root", "src"]);

        let entries = visible_entries(&nodes, &state);

        assert_eq!(
            row_ids(&entries),
            vec!["root", "src", "main", "lib", "tests", "target"]
        );
        assert_row(&entries[0], "root", 0, None, 0, Some(true));
        assert_row(&entries[1], "src", 1, Some("root"), 1, Some(true));
        assert_row(&entries[2], "main", 2, Some("src"), 2, None);
        assert_row(&entries[3], "lib", 2, Some("src"), 3, None);
        assert_row(&entries[4], "tests", 1, Some("root"), 4, None);
        assert_row(&entries[5], "target", 0, None, 5, None);
    }

    #[test]
    fn empty_loaded_branch_renders_one_canonical_empty_row() {
        let nodes = vec![
            TreeNode::branch("empty", "Empty", Vec::<TestNode>::new()),
            TreeNode::leaf("after", "After"),
        ];
        let state = expanded_state(["empty"]);

        let entries = visible_entries(&nodes, &state);

        assert_eq!(entries.len(), 3);
        assert_eq!(row_ids(&entries), vec!["empty", "after"]);
        assert_row(&entries[0], "empty", 0, None, 0, Some(true));
        assert_empty(&entries[1], "empty", 1, 1);
        assert_row(&entries[2], "after", 0, None, 2, None);
    }

    #[test]
    fn expanded_deferred_branch_inserts_one_loading_placeholder() {
        let nodes = vec![
            TreeNode::branch_deferred("remote", "Remote"),
            TreeNode::leaf("after", "After"),
        ];
        let state = expanded_state(["remote"]);

        let entries = visible_entries(&nodes, &state);

        assert_eq!(entries.len(), 3);
        assert_eq!(row_ids(&entries), vec!["remote", "after"]);
        assert_row(&entries[0], "remote", 0, None, 0, Some(true));
        assert_loading(&entries[1], "remote", 1, 1);
        assert_row(&entries[2], "after", 0, None, 2, None);
    }

    struct TestError {
        summary: &'static str,
    }

    impl nive_core::ErrorPresentation for TestError {
        fn summary(&self) -> &str {
            self.summary
        }

        fn detail(&self) -> &str {
            self.summary
        }
    }

    #[test]
    fn expanded_failed_branch_inserts_one_error_row() {
        let error = TestError {
            summary: "Load failed",
        };
        let nodes = vec![
            TreeNode::branch_failed("remote", "Remote", &error),
            TreeNode::leaf("after", "After"),
        ];
        let state = expanded_state(["remote"]);

        let entries = visible_entries(&nodes, &state);

        assert_eq!(entries.len(), 3);
        assert_eq!(row_ids(&entries), vec!["remote", "after"]);
        assert_row(&entries[0], "remote", 0, None, 0, Some(true));
        assert_failed(&entries[1], "remote", 1, 1, "Load failed");
        assert_row(&entries[2], "after", 0, None, 2, None);
    }

    #[test]
    fn disabled_rows_and_display_metadata_remain_visible() {
        let nodes = vec![TreeNode::leaf("readme", "README.md")
            .leading_icon(IconRole::Folder)
            .status_text(crate::theme::ToneRole::Warning, "Warning")
            .trailing_text("4 KB")
            .disabled(true)];
        let state = TreeState::default();

        let entries = visible_entries(&nodes, &state);

        assert_eq!(entries.len(), 1);
        let VisibleTreeEntry::Row(row) = &entries[0] else {
            panic!("expected disabled node row");
        };
        assert_eq!(row.id, "readme");
        assert_eq!(row.label.as_ref(), "README.md");
        assert_eq!(row.depth, 0);
        assert_eq!(row.parent, None);
        assert_eq!(row.visible_index, 0);
        assert_eq!(row.expanded, None);
        assert!(row.disabled);
        assert_eq!(row.leading_icon, Some(IconRole::Folder));
        assert_eq!(
            row.status.as_ref().map(StatusIndicator::tone),
            Some(crate::theme::ToneRole::Warning)
        );
        assert_eq!(row.trailing_text.as_deref(), Some("4 KB"));
    }
}
