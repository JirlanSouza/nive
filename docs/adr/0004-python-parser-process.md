# ADR 0004: Run The Document Parser As An Isolated Python Process

## Status

Accepted

## Context

PDF parsing relies on Python and Docling. Parsing can be heavy, failure-prone, and dependency-sensitive compared with the Rust desktop app.

The Rust application needs parser progress and completion events without embedding parser runtime concerns directly into UI code.

## Decision

Run the document parser as an isolated Python child process managed by `document-parser-worker`.

Communicate with the parser process over NDJSON through stdio. Keep process lifecycle, command dispatch, parser event parsing, and restart behavior inside `document-parser-worker`.

Use `app-core` parse services and events to coordinate document status, parse jobs, and downstream rule behavior.

## Consequences

- Parser failures are isolated from the UI process.
- Protocol changes must update both Rust worker protocol handling and Python parser behavior.
- UI code observes parse state through services/events, not direct process handles.
- Parser package/build workflows remain part of the active development toolchain.
