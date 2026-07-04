use std::time::Duration;

use nive::prelude::*;
use nive::ui::widgets::{TreeDropTarget, TreeEvent, TreeEventKind, TreePasteTarget};
use nive::ui::{
    CollectionTransferPayload, ContextInvocation, ContextRequest, ContextTarget, Transfer,
    TransferOperation,
};

use crate::app::{DemoTreeNode, LayoutState, Message};

pub fn handle_tree_event(
    layout: &mut LayoutState,
    event: TreeEvent<DemoTreeNode>,
) -> Option<Task<Message>> {
    layout.tree_state.apply(&event);
    layout.tree_event_feedback = describe_tree_event(&event.kind);
    layout.tree_drop_feedback = describe_tree_transfer(&layout.tree_state.transfer);

    match &event.kind {
        TreeEventKind::ExpandRequested { id }
            if *id == DemoTreeNode::RemotePackages && !layout.tree_deferred_loaded =>
        {
            if layout.tree_deferred_loading {
                None
            } else {
                layout.tree_deferred_loading = true;
                layout.tree_event_feedback = "Loading remote-packages...".to_owned();
                Some(load_deferred_tree_branch())
            }
        }
        TreeEventKind::ContextRequested(request) => {
            layout.tree_context_feedback = describe_context_request(request);
            None
        }
        TreeEventKind::CopyRequested(payload) => {
            layout.tree_clipboard_feedback = format!("Copy: {}", describe_payload(payload));
            None
        }
        TreeEventKind::CutRequested(payload) => {
            layout.tree_clipboard_feedback = format!("Cut: {}", describe_payload(payload));
            None
        }
        TreeEventKind::PasteRequested(target) => {
            layout.tree_clipboard_feedback = format!("Paste: {}", describe_paste_target(target));
            None
        }
        TreeEventKind::DropRequested(drop) => {
            layout.tree_drop_feedback = format!(
                "Drop: {} {}",
                describe_payload(&drop.payload),
                describe_drop_target(&drop.target)
            );
            None
        }
        _ => None,
    }
}

pub fn load_deferred_tree_branch() -> Task<Message> {
    Task::perform(
        async {
            std::thread::sleep(Duration::from_millis(900));
            DemoTreeNode::RemotePackages
        },
        Message::TreeDeferredLoaded,
    )
}

fn describe_tree_event(event: &TreeEventKind<DemoTreeNode>) -> String {
    match event {
        TreeEventKind::StateChanged => "State changed".to_owned(),
        TreeEventKind::ExpandRequested { id } => format!("Expand requested: {}", id.label()),
        TreeEventKind::Activate { id, trigger } => {
            format!("Activate: {} via {trigger:?}", id.label())
        }
        TreeEventKind::RenameRequested { id } => format!("Rename requested: {}", id.label()),
        TreeEventKind::ContextRequested(_) => "Context requested".to_owned(),
        TreeEventKind::CopyRequested(payload) => format!("Copy requested: {}", describe_payload(payload)),
        TreeEventKind::CutRequested(payload) => format!("Cut requested: {}", describe_payload(payload)),
        TreeEventKind::PasteRequested(target) => {
            format!("Paste requested: {}", describe_paste_target(target))
        }
        TreeEventKind::DropRequested(drop) => {
            format!("Drop requested: {}", describe_drop_target(&drop.target))
        }
        _ => "Tree event".to_owned(),
    }
}

fn describe_context_request(request: &ContextRequest<DemoTreeNode>) -> String {
    let target = match &request.target {
        ContextTarget::Item(id) => id.label(),
        ContextTarget::Empty => "empty space",
        _ => "unknown target",
    };
    let source = match request.invocation {
        ContextInvocation::SecondaryClick => "secondary click",
        ContextInvocation::Keyboard => "keyboard",
        _ => "unknown source",
    };

    format!(
        "Context: {target} via {source}, selected {}",
        describe_nodes(&request.selection.selected)
    )
}

fn describe_payload(payload: &CollectionTransferPayload<DemoTreeNode>) -> String {
    format!(
        "{} (roots: {})",
        describe_nodes(&payload.ids),
        describe_nodes(&payload.root_ids)
    )
}

fn describe_nodes(nodes: &[DemoTreeNode]) -> String {
    if nodes.is_empty() {
        "none".to_owned()
    } else {
        nodes
            .iter()
            .map(|id| id.label())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn describe_paste_target(target: &TreePasteTarget<DemoTreeNode>) -> String {
    match target {
        TreePasteTarget::Into(id) => format!("into {}", id.label()),
        TreePasteTarget::Root => "at root".to_owned(),
    }
}

fn describe_drop_target(target: &TreeDropTarget<DemoTreeNode>) -> String {
    match target {
        TreeDropTarget::Before(id) => format!("before {}", id.label()),
        TreeDropTarget::After(id) => format!("after {}", id.label()),
        TreeDropTarget::Into(id) => format!("into {}", id.label()),
        TreeDropTarget::Root => "at root".to_owned(),
    }
}

fn describe_tree_transfer(
    transfer: &Transfer<DemoTreeNode, TreeDropTarget<DemoTreeNode>>,
) -> String {
    match transfer {
        Transfer::None => "Transfer: idle".to_owned(),
        Transfer::Clipboard { payload, operation } => format!(
            "Clipboard feedback: {} {}",
            describe_operation(*operation),
            describe_payload(payload)
        ),
        Transfer::Dragging {
            payload,
            operation,
            target,
        } => {
            let target = target
                .as_ref()
                .map(describe_drop_target)
                .unwrap_or_else(|| "no valid target".to_owned());
            format!(
                "Dragging: {} {} toward {target}",
                describe_operation(*operation),
                describe_payload(payload)
            )
        }
    }
}

fn describe_operation(operation: TransferOperation) -> &'static str {
    match operation {
        TransferOperation::Copy => "copy",
        TransferOperation::Move => "move",
        TransferOperation::Link => "link",
        _ => "transfer",
    }
}
