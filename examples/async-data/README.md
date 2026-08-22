# Async Data Example

Demonstrates the direct, scoped `Resource` path with tracked cancellation.

## What it demonstrates

Loading and refreshing data while retaining the last usable value, rejecting
stale delivery, and cancelling the underlying tracked request.

## Concepts exercised

- `Resource::load(context.app_scope(), …)` returning `RequestTask`
- `CancelSignal` available to the application future
- retained values while `is_refreshing()` is true
- explicit `Resource::cancel()` forwarded through `Effect::cancel`
- typed `Settled<T>` and `SettleOutcome`
- restart policy: clicking “Load” again stops and replaces the prior request

## How to run

```bash
cd examples/async-data
cargo run
```
