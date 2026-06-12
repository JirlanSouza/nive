use iced::{window, Size, Task};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowMode {
    #[default]
    Windowed,
    Maximized,
    Fullscreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowChrome {
    Native,
    #[default]
    UnifiedTitlebar,
    AppOwned,
}

impl WindowChrome {
    #[cfg(target_os = "macos")]
    fn effective(self) -> Self {
        self
    }

    #[cfg(not(target_os = "macos"))]
    fn effective(self) -> Self {
        match self {
            Self::UnifiedTitlebar => Self::Native,
            chrome => chrome,
        }
    }

    fn uses_app_owned_chrome(self) -> bool {
        matches!(self.effective(), Self::AppOwned)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WindowSpec {
    pub size: Size,
    pub position: window::Position,
    pub min_size: Option<Size>,
    pub max_size: Option<Size>,
    pub resizable: bool,
    pub mode: WindowMode,
    pub chrome: WindowChrome,
    pub level: window::Level,
}

impl WindowSpec {
    pub fn settings(self, icon: Option<window::Icon>) -> window::Settings {
        let chrome = self.chrome.effective();
        let mut settings = window::Settings {
            size: self.size,
            position: self.position,
            min_size: self.min_size,
            max_size: self.max_size,
            resizable: self.resizable,
            decorations: !chrome.uses_app_owned_chrome(),
            maximized: matches!(self.mode, WindowMode::Maximized),
            fullscreen: matches!(self.mode, WindowMode::Fullscreen),
            level: self.level,
            icon,
            ..window::Settings::default()
        };

        apply_platform_chrome(chrome, &mut settings);

        settings
    }
}

impl From<WindowSpec> for window::Settings {
    fn from(spec: WindowSpec) -> Self {
        spec.settings(None)
    }
}

pub fn open_window<Message>(
    spec: WindowSpec,
    icon: Option<window::Icon>,
    on_open: impl Fn(window::Id) -> Message + Send + 'static,
) -> (window::Id, Task<Message>)
where
    Message: Send + 'static,
{
    let (window_id, open_task) = window::open(spec.settings(icon));
    (window_id, open_task.map(on_open))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowRole {
    App,
    Auxiliary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowHandle<K> {
    pub kind: K,
    pub id: window::Id,
    pub role: WindowRole,
}

impl<K> WindowHandle<K> {
    pub fn new(kind: K, id: window::Id) -> Self {
        Self {
            kind,
            id,
            role: WindowRole::App,
        }
    }

    pub fn auxiliary(kind: K, id: window::Id) -> Self {
        Self {
            kind,
            id,
            role: WindowRole::Auxiliary,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WindowRegistry<K> {
    windows: Vec<WindowHandle<K>>,
}

impl<K> Default for WindowRegistry<K> {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
        }
    }
}

impl<K> WindowRegistry<K>
where
    K: Copy + Eq,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_opened(&mut self, handle: WindowHandle<K>) {
        if let Some(existing) = self
            .windows
            .iter_mut()
            .find(|existing| existing.kind == handle.kind)
        {
            *existing = handle;
        } else {
            self.windows.push(handle);
        }
    }

    pub fn set_closed(&mut self, window_id: window::Id) -> Option<K> {
        let index = self
            .windows
            .iter()
            .position(|handle| handle.id == window_id)?;

        Some(self.windows.remove(index).kind)
    }

    pub fn kind(&self, window_id: window::Id) -> Option<K> {
        self.windows
            .iter()
            .find(|handle| handle.id == window_id)
            .map(|handle| handle.kind)
    }

    pub fn kind_or(&self, window_id: window::Id, fallback: K) -> K {
        self.kind(window_id).unwrap_or(fallback)
    }

    pub fn id(&self, kind: K) -> Option<window::Id> {
        self.windows
            .iter()
            .find(|handle| handle.kind == kind)
            .map(|handle| handle.id)
    }

    pub fn take(&mut self, kind: K) -> Option<window::Id> {
        let index = self.windows.iter().position(|handle| handle.kind == kind)?;

        Some(self.windows.remove(index).id)
    }

    pub fn is_empty(&self) -> bool {
        !self
            .windows
            .iter()
            .any(|handle| handle.role == WindowRole::App)
    }

    pub fn has_app_windows(&self) -> bool {
        !self.is_empty()
    }
}

#[cfg(target_os = "macos")]
fn apply_platform_chrome(chrome: WindowChrome, settings: &mut window::Settings) {
    use iced::window::settings::PlatformSpecific;

    if matches!(chrome, WindowChrome::UnifiedTitlebar) {
        settings.platform_specific = PlatformSpecific {
            title_hidden: true,
            titlebar_transparent: true,
            fullsize_content_view: true,
        };
    }
}

#[cfg(not(target_os = "macos"))]
fn apply_platform_chrome(_chrome: WindowChrome, _settings: &mut window::Settings) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestWindow {
        Bootstrap,
        Devtools,
        Welcome,
        Workspace,
    }

    fn fixed_spec(size: Size) -> WindowSpec {
        WindowSpec {
            size,
            position: window::Position::Centered,
            min_size: Some(size),
            max_size: Some(size),
            resizable: false,
            mode: WindowMode::Windowed,
            chrome: WindowChrome::AppOwned,
            level: window::Level::Normal,
        }
    }

    fn workspace_spec() -> WindowSpec {
        WindowSpec {
            size: Size::new(1280.0, 800.0),
            position: window::Position::default(),
            min_size: Some(Size::new(960.0, 640.0)),
            max_size: None,
            resizable: true,
            mode: WindowMode::Maximized,
            chrome: WindowChrome::UnifiedTitlebar,
            level: window::Level::Normal,
        }
    }

    #[test]
    fn fixed_app_owned_spec_converts_to_undecorated_settings() {
        let spec = fixed_spec(Size::new(560.0, 360.0));
        let settings = spec.settings(None);

        assert_eq!(settings.size, Size::new(560.0, 360.0));
        assert!(matches!(settings.position, window::Position::Centered));
        assert_eq!(settings.min_size, Some(Size::new(560.0, 360.0)));
        assert_eq!(settings.max_size, Some(Size::new(560.0, 360.0)));
        assert!(!settings.resizable);
        assert!(!settings.maximized);
        assert!(!settings.fullscreen);
        assert_eq!(settings.level, window::Level::Normal);
        assert!(!settings.decorations);
        assert!(settings.icon.is_none());
    }

    #[test]
    fn workspace_spec_converts_to_resizable_maximized_settings() {
        let spec = workspace_spec();
        let settings = window::Settings::from(spec);

        assert_eq!(settings.size, Size::new(1280.0, 800.0));
        assert_eq!(settings.min_size, Some(Size::new(960.0, 640.0)));
        assert!(settings.resizable);
        assert!(settings.maximized);
        assert!(!settings.fullscreen);
        assert_eq!(settings.level, window::Level::Normal);
        assert!(settings.decorations);
    }

    #[test]
    fn fullscreen_mode_converts_to_fullscreen_without_maximized() {
        let spec = WindowSpec {
            mode: WindowMode::Fullscreen,
            ..workspace_spec()
        };
        let settings = window::Settings::from(spec);

        assert!(!settings.maximized);
        assert!(settings.fullscreen);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn unified_titlebar_applies_macos_platform_settings() {
        let settings = workspace_spec().settings(None);

        assert!(settings.platform_specific.title_hidden);
        assert!(settings.platform_specific.titlebar_transparent);
        assert!(settings.platform_specific.fullsize_content_view);
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn unified_titlebar_uses_native_chrome_off_macos() {
        let settings = workspace_spec().settings(None);

        assert!(settings.decorations);
    }

    #[test]
    fn registry_tracks_opened_window_by_kind() {
        let bootstrap_id = window::Id::unique();
        let devtools_id = window::Id::unique();
        let welcome_id = window::Id::unique();
        let workspace_id = window::Id::unique();
        let mut registry = WindowRegistry::new();

        registry.set_opened(WindowHandle::new(TestWindow::Bootstrap, bootstrap_id));
        registry.set_opened(WindowHandle::auxiliary(TestWindow::Devtools, devtools_id));
        registry.set_opened(WindowHandle::new(TestWindow::Welcome, welcome_id));
        registry.set_opened(WindowHandle::new(TestWindow::Workspace, workspace_id));

        assert_eq!(registry.kind(bootstrap_id), Some(TestWindow::Bootstrap));
        assert_eq!(registry.kind(devtools_id), Some(TestWindow::Devtools));
        assert_eq!(registry.kind(welcome_id), Some(TestWindow::Welcome));
        assert_eq!(registry.kind(workspace_id), Some(TestWindow::Workspace));
        assert_eq!(registry.id(TestWindow::Bootstrap), Some(bootstrap_id));
        assert_eq!(registry.id(TestWindow::Devtools), Some(devtools_id));
        assert_eq!(registry.id(TestWindow::Workspace), Some(workspace_id));
        assert!(!registry.is_empty());
        assert!(registry.has_app_windows());
    }

    #[test]
    fn registry_closing_known_window_clears_only_that_kind() {
        let bootstrap_id = window::Id::unique();
        let welcome_id = window::Id::unique();
        let workspace_id = window::Id::unique();
        let mut registry = WindowRegistry::new();

        registry.set_opened(WindowHandle::new(TestWindow::Bootstrap, bootstrap_id));
        registry.set_opened(WindowHandle::new(TestWindow::Welcome, welcome_id));
        registry.set_opened(WindowHandle::new(TestWindow::Workspace, workspace_id));

        assert_eq!(
            registry.set_closed(bootstrap_id),
            Some(TestWindow::Bootstrap)
        );
        assert_eq!(registry.id(TestWindow::Welcome), Some(welcome_id));
        assert_eq!(registry.id(TestWindow::Workspace), Some(workspace_id));
        assert!(!registry.is_empty());

        assert_eq!(registry.set_closed(welcome_id), Some(TestWindow::Welcome));
        assert_eq!(registry.id(TestWindow::Workspace), Some(workspace_id));
        assert!(!registry.is_empty());

        assert_eq!(
            registry.set_closed(workspace_id),
            Some(TestWindow::Workspace)
        );
        assert!(registry.is_empty());
    }

    #[test]
    fn registry_auxiliary_window_does_not_keep_app_alive() {
        let devtools_id = window::Id::unique();
        let mut registry = WindowRegistry::new();

        registry.set_opened(WindowHandle::auxiliary(TestWindow::Devtools, devtools_id));

        assert_eq!(registry.id(TestWindow::Devtools), Some(devtools_id));
        assert!(registry.is_empty());
        assert!(!registry.has_app_windows());
        assert_eq!(registry.set_closed(devtools_id), Some(TestWindow::Devtools));
        assert_eq!(registry.id(TestWindow::Devtools), None);
    }

    #[test]
    fn registry_unknown_window_close_is_ignored() {
        let mut registry: WindowRegistry<TestWindow> = WindowRegistry::new();

        assert_eq!(registry.set_closed(window::Id::unique()), None);
        assert!(registry.is_empty());
    }
}
