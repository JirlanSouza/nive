use crate::{RequestId, UserFacingError};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum OperationState<C> {
    #[default]
    Idle,
    Running {
        request_id: RequestId,
        context: C,
    },
    Failed {
        error: UserFacingError,
        context: C,
    },
}

impl<C> OperationState<C> {
    pub fn start(&mut self, request_id: RequestId, context: C) {
        *self = Self::Running {
            request_id,
            context,
        };
    }

    pub fn finish(&mut self, request_id: RequestId) -> Option<C> {
        let current = std::mem::replace(self, Self::Idle);
        match current {
            Self::Running {
                request_id: running_request_id,
                context,
            } if running_request_id == request_id => Some(context),
            other => {
                *self = other;
                None
            }
        }
    }

    pub fn fail(&mut self, request_id: RequestId, error: UserFacingError) -> bool {
        let current = std::mem::replace(self, Self::Idle);
        match current {
            Self::Running {
                request_id: running_request_id,
                context,
            } if running_request_id == request_id => {
                *self = Self::Failed { error, context };
                true
            }
            other => {
                *self = other;
                false
            }
        }
    }

    pub fn clear(&mut self) {
        *self = Self::Idle;
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    pub fn error(&self) -> Option<&UserFacingError> {
        match self {
            Self::Failed { error, .. } => Some(error),
            Self::Idle | Self::Running { .. } => None,
        }
    }

    pub fn failed_context(&self) -> Option<&C> {
        match self {
            Self::Failed { context, .. } => Some(context),
            Self::Idle | Self::Running { .. } => None,
        }
    }
}

#[cfg(test)]
mod operation_state_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Context {
        id: String,
    }

    #[test]
    fn start_marks_operation_running_with_context() {
        let mut state = OperationState::default();

        state.start(RequestId::new(7), Context { id: "p1".into() });

        assert!(state.is_running());
        assert!(state.error().is_none());
    }

    #[test]
    fn finish_matching_request_returns_context_and_clears_state() {
        let mut state = OperationState::default();
        state.start(RequestId::new(7), Context { id: "p1".into() });

        let context = state.finish(RequestId::new(7));

        assert_eq!(context.map(|context| context.id), Some("p1".into()));
        assert!(!state.is_running());
        assert!(state.error().is_none());
    }

    #[test]
    fn finish_stale_request_preserves_running_state() {
        let mut state = OperationState::default();
        state.start(RequestId::new(7), Context { id: "p1".into() });

        let context = state.finish(RequestId::new(6));

        assert!(context.is_none());
        assert!(state.is_running());
    }

    #[test]
    fn fail_matching_request_preserves_context_and_exposes_error() {
        let mut state = OperationState::default();
        state.start(RequestId::new(7), Context { id: "p1".into() });

        let failed = state.fail(
            RequestId::new(7),
            UserFacingError::project_catalog("Project not found (project_id: p1)"),
        );

        assert!(failed);
        assert_eq!(
            state.error().map(UserFacingError::summary),
            Some("Project not found")
        );
        assert_eq!(
            state.failed_context().map(|context| context.id.as_str()),
            Some("p1")
        );
    }

    #[test]
    fn fail_stale_request_preserves_running_state() {
        let mut state = OperationState::default();
        state.start(RequestId::new(7), Context { id: "p1".into() });

        let failed = state.fail(
            RequestId::new(6),
            UserFacingError::project_catalog("Project not found (project_id: p1)"),
        );

        assert!(!failed);
        assert!(state.is_running());
        assert!(state.error().is_none());
    }
}
