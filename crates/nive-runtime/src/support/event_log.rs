use std::collections::VecDeque;
use std::sync::Mutex;

use super::runtime_event::{RuntimeEvent, RuntimeEventKind};

const DEFAULT_CAPACITY: usize = 256;

#[derive(Debug)]
pub struct RuntimeEventLog {
    events: Mutex<VecDeque<RuntimeEvent>>,
    capacity: usize,
}

impl RuntimeEventLog {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn record(&self, event: RuntimeEvent) {
        let mut events = self.events.lock().expect("event log poisoned");
        if events.len() == self.capacity {
            events.pop_front();
        }
        events.push_back(event);
    }

    pub fn record_kind(
        &self,
        kind: RuntimeEventKind,
        category: impl Into<std::borrow::Cow<'static, str>>,
        message: impl Into<std::borrow::Cow<'static, str>>,
    ) {
        self.record(RuntimeEvent::new(kind, category, message));
    }

    pub fn len(&self) -> usize {
        self.events.lock().expect("event log poisoned").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&self) {
        self.events.lock().expect("event log poisoned").clear();
    }

    pub fn snapshot(&self) -> Vec<RuntimeEvent> {
        self.events
            .lock()
            .expect("event log poisoned")
            .iter()
            .cloned()
            .collect()
    }

    pub fn recent(&self, limit: usize) -> Vec<RuntimeEvent> {
        let events = self.events.lock().expect("event log poisoned");
        let start = events.len().saturating_sub(limit);
        events.iter().skip(start).cloned().collect()
    }
}

impl Default for RuntimeEventLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod event_log_tests {
    use super::*;

    #[test]
    fn record_stores_event_in_order() {
        let log = RuntimeEventLog::new();
        log.record(RuntimeEvent::info("settings", "loaded"));
        log.record(RuntimeEvent::warning("settings", "missing key"));

        let snapshot = log.snapshot();
        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot[0].message, "loaded");
        assert_eq!(snapshot[1].message, "missing key");
    }

    #[test]
    fn ring_buffer_drops_oldest_when_full() {
        let log = RuntimeEventLog::with_capacity(2);
        log.record(RuntimeEvent::info("test", "first"));
        log.record(RuntimeEvent::info("test", "second"));
        log.record(RuntimeEvent::info("test", "third"));

        let snapshot = log.snapshot();
        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot[0].message, "second");
        assert_eq!(snapshot[1].message, "third");
    }

    #[test]
    fn recent_returns_last_n_events() {
        let log = RuntimeEventLog::new();
        for index in 0..5 {
            log.record(RuntimeEvent::info("test", format!("event-{index}")));
        }

        let recent = log.recent(3);

        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].message, "event-2");
        assert_eq!(recent[1].message, "event-3");
        assert_eq!(recent[2].message, "event-4");
    }

    #[test]
    fn clear_empties_log() {
        let log = RuntimeEventLog::new();
        log.record(RuntimeEvent::info("test", "kept"));
        assert!(!log.is_empty());

        log.clear();

        assert!(log.is_empty());
    }

    #[test]
    fn record_kind_helper_constructs_event() {
        let log = RuntimeEventLog::new();
        log.record_kind(RuntimeEventKind::Error, "ingest", "disk full");

        let snapshot = log.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].kind, RuntimeEventKind::Error);
        assert_eq!(snapshot[0].category, "ingest");
        assert_eq!(snapshot[0].message, "disk full");
    }
}
