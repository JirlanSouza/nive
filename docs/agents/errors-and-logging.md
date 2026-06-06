# Errors And Logging Guidelines

## Domain Errors

Use domain-specific errors such as `DocumentError` or `ParseError` instead of relying only on a global `AppError`.

Integrate domain errors into `AppError` with `#[from]`.

## Error Messages

Use key-value context in parentheses:

```rust
#[error("Document not found (document_id: {0})")]
NotFound(String)
```

For multiple identifiers, prefer struct variants:

```rust
#[error("Chunk not found (chunk_id: {chunk_id}, document_id: {document_id})")]
ChunkNotFound {
    chunk_id: String,
    document_id: String,
}
```

Instantiation should remain simple: `NotFound(String)` or `NotFound { id1, id2 }`.

## Log Messages

Use concise, sentence-style log messages. When including values, append them in key-value format with spaces inside parentheses:

```text
Message ( key: value, key: value )
```

Include only identifiers and context needed to debug the event. Avoid dumping full structs with `{:?}` on normal paths.

Use log levels consistently:

- `trace`: noisy internal flow or high-frequency progress.
- `debug`: routine lifecycle details.
- `info`: important successful milestones.
- `warn`: unexpected but recoverable states.
- `error`: failures that need attention.
