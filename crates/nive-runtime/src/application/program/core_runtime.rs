use std::collections::{HashMap, HashSet};

use iced::{window, Size};

use super::{NiveCore, SettingsRuntime};
use crate::application::{ApplicationConfig, Context, WindowContext, WindowQuery};
use crate::{
    ThemeController, ToastInsets, ToastPosition, ToastState, WindowHandle, WindowRegistry,
    WindowRole, WindowSpec,
};

pub(super) fn clamp_window_size(
    size: Size,
    min_size: Option<Size>,
    max_size: Option<Size>,
) -> Size {
    let min_width = min_size.map(|size| size.width).unwrap_or(1.0);
    let min_height = min_size.map(|size| size.height).unwrap_or(1.0);
    let max_width = max_size
        .map(|size| size.width)
        .unwrap_or(f32::MAX)
        .max(min_width);
    let max_height = max_size
        .map(|size| size.height)
        .unwrap_or(f32::MAX)
        .max(min_height);

    Size::new(
        size.width.clamp(min_width, max_width),
        size.height.clamp(min_height, max_height),
    )
}

impl<K, Message> NiveCore<K, Message>
where
    K: Copy + Eq,
{
    pub(super) fn new<B>(
        config: &ApplicationConfig<K, B>,
        settings: Option<SettingsRuntime>,
    ) -> Self {
        let core = Self {
            app_id: config.app_id.clone(),
            app_name: config.app_name.clone(),
            windows: config
                .windows
                .iter()
                .map(|registration| (registration.kind, registration.spec))
                .collect(),
            initial_windows: config.initial_windows.clone(),
            registry: WindowRegistry::new(),
            theme: ThemeController::with_catalog(config.theme_preference, config.theme_catalog),
            exiting: false,
            window_icon: config.window_icon.clone(),
            toast_position: config.toast_position,
            toast_insets: config.toast_insets,
            toasts: ToastState::default(),
            pending_app_closes: HashSet::new(),
            pending_replacements: HashMap::new(),
            settings,
        };
        core.report_duplicate_window_session_keys();
        core
    }

    pub(super) fn context(&self) -> Context<'_, K> {
        Context {
            app_id: self.app_id.as_str(),
            app_name: self.app_name.as_str(),
            theme: self.theme.effective(),
            theme_preference: self.theme.preference(),
            windows: WindowQuery {
                registry: &self.registry,
            },
            exiting: self.exiting,
        }
    }

    pub(super) fn window_context(&self, window_id: window::Id) -> Option<WindowContext<K>> {
        self.registry.get(window_id).map(Into::into)
    }

    pub(super) fn window_spec(&self, kind: K) -> Option<WindowSpec> {
        self.windows
            .iter()
            .find(|(registered_kind, _)| *registered_kind == kind)
            .map(|(_, spec)| self.apply_window_session(*spec))
    }

    pub(super) fn apply_window_session(&self, mut spec: WindowSpec) -> WindowSpec {
        let Some(key) = spec.configured_session_key() else {
            return spec;
        };
        let Some(session) = self
            .settings
            .as_ref()
            .and_then(|settings| settings.session.window(key))
        else {
            return spec;
        };

        if let Some(size) = session.size() {
            spec.size = clamp_window_size(size, spec.min_size, spec.max_size);
        }
        if let Some(position) = session.position() {
            spec.position = window::Position::Specific(position);
        }

        spec
    }

    pub(super) fn window_session_key(&self, window_id: window::Id) -> Option<&'static str> {
        let kind = self.registry.kind(window_id)?;
        self.windows
            .iter()
            .find(|(registered_kind, _)| *registered_kind == kind)
            .and_then(|(_, spec)| spec.configured_session_key())
    }

    pub(super) fn toast_position(&self) -> ToastPosition {
        self.toast_position
    }

    pub(super) fn toast_insets(&self) -> ToastInsets {
        self.toast_insets
    }

    pub(super) fn effective_app_window_count(&self) -> usize {
        self.registry
            .handles()
            .filter(|handle| {
                handle.role == WindowRole::App && !self.pending_app_closes.contains(&handle.id)
            })
            .count()
    }

    pub(super) fn report_duplicate_window_session_keys(&self) {
        let mut seen = Vec::new();
        for (_, spec) in &self.windows {
            let Some(key) = spec.configured_session_key() else {
                continue;
            };
            if seen.contains(&key) {
                log::warn!(
                    target: "nive_runtime::settings",
                    "settings.duplicate_window_key key={key}"
                );
            } else {
                seen.push(key);
            }
        }
    }
}

impl<K> From<WindowHandle<K>> for WindowContext<K> {
    fn from(handle: WindowHandle<K>) -> Self {
        Self {
            id: handle.id,
            kind: handle.kind,
            role: handle.role,
        }
    }
}
