use iced::{system, Subscription, Task};
use nive_ui::theme::{self, Theme, ThemePreference};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeEvent(iced::theme::Mode);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeController {
    preference: ThemePreference,
    system_mode: iced::theme::Mode,
    effective: Theme,
}

impl ThemeController {
    pub fn new(preference: ThemePreference) -> Self {
        Self::with_system_mode(preference, iced::theme::Mode::None)
    }

    fn with_system_mode(preference: ThemePreference, system_mode: iced::theme::Mode) -> Self {
        let effective = Theme::from_mode(preference.resolve(system_mode));
        let controller = Self {
            preference,
            system_mode,
            effective,
        };
        controller.activate();
        controller
    }

    pub fn preference(self) -> ThemePreference {
        self.preference
    }

    pub fn effective(self) -> Theme {
        self.effective
    }

    pub fn initial_task(&self) -> Task<ThemeEvent> {
        system::theme().map(ThemeEvent)
    }

    pub fn subscription(&self) -> Subscription<ThemeEvent> {
        system::theme_changes().map(ThemeEvent)
    }

    pub fn handle(&mut self, event: ThemeEvent) -> bool {
        self.set_system_mode(event.0)
    }

    pub fn set_preference(&mut self, preference: ThemePreference) -> bool {
        self.preference = preference;
        self.synchronize()
    }

    fn set_system_mode(&mut self, system_mode: iced::theme::Mode) -> bool {
        self.system_mode = system_mode;
        self.synchronize()
    }

    fn activate(self) {
        theme::runtime::set_active(self.effective);
    }

    fn synchronize(&mut self) -> bool {
        let effective = Theme::from_mode(self.preference.resolve(self.system_mode));
        let changed = effective != self.effective;
        self.effective = effective;
        self.activate();
        changed
    }
}

#[cfg(test)]
mod theme_controller_tests {
    use super::*;
    use nive_ui::theme::testing::ThemeTestGuard;

    #[test]
    fn system_preference_tracks_system_mode() {
        let _guard = ThemeTestGuard::activate(Theme::Dark);
        let mut controller =
            ThemeController::with_system_mode(ThemePreference::System, iced::theme::Mode::Light);

        assert_eq!(controller.effective(), Theme::Light);
        assert_eq!(theme::active(), Theme::Light);
        assert!(controller.handle(ThemeEvent(iced::theme::Mode::Dark)));
        assert_eq!(controller.effective(), Theme::Dark);
        assert_eq!(theme::active(), Theme::Dark);
    }

    #[test]
    fn explicit_preference_ignores_system_changes() {
        let _guard = ThemeTestGuard::activate(Theme::Dark);
        let mut controller =
            ThemeController::with_system_mode(ThemePreference::Light, iced::theme::Mode::Dark);

        assert!(!controller.handle(ThemeEvent(iced::theme::Mode::Light)));
        assert_eq!(controller.effective(), Theme::Light);
        assert_eq!(theme::active(), Theme::Light);
    }

    #[test]
    fn explicit_preference_changes_synchronize_active_theme() {
        let _guard = ThemeTestGuard::activate(Theme::Dark);
        let mut controller = ThemeController::new(ThemePreference::System);

        assert!(controller.set_preference(ThemePreference::Light));
        assert_eq!(controller.effective(), Theme::Light);
        assert_eq!(theme::active(), Theme::Light);
    }

    #[test]
    fn owns_system_theme_task_and_subscription() {
        let _guard = ThemeTestGuard::activate(Theme::Dark);
        let controller = ThemeController::new(ThemePreference::System);

        let _: Task<ThemeEvent> = controller.initial_task();
        let _: Subscription<ThemeEvent> = controller.subscription();
    }
}
