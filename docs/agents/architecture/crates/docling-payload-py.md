# docling-payload-py

## Role

`crates/docling-payload-py` exposes `docling-payload-core` to Python through PyO3. It is a binding layer, not the source of payload semantics.

## Internal Modules

- `lib.rs`: PyO3 module, Python-facing functions, object conversion, and binding tests.

## Architectural Dependencies

Depends on:

- `docling-payload-core` for payload behavior.
- `pyo3` for Python extension bindings.
- `serde-pyobject` for Python object conversion.

Used by:

- Python parser workflows that need to create or interact with payloads.

Must not depend on:

- `app-core`
- `app-database`
- `app-gui`

## Workflow

Binding changes should usually:

1. implement semantic changes in `docling-payload-core` first
2. expose only the needed Python API in this crate
3. preserve feature behavior for extension modules and Python tests
4. verify through Python-enabled tests when bindings change

## Testing

- Rust-only focused command: `cargo test -p docling-payload-py`.
- Python-enabled focused command: `cargo test --manifest-path crates/docling-payload-py/Cargo.toml --features python-tests --no-default-features`.
- Active parser workflow command: `just payload-py-test`.

## Rules

Do:

- keep Python API shape small and explicit
- convert Python objects at the boundary
- rely on `docling-payload-core` for validation and payload behavior

Do not:

- duplicate payload semantics in binding code
- add application service dependencies
- make Python tests mandatory for unrelated Rust changes
