# Nive UI Contracts

`nive_ui::prelude` exposes the Nive `Element`, renderer, theme types, common
layout primitives and reusable widgets.

Theme definitions remain in `nive-ui`. `ThemeId` identifies a theme,
`ThemeCatalog` resolves its `ThemeData`, and `theme::active()` provides the
current snapshot used by view helpers.

Generic feedback, dialog host and toast host extraction is scheduled for the
later UI implementation slices.
