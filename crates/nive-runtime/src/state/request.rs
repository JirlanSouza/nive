use std::num::NonZeroU64;

use crate::UserFacingResult;

/// Identifies an in-flight async request. Stays `Copy` and message-friendly.
///
/// On the blessed path, `RequestId` is carried inside [`Settled<T>`] and apps
/// never name it directly. It remains public for the low-level manual path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(NonZeroU64);

/// Monotonic counter that mints [`RequestId`]s.
///
/// Used internally by [`Resource<T>`](super::resource::Resource) and
/// [`Operation<C>`](super::operation::Operation). Not part of the public API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RequestCounter {
    next_id: u64,
}

impl RequestId {
    pub(crate) fn new(value: u64) -> Self {
        Self(NonZeroU64::new(value).unwrap_or(NonZeroU64::MIN))
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl RequestCounter {
    pub(crate) fn next(&mut self) -> RequestId {
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

/// Carries the in-flight [`RequestId`] together with the async result.
///
/// `Message` variants carry a single `Settled<T>` value, and
/// [`Resource::settle`](super::resource::Resource::settle) /
/// [`Operation::settle`](super::operation::Operation::settle) consume it.
/// The token is invisible plumbing on the blessed `load`/`run` path.
#[derive(Debug, Clone)]
pub struct Settled<T> {
    pub(crate) token: RequestId,
    pub(crate) result: UserFacingResult<T>,
}

impl<T> Settled<T> {
    /// Bundle a `RequestId` and a `UserFacingResult<T>` into a `Settled`.
    pub fn new(token: RequestId, result: UserFacingResult<T>) -> Self {
        Self { token, result }
    }

    pub fn token(&self) -> RequestId {
        self.token
    }

    pub fn result(&self) -> &UserFacingResult<T> {
        &self.result
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
