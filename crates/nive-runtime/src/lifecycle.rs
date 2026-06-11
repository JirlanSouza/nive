use std::time::{Duration, Instant};

use crate::UserFacingError;

const DEFAULT_MIN_SPLASH_DURATION: Duration = Duration::from_millis(900);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplashConfig {
    pub min_duration: Duration,
}

impl SplashConfig {
    pub const DEFAULT: SplashConfig = SplashConfig {
        min_duration: DEFAULT_MIN_SPLASH_DURATION,
    };
}

impl Default for SplashConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug)]
pub enum AppPhase<E = UserFacingError, P = (), R = ()> {
    Booting {
        started_at: Instant,
        pending_success: Option<P>,
    },
    BootFailed(E),
    Ready(R),
}

impl<E, P, R> AppPhase<E, P, R> {
    pub fn booting(started_at: Instant) -> Self {
        Self::Booting {
            started_at,
            pending_success: None,
        }
    }

    pub fn failed(error: E) -> Self {
        Self::BootFailed(error)
    }

    pub fn ready(state: R) -> Self {
        Self::Ready(state)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    pub fn is_booting(&self) -> bool {
        matches!(self, Self::Booting { .. })
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::BootFailed(_))
    }

    pub fn booting_started_at(&self) -> Option<Instant> {
        match self {
            Self::Booting { started_at, .. } => Some(*started_at),
            _ => None,
        }
    }

    pub fn splash_elapsed(&self, now: Instant, config: &SplashConfig) -> bool {
        if let Self::Booting { started_at, .. } = self {
            now.duration_since(*started_at) >= config.min_duration
        } else {
            false
        }
    }

    pub fn accepts_splash_elapsed(
        &self,
        expected_started_at: Instant,
        now: Instant,
        config: &SplashConfig,
    ) -> bool {
        if let Self::Booting { started_at, .. } = self {
            *started_at == expected_started_at
                && now.duration_since(*started_at) >= config.min_duration
        } else {
            false
        }
    }

    pub fn take_pending_success(&mut self) -> Option<P> {
        if let Self::Booting {
            pending_success, ..
        } = self
        {
            pending_success.take()
        } else {
            None
        }
    }

    pub fn set_pending_success(&mut self, pending: P) {
        if let Self::Booting {
            pending_success, ..
        } = self
        {
            *pending_success = Some(pending);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn booting_starts_with_no_pending_success() {
        let started_at = Instant::now();
        let phase: AppPhase<(), (), ()> = AppPhase::booting(started_at);

        assert!(phase.is_booting());
        assert!(!phase.is_ready());
        assert!(!phase.is_failed());
        assert_eq!(phase.booting_started_at(), Some(started_at));
    }

    #[test]
    fn booting_with_pending_success_stores_value() {
        let started_at = Instant::now();
        let mut phase: AppPhase<(), String, ()> = AppPhase::booting(started_at);
        phase.set_pending_success("test_data".to_string());

        assert_eq!(phase.take_pending_success(), Some("test_data".to_string()));
        assert_eq!(phase.take_pending_success(), None);
    }

    #[test]
    fn splash_elapsed_checks_minimum_duration() {
        let config = SplashConfig::default();
        let started_at = Instant::now();
        let phase: AppPhase<(), (), ()> = AppPhase::booting(started_at);

        let before_minimum = started_at + Duration::from_millis(899);
        assert!(!phase.splash_elapsed(before_minimum, &config));
        assert!(phase.splash_elapsed(started_at + config.min_duration, &config));
    }

    #[test]
    fn accepts_splash_elapsed_rejects_stale_started_at() {
        let config = SplashConfig::default();
        let previous_started_at = Instant::now();
        let current_started_at = previous_started_at + Duration::from_millis(10);
        let phase: AppPhase<(), (), ()> = AppPhase::booting(current_started_at);
        let after_minimum = current_started_at + config.min_duration;

        assert!(!phase.accepts_splash_elapsed(previous_started_at, after_minimum, &config));
        assert!(phase.accepts_splash_elapsed(current_started_at, after_minimum, &config));
    }

    #[test]
    fn set_pending_success_is_no_op_outside_booting() {
        let mut phase: AppPhase<(), String, ()> = AppPhase::ready(());
        phase.set_pending_success("ignored".to_string());

        assert!(phase.is_ready());
    }

    #[test]
    fn take_pending_success_returns_none_outside_booting() {
        let mut phase: AppPhase<(), (), String> = AppPhase::ready("done".to_string());

        assert_eq!(phase.take_pending_success(), None);
    }
}
