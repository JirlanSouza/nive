# Nive Development Guide

This guide is for contributors working on the Nive framework itself.

## Requirements

- Rust 1.92 or later
- Cargo
- just
- curl (for icon syncing)

## Setup

Clone the repository:

```bash
git clone https://github.com/yourusername/nive.git
cd nive
```

## Development Commands

```bash
# Format code
just fmt

# Check formatting
just fmt-check

# Check all crates
just check

# Lint all crates
just lint

# Run all tests
just test

# Build documentation
just doc

# Build all crates
just build

# Build release
just release
```

## Icon Management

The framework maintains a set of essential icons in `nive-ui`.

```bash
# List framework icons
just icons-list

# Sync framework icons
just icons-sync

# Check icons are up to date
just icons-check

# Add icon to framework
just icons-add <Variant> <lucide-name>
```

## Creating Test Apps

```bash
# Create a new app using the framework
just create-app test-app

# Or manually
cargo run --package create-nive-app -- test-app
```

## Architecture

See [docs/agents/architecture.md](agents/architecture.md) for framework architecture.

## Testing

Run the full test suite:

```bash
just test
```

Run tests for a specific crate:

```bash
cargo test --package nive-ui
cargo test --package nive-runtime
```

## Documentation

Build and open documentation:

```bash
just doc
```

Or manually:

```bash
cargo doc --workspace --no-deps --open
```

## Publishing

When ready to publish:

```bash
# Dry run
cargo publish --package nive-ui --dry-run
cargo publish --package nive-runtime --dry-run
cargo publish --package nive --dry-run
cargo publish --package create-nive-app --dry-run

# Publish
cargo publish --package nive-ui
cargo publish --package nive-runtime
cargo publish --package nive
cargo publish --package create-nive-app
```

Note: Publish order matters due to dependencies.
