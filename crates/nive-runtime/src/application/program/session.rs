use iced::{window, Point, Size, Task};

use crate::application::program::{
    CoreMessage, NiveMessage, ProbeCatalogEntry, Program, RuntimeMessage, SettingsRuntime,
};
use crate::application::{Application, WindowContext};
use crate::{RuntimeSession, SettingsConfig, SettingsErrorKind};

impl<A, P> Program<A, P>
where
    A: Application,
    P: ProbeCatalogEntry,
{
    /// Resolves the theme preference after consulting `Application::theme`
    /// using the documented tie-break rule:
    /// - `Light`/`Dark` from `Application::theme` wins.
    /// - `System` (or no app yet) defers to `emitted` if `Some`, else the
    ///   runtime's current preference (already set by the OS handler or
    ///   settings).
    pub(super) fn resolve_theme_preference(
        &self,
        emitted: Option<crate::ThemePreference>,
    ) -> crate::ThemePreference {
        let context = self.core.context();
        let window = self.first_app_window_context();
        let app_pref = self.app.as_ref().map(|app| app.theme(context, window));
        match app_pref {
            Some(crate::ThemePreference::Light) | Some(crate::ThemePreference::Dark) => {
                app_pref.unwrap()
            }
            _ => emitted.unwrap_or_else(|| self.core.theme.preference()),
        }
    }

    pub(super) fn first_app_window_context(&self) -> Option<WindowContext<A::Window>> {
        self.core
            .registry
            .most_recent_app_window()
            .map(|handle| WindowContext {
                id: handle.id,
                kind: handle.kind,
                role: handle.role,
            })
    }

    pub(super) fn save_theme_preference(
        &mut self,
        preference: crate::ThemePreference,
    ) -> Task<RuntimeMessage<A, P>> {
        let Some(settings) = self.core.settings.as_mut() else {
            return Task::none();
        };

        settings.session.set_theme_preference(Some(preference));
        self.save_runtime_session()
    }

    pub(super) fn save_window_size(
        &mut self,
        window_id: window::Id,
        size: Size,
    ) -> Task<RuntimeMessage<A, P>> {
        let Some(key) = self.core.window_session_key(window_id) else {
            return Task::none();
        };
        let Some(settings) = self.core.settings.as_mut() else {
            return Task::none();
        };

        if settings.session.set_window_size(key, size) {
            self.save_runtime_session()
        } else {
            Task::none()
        }
    }

    pub(super) fn save_window_position(
        &mut self,
        window_id: window::Id,
        position: Point,
    ) -> Task<RuntimeMessage<A, P>> {
        let Some(key) = self.core.window_session_key(window_id) else {
            return Task::none();
        };
        let Some(settings) = self.core.settings.as_mut() else {
            return Task::none();
        };

        if settings.session.set_window_position(key, position) {
            self.save_runtime_session()
        } else {
            Task::none()
        }
    }

    pub(super) fn save_runtime_session(&self) -> Task<RuntimeMessage<A, P>> {
        let Some(settings) = self.core.settings.as_ref() else {
            return Task::none();
        };

        let config = settings.config.clone();
        let session = settings.session.clone();

        Task::perform(
            async move { crate::settings::save_session(&config, &session) },
            |result| NiveMessage::Core(CoreMessage::SettingsSaved(result)),
        )
    }
}

impl SettingsRuntime {
    pub(super) fn load(config: Option<&SettingsConfig>) -> Option<Self> {
        let config = config?.clone();
        let session = match crate::settings::load_session(&config) {
            Ok(Some(session)) => session,
            Ok(None) => {
                log::debug!(
                    target: "nive_runtime::settings",
                    "settings.load_missing path={}",
                    config.path().display()
                );
                RuntimeSession::default()
            }
            Err(error) if error.kind() == SettingsErrorKind::UnsupportedVersion => {
                log::warn!(
                    target: "nive_runtime::settings",
                    "settings.unsupported_version path={} version={}",
                    error.path().display(),
                    error.detail()
                );
                RuntimeSession::default()
            }
            Err(error) => {
                log::warn!(
                    target: "nive_runtime::settings",
                    "settings.load_failed path={} error={}",
                    error.path().display(),
                    error.detail()
                );
                RuntimeSession::default()
            }
        };

        Some(Self { config, session })
    }
}
