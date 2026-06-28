# Generic app-framework recipes for the Nive runtime.
# App-specific justfiles should set variables and call these recipes.

import './app-dev.just'

# Run dev loop with dev feature and devtools env.
app-devtools package binary:
    NIVE_APP_FEATURES="dev" NIVE_DEVTOOLS=1 just app-dev {{ package }} {{ binary }} "dev"

# Run dev loop with dev feature and a UI error scenario.
app-dev-error package binary scenario:
    NIVE_APP_FEATURES="dev" NIVE_DEV_ERROR="{{ scenario }}" just app-dev {{ package }} {{ binary }} "dev"

# Run the binary with --list-ui-error-scenarios.
app-dev-error-list package:
    cargo run --package {{ package }} --features dev -- --list-ui-error-scenarios

# Build a package in debug mode.
app-build package:
    cargo build --package {{ package }}

# Build a package in release mode.
app-release package:
    cargo build --package {{ package }} --release

# Check a package.
app-check package:
    cargo check --package {{ package }} --all-targets

# Test a package.
app-test package:
    cargo test --package {{ package }}
