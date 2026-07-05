use std::collections::VecDeque;
use std::sync::Mutex;

use super::runtime_event::{DiagnosticEvent, DiagnosticEventKind};

const DEFAULT_CAPACITY: usize = 256;

#[derive(Debug)]
pub struct DiagnosticEventLog {
    events: Mutex<VecDeque<DiagnosticEvent>>,
    capacity: usize,
}

impl DiagnosticEventLog {
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

    pub fn record(&self, event: DiagnosticEvent) {
        let mut events = self.events.lock().expect("event log poisoned");
        if events.len() == self.capacity {
            events.pop_front();
        }
        events.push_back(event);
    }

    pub fn record_kind(
        &self,
        kind: DiagnosticEventKind,
        category: impl Into<std::borrow::Cow<'static, str>>,
        message: impl Into<std::borrow::Cow<'static, str>>,
    ) {
        self.record(DiagnosticEvent::new(kind, category, message));
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

    pub fn snapshot(&self) -> Vec<DiagnosticEvent> {
        self.events
            .lock()
            .expect("event log poisoned")
            .iter()
            .cloned()
            .collect()
    }

    pub fn recent(&self, limit: usize) -> Vec<DiagnosticEvent> {
        let events = self.events.lock().expect("event log poisoned");
        let start = events.len().saturating_sub(limit);
        events.iter().skip(start).cloned().collect()
    }
}

impl Default for DiagnosticEventLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod event_log_tests {
    use super::*;

    #[test]
    fn record_stores_event_in_order() {
        let log = DiagnosticEventLog::new();
        log.record(DiagnosticEvent::info("settings", "loaded"));
        log.record(DiagnosticEvent::warning("settings", "missing key"));

        let snapshot = log.snapshot();
        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot[0].message, "loaded");
        assert_eq!(snapshot[1].message, "missing key");
    }

    #[test]
    fn ring_buffer_drops_oldest_when_full() {
        let log = DiagnosticEventLog::with_capacity(2);
        log.record(DiagnosticEvent::info("test", "first"));
        log.record(DiagnosticEvent::info("test", "second"));
        log.record(DiagnosticEvent::info("test", "third"));

        let snapshot = log.snapshot();
        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot[0].message, "second");
        assert_eq!(snapshot[1].message, "third");
    }

    #[test]
    fn recent_returns_last_n_events() {
        let log = DiagnosticEventLog::new();
        for index in 0..5 {
            log.record(DiagnosticEvent::info("test", format!("event-{index}")));
        }

        let recent = log.recent(3);

        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].message, "event-2");
        assert_eq!(recent[1].message, "event-3");
        assert_eq!(recent[2].message, "event-4");
    }

    #[test]
    fn clear_empties_log() {
        let log = DiagnosticEventLog::new();
        log.record(DiagnosticEvent::info("test", "kept"));
        assert!(!log.is_empty());

        log.clear();

        assert!(log.is_empty());
    }

    #[test]
    fn record_kind_helper_constructs_event() {
        let log = DiagnosticEventLog::new();
        log.record_kind(DiagnosticEventKind::Error, "ingest", "disk full");

        let snapshot = log.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].kind, DiagnosticEventKind::Error);
        assert_eq!(snapshot[0].category, "ingest");
        assert_eq!(snapshot[0].message, "disk full");
    }
}
