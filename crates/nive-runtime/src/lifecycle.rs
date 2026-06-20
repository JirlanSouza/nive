pub(crate) mod bootstrap;
mod command;
mod window;

pub use bootstrap::{BackgroundFit, BootstrapSpec, BrandContent, SplashBackground};
pub use command::{
    CloseDecision, CommandRejected, CommandRejectionReason, ExitDecision, PlatformError,
    WindowCommand,
};
pub use window::{
    open_window, WindowCardinality, WindowChrome, WindowHandle, WindowMode, WindowRegistry,
    WindowRole, WindowSpec,
};
