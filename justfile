# Nive Framework - Development Commands

import 'just/app-framework.just'

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

# Run a standalone example with the reusable dev reload loop.
example-dev example:
    just app-dev-cwd examples/{{ example }} nive-example-{{ example }} {{ example }} ""

# Run the widget gallery with terminal-triggered reload.
widget-gallery-dev:
    just example-dev widget-gallery

# Run the widget gallery with devtools and terminal-triggered reload.
widget-gallery-devtools:
    NIVE_DEVTOOLS=1 just app-dev-cwd examples/widget-gallery nive-example-widget-gallery widget-gallery "devtools"

# Smoke-check a basic scaffold outside the workspace
scaffold-smoke-basic:
    bash scripts/scaffold-smoke.sh basic

# Smoke-check a dashboard scaffold outside the workspace
scaffold-smoke-dashboard:
    bash scripts/scaffold-smoke.sh dashboard

# Smoke-check all scaffolds outside the workspace (local [patch.crates-io] form)
scaffold-smoke: scaffold-smoke-basic scaffold-smoke-dashboard

# Smoke-check a basic scaffold using a Git dependency (file:// URL + HEAD rev)
scaffold-smoke-github-basic:
    bash scripts/scaffold-smoke-github.sh basic

# Smoke-check a dashboard scaffold using a Git dependency (file:// URL + HEAD rev)
scaffold-smoke-github-dashboard:
    bash scripts/scaffold-smoke-github.sh dashboard

# Smoke-check all scaffolds using the GitHub alpha dependency shape
scaffold-smoke-github: scaffold-smoke-github-basic scaffold-smoke-github-dashboard

# Verify publishable crates can be packaged in dependency order
package-check:
    cargo package --package nive-core
    cargo package --package nive-ui --config 'patch.crates-io.nive-core.path="crates/nive-core"'
    cargo package --package nive-runtime-derive
    cargo package --package nive-runtime --config 'patch.crates-io.nive-core.path="crates/nive-core"' --config 'patch.crates-io.nive-runtime-derive.path="crates/nive-runtime-derive"' --config 'patch.crates-io.nive-ui.path="crates/nive-ui"'
    cargo package --package nive-workbench --config 'patch.crates-io.nive-core.path="crates/nive-core"' --config 'patch.crates-io.nive-runtime.path="crates/nive-runtime"' --config 'patch.crates-io.nive-ui.path="crates/nive-ui"'
    cargo package --package nive --config 'patch.crates-io.nive-core.path="crates/nive-core"' --config 'patch.crates-io.nive-runtime-derive.path="crates/nive-runtime-derive"' --config 'patch.crates-io.nive-runtime.path="crates/nive-runtime"' --config 'patch.crates-io.nive-ui.path="crates/nive-ui"' --config 'patch.crates-io.nive-workbench.path="crates/nive-workbench"'
    cargo package --package nive-cli

# Run local readiness checks that mirror CI categories
readiness: fmt-check check test lint doc-check examples-check scaffold-smoke scaffold-smoke-github package-check icons-check

# List framework icons
icons-list:
    cd crates/nive-ui && cargo run -p nive-cli -- icons list

# Sync framework icons
icons-sync:
    cd crates/nive-ui && cargo run -p nive-cli -- icons sync --framework

# Check framework icons are up to date
icons-check:
    cd crates/nive-ui && cargo run -p nive-cli -- icons check --framework

# Add framework icon symbol. Usage: just icons-add-symbol User user
icons-add-symbol variant provider_ref:
    cd crates/nive-ui && cargo run -p nive-cli -- icons add-symbol {{ variant }} {{ provider_ref }}

# Set framework icon role. Usage: just icons-set-role window-close lucide:x
icons-set-role role provider_ref:
    cd crates/nive-ui && cargo run -p nive-cli -- icons set-role {{ role }} {{ provider_ref }}

# Create new app using Nive. Usage: just create-app my-app
create-app name:
    nive new {{ name }}

# Build all crates
build:
    cargo build --workspace

# Build release
release:
    cargo build --workspace --release
