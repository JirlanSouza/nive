# Repository Guidelines

## Project Structure & Module Organization

`rag-studio` is a pnpm workspace with two active app layers:

- `src-ui/`: React + TypeScript + Vite frontend. Main code lives in `src-ui/src`, organized by `components/`, `screens/`, `stores/`, `data/`, and `styles/`.
- `src-tauri/`: Tauri v2 Rust backend. Core modules live in `src-tauri/src`, with domain areas such as `app/`, `commands/`, `db/`, `models/`, `search/`, and `document_parser/`.
- `src-tauri/migrations/`: SQLite schema migrations for global and project databases.
- `design/` and `.kiro/specs/`: design references and feature specs.

Keep new frontend tests under `src-ui/src/__tests__/`. Keep Rust tests close to the module they cover, following the existing `*_tests.rs` pattern.

## Build, Test, and Development Commands

- `pnpm install`: install workspace dependencies.
- `pnpm dev`: run the full Tauri desktop app in development.
- `pnpm ui:dev`: run only the Vite frontend.
- `pnpm ui:build`: type-check and build the UI bundle.
- `pnpm --filter rag-studio-ui exec vitest run`: run frontend tests in `jsdom`.
- `cargo test --manifest-path src-tauri/Cargo.toml`: run Rust unit and property tests.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: format Rust code before opening a PR.

## Coding Style & Naming Conventions

Use 2 spaces in TypeScript/TSX and default Rust formatting in `src-tauri/`. Prettier is configured for `printWidth: 120`, semicolons, and double quotes. Frontend files use `kebab-case` and named exports only; component symbols stay `PascalCase` (`document-status-icon.tsx` exports `DocumentStatusIcon`). In Rust, follow the `<name>.rs` + `<name>/` module pattern, use `snake_case` for files/functions, and keep imports grouped as `std`, external crates, then `crate::`. Do not add code comments unless a public Rust API truly needs a short doc comment.

## Architecture & Error Handling

- **Domain Errors:** Use domain-specific errors (e.g. `DocumentError`, `ParseError`) instead of relying solely on the global `AppError`. Integrate domain errors into `AppError` using `#[from]`.
- **Memory Efficiency:** Service methods should prefer references (`&str`) instead of owned strings (`String`) for IDs to avoid unnecessary allocations. The Tauri Command layer acts as the data owner and passes references down to the services.
- **Data Isolation:** All database repository methods must explicitly require a `project_id` parameter to enforce strict multi-tenant data isolation directly at the SQL query level (e.g., `WHERE project_id = ?`).
- **Error Message Formatting:** Use the Key-Value format for error messages to ensure consistency in logs and ease of parsing in the UI. 
  - Instantiation should remain simple: `NotFound(String)` or `NotFound { id1, id2 }`.
  - Format messages explicitly with keys in parentheses: `#[error("Document not found (document_id: {0})")]`.
  - For multiple IDs, use struct variants: `#[error("Chunk not found (chunk_id: {chunk_id}, document_id: {document_id})")] ChunkNotFound { chunk_id: String, document_id: String }`.
  - The frontend will gracefully handle these by splitting the error message by `(` to display a clean, user-friendly text, while the backend retains the full contextual logs.


## Agent Working Rules

- Clarify before acting when the request is ambiguous. Do not guess hidden intent.
- Keep output minimal and task-focused. Do not add features, explanations, or scope that were not requested.
- Edit surgically. Change only the lines or sections needed for the verified target.
- Convert vague requests into a verifiable target before execution. State the target briefly when needed.

## Testing Guidelines

Frontend tests use Vitest with Testing Library and live under `src-ui/src/__tests__/property/*.test.ts`. Add new files as `featureName.test.ts`. Rust uses `cargo test`, with focused test modules such as `docling_tests.rs` and regression fixtures in `src-tauri/proptest-regressions/`. Add or update tests with every behavior change; cover parsing, chunking, and IPC-facing flows when touched.

## Commit & Pull Request Guidelines

Recent history follows Conventional Commit style: `feat: ...`, `refactor: ...`, and scoped variants like `refactor(workspace): ...`. Keep messages imperative and specific.

PRs should include a short problem statement, a summary of user-visible changes, linked issues or specs when applicable, and screenshots or recordings for UI work. Call out schema changes, new migrations, or document parser protocol updates explicitly.
