use super::*;

#[test]
fn fail_resource_preserves_value_when_status_has_cache() {
    assert!(async_status_has_value(&DevtoolAsyncStatus::Loaded));
    assert!(async_status_has_value(&DevtoolAsyncStatus::Loading {
        has_value: true,
    }));
    assert!(async_status_has_value(&DevtoolAsyncStatus::Failed {
        has_value: true,
        summary: "Already failed".to_string(),
    }));
    assert!(!async_status_has_value(&DevtoolAsyncStatus::Idle));
    assert!(!async_status_has_value(&DevtoolAsyncStatus::Loading {
        has_value: false,
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
