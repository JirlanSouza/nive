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

Generic feedback and toast host extraction remains scheduled for later UI
implementation slices.
