# Generic app-framework recipes for the Nive runtime.
# App-specific justfiles should set variables and call these recipes.

# Build an app package, run the binary, and watch terminal keys for reload/quit.
# Set NIVE_APP_FEATURES, NIVE_DEV_ERROR, or NIVE_DEVTOOLS
# before calling this recipe to enable dev features.
app-dev package binary features_env:
    #!/usr/bin/env bash
    set -euo pipefail

    app_pid=""
    original_stty=""

    cleanup() {
      if [ -n "$original_stty" ]; then
        stty "$original_stty" 2>/dev/null || true
      fi

      if [ -n "$app_pid" ] && kill -0 "$app_pid" 2>/dev/null; then
        kill "$app_pid" 2>/dev/null || true
        wait "$app_pid" 2>/dev/null || true
      fi
    }

    app_running() {
      local pid

      for pid in $(jobs -pr); do
        if [ "$pid" = "$app_pid" ]; then
          return 0
        fi
      done

      return 1
    }

    drain_keys() {
      local ignored

      while IFS= read -rsn1 -t 1 ignored; do
        :
      done
    }

    trap cleanup EXIT INT TERM

    while :; do
      if [ -n "{{ features_env }}" ]; then
        NIVE_APP_FEATURES="{{ features_env }}" cargo build --package {{ package }} --features "{{ features_env }}"
      else
        cargo build --package {{ package }}
      fi

      if [ -n "${NIVE_DEV_ERROR:-}" ]; then
        printf 'Dev probes enabled: %s\n' "$NIVE_DEV_ERROR"
      fi
      if [ -n "${NIVE_DEVTOOLS:-}" ]; then
        printf 'Devtools window enabled: %s\n' "$NIVE_DEVTOOLS"
      fi
      ./target/debug/{{ binary }} 2>&1 &
      app_pid=$!
      reload_requested=0

      printf '\n{{ binary }} running. stdout/stderr are shown here. Focus this terminal and press r to rebuild/reload, Ctrl+R to rebuild/reload, or q to quit.\n'

      original_stty="$(stty -g)"
      stty raw -echo

      while app_running; do
        key=""
        IFS= read -rsn1 -t 1 key || true

        case "$key" in
          r|R|$'\022')
            reload_requested=1
            kill "$app_pid" 2>/dev/null || true
            wait "$app_pid" 2>/dev/null || true
            drain_keys
            break
            ;;
          q|Q)
            kill "$app_pid" 2>/dev/null || true
            wait "$app_pid" 2>/dev/null || true
            exit 0
            ;;
        esac
      done

      stty "$original_stty"
      original_stty=""

      if [ "$reload_requested" -eq 1 ]; then
        printf '\nReload requested from terminal. Rebuilding {{ package }}...\n\n'
        sleep 0.8
      else
        if wait "$app_pid"; then
          status=0
        else
          status=$?
        fi
        exit "$status"
      fi
    done

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