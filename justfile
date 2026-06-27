# Nive Framework - Development Commands

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Check all crates
check:
    cargo check --workspace --all-targets --all-features

# Lint all crates
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run all tests
test:
    cargo test --workspace --all-features

# Build documentation
doc:
    cargo doc --workspace --no-deps --all-features --open

# Build documentation without opening a browser
doc-check:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features

# Check every standalone example
examples-check:
    for manifest in examples/*/Cargo.toml; do cargo check --manifest-path "$manifest"; done

# Smoke-check a basic scaffold outside the workspace
scaffold-smoke-basic:
    bash scripts/scaffold-smoke.sh basic

# Smoke-check a dashboard scaffold outside the workspace
scaffold-smoke-dashboard:
    bash scripts/scaffold-smoke.sh dashboard

# Smoke-check all scaffolds outside the workspace
scaffold-smoke: scaffold-smoke-basic scaffold-smoke-dashboard

# Verify publishable crates can be packaged in dependency order
package-check:
    cargo package --package nive-ui
    cargo package --package nive-runtime-derive
    cargo package --package nive-runtime --config 'patch.crates-io.nive-runtime-derive.path="crates/nive-runtime-derive"' --config 'patch.crates-io.nive-ui.path="crates/nive-ui"'
    cargo package --package nive --config 'patch.crates-io.nive-runtime-derive.path="crates/nive-runtime-derive"' --config 'patch.crates-io.nive-runtime.path="crates/nive-runtime"' --config 'patch.crates-io.nive-ui.path="crates/nive-ui"'
    cargo package --package nive-cli

# Run local readiness checks that mirror CI categories
readiness: fmt-check check test lint doc-check examples-check scaffold-smoke package-check icons-check

# List framework icons
icons-list:
    nive icons list

# Sync framework icons
icons-sync:
    nive icons sync

# Check framework icons are up to date
icons-check:
    nive icons check

# Add icon to framework. Usage: just icons-add Search search
icons-add variant lucide_name:
    nive icons add {{ variant }} {{ lucide_name }}

# Create new app using Nive. Usage: just create-app my-app
create-app name:
    nive new {{ name }}

# Build all crates
build:
    cargo build --workspace

# Build release
release:
    cargo build --workspace --release
