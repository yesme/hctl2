#!/usr/bin/env bash
# Build the pinned zstd CLI from the source archive Buck unpacked into the action.
#
# The tool is build-only: it compresses the release archives on the build host
# and never ships to users. Building from source (rather than pinning a
# prebuilt binary) keeps one upstream artifact — the official release tarball
# and its published SHA-256 — and avoids the @rpath dependency chain the pkgx
# zstd carries (zlib, liblzma, liblz4). Costs a C compiler on the build host.
set -euo pipefail

: "${SRCDIR:?Buck must provide the action sources}"
: "${OUT:?Buck must provide the action output}"
: "${TMP:?Buck must provide the action scratch directory}"

die() {
    echo "zstd-build: $*" >&2
    exit 1
}

command -v make >/dev/null 2>&1 || die "make is required to build the pinned zstd"
command -v cc >/dev/null 2>&1 || die "a C compiler is required to build the pinned zstd"

source_root="$PWD/$SRCDIR/src"
output_root="$PWD/$OUT"
work="$TMP/zstd-build-source"

[[ -f "$source_root/Makefile" ]] || die "zstd source tree not found at $source_root"

# Buck materializes inputs read-only; build a copy under $TMP.
rm -rf "$work"
mkdir -p "$work" "$output_root/bin"
cp -a "$source_root/." "$work/"

jobs="$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 2)"
build_log="$work/build.log"
if ! make -C "$work" -j"$jobs" zstd >"$build_log" 2>&1; then
    cat "$build_log" >&2
    die "make failed to build zstd"
fi
# Without pthread the build silently drops multithreading; the version string
# stays the same but the compressed bytes change, so fail loudly instead.
if grep -F 'without multithreading support' "$build_log" >/dev/null; then
    cat "$build_log" >&2
    die "the pinned zstd built without multithreading support"
fi

install -m 0755 "$work/programs/zstd" "$output_root/bin/zstd"
install -m 0644 "$work/LICENSE" "$output_root/LICENSE"
install -m 0644 "$work/COPYING" "$output_root/COPYING"

"$output_root/bin/zstd" --version
