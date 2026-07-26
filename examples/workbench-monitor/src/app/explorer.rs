//! Left-panel Tree explorer: hosts grouping their services, plus a
//! `diagnostics` branch that always fails to load, demonstrating
//! `TreeChildren::Failed` and the canonical retry row. Context requests are
//! intent only: this module hosts the canonical `Menu` at the request
//! position, matching the boundary Tree itself never crosses.

use std::time::Duration;

use nive::prelude::*;
#[cfg(test)]
use nive::ui::interaction::{ContextInvocation, ContextRequest, SelectionSnapshot};
use nive::ui::interaction::{ContextPosition, ContextTarget, SelectionMode};

use super::tone::tone_label;
use super::{Message, Selection, WorkbenchMonitor};
use crate::icons::IconSymbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ExplorerNodeId {
    Host(&'static str),
    Service(&'static str),
    Diagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ExplorerContextMenuState {
    pub(crate) target: ExplorerNodeId,
    pub(crate) position: Point,
}

/// Simulated diagnostics-collector failure, projected through the shared
/// `ErrorPresentation` contract the way a real `UserFacingError` would be.
struct ExplorerDiagnosticsError;

impl ErrorPresentation for ExplorerDiagnosticsError {
    fn summary(&self) -> &str {
        "Diagnostics collector unreachable"
    }

    fn detail(&self) -> &str {
        "Diagnostics collector unreachable: fleet-diagnostics.internal timed out after 3 retries"
    }
}

impl WorkbenchMonitor {
    fn explorer_nodes(&self) -> Vec<TreeNode<'static, ExplorerNodeId>> {
        let mut roots: Vec<TreeNode<'static, ExplorerNodeId>> = self
            .model
            .hosts
            .iter()
            .map(|host| {
                let services = self
                    .model
                    .services
                    .iter()
                    .filter(|service| service.host_id == host.id)
                    .map(|service| {
                        TreeNode::leaf(ExplorerNodeId::Service(service.id), service.name)
                            .leading_icon(IconSymbol::Server)
                            .status_text(service.health, tone_label(service.health))
                    })
                    .collect::<Vec<_>>();

                TreeNode::branch(ExplorerNodeId::Host(host.id), host.name, services)
                    .leading_icon(IconSymbol::Server)
                    .status_text(host.health, tone_label(host.health))
            })
            .collect();

        let diagnostics = if self.explorer_diagnostics_failed {
            TreeNode::branch_failed(
                ExplorerNodeId::Diagnostics,
                "diagnostics",
                &ExplorerDiagnosticsError,
            )
        } else {
            TreeNode::branch_deferred(ExplorerNodeId::Diagnostics, "diagnostics").trailing_text(
                if self.explorer_diagnostics_loading {
                    "loading"
                } else {
                    "deferred"
                },
            )
        };
        roots.push(diagnostics.leading_icon(IconSymbol::Diagnostics));

        roots
    }

    pub(super) fn explorer_view(&self) -> Element<'_, Message> {
        let tree = Tree::new(self.explorer_nodes())
            .state(&self.explorer)
            .selection_mode(SelectionMode::Single)
            .height(Length::Fill)
            .on_event(Message::ExplorerEvent);

        self.explorer_with_context_menu(tree.into())
    }

    /// Hosts the canonical `Menu` at the captured context-request position.
    /// Tree itself owns no menu; it only emits `ContextRequested`.
    fn explorer_with_context_menu<'a>(
        &'a self,
        tree: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let Some(menu) = self.explorer_context_menu else {
            return tree;
        };

        let anchor = nive::widget::Space::new()
            .width(Length::Fixed(0.0))
            .height(Length::Fixed(0.0));

        let mut hosted = Menu::new(anchor)
            .open(true)
            .on_dismiss(Message::ExplorerContextDismissed)
            .command(
                MenuCommand::new("Inspect").on_press(Message::ExplorerContextAction("Inspect")),
            );
        if matches!(menu.target, ExplorerNodeId::Service(_)) {
            hosted = hosted.command(
                MenuCommand::new("Open document").on_press(Message::ExplorerContextAction("Open")),
            );
        }
        let hosted: Element<'a, Message> = hosted.into();

        let overlay = nive::widget::container(hosted)
            .padding(Padding {
                top: menu.position.y,
                right: 0.0,
                bottom: 0.0,
                left: menu.position.x,
            })
            .width(Length::Fill)
            .height(Length::Fill);

        nive::widget::stack![tree, overlay].into()
    }

    pub(super) fn apply_explorer_event(
        &mut self,
        event: TreeEvent<ExplorerNodeId>,
    ) -> Option<Task<Message>> {
        self.explorer.apply(&event);

        match &event.kind {
            TreeEventKind::ExpandRequested {
                id: ExplorerNodeId::Diagnostics,
            } => {
                if self.explorer_diagnostics_loading {
                    None
                } else {
                    self.explorer_diagnostics_loading = true;
                    self.explorer_diagnostics_failed = false;
                    Some(Task::perform(
                        async {
                            std::thread::sleep(Duration::from_millis(700));
                        },
                        |()| Message::ExplorerDiagnosticsFailed,
                    ))
                }
            }
            TreeEventKind::Activate { id, .. } => {
                match id {
                    ExplorerNodeId::Service(service_id) => {
                        self.select(Selection::Service(service_id))
                    }
                    ExplorerNodeId::Host(host_id) => self.select(Selection::Host(host_id)),
                    ExplorerNodeId::Diagnostics => {}
                }
                None
            }
            TreeEventKind::ContextRequested(request) => {
                self.explorer_context_menu = match (&request.target, request.position) {
                    (ContextTarget::Item(id), ContextPosition::Pointer(position)) => {
                        Some(ExplorerContextMenuState {
                            target: *id,
                            position,
                        })
                    }
                    _ => None,
                };
                None
            }
            _ => None,
        }
    }

    pub(super) fn apply_explorer_diagnostics_failed(&mut self) {
        self.explorer_diagnostics_loading = false;
        self.explorer_diagnostics_failed = true;
    }

    pub(super) fn apply_explorer_context_action(&mut self, action: &'static str) {
        if let Some(menu) = self.explorer_context_menu.take() {
            match (action, menu.target) {
                ("Inspect", ExplorerNodeId::Service(id)) => self.select(Selection::Service(id)),
                ("Inspect", ExplorerNodeId::Host(id)) => self.select(Selection::Host(id)),
                ("Open", ExplorerNodeId::Service(id)) => {
                    self.open_document(super::DocumentId::Service(id));
                    self.select(Selection::Service(id));
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explorer_nodes_group_services_under_their_host() {
        let app = WorkbenchMonitor::seeded();
        let nodes = app.explorer_nodes();

        let host = nodes
            .iter()
            .find(|node| *node.id() == ExplorerNodeId::Host("edge-01"))
            .expect("seeded edge-01 host node");
        let Some(TreeChildren::Loaded(children)) = host.children() else {
            panic!("expected loaded host children");
        };
        assert!(children
            .iter()
            .any(|child| *child.id() == ExplorerNodeId::Service("api")));
    }

    #[test]
    fn diagnostics_branch_starts_deferred_then_fails_and_can_retry() {
        let mut app = WorkbenchMonitor::seeded();
        assert!(matches!(
            app.explorer_nodes()
                .into_iter()
                .find(|node| *node.id() == ExplorerNodeId::Diagnostics)
                .and_then(|node| node.children().cloned()),
            Some(TreeChildren::Deferred)
        ));

        let expand = TreeEvent {
            state_change: None,
            kind: TreeEventKind::ExpandRequested {
                id: ExplorerNodeId::Diagnostics,
            },
        };
        assert!(app.apply_explorer_event(expand.clone()).is_some());
        assert!(app.explorer_diagnostics_loading);

        app.apply_explorer_diagnostics_failed();
        assert!(!app.explorer_diagnostics_loading);
        assert!(app.explorer_diagnostics_failed);
        assert!(matches!(
            app.explorer_nodes()
                .into_iter()
                .find(|node| *node.id() == ExplorerNodeId::Diagnostics)
                .and_then(|node| node.children().cloned()),
            Some(TreeChildren::Failed { .. })
        ));

        // Retry re-emits the same expand intent and resets to loading.
        assert!(app.apply_explorer_event(expand).is_some());
        assert!(app.explorer_diagnostics_loading);
        assert!(!app.explorer_diagnostics_failed);
    }

    #[test]
    fn context_request_hosts_menu_and_action_updates_selection() {
        let mut app = WorkbenchMonitor::seeded();
        let context = TreeEvent {
            state_change: None,
            kind: TreeEventKind::ContextRequested(ContextRequest {
                target: ContextTarget::Item(ExplorerNodeId::Service("api")),
                selection: SelectionSnapshot {
                    selected: vec![],
                    focused: None,
                    anchor: None,
                },
                position: ContextPosition::Pointer(Point::new(12.0, 34.0)),
                invocation: ContextInvocation::SecondaryClick,
            }),
        };
        app.apply_explorer_event(context);
        assert_eq!(
            app.explorer_context_menu,
            Some(ExplorerContextMenuState {
                target: ExplorerNodeId::Service("api"),
                position: Point::new(12.0, 34.0),
            })
        );

        app.apply_explorer_context_action("Inspect");
        assert_eq!(app.selected, Selection::Service("api"));
        assert!(app.explorer_context_menu.is_none());
    }
}
