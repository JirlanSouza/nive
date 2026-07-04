use iced::keyboard::Modifiers;
use nive_ui::interaction::{
    CollectionTransferPayload, Drag, DropCommit, DropContext, DropDecision, TransferData,
    TransferOperation, TransferOperations,
};
use nive_ui::Point;

#[test]
fn component_library_payload_can_drop_on_canvas_target() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum LibraryPayload {
        Component(&'static str),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Origin {
        LibraryTree,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum CanvasTarget {
        Point(Point),
    }

    let drag = Drag {
        payload: TransferData::local(LibraryPayload::Component("button")),
        origin: Origin::LibraryTree,
        operations: TransferOperations::COPY,
        preferred: TransferOperation::Copy,
    };
    let context = DropContext {
        payload: &drag.payload,
        origin: &drag.origin,
        operations: drag.operations,
        preferred: drag.preferred,
        requested: None,
        position: Point::new(12.0, 18.0),
        modifiers: Modifiers::NONE,
    };
    let decision = DropDecision::accept(
        CanvasTarget::Point(context.position),
        TransferOperation::Copy,
    );
    let DropDecision::Accept { target, operation } = decision else {
        panic!("canvas target should accept component payload");
    };
    let position = context.position;
    let commit = DropCommit {
        payload: drag.payload,
        origin: drag.origin,
        target,
        operation,
        position,
    };

    assert_eq!(commit.operation, TransferOperation::Copy);
}

#[test]
fn collection_reorder_uses_shared_drag_and_drop_flow() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum ListTarget {
        Before(u32),
    }

    let payload = CollectionTransferPayload::flat([2, 3]);
    let drag = Drag {
        payload: TransferData::local(payload),
        origin: "list",
        operations: TransferOperations::MOVE,
        preferred: TransferOperation::Move,
    };
    let context = DropContext {
        payload: &drag.payload,
        origin: &drag.origin,
        operations: drag.operations,
        preferred: drag.preferred,
        requested: Some(TransferOperation::Move),
        position: Point::ORIGIN,
        modifiers: Modifiers::SHIFT,
    };
    let decision = DropDecision::accept(ListTarget::Before(4), TransferOperation::Move);
    let DropDecision::Accept { target, operation } = decision else {
        panic!("list target should accept reorder payload");
    };
    let position = context.position;
    let commit = DropCommit {
        payload: drag.payload,
        origin: drag.origin,
        target,
        operation,
        position,
    };

    assert!(matches!(commit.payload, TransferData::Local(_)));
    assert_eq!(commit.target, ListTarget::Before(4));
    assert_eq!(commit.operation, TransferOperation::Move);
}

#[test]
fn drag_drop_callback_shape_compiles_without_widget_builders() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum ListTarget {
        After(u32),
    }

    #[derive(Debug, Clone, PartialEq)]
    enum Message {
        Dropped(TransferOperation),
    }

    accepts_drag_drop_callbacks(
        || {
            Some(Drag {
                payload: TransferData::local(CollectionTransferPayload::flat([1, 2])),
                origin: "tree",
                operations: TransferOperations::MOVE,
                preferred: TransferOperation::Move,
            })
        },
        |context| {
            let operation = context.preferred_operation().expect("allowed operation");

            DropDecision::accept(ListTarget::After(3), operation)
        },
        |commit| Message::Dropped(commit.operation),
    );
}

fn accepts_drag_drop_callbacks<Payload, Origin, Target, Message>(
    _drag_source: impl Fn() -> Option<Drag<Payload, Origin>>,
    _drop_target: impl for<'a> Fn(DropContext<'a, Payload, Origin>) -> DropDecision<Target>,
    _on_drop: impl Fn(DropCommit<Payload, Origin, Target>) -> Message,
) {
}
