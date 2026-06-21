# Nive Framework - Development Commands

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Check all crates
check:
    cargo check --workspace --all-targets

# Lint all crates
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Run all tests
test:
    cargo test --workspace

# Build documentation
doc:
    cargo doc --workspace --no-deps --open

# List framework icons
icons-list:
    cargo run --quiet --package xtask -- icons list

# Sync framework icons
icons-sync:
    cargo run --quiet --package xtask -- icons sync

# Check framework icons are up to date
icons-check:
    cargo run --quiet --package xtask -- icons check

# Add icon to framework. Usage: just icons-add Search search
icons-add variant lucide_name:
    cargo run --quiet --package xtask -- icons add {{ variant }} {{ lucide_name }}

# Create new app using Nive. Usage: just create-app my-app
create-app name:
    cargo run --quiet --package create-nive-app -- {{ name }}

# Build all crates
build:
    cargo build --workspace

# Build release
release:
    cargo build --workspace --release
