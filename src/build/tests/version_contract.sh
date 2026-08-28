#!/usr/bin/env bash
set -euo pipefail

expected="$1"
cargo_toml="$2"
actual="$(awk '
    $0 == "[workspace.package]" { in_package = 1; next }
    in_package && $1 == "version" { gsub(/"/, "", $3); print $3; exit }
' "$cargo_toml")"

[[ -n "$actual" ]] || {
    printf 'could not read workspace package version from %s\n' "$cargo_toml" >&2
    exit 1
}
[[ "$actual" == "$expected" ]] || {
    printf 'Cargo workspace version %s does not match Buck version %s\n' \
        "$actual" "$expected" >&2
    exit 1
}
