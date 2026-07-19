//! Nive - A Rust/Iced framework for building desktop applications.
//!
//! This is the umbrella crate that re-exports `nive-ui`, `nive-runtime`, and
//! `nive-workbench` for convenient app development.
//!
//! # Quick Start
//!
//! ```ignore
//! use nive::prelude::*;
//!
//! struct MyApp;
//!
//! impl Application for MyApp {
//!     // ... implement Application trait (use `type Window = ()` and
//!     // `type Bootstrap = ()` for a single-window, no-splash app)
//! }
//!
//! fn main() -> nive::Result {
//!     nive::run::<MyApp>()
//! }
//! ```
//!
//! # Prelude tiers
//!
//! `nive::prelude::*` exposes the minimal template-stable surface: it
//! compiles the scaffolded counter template out of the box, and already
//! includes basic toasts, theming, icon roles/catalogs, shortcuts, actions,
//! runtime events, and the window *declaration* types (`WindowSpec`/`WindowRole`). Apps switch to
//! `nive::prelude::ui::*` for the extended surface: async state
//! (`Resource`/`Operation`), dialogs, `UserFacingError`, bootstrap/splash,
//! runtime window handles (`WindowHandle`/`WindowRegistry`/`WindowMode`),
//! `ToastDuration`, and file picker params.
//!
//! The crate root also re-exports the runtime and UI crates for convenience,
//! and exposes workbench APIs through `nive::workbench` plus the prelude tiers.
//! Broad crate-root exports are beta before 1.0; application templates should
//! prefer the prelude tiers above for the most stable import shape.
//!
//! Feature-gated APIs are exposed through the corresponding `nive` feature
//! flags, including `devtools` and `file-picker`. Devtools simulator internals
//! and generated inspect support are experimental before 1.0 and should not be
//! treated as production template dependencies.
//!
//! # Crates
//!
//! - `nive-ui`: visual design system (tokens, theme, widgets, icons)
//! - `nive-runtime`: application lifecycle, window management, feedback, devtools
//! - `nive-workbench`: fixed-region professional desktop shell
//!
//! # Status
//!
//! Part of Nive **v0.1.0**, a beta release. Public APIs may change before 1.0.

pub use nive_runtime as runtime;
pub use nive_ui as ui;
pub use nive_workbench as workbench;

/// The minimal `nive::prelude::*` tier compiles the canonical UI Dialog
/// family (widgets, not runtime hosting): `Dialog`, `DialogHeader`,
/// `DialogActionFooter`, and friends. Runtime request/dismissal policy
/// (`DialogRequest`/`DialogDismiss`) needs the extended
/// `nive::prelude::ui::*` tier:
///
/// ```compile_fail
/// use nive::prelude::*;
///
/// let _dialog: Option<DialogRequest<'static, ()>> = None;
/// ```
pub mod prelude {
    // Minimal template-stable surface. Compiles the scaffolded counter
    // template without extra `use` statements. The extended surface lives
    // in `nive::prelude::ui::*`.
    pub use nive_runtime::prelude::{
        install_diagnostic_panic_hook, keyboard_navigation_subscription, relative_time_label, run,
        time, unix_now, window, Action, ActionId, ActionMap, Application, ApplicationConfig,
        CloseDecision, CommandRejected, CommandRejectionReason, Context, DiagnosticEvent,
        DiagnosticEventKind, DiagnosticEventLog, DiagnosticSnapshot, DuplicateActionId, Effect,
        Error, ExitDecision, KeyboardNavigation, MessageContext, MessageSource, NamedShortcutKey,
        Never, PlatformError, Point, RequestId, Result, RuntimeEvent, RuntimeSession, ScreenView,
        SettingsConfig, SettingsError, SettingsErrorKind, ShortcutBinding, ShortcutKey,
        ShortcutMap, ShortcutModifiers, SimpleApplication, Size, Subscription, Task, Theme,
        ThemeBuilder, ThemeCatalog, ThemeController, ThemeEvent, ThemeMode, ThemePreference, Toast,
        ToastPosition, WindowCardinality, WindowCommand, WindowContext, WindowQuery, WindowRole,
        WindowSession, WindowSessionPosition, WindowSessionSize, WindowSpec,
    };
    pub use nive_ui::prelude::*;
    pub use nive_workbench::prelude::*;

    pub use nive_ui::widgets::Icon;

    /// Extended surface for app code that uses toasts, async state,
    /// dialogs, file picker params, theming, shortcuts, or window-handle
    /// types. Use `nive::prelude::ui::*`.
    ///
    /// This sub-module re-exports the minimal tier (via `super::*`) plus
    /// extended runtime types. The single `use nive::prelude::ui::*;` is
    /// enough for an app that renders `nive-ui` widgets and exercises
    /// toasts, async state, dialogs, file picker, theming, shortcuts, etc.
    pub mod ui {
        pub use super::*;
        // Extended runtime tier. `Application`, `ApplicationConfig`, etc.
        // come in through `super::*`.
        pub use crate::{
            BackgroundFit, BootstrapSpec, BrandContent, DialogDismiss, DialogRequest, ErrorCode,
            InvalidErrorCode, Operation, OperationDescriptor, OperationEntry, OperationId,
            OperationProgress, OperationRegistry, OperationStatus, RequestId, Resource,
            ScreenEffect, Settled, Toast, ToastDuration, UserFacingError, UserFacingErrorKind,
            UserFacingResult, WindowChrome, WindowHandle, WindowMode, WindowRegistry,
        };
        #[cfg(feature = "file-picker")]
        pub use crate::{FileFilter, PickFileParams, SaveFileParams};
    }
}

pub use nive_runtime::*;
pub use nive_ui::*;

#[cfg(feature = "devtools")]
#[doc(hidden)]
pub use nive_runtime::__inspect;

#[cfg(feature = "devtools")]
pub use nive_runtime::devtools;
