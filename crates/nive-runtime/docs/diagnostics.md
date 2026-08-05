# Diagnostics And Recovery Contract

`nive-runtime::support` provides a reusable diagnostics surface. Apps can
offer "copy diagnostics" or "export report" without duplicating runtime
state collection. The runtime does **not** pick a logging backend, does
**not** own the global panic hook, and does **not** depend on a specific
error reporting service.

## Public API

- `DiagnosticEvent` — one entry with `timestamp` (Unix seconds), `kind`
  (`Info` / `Warning` / `Error` / `Panic`), `category` (a `Cow<'static,
  str>` like `"settings"`, `"window"`, `"panic"`), and `message`
  (also `Cow<'static, str>`).
- `DiagnosticEventLog` — `Arc<...>`-friendly ring buffer with a bounded
  capacity (default 256). `record` drops the oldest event when full.
  `recent(limit)` and `snapshot()` are the read paths.
- `install_diagnostic_panic_hook(log: Arc<DiagnosticEventLog>)` — captures
  the previously installed panic hook, installs a new hook that records
  the panic (with `file:line`) into the log and then calls the
  previous hook. The app can install additional diagnostic hooks
  earlier in the chain; the helper preserves them.
- `DiagnosticSnapshot` — serializable report with `generated_at`,
  `events`, and an optional `app_metadata` list of `(key, value)` pairs.
  `to_json` / `from_json` round-trip the snapshot; `pretty` returns a
  short human-readable summary.

## Boundaries

- Apps own the logger backend (`env_logger`, `tracing-subscriber`,
  custom). The runtime uses the `log` facade; init code is not added
  by `nive-runtime`.
- The runtime does not register an `unwind-safe` boundary. The
  diagnostic panic hook is a recorder, not a recovery handler.
- `install_diagnostic_panic_hook` must be called once at startup. The
  helper captures whatever hook was in place at that moment, so apps
  should call it after any earlier hook installation (for example, a
  custom logger).
- `DiagnosticEventLog` is `Send + Sync` (the inner `VecDeque` is guarded
  by a `Mutex`). Apps that want lock-free logging can layer a
  channel-based collector on top; the runtime does not need to.

## Integration Pattern

```rust
use std::sync::Arc;

use nive_runtime::{
    install_diagnostic_panic_hook, DiagnosticEventLog, DiagnosticSnapshot,
};

let diagnostics = Arc::new(DiagnosticEventLog::new());
install_diagnostic_panic_hook(Arc::clone(&diagnostics));

// "Copy diagnostics" button
let snapshot = DiagnosticSnapshot::capture(&diagnostics)
    .add_metadata("app_name", env!("CARGO_PKG_NAME"))
    .add_metadata("version", env!("CARGO_PKG_VERSION"));
let report = snapshot.to_json().expect("serialize");

// "Export report" file
std::fs::write("diagnostics.json", report).expect("write report");
```

## Test Names

- `event_log_tests::record_stores_event_in_order`
- `event_log_tests::ring_buffer_drops_oldest_when_full`
- `event_log_tests::recent_returns_last_n_events`
- `event_log_tests::clear_empties_log`
- `event_log_tests::record_kind_helper_constructs_event`
- `snapshot_tests::capture_collects_log_events`
- `snapshot_tests::metadata_builder_appends_entries`
- `snapshot_tests::json_round_trip_preserves_events`
- `snapshot_tests::pretty_summary_includes_event_count`
- `panic_hook_tests::panic_hook_records_message_and_delegates_to_previous`

## Verification

```text
rtk cargo fmt --package nive-runtime
rtk cargo check -p nive-runtime --all-targets --all-features
rtk cargo test -p nive-runtime --all-features
rtk just doc-check
```
