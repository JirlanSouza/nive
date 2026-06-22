use std::collections::BTreeMap;

use crate::UserFacingError;

use super::operation_descriptor::{
    OperationDescriptor, OperationId, OperationProgress, OperationStatus,
};

/// A runtime-level command that drives the internal [`OperationRegistry`] via
/// `AppUpdate::op_start` / `op_complete` / `op_fail` / `op_cancel`.
///
/// Apps normally emit these through the [`AppUpdate`] builder methods rather
/// than constructing this enum directly; the runtime collects each emitted
/// command into a [`RuntimeCommand::Operation`] and applies it against its
/// internal registry.
///
/// [`AppUpdate`]: crate::AppUpdate
/// [`RuntimeCommand::Operation`]: crate::RuntimeCommand::Operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationCommand {
    /// Register a new operation (or replace an existing one with the same id).
    Start(OperationDescriptor),
    /// Mark an in-flight operation as completed. Ignored if the id is unknown
    /// or already terminal.
    Complete(OperationId),
    /// Mark an in-flight operation as failed. Ignored if the id is unknown or
    /// already terminal.
    Fail(OperationId, UserFacingError),
    /// Mark a cancellable in-flight operation as cancelled. Ignored if the id
    /// is unknown, already terminal, or not cancellable.
    Cancel(OperationId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationEntry {
    pub descriptor: OperationDescriptor,
    pub status: OperationStatus,
}

impl OperationEntry {
    pub fn is_running(&self) -> bool {
        self.status.is_running()
    }

    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }
}

#[derive(Debug, Default, Clone)]
pub struct OperationRegistry {
    entries: BTreeMap<OperationId, OperationEntry>,
}

impl OperationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, descriptor: OperationDescriptor) -> Option<OperationDescriptor> {
        let id = descriptor.id.clone();
        let entry = OperationEntry {
            descriptor,
            status: OperationStatus::Running,
        };
        self.entries
            .insert(id, entry)
            .map(|previous| previous.descriptor)
    }

    pub fn update_progress(&mut self, id: OperationId, progress: OperationProgress) -> bool {
        let Some(entry) = self.entries.get_mut(&id) else {
            return false;
        };
        if !entry.is_running() {
            return false;
        }
        entry.descriptor.progress = progress;
        true
    }

    pub fn complete(&mut self, id: OperationId) -> Option<OperationDescriptor> {
        self.finish(id, OperationStatus::Completed)
    }

    pub fn fail(&mut self, id: OperationId, error: UserFacingError) -> Option<OperationDescriptor> {
        self.finish(id, OperationStatus::Failed(error))
    }

    pub fn cancel(&mut self, id: OperationId) -> Option<OperationDescriptor> {
        let entry = self.entries.get_mut(&id)?;
        if !entry.is_running() || !entry.descriptor.cancellable {
            return None;
        }
        entry.status = OperationStatus::Cancelled;
        Some(entry.descriptor.clone())
    }

    fn finish(&mut self, id: OperationId, next: OperationStatus) -> Option<OperationDescriptor> {
        let entry = self.entries.get_mut(&id)?;
        if !entry.is_running() {
            return None;
        }
        entry.status = next;
        Some(entry.descriptor.clone())
    }

    pub fn remove(&mut self, id: OperationId) -> Option<OperationEntry> {
        self.entries.remove(&id)
    }

    pub fn get(&self, id: OperationId) -> Option<&OperationEntry> {
        self.entries.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&OperationId, &OperationEntry)> {
        self.entries.iter()
    }

    pub fn running(&self) -> impl Iterator<Item = (&OperationId, &OperationEntry)> {
        self.entries.iter().filter(|(_, entry)| entry.is_running())
    }

    pub fn cancellable(&self) -> impl Iterator<Item = (&OperationId, &OperationEntry)> {
        self.entries
            .iter()
            .filter(|(_, entry)| entry.is_running() && entry.descriptor.cancellable)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn running_count(&self) -> usize {
        self.entries
            .values()
            .filter(|entry| entry.is_running())
            .count()
    }

    pub fn clear_terminal(&mut self) -> usize {
        let terminal: Vec<OperationId> = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.is_terminal())
            .map(|(id, _)| id.clone())
            .collect();
        let count = terminal.len();
        for id in terminal {
            self.entries.remove(&id);
        }
        count
    }
}

#[cfg(test)]
mod operation_registry_tests {
    use super::*;

    fn descriptor(id: &'static str) -> OperationDescriptor {
        OperationDescriptor::new(id, "Test")
    }

    #[test]
    fn register_starts_running_and_returns_replaced_descriptor() {
        let mut registry = OperationRegistry::new();

        let replaced = registry.register(
            descriptor("ingest")
                .progress(OperationProgress::fraction(2, 10))
                .cancellable(true),
        );

        assert!(replaced.is_none());
        let entry = registry.get("ingest".into()).expect("entry present");
        assert!(entry.is_running());
        assert_eq!(entry.descriptor.progress.ratio(), Some(0.2));
        assert!(entry.descriptor.cancellable);
    }

    #[test]
    fn register_replaces_existing_descriptor_and_resets_status() {
        let mut registry = OperationRegistry::new();
        registry.register(descriptor("ingest"));

        let replaced =
            registry.register(descriptor("ingest").progress(OperationProgress::fraction(0, 5)));

        assert!(replaced.is_some());
        let entry = registry.get("ingest".into()).expect("entry present");
        assert!(entry.is_running());
        assert_eq!(entry.descriptor.progress.ratio(), Some(0.0));
    }

    #[test]
    fn update_progress_advances_fraction() {
        let mut registry = OperationRegistry::new();
        registry.register(descriptor("ingest"));

        let updated = registry.update_progress("ingest".into(), OperationProgress::fraction(7, 10));

        assert!(updated);
        let entry = registry.get("ingest".into()).expect("entry present");
        assert_eq!(entry.descriptor.progress.ratio(), Some(0.7));
    }

    #[test]
    fn update_progress_after_completion_is_ignored() {
        let mut registry = OperationRegistry::new();
        registry.register(descriptor("ingest"));
        registry.complete("ingest".into());

        let updated = registry.update_progress("ingest".into(), OperationProgress::fraction(1, 1));

        assert!(!updated);
        assert!(registry.get("ingest".into()).is_some());
    }

    #[test]
    fn complete_moves_entry_out_of_running() {
        let mut registry = OperationRegistry::new();
        registry.register(descriptor("ingest"));

        assert!(registry.complete("ingest".into()).is_some());
        assert_eq!(registry.running_count(), 0);

        let entry = registry.get("ingest".into()).expect("entry present");
        assert!(matches!(entry.status, OperationStatus::Completed));
    }

    #[test]
    fn fail_records_error_in_status() {
        let mut registry = OperationRegistry::new();
        registry.register(descriptor("ingest"));

        let error = UserFacingError::custom("ingest", "disk full");
        assert!(registry.fail("ingest".into(), error.clone()).is_some());

        let entry = registry.get("ingest".into()).expect("entry present");
        match &entry.status {
            OperationStatus::Failed(failed) => assert_eq!(failed, &error),
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn cancel_only_affects_cancellable_running_entries() {
        let mut registry = OperationRegistry::new();
        registry.register(descriptor("export"));
        registry.register(descriptor("ingest").cancellable(true));

        assert!(registry.cancel("export".into()).is_none());
        assert!(registry.cancel("ingest".into()).is_some());

        let entry = registry.get("ingest".into()).expect("entry present");
        assert!(matches!(entry.status, OperationStatus::Cancelled));
    }

    #[test]
    fn double_complete_is_idempotent() {
        let mut registry = OperationRegistry::new();
        registry.register(descriptor("ingest"));

        assert!(registry.complete("ingest".into()).is_some());
        assert!(registry.complete("ingest".into()).is_none());
        assert!(registry
            .fail("ingest".into(), UserFacingError::custom("ingest", "late"))
            .is_none());
    }

    #[test]
    fn clear_terminal_removes_only_terminal_entries() {
        let mut registry = OperationRegistry::new();
        registry.register(descriptor("export"));
        registry.register(descriptor("ingest"));
        registry.complete("ingest".into());

        let removed = registry.clear_terminal();

        assert_eq!(removed, 1);
        assert!(registry.get("ingest".into()).is_none());
        assert!(registry.get("export".into()).is_some());
    }

    #[test]
    fn cancellable_filter_returns_only_running_cancellable_entries() {
        let mut registry = OperationRegistry::new();
        registry.register(descriptor("export"));
        registry.register(descriptor("ingest").cancellable(true));
        registry.register(descriptor("backup").cancellable(true));
        registry.complete("backup".into());

        let cancellable: Vec<OperationId> =
            registry.cancellable().map(|(id, _)| id.clone()).collect();

        assert_eq!(cancellable, vec!["ingest".into()]);
    }

    #[test]
    fn mixed_static_and_owned_ids_lookup_by_either_form() {
        let mut registry = OperationRegistry::new();
        // Register using the static constructor (zero-cost Cow::Borrowed).
        registry.register(OperationDescriptor::new(
            OperationId::from_static("load-projects"),
            "Load",
        ));

        // Look up using an owned id (Cow::Owned) with the same content — the
        // BTreeMap orders by content so this succeeds.
        let owned_id = OperationId::from_owned(String::from("load-projects"));
        assert!(registry.get(owned_id.clone()).is_some());
        assert!(registry.complete(owned_id).is_some());

        // Register using an owned id; look up with a static id.
        let dynamic = OperationId::from_owned(format!("user:{}:load", 42));
        registry.register(OperationDescriptor::new(dynamic.clone(), "Per-user"));

        assert!(registry
            .get(OperationId::from_static("user:42:load"))
            .is_some());
    }

    #[test]
    fn owned_id_round_trips_through_display_and_as_str() {
        let owned = OperationId::from_owned(String::from("user:7:fetch"));
        assert_eq!(owned.as_str(), "user:7:fetch");
        assert_eq!(owned.to_string(), "user:7:fetch");
    }
}
