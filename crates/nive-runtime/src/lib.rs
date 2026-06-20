mod application;
#[cfg(feature = "devtools")]
pub mod devtools;
mod feedback;
mod input;
mod lifecycle;
pub mod platform;
mod screen;
mod state;

#[cfg(feature = "devtools")]
pub use application::run_with_devtools;
pub use application::{
    client_task, run, AppUpdate, Application, ApplicationConfig, Context, CoreEvent, Error, Never,
    Result, RuntimeCommand, ThemeController, ThemeEvent, Update, WindowContext, WindowQuery,
    WindowRegistration,
};
#[cfg(feature = "devtools")]
pub use devtools::{
    run_devtools_panel_effect, DevtoolStateCatalog, DevtoolStateHost, DevtoolsApp, DevtoolsConfig,
    DevtoolsHostState, DevtoolsPanelEffect, DevtoolsPanelMessage, DevtoolsPanelState,
    DevtoolsPanelTab, DevtoolsWindowSpec, ProbePanelState,
};
pub use feedback::{
    ErrorCode, InvalidErrorCode, Toast, ToastDuration, ToastId, ToastItem, ToastMessage,
    ToastPosition, ToastRequest, ToastState, ToastTone, UserFacingError, UserFacingErrorKind,
    UserFacingResult,
};
pub use input::{
    keyboard_navigation_subscription, KeyboardNavigation, ShortcutBinding, ShortcutKey, ShortcutMap,
};
pub use lifecycle::{
    open_window, BackgroundFit, BootstrapSpec, BrandContent, CloseDecision, CommandRejected,
    CommandRejectionReason, ExitDecision, PlatformError, SplashBackground, WindowCardinality,
    WindowChrome, WindowCommand, WindowHandle, WindowMode, WindowRegistry, WindowRole, WindowSpec,
};
pub use nive_ui::focus_trap::{
    direction_from_event, direction_from_keyboard_event, FocusDirection,
};
pub use platform::file_picker::{FileFilter, PickFileParams};
pub use screen::{is_escape_key_press, DialogDismiss, DialogRequest, ScreenUpdate, ScreenView};
pub use state::{
    relative_time_label, unix_now, AsyncState, OperationState, RequestCounter, RequestId,
};

pub use iced::{time, window, Size, Subscription, Task};
pub use nive_ui::theme::ThemePreference;

#[cfg(feature = "devtools")]
pub use nive_runtime_derive::{
    runtime_client, DevtoolOperationContext, DevtoolStateCatalog, DevtoolStateHost, Devtools,
    UiErrorProbeCatalog,
};

pub use platform::app_icon;
#[cfg(feature = "file-picker")]
pub use platform::file_picker::{pick_file, pick_files, pick_folder};

pub mod prelude {
    pub use crate::{
        keyboard_navigation_subscription, relative_time_label, run, time, unix_now, window,
        AppUpdate, Application, ApplicationConfig, BackgroundFit, BootstrapSpec, BrandContent,
        CloseDecision, CommandRejected, CommandRejectionReason, Context, CoreEvent, Error,
        ErrorCode, ExitDecision, KeyboardNavigation, Never, PlatformError, RequestCounter,
        RequestId, Result, RuntimeCommand, ScreenView, ShortcutBinding, ShortcutKey, ShortcutMap,
        Size, SplashBackground, Subscription, Task, ThemeController, ThemeEvent, ThemePreference,
        Toast, ToastPosition, Update, UserFacingError, WindowCardinality, WindowCommand,
        WindowContext, WindowQuery, WindowRole, WindowSpec,
    };
}
