#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
}

impl ThemeMode {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Light => "Nive Light",
            Self::Dark => "Nive Dark",
        }
    }

    pub const fn from_system(mode: iced::theme::Mode) -> Self {
        match mode {
            iced::theme::Mode::Light => Self::Light,
            iced::theme::Mode::Dark | iced::theme::Mode::None => Self::Dark,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub const fn resolve(self, system_mode: iced::theme::Mode) -> ThemeMode {
        match self {
            Self::System => ThemeMode::from_system(system_mode),
            Self::Light => ThemeMode::Light,
            Self::Dark => ThemeMode::Dark,
        }
    }
}

#[cfg(test)]
mod theme_mode_tests {
    use super::*;

    #[test]
    fn maps_system_light_mode_to_light_theme() {
        assert_eq!(
            ThemeMode::from_system(iced::theme::Mode::Light),
            ThemeMode::Light
        );
    }

    #[test]
    fn maps_system_dark_mode_to_dark_theme() {
        assert_eq!(
            ThemeMode::from_system(iced::theme::Mode::Dark),
            ThemeMode::Dark
        );
    }

    #[test]
    fn keeps_no_system_preference_on_default_dark_theme() {
        assert_eq!(
            ThemeMode::from_system(iced::theme::Mode::None),
            ThemeMode::default()
        );
    }

    #[test]
    fn system_preference_resolves_from_system_mode() {
        assert_eq!(
            ThemePreference::System.resolve(iced::theme::Mode::Light),
            ThemeMode::Light
        );
        assert_eq!(
            ThemePreference::System.resolve(iced::theme::Mode::Dark),
            ThemeMode::Dark
        );
    }

    #[test]
    fn explicit_preference_ignores_system_mode() {
        assert_eq!(
            ThemePreference::Light.resolve(iced::theme::Mode::Dark),
            ThemeMode::Light
        );
        assert_eq!(
            ThemePreference::Dark.resolve(iced::theme::Mode::Light),
            ThemeMode::Dark
        );
    }
}
