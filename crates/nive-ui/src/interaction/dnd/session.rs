use iced::{keyboard, mouse, Point};

use super::{Drag, DropCommit, DropContext, DropDecision};
use crate::interaction::{
    PointerGesture, PointerGestureKind, TransferOperation, TransferOperations,
};

#[derive(Debug, Clone)]
pub(crate) struct DragSession<T, Origin, Target> {
    phase: DragPhase<T, Origin, Target>,
}

impl<T, Origin, Target> Default for DragSession<T, Origin, Target> {
    fn default() -> Self {
        Self {
            phase: DragPhase::Idle,
        }
    }
}

impl<T, Origin, Target> DragSession<T, Origin, Target> {
    pub(crate) fn handle_gesture<Region>(
        &mut self,
        gesture: &PointerGesture<Region>,
        source: impl FnOnce() -> Option<Drag<T, Origin>>,
        target: impl FnOnce(DropContext<'_, T, Origin>) -> DropDecision<Target>,
    ) -> DragSessionOutcome<T, Origin, Target>
    where
        Target: Clone,
    {
        match gesture.kind {
            PointerGestureKind::DragStarted => {
                let Some(drag) = source() else {
                    self.phase = DragPhase::Idle;
                    return DragSessionOutcome::Feedback(DragSessionFeedback::Rejected);
                };

                self.phase = DragPhase::Active {
                    drag,
                    target: None,
                    requested: requested_operation_from_modifiers(gesture.modifiers),
                    modifiers: gesture.modifiers,
                    cancelled: false,
                };

                self.probe_target(gesture.position, target)
            }
            PointerGestureKind::DragMoved => self.probe_target(gesture.position, target),
            PointerGestureKind::DragReleased => {
                DragSessionOutcome::Commit(self.release(gesture.position))
            }
            PointerGestureKind::DragCancelled => {
                self.cancel();
                DragSessionOutcome::Feedback(DragSessionFeedback::Rejected)
            }
            _ => DragSessionOutcome::Feedback(DragSessionFeedback::Idle),
        }
    }

    #[allow(dead_code)] // consumidor futuro: Tree (parte 2)
    pub(crate) fn update_modifiers(&mut self, modifiers: keyboard::Modifiers) {
        if let DragPhase::Active {
            requested,
            modifiers: active_modifiers,
            ..
        } = &mut self.phase
        {
            *requested = requested_operation_from_modifiers(modifiers);
            *active_modifiers = modifiers;
        }
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

    #[allow(dead_code)] // consumidor futuro: Tree (parte 2)
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

    #[allow(dead_code)] // consumidor futuro: Tree (parte 2)
    pub(crate) fn grabbing_interaction(&self) -> mouse::Interaction {
        if matches!(self.phase, DragPhase::Active { .. }) {
            mouse::Interaction::Grabbing
        } else {
            mouse::Interaction::Grab
        }
    }

    fn probe_target(
        &mut self,
        position: Point,
        target: impl FnOnce(DropContext<'_, T, Origin>) -> DropDecision<Target>,
    ) -> DragSessionOutcome<T, Origin, Target>
    where
        Target: Clone,
    {
        let DragPhase::Active {
            drag,
            target: current_target,
            requested,
            modifiers,
            cancelled,
        } = &mut self.phase
        else {
            return DragSessionOutcome::Feedback(DragSessionFeedback::Idle);
        };

        if *cancelled {
            return DragSessionOutcome::Feedback(DragSessionFeedback::Rejected);
        }

        let context = DropContext {
            payload: &drag.payload,
            origin: &drag.origin,
            operations: drag.operations,
            preferred: drag.preferred,
            requested: *requested,
            position,
            modifiers: *modifiers,
        };
        let decision = target(context);
        *current_target = accepted_target(drag.operations, decision);

        DragSessionOutcome::Feedback(DragSessionFeedback::from_target(
            current_target.as_ref().map(|target| target.operation),
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DragSessionOutcome<T, Origin, Target> {
    Feedback(DragSessionFeedback),
    Commit(Option<DropCommit<T, Origin, Target>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragSessionFeedback {
    Idle,
    Rejected,
    Accepted(TransferOperation),
}

impl DragSessionFeedback {
    fn from_target(operation: Option<TransferOperation>) -> Self {
        operation.map_or(Self::Rejected, Self::Accepted)
    }
}

#[derive(Debug, Clone)]
enum DragPhase<T, Origin, Target> {
    Idle,
    Active {
        drag: Drag<T, Origin>,
        target: Option<AcceptedDrop<Target>>,
        requested: Option<TransferOperation>,
        modifiers: keyboard::Modifiers,
        cancelled: bool,
    },
}

#[derive(Debug, Clone)]
struct AcceptedDrop<Target> {
    target: Target,
    operation: TransferOperation,
}

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

#[cfg(test)]
mod dnd_tests {
    use super::super::TransferData;
    use super::*;
    use crate::interaction::PointerButton;

    fn drag(operations: TransferOperations, preferred: TransferOperation) -> Drag<&'static str> {
        Drag {
            payload: TransferData::local("payload"),
            origin: (),
            operations,
            preferred,
        }
    }

    fn gesture(
        kind: PointerGestureKind,
        position: Point,
        modifiers: keyboard::Modifiers,
    ) -> PointerGesture<&'static str> {
        PointerGesture {
            kind,
            button: PointerButton::Primary,
            region: "row",
            position,
            modifiers,
        }
    }

    #[test]
    fn drag_started_begins_session_and_probes_target() {
        let mut session = DragSession::default();

        let outcome = session.handle_gesture(
            &gesture(
                PointerGestureKind::DragStarted,
                Point::new(2.0, 0.0),
                keyboard::Modifiers::NONE,
            ),
            || Some(drag(TransferOperations::MOVE, TransferOperation::Move)),
            |context| {
                DropDecision::accept("target", context.preferred_operation().expect("operation"))
            },
        );
        let commit = session.release(Point::new(2.0, 0.0)).expect("commit");

        assert_eq!(
            outcome,
            DragSessionOutcome::Feedback(DragSessionFeedback::Accepted(TransferOperation::Move))
        );
        assert_eq!(commit.target, "target");
        assert_eq!(commit.operation, TransferOperation::Move);
    }

    #[test]
    fn drag_moved_updates_candidate_without_restarting_source() {
        let mut session = DragSession::default();

        session.handle_gesture(
            &gesture(
                PointerGestureKind::DragStarted,
                Point::new(2.0, 0.0),
                keyboard::Modifiers::NONE,
            ),
            || Some(drag(TransferOperations::MOVE, TransferOperation::Move)),
            |_| DropDecision::accept("first", TransferOperation::Move),
        );
        let outcome = session.handle_gesture(
            &gesture(
                PointerGestureKind::DragMoved,
                Point::new(4.0, 0.0),
                keyboard::Modifiers::NONE,
            ),
            || panic!("drag source should only be consumed at start"),
            |_| DropDecision::accept("second", TransferOperation::Move),
        );
        let commit = session.release(Point::new(4.0, 0.0)).expect("commit");

        assert_eq!(
            outcome,
            DragSessionOutcome::Feedback(DragSessionFeedback::Accepted(TransferOperation::Move))
        );
        assert_eq!(commit.target, "second");
    }

    #[test]
    fn rejected_target_emits_no_commit() {
        let mut session = DragSession::default();

        let outcome = session.handle_gesture(
            &gesture(
                PointerGestureKind::DragStarted,
                Point::new(2.0, 0.0),
                keyboard::Modifiers::NONE,
            ),
            || Some(drag(TransferOperations::MOVE, TransferOperation::Move)),
            |_| DropDecision::<&'static str>::Reject,
        );

        assert_eq!(
            outcome,
            DragSessionOutcome::Feedback(DragSessionFeedback::Rejected)
        );
        assert!(session.release(Point::new(2.0, 0.0)).is_none());
    }

    #[test]
    fn disallowed_target_operation_emits_no_commit() {
        let mut session = DragSession::default();

        let outcome = session.handle_gesture(
            &gesture(
                PointerGestureKind::DragStarted,
                Point::new(2.0, 0.0),
                keyboard::Modifiers::NONE,
            ),
            || Some(drag(TransferOperations::MOVE, TransferOperation::Move)),
            |_| DropDecision::accept("target", TransferOperation::Copy),
        );

        assert_eq!(
            outcome,
            DragSessionOutcome::Feedback(DragSessionFeedback::Rejected)
        );
        assert_eq!(session.mouse_interaction(), mouse::Interaction::NoDrop);
        assert!(session.release(Point::new(2.0, 0.0)).is_none());
    }

    #[test]
    fn modifier_requested_operation_is_visible_to_target() {
        let mut session = DragSession::default();

        let outcome = session.handle_gesture(
            &gesture(
                PointerGestureKind::DragStarted,
                Point::new(2.0, 0.0),
                keyboard::Modifiers::COMMAND,
            ),
            || {
                Some(drag(
                    TransferOperations::COPY | TransferOperations::MOVE,
                    TransferOperation::Move,
                ))
            },
            |context| DropDecision::accept("target", context.requested.expect("requested")),
        );

        assert_eq!(
            outcome,
            DragSessionOutcome::Feedback(DragSessionFeedback::Accepted(TransferOperation::Copy))
        );
        assert_eq!(session.mouse_interaction(), mouse::Interaction::Copy);
    }

    #[test]
    fn drag_released_commits_active_target() {
        let mut session = DragSession::default();

        session.handle_gesture(
            &gesture(
                PointerGestureKind::DragStarted,
                Point::new(2.0, 0.0),
                keyboard::Modifiers::NONE,
            ),
            || Some(drag(TransferOperations::MOVE, TransferOperation::Move)),
            |_| DropDecision::accept("target", TransferOperation::Move),
        );
        let outcome = session.handle_gesture(
            &gesture(
                PointerGestureKind::DragReleased,
                Point::new(3.0, 0.0),
                keyboard::Modifiers::NONE,
            ),
            || panic!("release should not consume source"),
            |_| panic!("release should not probe target"),
        );

        let DragSessionOutcome::Commit(Some(commit)) = outcome else {
            panic!("expected commit");
        };
        assert_eq!(commit.target, "target");
        assert_eq!(commit.position, Point::new(3.0, 0.0));
    }

    #[test]
    fn cancellation_emits_no_commit() {
        let mut session = DragSession::default();

        session.handle_gesture(
            &gesture(
                PointerGestureKind::DragStarted,
                Point::new(2.0, 0.0),
                keyboard::Modifiers::NONE,
            ),
            || Some(drag(TransferOperations::MOVE, TransferOperation::Move)),
            |_| DropDecision::accept("target", TransferOperation::Move),
        );
        session.handle_gesture(
            &gesture(
                PointerGestureKind::DragCancelled,
                Point::new(2.0, 0.0),
                keyboard::Modifiers::NONE,
            ),
            || panic!("cancel should not consume source"),
            |_| panic!("cancel should not probe target"),
        );

        assert_eq!(session.mouse_interaction(), mouse::Interaction::NoDrop);
        assert!(session.release(Point::new(2.0, 0.0)).is_none());
    }
}
