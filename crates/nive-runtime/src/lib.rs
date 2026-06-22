//! Reusable runtime foundation for Rust/Iced desktop applications.
//!
//! `nive-runtime` owns the application/update contracts, window lifecycle,
//! reusable state machines, user-facing feedback, and an optional devtools
//! layer that are reused by Rust/Iced apps without depending on app-domain
//! services. It sits above `nive-ui` and re-exports stable helper APIs from it.
//!
//! # Scope
//!
//! - `Application`, `ApplicationConfig`, `Context`, and `run` — the stable
//!   product contract and the private Iced program runner.
//! - `Action`, `ActionId`, and `ActionMap` — product action catalogs that can
//!   power shortcuts and future command surfaces.
//! - `Update`, `AppUpdate`, and `RuntimeCommand` — ordered task and
//!   runtime-effect composition.
//! - `BootstrapSpec` — repeatable startup task attempts, stale-result
//!   rejection, minimum splash duration, retry, and cancellation.
//! - `WindowSpec`, `WindowCommand`, `WindowRegistry` — generic window
//!   contracts, cardinality, and open/close/exit handshakes.
//! - `AsyncState` and `OperationState` — reusable resource and operation
//!   state machines.
//! - `UserFacingError` and toast state (`ToastState`, `ToastItem`) —
//!   user-facing feedback.
//! - `ScreenView` and `ScreenUpdate` — screen composition contracts.
//! - `platform` — cross-platform app icon installer and optional file picker.
//! - `SettingsConfig` and `RuntimeSession` — opt-in runtime settings/session
//!   persistence.
//! - `keyboard_navigation_subscription` and `ShortcutMap` — lower-level input
//!   helpers.
//!
//! # Public API
//!
//! Application crates should treat the crate root and `nive_runtime::prelude`
//! as the app-facing API. The root exports the stable runtime contract,
//! action catalog types, lifecycle/window types, feedback/state helpers,
//! task/subscription aliases, theme configuration reexports, and feature-gated
//! platform/devtools entry points.
//!
//! Modules that remain private (`application`, `feedback`, `input`,
//! `lifecycle`, `screen`, and `state`) are implementation boundaries. App code
//! should not depend on runner internals. Public modules such as `platform` and
//! feature-gated `devtools` are explicit extension surfaces.
//!
//! # Feature flags
//!
//! - `devtools` (off by default) — enables the optional devtools layer
//!   (`devtools`), the `run_with_devtools` entry point, and the derive macros
//!   from `nive-runtime-derive`. This is the most experimental part of Nive.
//! - `file-picker` (off by default) — enables `pick_file`, `pick_files`,
//!   `pick_folder`, and `save_file` backed by `rfd`.
//!
//! # Status
//!
//! Part of Nive **v0.1.0**, a beta release. Public APIs may change before 1.0.
//! See `docs/` for contract details on the application, lifecycle, and
//! devtools layers.

pub mod actions;
mod application;
#[cfg(feature = "devtools")]
pub mod devtools;
mod feedback;
mod input;
mod lifecycle;
pub mod platform;
mod screen;
pub mod settings;
mod state;
pub mod support;

pub use actions::{command_palette_rows, Action, ActionId, ActionMap, DuplicateActionId};
#[cfg(feature = "devtools")]
pub use application::run_with_devtools;
pub use application::{
    client_task, run, AppUpdate, Application, ApplicationConfig, Context, CoreEvent, Error, Never,
    Result, RuntimeCommand, SimpleApplication, ThemeController, ThemeEvent, Update, WindowContext,
    WindowQuery, WindowRegistration,
};
#[cfg(feature = "devtools")]
pub use devtools::{
    run_devtools_panel_effect, DevtoolStateCatalog, DevtoolStateHost, DevtoolsApp, DevtoolsConfig,
    DevtoolsHostState, DevtoolsPanelEffect, DevtoolsPanelMessage, DevtoolsPanelState,
    DevtoolsPanelTab, DevtoolsWindowSpec, ProbePanelState,
};
#[allow(deprecated)]
pub use feedback::ToastRequest;
pub use feedback::{
    ErrorCode, InvalidErrorCode, Toast, ToastDuration, ToastId, ToastItem, ToastMessage,
    ToastPosition, ToastState, ToastTone, UserFacingError, UserFacingErrorKind, UserFacingResult,
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
#[cfg(feature = "file-picker")]
pub use platform::file_picker::{FileFilter, PickFileParams, SaveFileParams};
pub use screen::{is_escape_key_press, DialogDismiss, DialogRequest, ScreenUpdate, ScreenView};
pub use settings::{
    RuntimeSession, SettingsConfig, SettingsError, SettingsErrorKind, WindowSession,
    WindowSessionPosition, WindowSessionSize,
};
pub use state::{
    relative_time_label, unix_now, AsyncState, OperationCommand, OperationDescriptor,
    OperationEntry, OperationId, OperationProgress, OperationRegistry, OperationState,
    OperationStatus, RequestCounter, RequestId,
};
pub use support::{
    install_diagnostic_panic_hook, DiagnosticSnapshot, RuntimeEvent, RuntimeEventKind,
    RuntimeEventLog,
};

pub use iced::{time, window, Point, Size, Subscription, Task};
pub use nive_ui::theme::{Theme, ThemeBuilder, ThemeCatalog, ThemeMode, ThemePreference};

#[cfg(feature = "devtools")]
pub use nive_runtime_derive::{
    runtime_client, DevtoolOperationContext, DevtoolStateCatalog, DevtoolStateHost, Devtools,
    UiErrorProbeCatalog,
};

pub use platform::app_icon;
#[cfg(feature = "file-picker")]
pub use platform::file_picker::{pick_file, pick_files, pick_folder, save_file};
pub mod prelude {
    /// Minimal surface that compiles the scaffolded counter template without
    /// extra `use` statements. Apps that use toasts, async state, dialogs,
    /// file-picker params, theming, or window-management types pull in
    /// [`crate::prelude::ui`] instead.
    pub use crate::{
        command_palette_rows, install_diagnostic_panic_hook, keyboard_navigation_subscription,
        relative_time_label, run, time, unix_now, window, Action, ActionId, ActionMap, AppUpdate,
        Application, ApplicationConfig, CloseDecision, CommandRejected, CommandRejectionReason,
        Context, CoreEvent, DiagnosticSnapshot, DuplicateActionId, Error, ExitDecision,
        KeyboardNavigation, Never, PlatformError, Point, RequestCounter, RequestId, Result,
        RuntimeCommand, RuntimeEvent, RuntimeEventKind, RuntimeEventLog, RuntimeSession,
        ScreenView, SettingsConfig, SettingsError, SettingsErrorKind, ShortcutBinding, ShortcutKey,
        ShortcutMap, SimpleApplication, Size, Subscription, Task, Theme, ThemeBuilder,
        ThemeCatalog, ThemeController, ThemeEvent, ThemeMode, ThemePreference, Toast,
        ToastPosition, Update, WindowCardinality, WindowCommand, WindowContext, WindowQuery,
        WindowRole, WindowSession, WindowSessionPosition, WindowSessionSize, WindowSpec,
    };

    /// Extended surface for app code that uses toasts, async state,
    /// dialogs, file picker params, theming, shortcuts, window handle types,
    /// or splash-related types. Use `nive::prelude::ui::*`.
    pub mod ui {
        pub use super::*;
        /// Deprecated `ToastRequest` alias. Kept for one release cycle
        /// (v0.1) so external apps gradually migrate to `Toast`.
        #[allow(deprecated)]
        pub use crate::ToastRequest;
        pub use crate::{
            AsyncState, BackgroundFit, BootstrapSpec, BrandContent, DialogDismiss, DialogRequest,
            ErrorCode, InvalidErrorCode, OperationDescriptor, OperationEntry, OperationId,
            OperationProgress, OperationRegistry, OperationState, OperationStatus, ScreenUpdate,
            ShortcutBinding, ShortcutKey, ShortcutMap, SplashBackground, ThemeBuilder,
            ThemeCatalog, ThemeMode, ToastDuration, ToastTone, UserFacingError,
            UserFacingErrorKind, UserFacingResult, WindowChrome, WindowHandle, WindowMode,
            WindowRegistry,
        };

        /// File-picker param structs surfaced in the extended tier only when
        /// the `file-picker` feature is enabled. Without the feature these
        /// `pub use`s are absent and downstream code that constructs them
        /// fails to compile (no orphaned types).
        #[cfg(feature = "file-picker")]
        pub use crate::{FileFilter, PickFileParams, SaveFileParams};
    }
}
