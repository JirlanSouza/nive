#!/usr/bin/env bash
# Scaffolding and adoption inside a Cargo workspace the user already owns.
#
# `nive new` inside a workspace produced a project that did not build at all:
# Cargo refuses a package under a workspace root that is not a member of it.
# Only a real `cargo check` proves the registration is what Cargo wanted, so
# this runs one for each of the two doors — `new` and `init`.
set -euo pipefail

mode="${1:-new}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

cargo build --package nive-cli

cli="$root/target/debug/nive"
if [[ ! -x "$cli" && -x "$cli.exe" ]]; then
    cli="$cli.exe"
fi

nive_ver="$(grep '^version = ' "$root/crates/nive/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')"

# A workspace with one existing member, comments, and a non-alphabetical key
# order — the formatting has to survive registration.
mkdir -p "$tmpdir/existing/src"
cat >"$tmpdir/Cargo.toml" <<EOF
# A product that is more than one crate.
[workspace]
resolver = "2"
members = [
  # the crate that was already here
  "existing",
]

[patch.crates-io]
nive = { path = "$root/crates/nive" }
EOF
cat >"$tmpdir/existing/Cargo.toml" <<'EOF'
[package]
name = "existing"
version = "0.1.0"
edition = "2021"
EOF
echo 'pub fn existing() {}' >"$tmpdir/existing/src/lib.rs"

cd "$tmpdir"

case "$mode" in
    new)
        app_name="workspace-app"
        "$cli" new "$app_name"
        ;;
    init)
        # `cargo new` shape, adopted in place.
        app_name="adopted-app"
        mkdir -p "$tmpdir/$app_name/src"
        cat >"$tmpdir/$app_name/Cargo.toml" <<EOF
[package]
name = "$app_name"
version = "0.1.0"
edition = "2021"
EOF
        (cd "$tmpdir/$app_name" && "$cli" init)
        ;;
    *)
        echo "Unknown workspace smoke mode: $mode" >&2
        exit 2
        ;;
esac

app_dir="$tmpdir/$app_name"

if ! grep -q "\"$app_name\"" "$tmpdir/Cargo.toml"; then
    echo "workspace smoke: $app_name was not registered as a workspace member" >&2
    cat "$tmpdir/Cargo.toml" >&2
    exit 1
fi

if ! grep -q "# A product that is more than one crate." "$tmpdir/Cargo.toml"; then
    echo "workspace smoke: registration destroyed the workspace manifest's comments" >&2
    exit 1
fi

if ! grep -q "# the crate that was already here" "$tmpdir/Cargo.toml"; then
    echo "workspace smoke: registration destroyed comments inside the members array" >&2
    exit 1
fi

# Pin nive to the exact workspace version so [patch.crates-io] resolves even
# when the version is a pre-release (e.g. 0.1.0-alpha.1).
if [[ "$(uname)" == "Darwin" ]]; then
    sed -i '' "s/nive = \"[0-9a-z.+-]*\"/nive = \"=$nive_ver\"/" "$app_dir/Cargo.toml"
    sed -i '' "s/nive = { version = \"[0-9a-z.+-]*\"/nive = { version = \"=$nive_ver\"/" "$app_dir/Cargo.toml"
else
    sed -i "s/nive = \"[0-9a-z.+-]*\"/nive = \"=$nive_ver\"/" "$app_dir/Cargo.toml"
    sed -i "s/nive = { version = \"[0-9a-z.+-]*\"/nive = { version = \"=$nive_ver\"/" "$app_dir/Cargo.toml"
fi

(cd "$app_dir" && "$cli" icons check)

# The point of the whole exercise: no manual edit to any manifest.
cargo check --manifest-path "$app_dir/Cargo.toml"
