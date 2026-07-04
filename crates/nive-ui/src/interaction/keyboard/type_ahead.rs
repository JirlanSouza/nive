use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct TypeAhead {
    buffer: String,
    timeout: Duration,
    last_input: Option<Instant>,
}

#[allow(dead_code)]
impl TypeAhead {
    pub(crate) fn new(timeout: Duration) -> Self {
        Self {
            buffer: String::new(),
            timeout,
            last_input: None,
        }
    }

    pub(crate) fn push(&mut self, ch: char, now: Instant) -> &str {
        if self
            .last_input
            .is_some_and(|last| now.duration_since(last) > self.timeout)
        {
            self.buffer.clear();
        }

        self.buffer.push(ch);
        self.last_input = Some(now);
        &self.buffer
    }

    pub(crate) fn reset(&mut self) {
        self.buffer.clear();
        self.last_input = None;
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub(crate) fn matches(label: &str, prefix: &str) -> bool {
        label.to_lowercase().starts_with(&prefix.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_ahead_matches_case_insensitive_prefix_and_resets() {
        let mut type_ahead = TypeAhead::new(Duration::from_secs(1));
        let now = Instant::now();

        assert_eq!(type_ahead.push('R', now), "R");
        assert_eq!(type_ahead.push('s', now + Duration::from_millis(200)), "Rs");
        assert!(TypeAhead::matches("Rust", "ru"));
        assert!(!TypeAhead::matches("Rust", "rs"));

        assert_eq!(type_ahead.push('c', now + Duration::from_secs(2)), "c");
        type_ahead.reset();
        assert!(type_ahead.is_empty());
    }
}
