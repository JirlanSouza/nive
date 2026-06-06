# Parser And Payload Architecture

## Parser Worker

`crates/document-parser-worker` manages the Python document parser process. It is responsible for process lifecycle, command dispatch, parser events, and recovery around parser failures.

The parser process communicates over NDJSON through stdio. Keep protocol parsing and process management in this crate rather than leaking process details into `app-gui`.

## Parse Service

`app-core::parsing` owns parse orchestration:

- validate document status and document type
- create parse jobs
- update document and parse job status
- enqueue parser work through the `ParseWorker` trait
- handle parser events and recovery

Use parse events such as job started, progress, done, and error to keep services decoupled.

## Docling Payloads

`crates/docling-payload-core` owns:

- Docling schema conversion
- payload construction
- Rkyv read/write
- compatibility checks
- document/table/media views

`crates/docling-payload-py` exposes the payload core to Python through PyO3. Keep binding-specific behavior there and shared payload behavior in `docling-payload-core`.

## Boundaries

- Parser protocol changes should be called out explicitly.
- Payload format or schema compatibility changes should include focused tests.
- Keep payload IO and compatibility errors specific enough for reparsing decisions.
