use super::*;
use crate::inspect::SimulableSnapshot;

#[test]
fn snapshot_has_value_returns_true_for_loaded_and_loading_with_cache() {
    assert!(snapshot_has_value(&SimulableSnapshot::Loaded));
    assert!(snapshot_has_value(&SimulableSnapshot::Loading {
        has_value: true
    }));
    assert!(snapshot_has_value(&SimulableSnapshot::Failed {
        has_value: true,
        summary: "err".to_string()
    }));
    assert!(!snapshot_has_value(&SimulableSnapshot::Idle));
    assert!(!snapshot_has_value(&SimulableSnapshot::Loading {
        has_value: false
    }));
}

#[test]
fn ellipsize_end_clamps_long_text() {
    assert_eq!(ellipsize_end("short".to_string(), 10), "short");
    assert_eq!(
        ellipsize_end("welcome.new_project".to_string(), 10),
        "welcome..."
    );
    assert_eq!(ellipsize_end("abc".to_string(), 0), "");
}
