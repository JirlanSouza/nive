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

snapshot="$tmpdir/nive-git-snapshot"
file_list="$tmpdir/nive-files.list"
mkdir -p "$snapshot"
(
    cd "$root"
    while IFS= read -r -d '' path; do
        if [[ -e "$path" ]]; then
            printf '%s\0' "$path"
        fi
    done < <(git ls-files -co --exclude-standard -z) > "$file_list"
    tar --null -T "$file_list" -cf -
) | tar -C "$snapshot" -xf -

git -C "$snapshot" init --quiet
git -C "$snapshot" add .
git -C "$snapshot" \
    -c user.name="Nive Scaffold Smoke" \
    -c user.email="nive-smoke@example.invalid" \
    commit --quiet -m "snapshot"

git_url="file://$snapshot"
rev="$(git -C "$snapshot" rev-parse HEAD)"

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

app_dir="$tmpdir/$app_name"

(cd "$app_dir" && "$cli" icons check)
cargo check --manifest-path "$app_dir/Cargo.toml"
