#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(u64);

#[derive(Debug, Clone, Default)]
pub struct RequestCounter {
    next_id: u64,
}

impl RequestId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}

impl RequestCounter {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> RequestId {
        let id = RequestId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        id
    }
}

#[cfg(test)]
mod request_id_tests {
    use super::*;

    #[test]
    fn next_returns_distinct_ids() {
        let mut counter = RequestCounter::default();

        assert_ne!(counter.next(), counter.next());
    }
}
