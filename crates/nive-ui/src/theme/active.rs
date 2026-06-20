use std::sync::{Mutex, MutexGuard};

use iced::Padding;

use super::component::{ControlMetrics, ControlMetricsScale, ControlSize};
use super::scheme::Theme;
use super::spacing::{GapRole, PaddingRole, SpaceStep, SpacingScale};

static ACTIVE_THEME: Mutex<Theme> = Mutex::new(Theme::Dark);
static TEST_THEME_LOCK: Mutex<()> = Mutex::new(());

pub fn active() -> Theme {
    *ACTIVE_THEME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(super) fn set_active(theme: Theme) {
    *ACTIVE_THEME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = theme;
}

pub fn spacing() -> SpacingScale {
    active().spacing()
}

pub fn controls() -> ControlMetricsScale {
    active().controls()
}

pub fn control_metrics(size: ControlSize) -> ControlMetrics {
    active().control_metrics(size)
}

pub fn space(step: SpaceStep) -> f32 {
    active().space(step)
}

pub fn gap(role: GapRole) -> f32 {
    active().gap(role)
}

pub fn padding(role: PaddingRole) -> Padding {
    active().padding(role)
}

#[doc(hidden)]
pub struct ThemeTestGuard {
    previous: Theme,
    _lock: MutexGuard<'static, ()>,
}

impl ThemeTestGuard {
    pub fn activate(theme: Theme) -> Self {
        let lock = TEST_THEME_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous = active();
        set_active(theme);

        Self {
            previous,
            _lock: lock,
        }
    }
}

impl Drop for ThemeTestGuard {
    fn drop(&mut self) {
        set_active(self.previous);
    }
}
