# Run app-gui in development mode.
app-gui-dev:
    just app-dev app-gui app-gui ""

# Run app-gui in development with a UI error scenario. Usage: just dev-error create_project
dev-error ui_error:
    just app-dev-error app-gui app-gui "{{ ui_error }}"

# Run app-gui with the Devtools auxiliary window enabled.
devtools:
    just app-devtools app-gui app-gui

# Deprecated alias for `just devtools`.
dev-error-panel:
    just app-devtools app-gui app-gui

# List available UI error-injection scenarios.
dev-error-list:
    just app-dev-error-list app-gui

# Build app-gui in debug mode.
app-gui-build:
    just app-build app-gui

# Build app-gui in release mode.
app-gui-release:
    just app-release app-gui

# Check app-gui.
app-gui-check:
    just app-check app-gui

# Check app-gui with Devtools/dev probes compiled.
app-gui-check-dev:
    cargo check --package app-gui --all-targets --features dev

# Test app-gui.
app-gui-test:
    just app-test app-gui

# Test app-gui with Devtools/dev probes compiled.
app-gui-test-dev:
    cargo test --package app-gui --features dev

alias gui-dev := app-gui-dev
alias gui-devtools := devtools
alias gui-dev-error := dev-error
alias gui-dev-error-panel := dev-error-panel
alias gui-dev-error-list := dev-error-list
alias gui-build := app-gui-build
alias gui-release := app-gui-release
alias gui-check-dev := app-gui-check-dev
alias gui-test-dev := app-gui-test-dev
