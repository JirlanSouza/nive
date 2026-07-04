use iced::{keyboard, mouse, Point};

use super::{Drag, DropCommit, DropContext, DropDecision, TransferOperation, TransferOperations};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct DragSession<T, Origin, Target> {
    phase: DragPhase<T, Origin, Target>,
    drag_threshold: f32,
    modifiers: keyboard::Modifiers,
}

impl<T, Origin, Target> Default for DragSession<T, Origin, Target> {
    fn default() -> Self {
        Self {
            phase: DragPhase::Idle,
            drag_threshold: 4.0,
            modifiers: keyboard::Modifiers::NONE,
        }
    }
}

#[allow(dead_code)]
impl<T, Origin, Target> DragSession<T, Origin, Target> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn with_drag_threshold(mut self, threshold: f32) -> Self {
        self.drag_threshold = threshold.max(0.0);
        self
    }

    pub(crate) fn begin(&mut self, position: Point) {
        self.phase = DragPhase::Pending { origin: position };
    }

    pub(crate) fn update_modifiers(&mut self, modifiers: keyboard::Modifiers) {
        self.modifiers = modifiers;
        if let DragPhase::Active { requested, .. } = &mut self.phase {
            *requested = requested_operation_from_modifiers(modifiers);
        }
    }

    pub(crate) fn move_to(
        &mut self,
        position: Point,
        source: impl FnOnce() -> Option<Drag<T, Origin>>,
        target: impl FnOnce(DropContext<'_, T, Origin>) -> DropDecision<Target>,
    ) -> DragSessionFeedback
    where
        Target: Clone,
    {
        if let DragPhase::Pending { origin } = self.phase {
            if distance(origin, position) < self.drag_threshold {
                return DragSessionFeedback::Pending;
            }

            let Some(drag) = source() else {
                self.phase = DragPhase::Idle;
                return DragSessionFeedback::Rejected;
            };

            self.phase = DragPhase::Active {
                drag,
                target: None,
                requested: requested_operation_from_modifiers(self.modifiers),
                cancelled: false,
            };
        }

        let DragPhase::Active {
            drag,
            target: current_target,
            requested,
            cancelled,
        } = &mut self.phase
        else {
            return DragSessionFeedback::Idle;
        };

        if *cancelled {
            return DragSessionFeedback::Rejected;
        }

        let context = DropContext {
            payload: &drag.payload,
            origin: &drag.origin,
            operations: drag.operations,
            preferred: drag.preferred,
            requested: *requested,
            position,
            modifiers: self.modifiers,
        };
        let decision = target(context);
        *current_target = accepted_target(drag.operations, decision);

        DragSessionFeedback::from_target(current_target.as_ref().map(|target| target.operation))
    }

    pub(crate) fn release(&mut self, position: Point) -> Option<DropCommit<T, Origin, Target>> {
        let phase = std::mem::replace(&mut self.phase, DragPhase::Idle);
        let DragPhase::Active {
            drag,
            target: Some(target),
            cancelled: false,
            ..
        } = phase
        else {
            return None;
        };

        Some(DropCommit {
            payload: drag.payload,
            origin: drag.origin,
            target: target.target,
            operation: target.operation,
            position,
        })
    }

    pub(crate) fn cancel(&mut self) {
        if let DragPhase::Active { cancelled, .. } = &mut self.phase {
            *cancelled = true;
        } else {
            self.phase = DragPhase::Idle;
        }
    }

    pub(crate) fn mouse_interaction(&self) -> mouse::Interaction {
        let DragPhase::Active {
            target, cancelled, ..
        } = &self.phase
        else {
            return mouse::Interaction::None;
        };

        if *cancelled {
            return mouse::Interaction::NoDrop;
        }

        match target.as_ref().map(|target| target.operation) {
            Some(TransferOperation::Copy) => mouse::Interaction::Copy,
            Some(TransferOperation::Move) => mouse::Interaction::Move,
            Some(TransferOperation::Link) => mouse::Interaction::Alias,
            None => mouse::Interaction::NoDrop,
        }
    }

    pub(crate) fn grabbing_interaction(&self) -> mouse::Interaction {
        if matches!(
            self.phase,
            DragPhase::Pending { .. } | DragPhase::Active { .. }
        ) {
            mouse::Interaction::Grabbing
        } else {
            mouse::Interaction::Grab
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragSessionFeedback {
    Idle,
    Pending,
    Rejected,
    Accepted(TransferOperation),
}

#[allow(dead_code)]
impl DragSessionFeedback {
    fn from_target(operation: Option<TransferOperation>) -> Self {
        operation.map_or(Self::Rejected, Self::Accepted)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum DragPhase<T, Origin, Target> {
    Idle,
    Pending {
        origin: Point,
    },
    Active {
        drag: Drag<T, Origin>,
        target: Option<AcceptedDrop<Target>>,
        requested: Option<TransferOperation>,
        cancelled: bool,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct AcceptedDrop<Target> {
    target: Target,
    operation: TransferOperation,
}

#[allow(dead_code)]
fn accepted_target<Target>(
    operations: TransferOperations,
    decision: DropDecision<Target>,
) -> Option<AcceptedDrop<Target>> {
    match decision {
        DropDecision::Reject => None,
        DropDecision::Accept { target, operation } if operations.contains(operation) => {
            Some(AcceptedDrop { target, operation })
        }
        DropDecision::Accept { .. } => None,
    }
}

#[allow(dead_code)]
pub(crate) fn requested_operation_from_modifiers(
    modifiers: keyboard::Modifiers,
) -> Option<TransferOperation> {
    if modifiers.alt() {
        Some(TransferOperation::Link)
    } else if modifiers.command() {
        Some(TransferOperation::Copy)
    } else if modifiers.shift() {
        Some(TransferOperation::Move)
    } else {
        None
    }
}

#[allow(dead_code)]
fn distance(a: Point, b: Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;

    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod dnd_tests {
    use super::super::TransferData;
    use super::*;

    fn drag(operations: TransferOperations, preferred: TransferOperation) -> Drag<&'static str> {
        Drag {
            payload: TransferData::local("payload"),
            origin: (),
            operations,
            preferred,
        }
    }

    #[test]
    fn successful_release_emits_commit() {
        let mut session = DragSession::new().with_drag_threshold(1.0);

        session.begin(Point::ORIGIN);
        let feedback = session.move_to(
            Point::new(2.0, 0.0),
            || Some(drag(TransferOperations::MOVE, TransferOperation::Move)),
            |context| {
                DropDecision::accept("target", context.preferred_operation().expect("operation"))
            },
        );
        let commit = session.release(Point::new(2.0, 0.0)).expect("commit");

        assert_eq!(
            feedback,
            DragSessionFeedback::Accepted(TransferOperation::Move)
        );
        assert_eq!(commit.target, "target");
        assert_eq!(commit.operation, TransferOperation::Move);
    }

    #[test]
    fn rejected_target_emits_no_commit() {
        let mut session = DragSession::new().with_drag_threshold(1.0);

        session.begin(Point::ORIGIN);
        let feedback = session.move_to(
            Point::new(2.0, 0.0),
            || Some(drag(TransferOperations::MOVE, TransferOperation::Move)),
            |_| DropDecision::<&'static str>::Reject,
        );

        assert_eq!(feedback, DragSessionFeedback::Rejected);
        assert!(session.release(Point::new(2.0, 0.0)).is_none());
    }

    #[test]
    fn disallowed_target_operation_emits_no_commit() {
        let mut session = DragSession::new().with_drag_threshold(1.0);

        session.begin(Point::ORIGIN);
        let feedback = session.move_to(
            Point::new(2.0, 0.0),
            || Some(drag(TransferOperations::MOVE, TransferOperation::Move)),
            |_| DropDecision::accept("target", TransferOperation::Copy),
        );

        assert_eq!(feedback, DragSessionFeedback::Rejected);
        assert_eq!(session.mouse_interaction(), mouse::Interaction::NoDrop);
        assert!(session.release(Point::new(2.0, 0.0)).is_none());
    }

    #[test]
    fn falls_back_to_preferred_operation() {
        let mut session = DragSession::new().with_drag_threshold(1.0);

        session.begin(Point::ORIGIN);
        let feedback = session.move_to(
            Point::new(2.0, 0.0),
            || {
                Some(drag(
                    TransferOperations::COPY | TransferOperations::MOVE,
                    TransferOperation::Move,
                ))
            },
            |context| {
                DropDecision::accept("target", context.preferred_operation().expect("operation"))
            },
        );

        assert_eq!(
            feedback,
            DragSessionFeedback::Accepted(TransferOperation::Move)
        );
    }

    #[test]
    fn modifier_requested_operation_is_visible_to_target() {
        let mut session = DragSession::new().with_drag_threshold(1.0);

        session.update_modifiers(keyboard::Modifiers::COMMAND);
        session.begin(Point::ORIGIN);
        let feedback = session.move_to(
            Point::new(2.0, 0.0),
            || {
                Some(drag(
                    TransferOperations::COPY | TransferOperations::MOVE,
                    TransferOperation::Move,
                ))
            },
            |context| DropDecision::accept("target", context.requested.expect("requested")),
        );

        assert_eq!(
            feedback,
            DragSessionFeedback::Accepted(TransferOperation::Copy)
        );
        assert_eq!(session.mouse_interaction(), mouse::Interaction::Copy);
    }

    #[test]
    fn cancellation_emits_no_commit() {
        let mut session = DragSession::new().with_drag_threshold(1.0);

        session.begin(Point::ORIGIN);
        session.move_to(
            Point::new(2.0, 0.0),
            || Some(drag(TransferOperations::MOVE, TransferOperation::Move)),
            |_| DropDecision::accept("target", TransferOperation::Move),
        );
        session.cancel();

        assert_eq!(session.mouse_interaction(), mouse::Interaction::NoDrop);
        assert!(session.release(Point::new(2.0, 0.0)).is_none());
    }

    #[test]
    fn release_without_target_emits_no_commit() {
        let mut session = DragSession::new().with_drag_threshold(1.0);

        session.begin(Point::ORIGIN);
        assert_eq!(
            session.move_to(
                Point::new(0.5, 0.0),
                || Some(drag(TransferOperations::MOVE, TransferOperation::Move)),
                |_| DropDecision::<&'static str>::Reject,
            ),
            DragSessionFeedback::Pending
        );
        assert!(session.release(Point::new(0.5, 0.0)).is_none());
    }
}
