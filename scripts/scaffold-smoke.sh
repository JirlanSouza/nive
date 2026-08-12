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

(cd "$app_dir" && "$cli" icons sync)
first_generated="$tmpdir/first-generated-$template"
mkdir -p "$first_generated"
cp "$app_dir/src/icons.rs" "$first_generated/icons.rs"
cp -R "$app_dir/src/icons" "$first_generated/icons"

(cd "$app_dir" && "$cli" icons sync)
diff -ru "$first_generated/icons.rs" "$app_dir/src/icons.rs"
diff -ru "$first_generated/icons" "$app_dir/src/icons"

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

if [[ "$template" == "dashboard" ]]; then
    main_rs="$app_dir/src/main.rs"

    # Legacy exhaustive DialogDismiss variants and direct DialogRequest field
    # construction: both were removed from the public API (breaking change),
    # so a real regression here would already fail `cargo check` below with a
    # private-field/missing-variant error, but a grep failure gives a clear,
    # specific message about which compatibility guarantee broke.
    if grep -qE 'DialogDismiss::(OnBackdrop|OnEscape|None)\b' "$main_rs"; then
        echo "scaffold smoke: dashboard template regressed to a legacy DialogDismiss enum variant" >&2
        exit 1
    fi
    if grep -qE 'DialogRequest[[:space:]]*\{' "$main_rs"; then
        echo "scaffold smoke: dashboard template regressed to direct DialogRequest field construction" >&2
        exit 1
    fi
    if grep -q 'backdrop_alpha(' "$main_rs"; then
        echo "scaffold smoke: dashboard template regressed to raw backdrop_alpha" >&2
        exit 1
    fi

    # Raw geometry, nested Panel chrome, and caller-owned scrolling on the
    # Dialog builder chain itself: these compile fine (Dialog's body accepts
    # any Element and has no width/height builder of its own to reject the
    # call), so only a source check catches the anatomy regression.
    dialog_chain="$(tr '\n' ' ' <"$main_rs" | grep -oE 'Dialog::new\([^;]*;' || true)"
    if [[ -z "$dialog_chain" ]]; then
        echo "scaffold smoke: dashboard template no longer builds a Dialog" >&2
        exit 1
    fi
    if grep -qE '\.(width|height)\(' <<<"$dialog_chain"; then
        echo "scaffold smoke: dashboard template regressed to raw Dialog geometry (.width/.height)" >&2
        exit 1
    fi
    if grep -q 'Panel::new(' <<<"$dialog_chain"; then
        echo "scaffold smoke: dashboard template regressed to nested Panel chrome inside Dialog" >&2
        exit 1
    fi
    if grep -q 'scrollable(' <<<"$dialog_chain"; then
        echo "scaffold smoke: dashboard template regressed to a caller-owned Dialog body Scrollable" >&2
        exit 1
    fi
fi

(cd "$app_dir" && "$cli" icons check)
cargo fmt --manifest-path "$app_dir/Cargo.toml" -- --check
cargo check --manifest-path "$app_dir/Cargo.toml"
