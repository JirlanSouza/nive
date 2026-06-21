use std::panic::{self, PanicHookInfo};
use std::sync::Arc;

use log::error;

use super::event_log::RuntimeEventLog;
use super::runtime_event::RuntimeEvent;

/// Installs a panic hook that records every panic into the provided
/// [`RuntimeEventLog`] before delegating to the previously installed hook.
///
/// The runtime does not own the global panic hook. Apps call this helper
/// once during startup after creating the log they want diagnostics to
/// use, and the helper preserves the existing hook (default printer in
/// tests, custom logger in production) so behaviour outside the log
/// does not change.
pub fn install_diagnostic_panic_hook(log: Arc<RuntimeEventLog>) {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info: &PanicHookInfo<'_>| {
        let message = panic_message(info);
        let location = info
            .location()
            .map(|loc| format!("{}:{}", loc.file(), loc.line()))
            .unwrap_or_default();
        log.record(RuntimeEvent::panic(
            "panic",
            if location.is_empty() {
                message.clone()
            } else {
                format!("{message} ({location})")
            },
        ));
        error!(target: "nive_runtime::support", "panic recorded: {message} at {location}");
        previous(info);
    }));
}

fn panic_message(info: &PanicHookInfo<'_>) -> String {
    if let Some(message) = info.payload().downcast_ref::<&'static str>() {
        (*message).to_string()
    } else if let Some(message) = info.payload().downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

#[cfg(test)]
mod panic_hook_tests {
    use super::*;
    use std::panic::AssertUnwindSafe;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    static PREVIOUS_HOOK_CALLED: AtomicBool = AtomicBool::new(false);

    #[test]
    fn panic_hook_records_message_and_delegates_to_previous() {
        PREVIOUS_HOOK_CALLED.store(false, Ordering::SeqCst);
        let log = Arc::new(RuntimeEventLog::new());
        let log_for_hook = Arc::clone(&log);

        let previous = panic::take_hook();
        panic::set_hook(Box::new(|_info| {
            PREVIOUS_HOOK_CALLED.store(true, Ordering::SeqCst);
        }));

        install_diagnostic_panic_hook(log_for_hook);

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            panic!("synthetic test panic");
        }));

        assert!(result.is_err());
        assert!(PREVIOUS_HOOK_CALLED.load(Ordering::SeqCst));

        let snapshot = log.snapshot();
        assert_eq!(snapshot.len(), 1);
        let event = &snapshot[0];
        assert_eq!(event.category, "panic");
        assert!(event.message.starts_with("synthetic test panic"));

        panic::set_hook(previous);
    }
}
