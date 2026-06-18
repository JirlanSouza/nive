# Application Contract

`nive-runtime` exposes the stable-Rust application contract through
`nive_runtime::prelude`.

Applications implement `Application` with product-owned message, window and
bootstrap types. Apps without bootstrap use `type Bootstrap = ();`. Rust stable
does not support the approved associated-type default syntax, so the explicit
associated type is required.

`ApplicationConfig` declares product windows, initial windows, theme preference,
toast position, fonts, a shared window icon and optional bootstrap
configuration. `Context` and `WindowContext` are read-only views; runtime-owned
mutable state is not exposed.

`Update` combines Iced tasks, one optional screen outcome and ordered
`RuntimeCommand` values. Application hooks use `AppUpdate`, which cannot carry a
screen outcome.

`run::<A>()` owns the private Iced daemon state. Product view messages are
correlated with their source window automatically, while task and subscription
messages remain unscoped. The runner processes ordered runtime commands,
configured initial windows, dynamic titles, app subscriptions and core events.

Bootstrap specifications are accepted by the public config builder but runner
execution for them starts in the bootstrap slice. Until then, configured
bootstrap returns `Error::BootstrapUnavailable`; applications using the current
runner use `Bootstrap = ()`.

## Theme Runtime

`ThemeController` owns the configured `ThemePreference`, current system mode,
effective Nive theme, initial system-theme detection task and system-change
subscription. The runner applies it internally and emits `CoreEvent::ThemeChanged`
when the effective theme changes.

Only `ThemeController` synchronizes the global `nive-ui` active-theme snapshot.
Application code must not call Iced system-theme APIs or mutate the snapshot
directly.
