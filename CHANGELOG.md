# Changelog

This file records notable user-visible changes to the Nive workspace. Versions
follow Semantic Versioning, and entries are grouped by their effect on users.
Internal refactors, tests, CI, formatting, and documentation maintenance without
user-visible impact are omitted.

## [Unreleased]

### Changed

- **Breaking:** replace per-state async tokens with affine, scoped tracked
  requests. `Resource` and `Operation<C, T>` now return explicit settlement
  outcomes, distinguish cancellation from failure, support typed mutation
  output, and are no longer `Clone`. Direct `load`/`run` callers must pass a
  scope and return the resulting `RequestTask` through `Effect`; see
  [runtime flows](docs/architecture/runtime-flows.md#5-tracked-requests-settlement-and-cancellation)
  for the direct, handle, external-owner, and manual migration tiers.

### Fixed

- Keep generated icon Rust deterministic and compatible with `cargo fmt
  --check` across application scaffolds and framework-owned manifests.
- Ensure hosted dialogs report modal activity so Toast expiry pauses while a
  dialog is open.

## [0.1.0-alpha.1] - 2026-07-28

### Added

- Initial GitHub alpha distribution for installing `nive-cli` from a tag and
  creating or initializing applications against the same Nive revision.

[Unreleased]: https://github.com/JirlanSouza/nive/compare/v0.1.0-alpha.1...HEAD
[0.1.0-alpha.1]: https://github.com/JirlanSouza/nive/tree/v0.1.0-alpha.1
