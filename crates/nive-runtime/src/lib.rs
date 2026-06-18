mod application;
mod async_state;
mod bootstrap;
mod client_task;
pub mod devtools;
mod dialog_dismiss;
mod dialog_request;
mod error;
mod keyboard_navigation;
mod operation_state;
pub mod platform;
mod probe;
mod request;
mod screen_update;
mod screen_view;
mod theme_controller;
mod toast;
mod update;
mod window_shell;

#[cfg(feature = "devtools")]
pub use application::run_with_devtools;
pub use application::{
    run, Application, ApplicationConfig, CloseDecision, CommandRejected, CommandRejectionReason,
    Context, CoreEvent, Error, ExitDecision, PlatformError, Result, ShortcutMap, WindowCommand,
    WindowContext, WindowQuery, WindowRegistration,
};
pub use async_state::AsyncState;
pub use bootstrap::{
    BackgroundFit, BackgroundPosition, BootstrapSpec, BrandContent, SplashBackground,
};
pub use client_task::{client_task, injected_client_task, ClientTaskInjection, ProbeEffect};
pub use devtools::{
    run_devtools_panel_effect, DevtoolStateCatalog, DevtoolStateHost, DevtoolsApp, DevtoolsConfig,
    DevtoolsHostState, DevtoolsPanelEffect, DevtoolsPanelMessage, DevtoolsPanelState,
    DevtoolsPanelTab, DevtoolsWindowSpec, ProbePanelState,
};
pub use dialog_dismiss::{is_escape_key_press, DialogDismiss};
pub use dialog_request::DialogRequest;
pub use error::{
    ErrorCode, InvalidErrorCode, UserFacingError, UserFacingErrorKind, UserFacingResult,
};
pub use keyboard_navigation::{keyboard_navigation_subscription, KeyboardNavigation};
pub use nive_ui::focus_trap::{
    direction_from_event, direction_from_keyboard_event, FocusDirection,
};
pub use operation_state::OperationState;
pub use platform::file_picker::{FileFilter, PickFileParams};
pub use probe::{
    composed_probe_ids, parse_probe_config, probe_catalog_items, probe_catalog_keys,
    probe_drafts_from_snapshot, update_probe_drafts, ComposedProbeId, NoProbe, ProbeCatalogEntry,
    ProbeCatalogItem, ProbeDraft, ProbeErrorScope, ProbeInjectionConfig, ProbeInjectionSnapshot,
    ProbeInjectionStore, ProbeMeta, ProbeMetaCatalog, ProbePanelEffect, ProbePanelMessage,
    ProbeRuntimeConfig, ProbeScenarioConfig, ProbeScenarioSnapshot,
};
pub use request::{RequestCounter, RequestId};
pub use screen_update::ScreenUpdate;
pub use screen_view::ScreenView;
pub use theme_controller::{ThemeController, ThemeEvent};
pub use toast::ToastRequest as Toast;
pub use toast::{
    ToastDuration, ToastId, ToastItem, ToastMessage, ToastPosition, ToastRequest, ToastState,
    ToastTone,
};
pub use update::{AppUpdate, Never, RuntimeCommand, Update};
pub use window_shell::{
    open_window, WindowCardinality, WindowChrome, WindowHandle, WindowMode, WindowRegistry,
    WindowRole, WindowSpec,
};

pub use iced::{window, Size, Subscription, Task};
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
        keyboard_navigation_subscription, run, window, AppUpdate, Application, ApplicationConfig,
        BackgroundFit, BackgroundPosition, BootstrapSpec, BrandContent, CloseDecision,
        CommandRejected, CommandRejectionReason, Context, CoreEvent, Error, ErrorCode,
        ExitDecision, KeyboardNavigation, Never, PlatformError, RequestCounter, RequestId, Result,
        RuntimeCommand, ScreenView, ShortcutMap, Size, SplashBackground, Subscription, Task,
        ThemeController, ThemeEvent, ThemePreference, Toast, ToastPosition, Update,
        UserFacingError, WindowCardinality, WindowCommand, WindowContext, WindowQuery, WindowRole,
        WindowSpec,
    };
}
