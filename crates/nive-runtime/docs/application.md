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

`BootstrapSpec` accepts a task factory so retries create independent attempts.
When configured, the runner opens an internal splash, correlates results with
their attempt, enforces the minimum splash duration and calls `Application::init`
only after success. The bootstrap value is transferred into `init`; the runtime
does not retain product clients or services afterward. The initial `AppUpdate`
is processed before configured initial product windows open.

Failure, retry, diagnostic details and close-during-bootstrap are runtime-owned.
Closing the splash exits without constructing the application. Apps without
bootstrap continue to use `Bootstrap = ()`.

## Theme Runtime

`ThemeController` owns the configured `ThemePreference`, current system mode,
effective Nive theme, initial system-theme detection task and system-change
subscription. The runner applies it internally and emits `CoreEvent::ThemeChanged`
when the effective theme changes.

Only `ThemeController` synchronizes the global `nive-ui` active-theme snapshot.
Application code must not call Iced system-theme APIs or mutate the snapshot
directly.
