use nive_ui::theme::Theme;

use crate::{CommandRejected, PlatformError, WindowContext};

/// An app-facing runtime lifecycle event, delivered through
/// [`Application::on_runtime_event`](super::Application::on_runtime_event).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeEvent<K> {
    WindowOpened(WindowContext<K>),
    WindowClosed(WindowContext<K>),
    WindowFocused(WindowContext<K>),
    LastAppWindowClosed,
    ThemeChanged(Theme),
    CommandRejected(CommandRejected<K>),
    PlatformError(PlatformError),
}
