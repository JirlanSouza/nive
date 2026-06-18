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

Generic feedback, dialog host and toast host extraction is scheduled for the
later UI implementation slices.
