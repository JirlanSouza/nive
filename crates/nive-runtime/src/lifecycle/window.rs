use iced::{window, Size, Task};

/// The display mode of a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowMode {
    /// A normal windowed window (default).
    #[default]
    Windowed,
    /// A maximized window.
    Maximized,
    /// A fullscreen window.
    Fullscreen,
}

/// The window chrome (titlebar/decoration) style.
///
/// `UnifiedTitlebar` is only effective on macOS; it falls back to `Native`
/// elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowChrome {
    /// The platform's native decorations.
    Native,
    /// A macOS unified titlebar (default; falls back to `Native` off-macOS).
    #[default]
    UnifiedTitlebar,
    /// App-owned chrome (no native decorations).
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

/// Declarative specification of a window's appearance and behavior.
///
/// Built via [`WindowSpec::app`] or [`WindowSpec::auxiliary`] defaults and
/// refined with builder methods. The runtime converts a `WindowSpec` into the
/// Iced window settings when opening a window.
#[derive(Debug, Clone, Copy)]
pub struct WindowSpec {
    /// The window's role (app, auxiliary, etc.).
    pub role: WindowRole,
    /// How many instances of this window kind may be open at once.
    pub cardinality: WindowCardinality,
    /// The initial window size.
    pub size: Size,
    /// The initial window position.
    pub position: window::Position,
    /// The optional minimum window size.
    pub min_size: Option<Size>,
    /// The optional maximum window size.
    pub max_size: Option<Size>,
    /// Whether the window is user-resizable.
    pub resizable: bool,
    /// Whether native window decorations are shown.
    pub decorations: bool,
    /// Whether the window background is transparent.
    pub transparent: bool,
    /// The display mode.
    pub mode: WindowMode,
    /// The chrome style.
    pub chrome: WindowChrome,
    /// The window z-order level.
    pub level: window::Level,
    /// Stable key used for opt-in runtime session persistence.
    pub session_key: Option<&'static str>,
}

impl WindowSpec {
    pub fn app() -> Self {
        Self {
            role: WindowRole::App,
            cardinality: WindowCardinality::Single,
            size: Size::new(1024.0, 720.0),
            position: window::Position::Centered,
            min_size: Some(Size::new(640.0, 480.0)),
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            mode: WindowMode::Windowed,
            chrome: WindowChrome::Native,
            level: window::Level::Normal,
            session_key: None,
        }
    }

    pub fn auxiliary() -> Self {
        Self {
            role: WindowRole::Auxiliary,
            size: Size::new(900.0, 640.0),
            ..Self::app()
        }
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Size::new(width, height);
        self
    }

    pub fn min_size(mut self, width: f32, height: f32) -> Self {
        self.min_size = Some(Size::new(width, height));
        self
    }

    pub fn max_size(mut self, width: f32, height: f32) -> Self {
        self.max_size = Some(Size::new(width, height));
        self
    }

    pub fn multiple(mut self) -> Self {
        self.cardinality = WindowCardinality::Multiple;
        self
    }

    pub fn session_key(mut self, key: &'static str) -> Self {
        self.session_key = Some(key);
        self
    }

    pub fn role(self) -> WindowRole {
        self.role
    }

    pub fn cardinality(self) -> WindowCardinality {
        self.cardinality
    }

    pub fn configured_session_key(self) -> Option<&'static str> {
        self.session_key
    }

    pub fn settings(self, icon: Option<window::Icon>) -> window::Settings {
        let chrome = self.chrome.effective();
        let mut settings = window::Settings {
            size: self.size,
            position: self.position,
            min_size: self.min_size,
            max_size: self.max_size,
            resizable: self.resizable,
            decorations: self.decorations && !chrome.uses_app_owned_chrome(),
            transparent: self.transparent,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowCardinality {
    #[default]
    Single,
    Multiple,
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
    windows: Vec<WindowEntry<K>>,
    activity_sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowLifecycle {
    Opening,
    Open,
}

#[derive(Debug, Clone, Copy)]
struct WindowEntry<K> {
    handle: WindowHandle<K>,
    lifecycle: WindowLifecycle,
    activity_sequence: u64,
}

impl<K> Default for WindowRegistry<K> {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            activity_sequence: 0,
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

    pub fn set_opening(&mut self, handle: WindowHandle<K>) {
        self.upsert(handle, WindowLifecycle::Opening);
    }

    pub fn set_opened(&mut self, handle: WindowHandle<K>) {
        self.upsert(handle, WindowLifecycle::Open);
    }

    pub fn mark_opened(&mut self, window_id: window::Id) -> Option<WindowHandle<K>> {
        let activity_sequence = self.next_activity_sequence();
        let entry = self
            .windows
            .iter_mut()
            .find(|entry| entry.handle.id == window_id)?;

        entry.lifecycle = WindowLifecycle::Open;
        entry.activity_sequence = activity_sequence;

        Some(entry.handle)
    }

    pub fn set_focused(&mut self, window_id: window::Id) -> Option<WindowHandle<K>> {
        let activity_sequence = self.next_activity_sequence();
        let entry = self
            .windows
            .iter_mut()
            .find(|entry| entry.handle.id == window_id)?;

        entry.activity_sequence = activity_sequence;

        Some(entry.handle)
    }

    pub fn set_closed(&mut self, window_id: window::Id) -> Option<K> {
        let index = self
            .windows
            .iter()
            .position(|entry| entry.handle.id == window_id)?;

        Some(self.windows.remove(index).handle.kind)
    }

    #[cfg(test)]
    fn lifecycle(&self, window_id: window::Id) -> Option<WindowLifecycle> {
        self.windows
            .iter()
            .find(|entry| entry.handle.id == window_id)
            .map(|entry| entry.lifecycle)
    }

    pub(crate) fn get(&self, window_id: window::Id) -> Option<WindowHandle<K>> {
        self.windows
            .iter()
            .find(|entry| entry.handle.id == window_id)
            .map(|entry| entry.handle)
    }

    pub fn kind(&self, window_id: window::Id) -> Option<K> {
        self.get(window_id).map(|handle| handle.kind)
    }

    pub fn kind_or(&self, window_id: window::Id, fallback: K) -> K {
        self.kind(window_id).unwrap_or(fallback)
    }

    pub fn id(&self, kind: K) -> Option<window::Id> {
        self.windows
            .iter()
            .filter(|entry| entry.handle.kind == kind)
            .max_by_key(|entry| entry.activity_sequence)
            .map(|entry| entry.handle.id)
    }

    pub fn take(&mut self, kind: K) -> Option<window::Id> {
        let index = self
            .windows
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.handle.kind == kind)
            .max_by_key(|(_, entry)| entry.activity_sequence)
            .map(|(index, _)| index)?;

        Some(self.windows.remove(index).handle.id)
    }

    pub(crate) fn contains(&self, kind: K) -> bool {
        self.windows.iter().any(|entry| entry.handle.kind == kind)
    }

    pub(crate) fn first(&self, kind: K) -> Option<WindowHandle<K>> {
        self.windows
            .iter()
            .filter(|entry| entry.handle.kind == kind)
            .max_by_key(|entry| entry.activity_sequence)
            .map(|entry| entry.handle)
    }

    pub(crate) fn all(&self, kind: K) -> impl Iterator<Item = WindowHandle<K>> + '_ {
        self.windows
            .iter()
            .filter(move |entry| entry.handle.kind == kind)
            .map(|entry| entry.handle)
    }

    pub(crate) fn handles(&self) -> impl Iterator<Item = WindowHandle<K>> + '_ {
        self.windows.iter().map(|entry| entry.handle)
    }

    /// Returns the most recently active app window (highest
    /// `activity_sequence`), or `None` when no app window is open.
    pub(crate) fn most_recent_app_window(&self) -> Option<WindowHandle<K>> {
        self.windows
            .iter()
            .filter(|entry| entry.handle.role == WindowRole::App)
            .max_by_key(|entry| entry.activity_sequence)
            .map(|entry| entry.handle)
    }

    pub(crate) fn app_window_count(&self) -> usize {
        self.windows
            .iter()
            .filter(|entry| entry.handle.role == WindowRole::App)
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.app_window_count() == 0
    }

    pub fn has_app_windows(&self) -> bool {
        !self.is_empty()
    }

    fn upsert(&mut self, handle: WindowHandle<K>, lifecycle: WindowLifecycle) {
        let activity_sequence = self.next_activity_sequence();

        if let Some(entry) = self
            .windows
            .iter_mut()
            .find(|entry| entry.handle.id == handle.id)
        {
            entry.handle = handle;
            entry.lifecycle = lifecycle;
            entry.activity_sequence = activity_sequence;
        } else {
            self.windows.push(WindowEntry {
                handle,
                lifecycle,
                activity_sequence,
            });
        }
    }

    fn next_activity_sequence(&mut self) -> u64 {
        self.activity_sequence = self.activity_sequence.wrapping_add(1);
        self.activity_sequence
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
            role: WindowRole::App,
            cardinality: WindowCardinality::Single,
            size,
            position: window::Position::Centered,
            min_size: Some(size),
            max_size: Some(size),
            resizable: false,
            decorations: true,
            transparent: false,
            mode: WindowMode::Windowed,
            chrome: WindowChrome::AppOwned,
            level: window::Level::Normal,
            session_key: None,
        }
    }

    fn workspace_spec() -> WindowSpec {
        WindowSpec {
            role: WindowRole::App,
            cardinality: WindowCardinality::Single,
            size: Size::new(1280.0, 800.0),
            position: window::Position::default(),
            min_size: Some(Size::new(960.0, 640.0)),
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            mode: WindowMode::Maximized,
            chrome: WindowChrome::UnifiedTitlebar,
            level: window::Level::Normal,
            session_key: None,
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

    #[test]
    fn registry_keeps_multiple_instances_of_the_same_kind() {
        let first_id = window::Id::unique();
        let second_id = window::Id::unique();
        let mut registry = WindowRegistry::new();

        registry.set_opened(WindowHandle::new(TestWindow::Workspace, first_id));
        registry.set_opened(WindowHandle::new(TestWindow::Workspace, second_id));

        assert_eq!(
            registry
                .all(TestWindow::Workspace)
                .map(|handle| handle.id)
                .collect::<Vec<_>>(),
            vec![first_id, second_id]
        );
        assert_eq!(registry.id(TestWindow::Workspace), Some(second_id));
        assert_eq!(registry.app_window_count(), 2);
    }

    #[test]
    fn registry_tracks_opening_until_the_window_opens() {
        let window_id = window::Id::unique();
        let handle = WindowHandle::new(TestWindow::Welcome, window_id);
        let mut registry = WindowRegistry::new();

        registry.set_opening(handle);

        assert_eq!(
            registry.lifecycle(window_id),
            Some(WindowLifecycle::Opening)
        );
        assert_eq!(registry.mark_opened(window_id), Some(handle));
        assert_eq!(registry.lifecycle(window_id), Some(WindowLifecycle::Open));
    }

    #[test]
    fn registry_removes_interrupted_opening_without_a_ghost_handle() {
        let window_id = window::Id::unique();
        let mut registry = WindowRegistry::new();

        registry.set_opening(WindowHandle::new(TestWindow::Welcome, window_id));

        assert_eq!(registry.set_closed(window_id), Some(TestWindow::Welcome));
        assert_eq!(registry.get(window_id), None);
        assert!(!registry.contains(TestWindow::Welcome));
    }

    #[test]
    fn registry_uses_the_most_recent_instance_as_kind_representative() {
        let first_id = window::Id::unique();
        let second_id = window::Id::unique();
        let mut registry = WindowRegistry::new();

        registry.set_opened(WindowHandle::new(TestWindow::Workspace, first_id));
        registry.set_opened(WindowHandle::new(TestWindow::Workspace, second_id));
        registry.set_focused(first_id);

        assert_eq!(
            registry.first(TestWindow::Workspace),
            Some(WindowHandle::new(TestWindow::Workspace, first_id))
        );
        assert_eq!(registry.id(TestWindow::Workspace), Some(first_id));
    }
}
