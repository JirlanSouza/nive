# Async Data Example

Demonstrates `Resource` with stale-request guarding and an app-owned `OperationRegistry`.

## What it demonstrates

Loading data asynchronously with protection against stale responses from outdated requests.

## Concepts exercised

- `Resource<T>` with `begin()` / `settle(Settled<T>)`
- Internal request IDs carried through `Settled<T>`
- App-owned `OperationRegistry` registration and completion
- `OperationDescriptor` with `cancellable` flag
- `perform` for async background work not tied to a `Resource`
- Stale-request guarding: clicking "Load" twice quickly discards the first response

## How to run

```bash
cd examples/async-data
cargo run
```
