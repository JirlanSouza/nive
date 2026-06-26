use std::future::Future;

use iced::Task;

use crate::UserFacingResult;

/// Spawns a `UserFacingResult<T>` future as a `Task<Message>`, for async work
/// not tied to a [`Resource`](crate::Resource) or [`Operation`](crate::Operation)
/// (e.g. fire-and-forget that only toasts on completion).
pub fn perform<T, Message, Fut>(
    future: Fut,
    map: impl Fn(UserFacingResult<T>) -> Message + Send + 'static,
) -> Task<Message>
where
    T: Send + 'static,
    Message: Send + 'static,
    Fut: Future<Output = UserFacingResult<T>> + Send + 'static,
{
    Task::perform(future, map)
}

#[cfg(test)]
mod perform_tests {
    use super::*;

    #[test]
    fn perform_wraps_future_result() {
        let _task = perform(async { Ok::<_, crate::UserFacingError>(()) }, |result| {
            result
        });
    }
}
