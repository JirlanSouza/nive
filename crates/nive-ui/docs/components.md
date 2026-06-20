# Nive UI Contracts

`nive_ui::prelude` exposes the Nive `Element`, renderer, theme types, common
layout primitives and reusable widgets.

Theme definitions remain in `nive-ui`. `ThemeId` identifies a theme,
`ThemeCatalog` is the single resolver for static `ThemeData`, and
`theme::active()` provides the current snapshot used by view helpers. The
active-theme atomic is private to `nive-ui`; runtime synchronization is exposed
only through the framework integration module.

Tests that change the global snapshot must hold
`theme::testing::ThemeTestGuard`, which restores the previous theme when
dropped.

## Dialog Infrastructure

`DialogHost` owns modal composition, backdrop rendering, pointer blocking and
focus trapping. Runtime integrations provide optional backdrop and Escape
messages; the host publishes those messages without changing product state.

Dialog content remains composed with `Dialog`, `DialogHeader`,
`DialogFooter` and `DialogActionFooter`.

## Feedback And Status

`nive-ui` owns the reusable feedback and status components:

- `ErrorFeedback`, `ErrorEmptyState`, `ErrorStatusLine` and `ErrorDetailsDialog`
- `ResourceStatusLine`, `OperationStatusLine` and `OperationActionGroup`
- `InitialAvatar`, `MetricCard` and `VersionBadge`

Presentation contracts keep runtime types out of the UI crate:

- `ErrorPresentation`
- `ResourceStatusPresentation`
- `OperationStatusPresentation`

`nive-runtime::UserFacingError`, `AsyncState<T>` and `OperationState<C>`
implement these contracts. Applications supply product copy and messages while
Nive owns the reusable visual composition.

## Bootstrap Template

`BootstrapView` owns the generic loading and startup-failure composition,
including brand placement, animated status dots, retry/details actions and the
error-details dialog content. Applications supply product assets and copy;
`nive-runtime` supplies lifecycle state and internal messages.

## Toast Host

`ToastHost` owns the generic toast overlay: corner positioning, hover
pause/resume wiring and dismissible toast rows built from the
`ToastPresentation` contract. `nive-runtime::ToastItem` implements
`ToastPresentation`, so the runtime owns toast identity, visible/queued state,
promotion and timing while `nive-ui` owns only the visual composition. The
runtime applies the host automatically to app-role windows; applications do not
mount it themselves and toasts may remain visible alongside a modal dialog.
