# Nive

A Rust/Iced framework for building desktop applications.

## Features

- **Design System**: Semantic theme contracts, tokens, and reusable widgets
- **Application Lifecycle**: Window management, bootstrap, feedback, and devtools
- **Icon Management**: Integrated Lucide icon system with xtask automation
- **Scaffolding**: `create-nive-app` CLI for quick project setup

## Quick Start

```bash
# Create a new app
cargo run --package create-nive-app -- my-app

# Build and run
cd my-app
cargo build
cargo run
```

## Crates

- [`nive`](crates/nive): Umbrella crate that re-exports everything
- [`nive-ui`](crates/nive-ui): Visual design system (tokens, theme, widgets, icons)
- [`nive-runtime`](crates/nive-runtime): Application lifecycle, window management, feedback
- [`nive-runtime-derive`](crates/nive-runtime-derive): Proc macros for devtools
- [`xtask`](crates/xtask): Icon management and project automation
- [`create-nive-app`](crates/create-nive-app): Scaffolding CLI

## Documentation

- [Getting Started Guide](docs/guides/getting-started.md)
- [Adding Icons](docs/guides/adding-icons.md)
- [Development Guide](docs/development.md)
- [Architecture](docs/agents/architecture.md)

## Development

```bash
# Format code
just fmt

# Check all crates
just check

# Run tests
just test

# Build documentation
just doc

# Sync framework icons
just icons-sync
```

## License

Apache-2.0
