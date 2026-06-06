# docling-payload-core

## Role

`crates/docling-payload-core` owns Docling payload semantics. It builds, validates, serializes, reads, and exposes views over Docling-derived payloads.

## Internal Modules

- `payload`: payload root structures and version metadata.
- `schema`: Docling schema types and conversion support.
- `reader`: payload file reader and archived view access.
- `views`: high-level read views such as table/document view helpers.
- `serialization`: Rkyv serialization support.
- `error`: compatibility, build, read, and write errors.

## Architectural Dependencies

Depends on:

- `serde_json` for input schema data.
- `rkyv`/`bytecheck` for archive format and validation.
- optional read/write IO crates behind features.

Used by:

- `app-core` for parsed document reads.
- `docling-payload-py` for Python binding exposure.

Must not depend on:

- `app-core`
- `app-database`
- `app-gui`
- `document-parser-worker`

## Workflow

Payload changes should usually:

1. update schema conversion or views
2. preserve compatibility checks and explicit reparse reasons
3. update read/write feature behavior if IO changes
4. add fixture or runtime tests for schema and archive behavior

## Testing

- Use focused unit tests for schema conversion and views.
- Use runtime tests for payload read/write compatibility.
- Focused command: `cargo test -p docling-payload-core`.

## Rules

Do:

- keep payload compatibility errors precise
- keep archive format changes deliberate and tested
- avoid leaking application service concepts into payload code

Do not:

- add dependencies on application crates
- silently accept incompatible payload/schema versions
- mix Python binding behavior into core payload semantics
