//! Nive - A Rust/Iced framework for building desktop applications.
//!
//! This is the umbrella crate that re-exports `nive-ui` and `nive-runtime`
//! for convenient app development.
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
//! compiles the scaffolded counter template out of the box. Apps that use
//! toasts, async state, dialogs, file picker, theming, shortcuts, or
//! window-handle types switch to `nive::prelude::ui::*` for the extended
//! surface.
//!
//! The crate root also re-exports the runtime and UI crates for convenience.
//! Those broad crate-root exports are beta before 1.0; application templates
//! should prefer the prelude tiers above for the most stable import shape.
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
//!
//! # Status
//!
//! Part of Nive **v0.1.0**, a beta release. Public APIs may change before 1.0.

pub use nive_runtime as runtime;
pub use nive_ui as ui;

pub mod prelude {
    // Minimal template-stable surface. Compiles the scaffolded counter
    // template without extra `use` statements. The extended surface lives
    // in `nive::prelude::ui::*`.
    pub use nive_runtime::prelude::{
        command_palette_rows, install_diagnostic_panic_hook, keyboard_navigation_subscription,
        relative_time_label, run, time, unix_now, window, Action, ActionId, ActionMap, AppUpdate,
        Application, ApplicationConfig, CloseDecision, CommandRejected, CommandRejectionReason,
        Context, CoreEvent, DiagnosticSnapshot, DuplicateActionId, Error, ExitDecision,
        KeyboardNavigation, Never, PlatformError, Point, RequestId, Result, RuntimeCommand,
        RuntimeEvent, RuntimeEventKind, RuntimeEventLog, RuntimeSession, ScreenView,
        SettingsConfig, SettingsError, SettingsErrorKind, ShortcutBinding, ShortcutKey,
        ShortcutMap, SimpleApplication, Size, Subscription, Task, Theme, ThemeBuilder,
        ThemeCatalog, ThemeController, ThemeEvent, ThemeMode, ThemePreference, Toast,
        ToastPosition, Update, WindowCardinality, WindowCommand, WindowContext, WindowQuery,
        WindowRole, WindowSession, WindowSessionPosition, WindowSessionSize, WindowSpec,
    };
    pub use nive_ui::prelude::*;

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
        // Extended runtime tier; `ToastTone` is omitted (already in
        // `super::*` via `nive_ui::prelude::*`) to avoid an ambiguous-glob
        // warning. Similarly, `Application`, `ApplicationConfig`, etc. come
        // in through `super::*`.
        #[allow(deprecated)]
        pub use crate::ToastRequest;
        pub use crate::{
            BackgroundFit, BootstrapSpec, BrandContent, DialogDismiss, DialogRequest, ErrorCode,
            InvalidErrorCode, Operation, OperationDescriptor, OperationEntry, OperationId,
            OperationProgress, OperationRegistry, OperationStatus, RequestId, Resource,
            ScreenUpdate, Settled, Toast, ToastDuration, UserFacingError, UserFacingErrorKind,
            UserFacingResult, WindowChrome, WindowHandle, WindowMode, WindowRegistry,
        };
        #[cfg(feature = "file-picker")]
        pub use crate::{FileFilter, PickFileParams, SaveFileParams};
    }
}

pub use nive_runtime::*;
pub use nive_ui::*;

pub use nive_runtime::{ToastPosition, ToastTone};

#[cfg(feature = "devtools")]
#[doc(hidden)]
pub use nive_runtime::__inspect;

#[cfg(feature = "devtools")]
pub use nive_runtime::devtools;
