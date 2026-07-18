use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, MutexGuard,
    },
};

static NEXT_FOCUS_TOKEN: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct FocusToken(u64);

impl FocusToken {
    pub(crate) fn new() -> Self {
        Self(NEXT_FOCUS_TOKEN.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum InputOrigin {
    Pointer,
    #[default]
    Keyboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActivationVisibility {
    Auto,
    Always,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FocusGeneration(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FocusSnapshot {
    pub(crate) active: Option<FocusToken>,
    pub(crate) anchor: Option<FocusToken>,
    pub(crate) origin: InputOrigin,
    pub(crate) visible: bool,
    pub(crate) window_active: bool,
    pub(crate) revision: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FocusTransition {
    Unchanged,
    Changed {
        previous: FocusSnapshot,
        current: FocusSnapshot,
    },
}

#[derive(Debug)]
pub(crate) struct FocusCoordinator {
    active: Option<FocusToken>,
    anchor: Option<FocusToken>,
    origin: InputOrigin,
    visible: bool,
    window_active: bool,
    generation: FocusGeneration,
    pending_liveness: Option<FocusGeneration>,
    live: HashMap<FocusToken, FocusGeneration>,
    pointer_transaction: bool,
    pointer_claimed: bool,
    revision: u64,
}

pub(crate) type SharedFocusCoordinator = Arc<Mutex<FocusCoordinator>>;

impl Default for FocusCoordinator {
    fn default() -> Self {
        Self {
            active: None,
            anchor: None,
            origin: InputOrigin::default(),
            visible: false,
            window_active: true,
            generation: FocusGeneration::default(),
            pending_liveness: None,
            live: HashMap::new(),
            pointer_transaction: false,
            pointer_claimed: false,
            revision: 0,
        }
    }
}

impl FocusCoordinator {
    pub(crate) fn shared() -> SharedFocusCoordinator {
        Arc::new(Mutex::new(Self::default()))
    }

    pub(crate) fn snapshot(&self) -> FocusSnapshot {
        FocusSnapshot {
            active: self.active,
            anchor: self.anchor,
            origin: self.origin,
            visible: self.visible,
            window_active: self.window_active,
            revision: self.revision,
        }
    }

    pub(crate) fn record_origin(&mut self, origin: InputOrigin) -> FocusTransition {
        self.transition(|coordinator| coordinator.origin = origin)
    }

    pub(crate) fn activate(
        &mut self,
        token: FocusToken,
        visibility: ActivationVisibility,
    ) -> FocusTransition {
        self.transition(|coordinator| {
            coordinator.revision = coordinator.revision.wrapping_add(1);
            coordinator.anchor = Some(token);
            if coordinator.window_active {
                coordinator.active = Some(token);
                coordinator.visible = visibility == ActivationVisibility::Always
                    || coordinator.origin == InputOrigin::Keyboard;
            } else {
                coordinator.active = None;
                coordinator.visible = false;
            }
        })
    }

    pub(crate) fn activate_from_pointer(
        &mut self,
        token: FocusToken,
        visibility: ActivationVisibility,
    ) -> FocusTransition {
        self.transition(|coordinator| {
            coordinator.revision = coordinator.revision.wrapping_add(1);
            coordinator.origin = InputOrigin::Pointer;
            coordinator.pointer_claimed = coordinator.pointer_transaction;
            coordinator.anchor = Some(token);
            if coordinator.window_active {
                coordinator.active = Some(token);
                coordinator.visible = visibility == ActivationVisibility::Always;
            } else {
                coordinator.active = None;
                coordinator.visible = false;
            }
        })
    }

    pub(crate) fn begin_pointer_transaction(&mut self) -> FocusTransition {
        if self.pointer_transaction {
            return FocusTransition::Unchanged;
        }
        self.pointer_transaction = true;
        self.pointer_claimed = false;
        self.record_origin(InputOrigin::Pointer)
    }

    pub(crate) fn finish_pointer_transaction(&mut self) -> FocusTransition {
        if !self.pointer_transaction {
            return FocusTransition::Unchanged;
        }

        let claimed = self.pointer_claimed;
        self.pointer_transaction = false;
        self.pointer_claimed = false;
        if claimed {
            FocusTransition::Unchanged
        } else {
            self.deactivate_current()
        }
    }

    pub(crate) fn deactivate(&mut self, owner: FocusToken) -> FocusTransition {
        self.transition(|coordinator| {
            if coordinator.active == Some(owner) {
                coordinator.active = None;
                coordinator.visible = false;
            }
        })
    }

    pub(crate) fn deactivate_current(&mut self) -> FocusTransition {
        self.transition(|coordinator| {
            coordinator.active = None;
            coordinator.visible = false;
        })
    }

    pub(crate) fn unfocus_if_owned(&mut self, owner: FocusToken) -> FocusTransition {
        self.transition(|coordinator| {
            if coordinator.active == Some(owner) {
                coordinator.active = None;
                coordinator.visible = false;
            }
            if coordinator.anchor == Some(owner) {
                coordinator.anchor = None;
            }
        })
    }

    pub(crate) fn clear(&mut self, owner: FocusToken) -> FocusTransition {
        self.unfocus_if_owned(owner)
    }

    pub(crate) fn deactivate_window(&mut self) -> FocusTransition {
        self.transition(|coordinator| {
            coordinator.window_active = false;
            coordinator.active = None;
            coordinator.visible = false;
        })
    }

    pub(crate) fn activate_window(&mut self) -> FocusTransition {
        self.transition(|coordinator| coordinator.window_active = true)
    }

    pub(crate) fn begin_liveness(&mut self) -> FocusGeneration {
        self.generation = FocusGeneration(self.generation.0.wrapping_add(1));
        self.pending_liveness = Some(self.generation);
        self.generation
    }

    pub(crate) fn ensure_liveness(&mut self) -> FocusGeneration {
        self.pending_liveness
            .unwrap_or_else(|| self.begin_liveness())
    }

    pub(crate) fn observe_live(&mut self, token: FocusToken, generation: FocusGeneration) {
        if generation == self.generation {
            self.live.insert(token, generation);
        }
    }

    pub(crate) fn finish_liveness(&mut self, generation: FocusGeneration) -> FocusTransition {
        if generation != self.generation {
            return FocusTransition::Unchanged;
        }

        let transition = self.transition(|coordinator| {
            if coordinator
                .active
                .is_some_and(|token| coordinator.live.get(&token) != Some(&generation))
            {
                coordinator.active = None;
                coordinator.visible = false;
            }
            if coordinator
                .anchor
                .is_some_and(|token| coordinator.live.get(&token) != Some(&generation))
            {
                coordinator.anchor = None;
            }
        });
        self.live.retain(|_, seen| *seen == generation);
        self.pending_liveness = None;
        transition
    }

    pub(crate) fn is_current(&self, token: FocusToken) -> bool {
        self.active == Some(token) || self.anchor == Some(token)
    }

    pub(crate) fn is_active(&self, token: FocusToken) -> bool {
        self.active == Some(token)
    }

    pub(crate) fn is_visible(&self, token: FocusToken) -> bool {
        self.active == Some(token) && self.visible
    }

    pub(crate) fn target_identity(&self) -> Option<(FocusToken, u64)> {
        self.anchor.map(|token| (token, self.revision))
    }

    pub(crate) fn is_live(&self, token: FocusToken) -> bool {
        self.live.contains_key(&token)
    }

    pub(crate) fn restore_if_current(
        &mut self,
        captured: FocusToken,
        expected: Option<(FocusToken, u64)>,
        visibility: ActivationVisibility,
    ) -> bool {
        if !self.is_live(captured) {
            return false;
        }

        let current = self.target_identity();
        if current != expected {
            return false;
        }

        self.activate(captured, visibility);
        true
    }

    pub(crate) fn restore_anchor_if_current(
        &mut self,
        captured: FocusToken,
        expected: Option<(FocusToken, u64)>,
    ) -> bool {
        if !self.is_live(captured) || self.target_identity() != expected {
            return false;
        }

        self.transition(|coordinator| {
            coordinator.revision = coordinator.revision.wrapping_add(1);
            coordinator.active = None;
            coordinator.anchor = Some(captured);
            coordinator.visible = false;
        });
        true
    }

    fn transition(&mut self, change: impl FnOnce(&mut Self)) -> FocusTransition {
        let previous = self.snapshot();
        change(self);
        self.enforce_invariants();
        let current = self.snapshot();

        if previous == current {
            FocusTransition::Unchanged
        } else {
            FocusTransition::Changed { previous, current }
        }
    }

    fn enforce_invariants(&mut self) {
        if let Some(active) = self.active {
            self.anchor = Some(active);
        }
        if self.active.is_none() {
            self.visible = false;
        }
    }
}

pub(crate) fn lock_coordinator(
    coordinator: &Mutex<FocusCoordinator>,
) -> MutexGuard<'_, FocusCoordinator> {
    coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
