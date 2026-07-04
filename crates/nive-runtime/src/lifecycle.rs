pub(crate) mod bootstrap;
mod command;
mod window;

pub use bootstrap::{BackgroundFit, BootstrapSpec, BrandContent, SplashBackground};
pub use command::{
    CloseDecision, CommandRejected, CommandRejectionReason, ExitDecision, PlatformError,
    WindowCommand,
};
#[cfg(feature = "devtools")]
pub(crate) use window::open_window;
pub use window::{
    WindowCardinality, WindowChrome, WindowHandle, WindowMode, WindowRegistry, WindowRole,
    WindowSpec,
};
