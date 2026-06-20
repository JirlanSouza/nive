use crate::UserFacingError;
use nive_ui::widgets::{ErrorPresentation, ResourceStatusPresentation};

/// Reusable state machine for an async-loaded resource.
///
/// Tracks idle, loading (optionally retaining a stale value for refreshing),
/// loaded, and failed states. Implements the UI presentation contracts
/// ([`ErrorPresentation`], [`ResourceStatusPresentation`]) so `nive-ui`
/// feedback widgets can render it without depending on app-domain types.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AsyncState<T> {
    /// No value has been requested yet.
    #[default]
    Idle,
    /// A load is in progress, optionally retaining a stale value while refreshing.
    Loading { value: Option<T> },
    /// A value was loaded successfully.
    Loaded(T),
    /// The last load failed, optionally retaining a stale value.
    Failed {
        value: Option<T>,
        error: UserFacingError,
    },
}

impl<T> AsyncState<T> {
    pub fn new(value: T) -> Self {
        Self::Loaded(value)
    }

    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Loaded(value)
            | Self::Loading { value: Some(value) }
            | Self::Failed {
                value: Some(value), ..
            } => Some(value),
            Self::Idle | Self::Loading { value: None } | Self::Failed { value: None, .. } => None,
        }
    }

    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading { .. })
    }

    pub fn error(&self) -> Option<&UserFacingError> {
        match self {
            Self::Failed { error, .. } => Some(error),
            Self::Idle | Self::Loading { .. } | Self::Loaded(_) => None,
        }
    }

    pub fn set_idle(&mut self) {
        *self = Self::Idle;
    }

    pub fn set_loading(&mut self) {
        *self = Self::Loading { value: None };
    }

    pub fn set_refreshing(&mut self) {
        let value = self.take_value();
        *self = Self::Loading { value };
    }

    pub fn set_loaded(&mut self, value: T) {
        *self = Self::Loaded(value);
    }

    pub fn set_failed(&mut self, error: UserFacingError) {
        let value = self.take_value();
        *self = Self::Failed { value, error };
    }

    pub fn set_failed_empty(&mut self, error: UserFacingError) {
        *self = Self::Failed { value: None, error };
    }

    pub fn dismiss_error(&mut self) {
        let current = std::mem::take(self);
        *self = match current {
            Self::Failed {
                value: Some(value), ..
            } => Self::Loaded(value),
            Self::Failed { value: None, .. } => Self::Idle,
            other => other,
        };
    }

    fn take_value(&mut self) -> Option<T> {
        match std::mem::replace(self, Self::Idle) {
            Self::Loaded(value)
            | Self::Loading { value: Some(value) }
            | Self::Failed {
                value: Some(value), ..
            } => Some(value),
            Self::Idle | Self::Loading { value: None } | Self::Failed { value: None, .. } => None,
        }
    }
}

impl<T> ResourceStatusPresentation for AsyncState<T> {
    fn is_refreshing(&self) -> bool {
        matches!(self, Self::Loading { value: Some(_) })
    }

    fn has_value(&self) -> bool {
        self.value().is_some()
    }

    fn error(&self) -> Option<&dyn ErrorPresentation> {
        self.error().map(|error| error as &dyn ErrorPresentation)
    }
}

#[cfg(test)]
mod async_state_tests {
    use super::*;
    use nive_ui::widgets::ResourceStatusPresentation;

    #[test]
    fn set_loaded_updates_value_and_clears_error() {
        let mut state = AsyncState::new(1);
        state.set_failed(UserFacingError::custom(
            "record_catalog",
            "Record not found (record_id: r1)",
        ));

        state.set_loaded(2);

        assert_eq!(state.value(), Some(&2));
        assert!(!state.is_loading());
        assert!(state.error().is_none());
    }

    #[test]
    fn set_refreshing_preserves_cached_value() {
        let mut state = AsyncState::new("cached");

        state.set_refreshing();

        assert!(state.is_loading());
        assert_eq!(state.value(), Some(&"cached"));
    }

    #[test]
    fn set_loading_clears_cached_value() {
        let mut state = AsyncState::new("cached");

        state.set_loading();

        assert!(state.is_loading());
        assert!(state.value().is_none());
    }

    #[test]
    fn set_failed_preserves_cached_value_and_exposes_error() {
        let mut state = AsyncState::new("cached");

        state.set_failed(UserFacingError::custom(
            "record_catalog",
            "Record not found (record_id: r1)",
        ));

        assert_eq!(state.value(), Some(&"cached"));
        assert_eq!(
            state.error().map(UserFacingError::summary),
            Some("Record not found")
        );
    }

    #[test]
    fn set_failed_empty_clears_cached_value_and_exposes_error() {
        let mut state = AsyncState::new("cached");

        state.set_failed_empty(UserFacingError::custom(
            "record_catalog",
            "Record not found (record_id: r1)",
        ));

        assert!(state.value().is_none());
        assert_eq!(
            state.error().map(UserFacingError::summary),
            Some("Record not found")
        );
    }

    #[test]
    fn dismiss_error_preserves_cached_value() {
        let mut state = AsyncState::new("cached");
        state.set_failed(UserFacingError::custom(
            "record_catalog",
            "Record not found (record_id: r1)",
        ));

        state.dismiss_error();

        assert_eq!(state.value(), Some(&"cached"));
        assert!(state.error().is_none());
    }

    #[test]
    fn dismiss_error_without_cache_returns_to_idle() {
        let mut state = AsyncState::<&str>::Idle;
        state.set_failed_empty(UserFacingError::custom(
            "record_catalog",
            "Record not found (record_id: r1)",
        ));

        state.dismiss_error();

        assert_eq!(state, AsyncState::Idle);
    }

    #[test]
    fn resource_presentation_distinguishes_refresh_from_initial_load() {
        let refreshing = AsyncState::Loading {
            value: Some("cached"),
        };
        let loading = AsyncState::<&str>::Loading { value: None };

        assert!(ResourceStatusPresentation::is_refreshing(&refreshing));
        assert!(!ResourceStatusPresentation::is_refreshing(&loading));
        assert!(ResourceStatusPresentation::has_value(&refreshing));
        assert!(!ResourceStatusPresentation::has_value(&loading));
    }

    #[test]
    fn resource_presentation_exposes_failed_error() {
        let state = AsyncState::<&str>::Failed {
            value: Some("cached"),
            error: UserFacingError::custom("record_catalog", "Refresh failed"),
        };

        assert_eq!(
            ResourceStatusPresentation::error(&state).map(ErrorPresentation::summary),
            Some("Refresh failed")
        );
    }
}
