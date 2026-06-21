use std::borrow::Cow;

use iced::{window, Font};
use nive_ui::theme::{ThemeCatalog, ThemePreference};

use crate::{BootstrapSpec, SettingsConfig, ToastPosition, WindowSpec};

pub struct ApplicationConfig<K, B> {
    pub(super) app_id: String,
    pub(super) app_name: String,
    pub(super) windows: Vec<WindowRegistration<K>>,
    pub(super) initial_windows: Vec<K>,
    pub(super) theme_preference: ThemePreference,
    pub(super) theme_catalog: ThemeCatalog,
    pub(super) toast_position: ToastPosition,
    pub(super) bootstrap: Option<BootstrapSpec<B>>,
    pub(super) immediate_bootstrap: Option<B>,
    pub(super) fonts: Vec<Cow<'static, [u8]>>,
    pub(super) default_font: Font,
    pub(super) window_icon: Option<window::Icon>,
    pub(super) settings: Option<SettingsConfig>,
}

#[derive(Debug, Clone, Copy)]
pub struct WindowRegistration<K> {
    pub kind: K,
    pub spec: WindowSpec,
}

impl<K> ApplicationConfig<K, ()> {
    pub fn new(app_id: impl Into<String>) -> Self {
        let app_id = app_id.into();
        Self {
            app_name: app_id.clone(),
            app_id,
            windows: Vec::new(),
            initial_windows: Vec::new(),
            theme_preference: ThemePreference::System,
            theme_catalog: ThemeCatalog::default(),
            toast_position: ToastPosition::BottomRight,
            bootstrap: None,
            immediate_bootstrap: Some(()),
            fonts: Vec::new(),
            default_font: Font::DEFAULT,
            window_icon: None,
            settings: None,
        }
    }
}

impl<K, B> ApplicationConfig<K, B> {
    pub fn app_id(&self) -> &str {
        self.app_id.as_str()
    }

    pub fn app_name(&self) -> &str {
        self.app_name.as_str()
    }

    pub fn name(mut self, app_name: impl Into<String>) -> Self {
        self.app_name = app_name.into();
        self
    }

    pub fn window(mut self, kind: K, spec: WindowSpec) -> Self {
        self.windows.push(WindowRegistration { kind, spec });
        self
    }

    pub fn initial_window(mut self, kind: K) -> Self {
        self.initial_windows.push(kind);
        self
    }

    pub fn theme_preference(mut self, preference: ThemePreference) -> Self {
        self.theme_preference = preference;
        self
    }

    pub fn theme_catalog(mut self, catalog: ThemeCatalog) -> Self {
        self.theme_catalog = catalog;
        self
    }

    pub fn toast_position(mut self, position: ToastPosition) -> Self {
        self.toast_position = position;
        self
    }

    pub fn settings(mut self, settings: SettingsConfig) -> Self {
        self.settings = Some(settings);
        self
    }

    pub fn bootstrap<T>(self, bootstrap: BootstrapSpec<T>) -> ApplicationConfig<K, T> {
        ApplicationConfig {
            app_id: self.app_id,
            app_name: self.app_name,
            windows: self.windows,
            initial_windows: self.initial_windows,
            theme_preference: self.theme_preference,
            theme_catalog: self.theme_catalog,
            toast_position: self.toast_position,
            bootstrap: Some(bootstrap),
            immediate_bootstrap: None,
            fonts: self.fonts,
            default_font: self.default_font,
            window_icon: self.window_icon,
            settings: self.settings,
        }
    }

    pub fn font(mut self, font: impl Into<Cow<'static, [u8]>>) -> Self {
        self.fonts.push(font.into());
        self
    }

    pub fn default_font(mut self, font: Font) -> Self {
        self.default_font = font;
        self
    }

    pub fn window_icon(mut self, icon: window::Icon) -> Self {
        self.window_icon = Some(icon);
        self
    }

    pub fn windows(&self) -> &[WindowRegistration<K>] {
        self.windows.as_slice()
    }

    pub fn initial_windows(&self) -> &[K] {
        self.initial_windows.as_slice()
    }

    pub fn initial_theme_preference(&self) -> ThemePreference {
        self.theme_preference
    }

    pub fn configured_theme_catalog(&self) -> ThemeCatalog {
        self.theme_catalog
    }

    pub fn initial_toast_position(&self) -> ToastPosition {
        self.toast_position
    }

    pub fn bootstrap_spec(&self) -> Option<&BootstrapSpec<B>> {
        self.bootstrap.as_ref()
    }

    pub fn fonts(&self) -> &[Cow<'static, [u8]>] {
        self.fonts.as_slice()
    }

    pub fn configured_default_font(&self) -> Font {
        self.default_font
    }

    pub fn configured_window_icon(&self) -> Option<&window::Icon> {
        self.window_icon.as_ref()
    }

    pub fn configured_settings(&self) -> Option<&SettingsConfig> {
        self.settings.as_ref()
    }
}
