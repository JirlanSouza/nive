use std::sync::atomic::Ordering;

use iced::Padding;

use super::component::{ControlMetrics, ControlMetricsScale, ControlSize};
use super::scheme::{Theme, ACTIVE_THEME};
use super::spacing::{GapRole, PaddingRole, SpaceStep, SpacingScale};
use super::ThemeMode;

pub fn active() -> Theme {
    let value = ACTIVE_THEME.load(Ordering::Relaxed);
    debug_assert!(
        value <= Theme::Dark as u8,
        "Invalid active theme value: {value}"
    );

    Theme::from_active_value(value)
}

pub fn active_mode() -> ThemeMode {
    active().mode()
}

pub fn set_active(theme: Theme) {
    ACTIVE_THEME.store(theme as u8, Ordering::Relaxed);
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
