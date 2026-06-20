use std::future::Future;

use iced::Task;

use crate::UserFacingResult;

pub fn client_task<T, Message, Fut>(
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
mod client_task_tests {
    use super::*;

    #[test]
    fn client_task_wraps_future_result() {
        let _task = client_task(async { Ok::<_, crate::UserFacingError>(()) }, |result| {
            result
        });
    }
}
