use std::{future::Future, time::Duration};

use iced::Task;

use crate::{UserFacingError, UserFacingResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeEffect {
    Fail,
    DelayOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientTaskInjection {
    kind: ClientTaskInjectionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ClientTaskInjectionKind {
    Fail {
        error: UserFacingError,
        delay: Option<Duration>,
    },
    DelayOnly {
        delay: Option<Duration>,
    },
}

impl ClientTaskInjection {
    pub fn fail(error: UserFacingError, delay: Option<Duration>) -> Self {
        Self {
            kind: ClientTaskInjectionKind::Fail { error, delay },
        }
    }

    pub fn delay_only(delay: Option<Duration>) -> Self {
        Self {
            kind: ClientTaskInjectionKind::DelayOnly { delay },
        }
    }

    pub fn effect(&self) -> ProbeEffect {
        match self.kind {
            ClientTaskInjectionKind::Fail { .. } => ProbeEffect::Fail,
            ClientTaskInjectionKind::DelayOnly { .. } => ProbeEffect::DelayOnly,
        }
    }

    pub fn delay(&self) -> Option<Duration> {
        match self.kind {
            ClientTaskInjectionKind::Fail { delay, .. }
            | ClientTaskInjectionKind::DelayOnly { delay } => delay,
        }
    }

    pub fn error(&self) -> Option<&UserFacingError> {
        match &self.kind {
            ClientTaskInjectionKind::Fail { error, .. } => Some(error),
            ClientTaskInjectionKind::DelayOnly { .. } => None,
        }
    }
}

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

pub fn injected_client_task<T, Message, Fut>(
    injection: ClientTaskInjection,
    future: Fut,
    map: impl Fn(UserFacingResult<T>) -> Message + Send + 'static,
) -> Task<Message>
where
    T: Send + 'static,
    Message: Send + 'static,
    Fut: Future<Output = UserFacingResult<T>> + Send + 'static,
{
    client_task(
        async move {
            match injection.kind {
                ClientTaskInjectionKind::Fail { error, delay } => {
                    if let Some(delay) = delay {
                        tokio::time::sleep(delay).await;
                    }
                    Err(error)
                }
                ClientTaskInjectionKind::DelayOnly { delay } => {
                    if let Some(delay) = delay {
                        tokio::time::sleep(delay).await;
                    }
                    future.await
                }
            }
        },
        map,
    )
}

#[cfg(test)]
mod client_task_tests {
    use super::*;

    #[test]
    fn fail_injection_exposes_error_and_delay() {
        let error = UserFacingError::custom("project_catalog", "Could not load projects");
        let injection = ClientTaskInjection::fail(error.clone(), Some(Duration::from_millis(25)));

        assert_eq!(injection.effect(), ProbeEffect::Fail);
        assert_eq!(injection.delay(), Some(Duration::from_millis(25)));
        assert_eq!(injection.error(), Some(&error));
    }

    #[test]
    fn delay_only_injection_has_no_error() {
        let injection = ClientTaskInjection::delay_only(Some(Duration::from_millis(25)));

        assert_eq!(injection.effect(), ProbeEffect::DelayOnly);
        assert_eq!(injection.delay(), Some(Duration::from_millis(25)));
        assert!(injection.error().is_none());
    }
}
