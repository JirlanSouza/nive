#!/usr/bin/env bash
set -euo pipefail

template="${1:-basic}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

cargo build --package nive-cli

cli="$root/target/debug/nive"
if [[ ! -x "$cli" && -x "$cli.exe" ]]; then
    cli="$cli.exe"
fi

cd "$tmpdir"

case "$template" in
    basic)
        app_name="smoke-basic"
        "$cli" new "$app_name"
        ;;
    dashboard)
        app_name="smoke-dashboard"
        "$cli" new "$app_name" --dashboard
        ;;
    *)
        echo "Unknown scaffold smoke template: $template" >&2
        exit 2
        ;;
esac

app_dir="$tmpdir/$app_name"
cat >> "$app_dir/Cargo.toml" <<EOF

[patch.crates-io]
nive = { path = "$root/crates/nive" }
EOF

cargo check --manifest-path "$app_dir/Cargo.toml"
