use std::{
    cell::RefCell,
    sync::{Mutex, MutexGuard},
};

use iced::Padding;

use super::component::{ControlMetrics, ControlMetricsScale, ControlSize, FormControlMetrics};
use super::density::ThemeDensity;
use super::scheme::Theme;
use super::spacing::{GapRole, PaddingRole, SpaceStep, SpacingScale};

static ACTIVE_THEME: Mutex<Theme> = Mutex::new(Theme::Dark);
static TEST_THEME_LOCK: Mutex<()> = Mutex::new(());

thread_local! {
    static TEST_THEME_OVERRIDE: RefCell<Option<Theme>> = const { RefCell::new(None) };
}

pub fn active() -> Theme {
    if let Some(theme) = TEST_THEME_OVERRIDE.with(|override_theme| *override_theme.borrow()) {
        return theme;
    }

    *ACTIVE_THEME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(super) fn set_active(theme: Theme) {
    let test_override_updated = TEST_THEME_OVERRIDE.with(|override_theme| {
        let mut override_theme = override_theme.borrow_mut();
        if override_theme.is_some() {
            *override_theme = Some(theme);
            true
        } else {
            false
        }
    });

    if test_override_updated {
        return;
    }

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

pub fn density() -> ThemeDensity {
    active().density()
}

pub fn control_metrics(size: ControlSize) -> ControlMetrics {
    active().control_metrics(size)
}

pub fn form_control_metrics(size: ControlSize) -> FormControlMetrics {
    active().form_control_metrics(size)
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
    previous: Option<Theme>,
    _lock: MutexGuard<'static, ()>,
}

impl ThemeTestGuard {
    pub fn activate(theme: Theme) -> Self {
        let lock = TEST_THEME_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous =
            TEST_THEME_OVERRIDE.with(|override_theme| override_theme.replace(Some(theme)));

        Self {
            previous,
            _lock: lock,
        }
    }
}

impl Drop for ThemeTestGuard {
    fn drop(&mut self) {
        TEST_THEME_OVERRIDE.with(|override_theme| {
            override_theme.replace(self.previous);
        });
    }
}
