use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ManagedFocusEntry {
    pub(crate) id: Option<Id>,
    pub(crate) bounds: Rectangle,
    pub(crate) token: crate::focus::FocusToken,
    pub(crate) root_identity: Option<usize>,
    pub(crate) active: bool,
    pub(crate) anchor_only: bool,
    pub(crate) visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ManagedFocusCall {
    Managed(crate::focus::FocusToken),
    Focusable(Option<Id>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ManagedFocusSnapshot {
    pub(crate) entries: Vec<ManagedFocusEntry>,
    pub(crate) calls: Vec<ManagedFocusCall>,
}

#[derive(Debug, Default)]
pub(crate) struct ManagedFocusProbe {
    snapshot: ManagedFocusSnapshot,
}

impl ManagedFocusProbe {
    pub(crate) fn finish(self) -> ManagedFocusSnapshot {
        self.snapshot
    }
}

impl operation::Operation for ManagedFocusProbe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
        operate(self);
    }

    fn custom(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn std::any::Any) {
        let Some(state) = state.downcast_ref::<crate::advanced::focus::FocusState>() else {
            return;
        };
        let current = operation::Focusable::is_focused(state);
        self.snapshot
            .calls
            .push(ManagedFocusCall::Managed(state.token()));
        self.snapshot.entries.push(ManagedFocusEntry {
            id: id.cloned(),
            bounds,
            token: state.token(),
            root_identity: state.root_identity(),
            active: state.is_active(),
            anchor_only: current && !state.is_active(),
            visible: state.is_focus_visible(),
        });
    }

    fn focusable(
        &mut self,
        id: Option<&Id>,
        _bounds: Rectangle,
        _state: &mut dyn operation::Focusable,
    ) {
        self.snapshot
            .calls
            .push(ManagedFocusCall::Focusable(id.cloned()));
    }
}

#[cfg(test)]
mod managed_focus_probe_tests {
    use iced::advanced::widget::Id;

    use super::{operation, ManagedFocusCall, ManagedFocusProbe, Point, Rectangle, Size};
    use crate::{
        advanced::focus::FocusState,
        focus::{lock_coordinator, FocusCoordinator},
    };

    #[test]
    fn probe_reports_anchor_bounds_identity_order_and_root_isolation() {
        let first_root = FocusCoordinator::shared();
        let second_root = FocusCoordinator::shared();
        let first_generation = lock_coordinator(&first_root).begin_liveness();
        let second_generation = lock_coordinator(&second_root).begin_liveness();
        let mut first = FocusState::default();
        let mut second = FocusState::default();
        first.bind(&first_root, first_generation);
        second.bind(&second_root, second_generation);
        first.focus_from_pointer();
        first.deactivate();
        operation::Focusable::focus(&mut second);
        let first_id = Id::unique();
        let second_id = Id::unique();
        let first_bounds = Rectangle::new(Point::new(1.0, 2.0), Size::new(3.0, 4.0));
        let second_bounds = Rectangle::new(Point::new(5.0, 6.0), Size::new(7.0, 8.0));
        let mut probe = ManagedFocusProbe::default();

        first.register(&mut probe, Some(&first_id), first_bounds);
        second.register(&mut probe, Some(&second_id), second_bounds);
        let snapshot = probe.finish();

        assert_eq!(snapshot.entries.len(), 2);
        assert_eq!(snapshot.entries[0].bounds, first_bounds);
        assert!(snapshot.entries[0].anchor_only);
        assert!(!snapshot.entries[0].active);
        assert!(!snapshot.entries[0].visible);
        assert_eq!(snapshot.entries[1].bounds, second_bounds);
        assert!(snapshot.entries[1].active);
        assert!(snapshot.entries[1].visible);
        assert_ne!(snapshot.entries[0].token, snapshot.entries[1].token);
        assert_ne!(
            snapshot.entries[0].root_identity,
            snapshot.entries[1].root_identity
        );
        assert_eq!(
            snapshot.calls,
            vec![
                ManagedFocusCall::Managed(snapshot.entries[0].token),
                ManagedFocusCall::Focusable(Some(first_id)),
                ManagedFocusCall::Managed(snapshot.entries[1].token),
                ManagedFocusCall::Focusable(Some(second_id)),
            ]
        );
    }
}
