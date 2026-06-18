use iced::theme::Mode;
use nive_ui::theme::{self, Theme, ThemePreference};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeController {
    preference: ThemePreference,
    system_mode: Mode,
    effective: Theme,
}

impl ThemeController {
    pub fn new(preference: ThemePreference, system_mode: Mode) -> Self {
        let effective = Theme::from_mode(preference.resolve(system_mode));
        Self {
            preference,
            system_mode,
            effective,
        }
    }

    pub fn preference(self) -> ThemePreference {
        self.preference
    }

    pub fn system_mode(self) -> Mode {
        self.system_mode
    }

    pub fn effective(self) -> Theme {
        self.effective
    }

    pub fn set_preference(&mut self, preference: ThemePreference) -> bool {
        self.preference = preference;
        self.synchronize()
    }

    pub fn set_system_mode(&mut self, system_mode: Mode) -> bool {
        self.system_mode = system_mode;
        self.synchronize()
    }

    pub fn activate(self) {
        theme::set_active(self.effective);
    }

    fn synchronize(&mut self) -> bool {
        let effective = Theme::from_mode(self.preference.resolve(self.system_mode));
        let changed = effective != self.effective;
        self.effective = effective;
        changed
    }
}

#[cfg(test)]
mod theme_controller_tests {
    use super::*;

    #[test]
    fn system_preference_tracks_system_mode() {
        let mut controller = ThemeController::new(ThemePreference::System, Mode::Light);

        assert_eq!(controller.effective(), Theme::Light);
        assert!(controller.set_system_mode(Mode::Dark));
        assert_eq!(controller.effective(), Theme::Dark);
    }

    #[test]
    fn explicit_preference_ignores_system_changes() {
        let mut controller = ThemeController::new(ThemePreference::Light, Mode::Dark);

        assert!(!controller.set_system_mode(Mode::Light));
        assert_eq!(controller.effective(), Theme::Light);
    }
}
