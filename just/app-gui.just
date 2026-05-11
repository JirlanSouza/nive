# Run app-gui in development.
app-gui-dev:
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
      cargo build --package app-gui
      ./target/debug/app-gui 2>&1 &
      app_pid=$!
      reload_requested=0

      printf '\napp-gui running. stdout/stderr are shown here. Focus this terminal and press r to rebuild/reload, Ctrl+R to rebuild/reload, or q to quit.\n'

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
        printf '\nReload requested from terminal. Rebuilding app-gui...\n\n'
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

# Build app-gui in debug mode.
app-gui-build:
    cargo build --package app-gui

# Build app-gui in release mode.
app-gui-release:
    cargo build --package app-gui --release

# Check app-gui.
app-gui-check:
    cargo check --package app-gui --all-targets

# Test app-gui.
app-gui-test:
    cargo test --package app-gui

alias gui-dev := app-gui-dev
alias gui-build := app-gui-build
alias gui-release := app-gui-release
