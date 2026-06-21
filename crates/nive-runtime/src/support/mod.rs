use std::borrow::Cow;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

mod event_log;
mod panic_hook;
mod runtime_event;

pub use event_log::RuntimeEventLog;
pub use panic_hook::install_diagnostic_panic_hook;
pub use runtime_event::{RuntimeEvent, RuntimeEventKind};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticSnapshot {
    pub generated_at: i64,
    pub events: Vec<RuntimeEvent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub app_metadata: Vec<(String, String)>,
}

impl DiagnosticSnapshot {
    pub fn capture(log: &RuntimeEventLog) -> Self {
        Self {
            generated_at: crate::unix_now(),
            events: log.snapshot(),
            app_metadata: Vec::new(),
        }
    }

    pub fn capture_arc(log: &Arc<RuntimeEventLog>) -> Self {
        Self::capture(log.as_ref())
    }

    pub fn with_metadata<I, K, V>(mut self, entries: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.app_metadata = entries
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }

    pub fn add_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.app_metadata.push((key.into(), value.into()));
        self
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }

    pub fn pretty(&self) -> Cow<'_, str> {
        if self.app_metadata.is_empty() {
            Cow::Owned(format!(
                "diagnostic @ {} ({} events)",
                self.generated_at,
                self.events.len()
            ))
        } else {
            Cow::Owned(format!(
                "diagnostic @ {} ({} events, {} metadata fields)",
                self.generated_at,
                self.events.len(),
                self.app_metadata.len()
            ))
        }
    }
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;

    #[test]
    fn capture_collects_log_events() {
        let log = RuntimeEventLog::new();
        log.record(RuntimeEvent::info("settings", "loaded"));
        log.record(RuntimeEvent::warning("ingest", "retry"));

        let snapshot = DiagnosticSnapshot::capture(&log);

        assert_eq!(snapshot.events.len(), 2);
        assert!(snapshot.app_metadata.is_empty());
        assert!(snapshot.generated_at > 0);
    }

    #[test]
    fn metadata_builder_appends_entries() {
        let log = RuntimeEventLog::new();
        let snapshot = DiagnosticSnapshot::capture(&log)
            .with_metadata([("app_name", "acme"), ("version", "0.1.0")])
            .add_metadata("channel", "dev");

        assert_eq!(snapshot.app_metadata.len(), 3);
        assert_eq!(snapshot.app_metadata[0].0, "app_name");
        assert_eq!(snapshot.app_metadata[2].1, "dev");
    }

    #[test]
    fn json_round_trip_preserves_events() {
        let log = RuntimeEventLog::new();
        log.record(RuntimeEvent::error("ingest", "disk full"));
        let snapshot = DiagnosticSnapshot::capture(&log).add_metadata("app", "acme");

        let json = snapshot.to_json().expect("serialize");
        let restored = DiagnosticSnapshot::from_json(&json).expect("deserialize");

        assert_eq!(restored.events.len(), 1);
        assert_eq!(restored.events[0].message, "disk full");
        assert_eq!(restored.app_metadata.len(), 1);
        assert_eq!(restored.app_metadata[0].0, "app");
    }

    #[test]
    fn pretty_summary_includes_event_count() {
        let log = RuntimeEventLog::new();
        log.record(RuntimeEvent::info("test", "a"));
        log.record(RuntimeEvent::info("test", "b"));
        let snapshot = DiagnosticSnapshot::capture(&log).add_metadata("app", "acme");

        let summary = snapshot.pretty();
        assert!(summary.contains("2 events"));
        assert!(summary.contains("1 metadata fields"));
    }
}
