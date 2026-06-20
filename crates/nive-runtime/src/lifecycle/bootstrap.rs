use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::Task;

use crate::{UserFacingError, UserFacingResult};

const DEFAULT_MINIMUM_DURATION: Duration = Duration::from_millis(900);

type BootstrapTaskFactory<B> = Arc<dyn Fn() -> Task<UserFacingResult<B>> + Send + Sync + 'static>;

/// Brand content shown on the bootstrap splash screen.
#[derive(Debug, Clone, PartialEq)]
pub struct BrandContent {
    title: String,
    subtitle: Option<String>,
    logo: Option<&'static [u8]>,
}

/// How a splash background image is fitted to the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackgroundFit {
    /// Scale the image to fit within the screen, preserving aspect ratio.
    Contain,
    /// Scale the image to cover the screen, preserving aspect ratio (default).
    #[default]
    Cover,
    /// Stretch the image to fill the screen, ignoring aspect ratio.
    Fill,
}

/// An SVG splash background with opacity and fit.
#[derive(Debug, Clone, PartialEq)]
pub struct SplashBackground {
    svg: &'static [u8],
    opacity: f32,
    fit: BackgroundFit,
}

/// Declarative specification of the application bootstrap sequence.
///
/// Carries the task factory that produces the bootstrap result, brand and
/// background assets, loading/failure copy, and a minimum splash duration. The
/// runtime owns the controller that runs repeatable attempts, rejects stale
/// results, enforces the minimum duration, supports retry and cancellation, and
/// transfers the result into [`Application::init`](crate::Application::init).
pub struct BootstrapSpec<B> {
    task: BootstrapTaskFactory<B>,
    brand: Option<BrandContent>,
    background: Option<SplashBackground>,
    loading_message: String,
    failure_title: String,
    failure_message: String,
    minimum_duration: Duration,
}

#[derive(Debug)]
pub(crate) struct BootstrapController<B> {
    attempt: u64,
    minimum_duration: Duration,
    state: BootstrapState<B>,
}

#[derive(Debug)]
enum BootstrapState<B> {
    Booting {
        started_at: Instant,
        pending_success: Option<B>,
    },
    Failed {
        error: UserFacingError,
        details_visible: bool,
    },
    Cancelled,
}

#[derive(Debug)]
pub(crate) enum BootstrapTransition<B> {
    Ignored,
    Pending,
    Failed,
    Ready(B),
}

pub(crate) fn minimum_duration_task<Message>(
    started_at: Instant,
    minimum_duration: Duration,
    on_elapsed: impl Fn() -> Message + Send + 'static,
) -> Task<Message>
where
    Message: Send + 'static,
{
    Task::perform(
        async move {
            let deadline = started_at + minimum_duration;
            let now = Instant::now();
            if deadline > now {
                tokio::time::sleep(deadline.duration_since(now)).await;
            }
        },
        move |()| on_elapsed(),
    )
}

impl BrandContent {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            logo: None,
        }
    }

    pub fn logo(mut self, svg: &'static [u8]) -> Self {
        self.logo = Some(svg);
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn subtitle_text(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }

    pub fn logo_svg(&self) -> Option<&'static [u8]> {
        self.logo
    }
}

impl SplashBackground {
    pub fn svg(svg: &'static [u8]) -> Self {
        Self {
            svg,
            opacity: 1.0,
            fit: BackgroundFit::Cover,
        }
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn fit(mut self, fit: BackgroundFit) -> Self {
        self.fit = fit;
        self
    }

    pub fn svg_bytes(&self) -> &'static [u8] {
        self.svg
    }

    pub fn opacity_value(&self) -> f32 {
        self.opacity
    }

    pub fn fit_mode(&self) -> BackgroundFit {
        self.fit
    }
}

impl<B> BootstrapSpec<B> {
    pub fn new(task: impl Fn() -> Task<UserFacingResult<B>> + Send + Sync + 'static) -> Self {
        Self {
            task: Arc::new(task),
            brand: None,
            background: None,
            loading_message: String::from("Preparing application"),
            failure_title: String::from("Application couldn't start"),
            failure_message: String::from(
                "A startup issue prevented the application from opening.",
            ),
            minimum_duration: DEFAULT_MINIMUM_DURATION,
        }
    }

    pub fn brand(mut self, brand: BrandContent) -> Self {
        self.brand = Some(brand);
        self
    }

    pub fn background(mut self, background: SplashBackground) -> Self {
        self.background = Some(background);
        self
    }

    pub fn loading_message(mut self, message: impl Into<String>) -> Self {
        self.loading_message = message.into();
        self
    }

    pub fn failure_title(mut self, title: impl Into<String>) -> Self {
        self.failure_title = title.into();
        self
    }

    pub fn failure_message(mut self, message: impl Into<String>) -> Self {
        self.failure_message = message.into();
        self
    }

    pub fn minimum_duration(mut self, duration: Duration) -> Self {
        self.minimum_duration = duration;
        self
    }

    pub(crate) fn run(&self) -> Task<UserFacingResult<B>> {
        (self.task)()
    }

    pub fn brand_content(&self) -> Option<&BrandContent> {
        self.brand.as_ref()
    }

    pub fn splash_background(&self) -> Option<&SplashBackground> {
        self.background.as_ref()
    }

    pub fn loading_text(&self) -> &str {
        self.loading_message.as_str()
    }

    pub fn failure_heading(&self) -> &str {
        self.failure_title.as_str()
    }

    pub fn failure_text(&self) -> &str {
        self.failure_message.as_str()
    }

    pub fn configured_minimum_duration(&self) -> Duration {
        self.minimum_duration
    }
}

impl<B> BootstrapController<B> {
    pub(crate) fn new(started_at: Instant, minimum_duration: Duration) -> Self {
        Self {
            attempt: 1,
            minimum_duration,
            state: BootstrapState::Booting {
                started_at,
                pending_success: None,
            },
        }
    }

    pub(crate) fn attempt(&self) -> u64 {
        self.attempt
    }

    pub(crate) fn finish(
        &mut self,
        attempt: u64,
        result: UserFacingResult<B>,
        now: Instant,
    ) -> BootstrapTransition<B> {
        if attempt != self.attempt {
            return BootstrapTransition::Ignored;
        }

        let BootstrapState::Booting {
            started_at,
            pending_success,
        } = &mut self.state
        else {
            return BootstrapTransition::Ignored;
        };

        match result {
            Ok(bootstrap) if now.duration_since(*started_at) >= self.minimum_duration => {
                BootstrapTransition::Ready(bootstrap)
            }
            Ok(bootstrap) => {
                *pending_success = Some(bootstrap);
                BootstrapTransition::Pending
            }
            Err(error) => {
                self.state = BootstrapState::Failed {
                    error,
                    details_visible: false,
                };
                BootstrapTransition::Failed
            }
        }
    }

    pub(crate) fn minimum_elapsed(&mut self, attempt: u64, now: Instant) -> BootstrapTransition<B> {
        if attempt != self.attempt {
            return BootstrapTransition::Ignored;
        }

        let BootstrapState::Booting {
            started_at,
            pending_success,
        } = &mut self.state
        else {
            return BootstrapTransition::Ignored;
        };

        if now.duration_since(*started_at) < self.minimum_duration {
            return BootstrapTransition::Ignored;
        }

        pending_success
            .take()
            .map(BootstrapTransition::Ready)
            .unwrap_or(BootstrapTransition::Pending)
    }

    pub(crate) fn retry(&mut self, started_at: Instant) -> Option<u64> {
        if !matches!(self.state, BootstrapState::Failed { .. }) {
            return None;
        }

        self.attempt = self.attempt.saturating_add(1);
        self.state = BootstrapState::Booting {
            started_at,
            pending_success: None,
        };
        Some(self.attempt)
    }

    pub(crate) fn cancel(&mut self) {
        self.state = BootstrapState::Cancelled;
    }

    pub(crate) fn error(&self) -> Option<&UserFacingError> {
        match &self.state {
            BootstrapState::Failed { error, .. } => Some(error),
            BootstrapState::Booting { .. } | BootstrapState::Cancelled => None,
        }
    }

    pub(crate) fn show_details(&mut self) {
        if let BootstrapState::Failed {
            details_visible, ..
        } = &mut self.state
        {
            *details_visible = true;
        }
    }

    pub(crate) fn close_details(&mut self) {
        if let BootstrapState::Failed {
            details_visible, ..
        } = &mut self.state
        {
            *details_visible = false;
        }
    }

    pub(crate) fn details_visible(&self) -> bool {
        matches!(
            self.state,
            BootstrapState::Failed {
                details_visible: true,
                ..
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn controller(started_at: Instant) -> BootstrapController<&'static str> {
        BootstrapController::new(started_at, Duration::from_millis(900))
    }

    #[test]
    fn successful_result_waits_for_minimum_duration() {
        let started_at = Instant::now();
        let mut controller = controller(started_at);

        let transition = controller.finish(1, Ok("ready"), started_at + Duration::from_millis(200));

        assert!(matches!(transition, BootstrapTransition::Pending));
        assert!(matches!(
            controller.minimum_elapsed(1, started_at + Duration::from_millis(900)),
            BootstrapTransition::Ready("ready")
        ));
    }

    #[test]
    fn failure_exposes_error_and_optional_details() {
        let started_at = Instant::now();
        let mut controller = controller(started_at);
        let error = UserFacingError::bootstrap("database unavailable");

        let transition = controller.finish(1, Err(error.clone()), started_at);

        assert!(matches!(transition, BootstrapTransition::Failed));
        assert_eq!(controller.error(), Some(&error));
        assert!(!controller.details_visible());

        controller.show_details();
        assert!(controller.details_visible());

        controller.close_details();
        assert!(!controller.details_visible());
    }

    #[test]
    fn retry_starts_new_attempt_and_rejects_stale_result() {
        let first_started_at = Instant::now();
        let mut controller = controller(first_started_at);
        let _ = controller.finish(
            1,
            Err(UserFacingError::bootstrap("failed")),
            first_started_at,
        );
        let second_started_at = first_started_at + Duration::from_secs(1);

        assert_eq!(controller.retry(second_started_at), Some(2));
        assert!(matches!(
            controller.finish(1, Ok("stale"), second_started_at),
            BootstrapTransition::Ignored
        ));
        assert_eq!(controller.attempt(), 2);
    }

    #[test]
    fn cancelled_controller_ignores_late_result() {
        let started_at = Instant::now();
        let mut controller = controller(started_at);

        controller.cancel();

        assert!(matches!(
            controller.finish(1, Ok("late"), started_at + Duration::from_secs(1)),
            BootstrapTransition::Ignored
        ));
    }

    #[test]
    fn stale_minimum_elapsed_is_ignored_after_retry() {
        let first_started_at = Instant::now();
        let mut controller = controller(first_started_at);
        let _ = controller.finish(
            1,
            Err(UserFacingError::bootstrap("failed")),
            first_started_at,
        );
        let second_started_at = first_started_at + Duration::from_secs(1);
        let _ = controller.retry(second_started_at);

        assert!(matches!(
            controller.minimum_elapsed(1, second_started_at + Duration::from_secs(1)),
            BootstrapTransition::Ignored
        ));
    }
}
