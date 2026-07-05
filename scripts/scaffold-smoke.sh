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

# Pin nive to the exact workspace version so [patch.crates-io] resolves
# correctly even when the workspace version is a pre-release (e.g. 0.1.0-alpha.1).
nive_ver="$(grep '^version = ' "$root/crates/nive/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')"
if [[ "$(uname)" == "Darwin" ]]; then
    sed -i '' "s/nive = \"[0-9a-z.+-]*\"/nive = \"=$nive_ver\"/" "$app_dir/Cargo.toml"
    sed -i '' "s/nive = { version = \"[0-9a-z.+-]*\"/nive = { version = \"=$nive_ver\"/" "$app_dir/Cargo.toml"
else
    sed -i "s/nive = \"[0-9a-z.+-]*\"/nive = \"=$nive_ver\"/" "$app_dir/Cargo.toml"
    sed -i "s/nive = { version = \"[0-9a-z.+-]*\"/nive = { version = \"=$nive_ver\"/" "$app_dir/Cargo.toml"
fi

cat >> "$app_dir/Cargo.toml" <<EOF

[patch.crates-io]
nive = { path = "$root/crates/nive" }
EOF

(cd "$app_dir" && "$cli" icons check)
cargo check --manifest-path "$app_dir/Cargo.toml"
