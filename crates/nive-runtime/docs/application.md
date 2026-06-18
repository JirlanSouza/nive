# Application Contract

`nive-runtime` exposes the stable-Rust application contract through
`nive_runtime::prelude`.

Applications implement `Application` with product-owned message, window and
bootstrap types. Apps without bootstrap use `type Bootstrap = ();`. Rust stable
does not support the approved associated-type default syntax, so the explicit
associated type is required.

`ApplicationConfig` declares product windows, initial windows, theme preference,
toast position and optional bootstrap configuration. `Context` and
`WindowContext` are read-only views; runtime-owned mutable state is not exposed.

`Update` combines Iced tasks, one optional screen outcome and ordered
`RuntimeCommand` values. Application hooks use `AppUpdate`, which cannot carry a
screen outcome.

The public runner signature is reserved, but runner execution is intentionally
unavailable until the core/runtime migration slice. Calling `run` currently
returns `Error::RunnerUnavailable`.
