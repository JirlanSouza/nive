#!/usr/bin/env bash
set -euo pipefail

# Smoke-checks a generated app compiled against the local repo via a Git
# dependency (file:// URL + exact HEAD rev). This validates the same dependency
# shape that real apps use with GitHub alpha tags, without requiring a pushed tag.

template="${1:-basic}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

cargo build --package nive-cli

cli="$root/target/debug/nive"
if [[ ! -x "$cli" && -x "$cli.exe" ]]; then
    cli="$cli.exe"
fi

git_url="file://$root"
rev="$(git -C "$root" rev-parse HEAD)"

cd "$tmpdir"

case "$template" in
    basic)
        app_name="smoke-github-basic"
        "$cli" new "$app_name" --git "$git_url" --rev "$rev"
        ;;
    dashboard)
        app_name="smoke-github-dashboard"
        "$cli" new "$app_name" --dashboard --git "$git_url" --rev "$rev"
        ;;
    *)
        echo "Unknown template: $template (expected basic or dashboard)" >&2
        exit 2
        ;;
esac

cargo check --manifest-path "$tmpdir/$app_name/Cargo.toml"
