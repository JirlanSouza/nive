//! Reusable runtime foundation for Rust/Iced desktop applications.
//!
//! `nive-runtime` owns the application/update contracts, window lifecycle,
//! reusable state machines, user-facing feedback, and an optional devtools
//! layer that are reused by Rust/Iced apps without depending on app-domain
//! services. It sits above `nive-ui` and re-exports stable helper APIs from
//! it. It also depends on `nive-core` directly, implementing its neutral
//! presentation contracts for `UserFacingError`, `Resource`, `Operation`, and
//! `ToastItem`.
//!
//! # Scope
//!
//! - `Application`, `ApplicationConfig`, `Context`, and `run` — the stable
//!   product contract and the private Iced program runner.
//! - `Action`, `ActionId`, and `ActionMap` — product action catalogs that can
//!   power shortcuts and future command surfaces.
//! - `Effect` — ordered task and runtime-effect composition for application
//!   hooks.
//! - `BootstrapSpec` — repeatable startup task attempts, stale-result
//!   rejection, minimum splash duration, retry, and cancellation.
//! - `WindowSpec`, `WindowCommand`, `WindowRegistry` — generic window
//!   contracts, cardinality, and open/close/exit handshakes.
//! - `Resource` and `Operation` — reusable async resource and operation
//!   state machines.
//! - `UserFacingError` and toast state (`ToastState`, `ToastItem`) —
//!   user-facing feedback.
//! - `ScreenView` and `ScreenEffect` — screen composition contracts.
//! - `platform` — cross-platform app icon installer and optional file picker.
//! - `SettingsConfig` and `RuntimeSession` — opt-in runtime settings/session
//!   persistence.
//! - `keyboard_navigation_subscription` and `ShortcutMap` — lower-level input
//!   helpers.
//!
//! # Public API
//!
//! Application crates should treat `nive_runtime::prelude` as the default
//! app-facing API. The crate root preserves the broad beta convenience export
//! surface, while public area modules (`application`, `actions`, `feedback`,
//! `input`, `lifecycle`, `screen`, `settings`, `state`, and `support`) provide
//! predictable direct imports for larger apps and layer-specific consumers.
//!
//! Runner internals remain hidden in private submodules. App code should not
//! depend on runner plumbing such as window-opening helpers; use
//! [`WindowCommand`] through [`Effect`] instead.
//!
//! # Feature flags
//!
//! - `devtools` (off by default) — enables the optional devtools layer
//!   (`devtools`), the `run_with_devtools` entry point, and `#[derive(Inspect)]`
//!   traversal from `nive-runtime-derive`. This is the most experimental part
//!   of Nive.
//! - `file-picker` (off by default) — enables `pick_file`, `pick_files`,
//!   `pick_folder`, and `save_file` backed by `rfd`.
//!
//! # Status
//!
//! Part of Nive **v0.1.0**, a beta release. Public APIs may change before 1.0.
//! See `docs/` for contract details on the application, lifecycle, and
//! devtools layers.

pub mod actions;
pub mod application;
#[cfg(feature = "devtools")]
pub mod devtools;
pub mod feedback;
pub mod input;
#[cfg(feature = "devtools")]
pub mod inspect;
pub mod lifecycle;
pub mod platform;
pub mod screen;
pub mod settings;
pub mod state;
pub mod support;

pub use actions::{command_palette_rows, Action, ActionId, ActionMap, DuplicateActionId};
#[cfg(feature = "devtools")]
pub use application::run_with_devtools;
pub use application::{
    perform, run, Application, ApplicationConfig, Context, Effect, Error, MessageContext,
    MessageSource, Never, Result, RuntimeEvent, SimpleApplication, ThemeController, ThemeEvent,
    WindowContext, WindowQuery,
};
#[cfg(feature = "devtools")]
pub use devtools::{
    apply_simulate, collect_snapshot, DevtoolStateSnapshot, DevtoolsApp, DevtoolsConfig,
    DevtoolsHostState, DevtoolsPanelEffect, DevtoolsPanelMessage, DevtoolsPanelState,
    DevtoolsPanelTab, DevtoolsWindowSpec, RegistryEntry, RegistryStatus, SimulateAction,
    SimulateResult, SimulatorCapabilities, SimulatorEntry, SimulatorKind,
};
pub use feedback::{
    ErrorCode, InvalidErrorCode, Toast, ToastDuration, ToastId, ToastItem, ToastMessage,
    ToastPosition, ToastState, ToastTone, UserFacingError, UserFacingErrorKind, UserFacingResult,
};
pub use input::{
    keyboard_navigation_subscription, KeyboardNavigation, ShortcutBinding, ShortcutKey, ShortcutMap,
};
pub use lifecycle::{
    BackgroundFit, BootstrapSpec, BrandContent, CloseDecision, CommandRejected,
    CommandRejectionReason, ExitDecision, PlatformError, SplashBackground, WindowCardinality,
    WindowChrome, WindowCommand, WindowHandle, WindowMode, WindowRegistry, WindowRole, WindowSpec,
};
pub use nive_ui::focus_trap::{
    direction_from_event, direction_from_keyboard_event, FocusDirection,
};
#[cfg(feature = "file-picker")]
pub use platform::file_picker::{FileFilter, PickFileParams, SaveFileParams};
pub use screen::{is_escape_key_press, DialogDismiss, DialogRequest, ScreenEffect, ScreenView};
pub use settings::{
    RuntimeSession, SettingsConfig, SettingsError, SettingsErrorKind, WindowSession,
    WindowSessionPosition, WindowSessionSize,
};
pub use state::{
    relative_time_label, unix_now, Operation, OperationDescriptor, OperationEntry, OperationId,
    OperationProgress, OperationRegistry, OperationStatus, RequestId, Resource, Settled,
};
pub use support::{
    install_diagnostic_panic_hook, DiagnosticEvent, DiagnosticEventKind, DiagnosticEventLog,
    DiagnosticSnapshot,
};

pub use iced::{time, window, Point, Size, Subscription, Task};
pub use nive_ui::theme::{Theme, ThemeBuilder, ThemeCatalog, ThemeMode, ThemePreference};

#[cfg(feature = "devtools")]
pub use inspect::{
    Inspect, InspectPath, InspectSink, OperationSimulator, ResourceSimulator, SimulableSnapshot,
    SimulableState,
};
pub use nive_runtime_derive::Inspect;

#[cfg(feature = "devtools")]
#[doc(hidden)]
pub mod __inspect {
    pub use crate::inspect::{
        Inspect, InspectPath, InspectSink, OperationSimulator, ResourceSimulator,
    };
}

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
        relative_time_label, run, time, unix_now, window, Action, ActionId, ActionMap, Application,
        ApplicationConfig, CloseDecision, CommandRejected, CommandRejectionReason, Context,
        DiagnosticEvent, DiagnosticEventKind, DiagnosticEventLog, DiagnosticSnapshot,
        DuplicateActionId, Effect, Error, ExitDecision, KeyboardNavigation, MessageContext,
        MessageSource, Never, PlatformError, Point, RequestId, Result, RuntimeEvent,
        RuntimeSession, ScreenView, SettingsConfig, SettingsError, SettingsErrorKind,
        ShortcutBinding, ShortcutKey, ShortcutMap, SimpleApplication, Size, Subscription, Task,
        Theme, ThemeBuilder, ThemeCatalog, ThemeController, ThemeEvent, ThemeMode, ThemePreference,
        Toast, ToastPosition, WindowCardinality, WindowCommand, WindowContext, WindowQuery,
        WindowRole, WindowSession, WindowSessionPosition, WindowSessionSize, WindowSpec,
    };

    /// Extended surface for app code that uses toasts, async state,
    /// dialogs, file picker params, theming, shortcuts, window handle types,
    /// or splash-related types. Use `nive::prelude::ui::*`.
    pub mod ui {
        pub use super::*;
        pub use crate::{
            BackgroundFit, BootstrapSpec, BrandContent, DialogDismiss, DialogRequest, ErrorCode,
            InvalidErrorCode, Operation, OperationDescriptor, OperationEntry, OperationId,
            OperationProgress, OperationRegistry, OperationStatus, Resource, ScreenEffect, Settled,
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
