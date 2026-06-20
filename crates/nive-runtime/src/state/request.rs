use std::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(NonZeroU64);

#[derive(Debug, Clone)]
pub struct RequestCounter {
    next_id: u64,
}

impl RequestId {
    pub fn new(value: u64) -> Self {
        Self(NonZeroU64::new(value).unwrap_or(NonZeroU64::MIN))
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl RequestCounter {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> RequestId {
        let id = RequestId::new(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        if self.next_id == 0 {
            self.next_id = 1;
        }
        id
    }
}

impl Default for RequestCounter {
    fn default() -> Self {
        Self { next_id: 1 }
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

    #[test]
    fn first_id_is_one() {
        assert_eq!(RequestCounter::default().next().get(), 1);
    }
}
