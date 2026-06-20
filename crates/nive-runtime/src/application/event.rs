use nive_ui::theme::Theme;

use crate::{CommandRejected, PlatformError, WindowContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreEvent<K> {
    WindowOpened(WindowContext<K>),
    WindowClosed(WindowContext<K>),
    WindowFocused(WindowContext<K>),
    LastAppWindowClosed,
    ThemeChanged(Theme),
    CommandRejected(CommandRejected<K>),
    PlatformError(PlatformError),
}
