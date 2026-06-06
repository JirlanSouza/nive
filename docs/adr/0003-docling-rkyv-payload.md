# ADR 0003: Use Rkyv As The Canonical Parsed Payload Format

## Status

Accepted

## Context

The document parser produces structured Docling document output. Runtime consumers need fast, validated access to parsed document data, including structural references and table/media views.

JSON is useful for debug output and fixtures, but it is not the production runtime format.

## Decision

Use `.rkyv` payload files as the canonical parsed artifact.

`docling-payload-core` owns payload construction, compatibility validation, read/write behavior, and high-level views.

`docling-payload-py` exposes the payload core to Python for parser integration.

## Consequences

- Runtime consumers read `.rkyv` payloads.
- Compatibility failures should produce explicit reparse reasons.
- JSON may be used for debug output and tests, but production parse success must not depend on JSON.
- Payload format or schema changes require focused compatibility tests.
