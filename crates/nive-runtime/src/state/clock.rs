use std::time::{SystemTime, UNIX_EPOCH};

pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

pub fn relative_time_label(updated_at: i64, now: i64) -> String {
    let elapsed = now.saturating_sub(updated_at).max(0);

    match elapsed {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{} min ago", elapsed / 60),
        3600..=86399 => format!("{}h ago", elapsed / 3600),
        86400..=604799 => format!("{} days ago", elapsed / 86400),
        _ => format!("{} weeks ago", elapsed / 604800),
    }
}

#[cfg(test)]
mod clock_tests {
    use super::*;

    #[test]
    fn relative_time_formats_recent_values() {
        assert_eq!(relative_time_label(1_000, 1_000), "just now");
        assert_eq!(relative_time_label(940, 1_000), "1 min ago");
        assert_eq!(relative_time_label(1_000 - 3600, 1_000), "1h ago");
    }

    #[test]
    fn relative_time_saturates_future_values() {
        assert_eq!(relative_time_label(2_000, 1_000), "just now");
    }
}
