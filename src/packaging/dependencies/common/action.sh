#!/usr/bin/env bash
# Minimal helpers shared by isolated external-component actions.

set -euo pipefail

P0_COMMON_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly P0_COMMON_DIR
P0_DEPENDENCY_SOURCE_ROOT="$(cd -- "$P0_COMMON_DIR/.." && pwd -P)"
readonly P0_DEPENDENCY_SOURCE_ROOT

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

note() {
    printf 'hctl2: %s\n' "$*"
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

run_xz() {
    : "${HCTL2_XZ_ROOT:?Buck must declare the xz tool input}"
    # pkgx's Linux RPATH points to a build-time directory. Scope its bundled
    # liblzma to this process; never fall back to PATH's xz or inherit options.
    LD_LIBRARY_PATH="$HCTL2_XZ_ROOT/lib" XZ_DEFAULTS= XZ_OPT= \
        "$HCTL2_XZ_ROOT/bin/xz" "$@"
}

require_pinned_xz() {
    local actual
    local expected
    : "${HCTL2_XZ_VERSION:?Buck must provide the pinned xz version}"
    actual="$(LC_ALL=C run_xz --version)" || die "could not run pinned xz"
    expected="$(printf 'xz (XZ Utils) %s\nliblzma %s' "$HCTL2_XZ_VERSION" "$HCTL2_XZ_VERSION")"
    [[ "$actual" == "$expected" ]] || die "xz/liblzma version mismatch: $actual"
}

# The pinned zstd CLI is build-only: it writes the release archives and never
# ships (docs/research/build-tools/zstd.md). The pinned xz above stays for
# unpacking Gitea's upstream .xz download.
run_zstd() {
    : "${HCTL2_ZSTD_ROOT:?Buck must declare the zstd tool input}"
    # ZSTD_CLEVEL and ZSTD_NBTHREADS let the caller's environment decide what
    # the tool produces; unset them so only the declared flags count.
    env -u ZSTD_CLEVEL -u ZSTD_NBTHREADS "$HCTL2_ZSTD_ROOT/bin/zstd" "$@"
}

require_pinned_zstd() {
    local actual
    local expected
    : "${HCTL2_ZSTD_VERSION:?Buck must provide the pinned zstd version}"
    actual="$(LC_ALL=C run_zstd --version)" || die "could not run pinned zstd"
    expected="*** Zstandard CLI (64-bit) v$HCTL2_ZSTD_VERSION, by Yann Collet ***"
    [[ "$actual" == "$expected" ]] || die "zstd version mismatch: $actual"
}

# HCTL2_ZSTD_PRESET selects the compression preset. `release` (--ultra -22) is
# the shipped artifact: compression happens once per release while every install
# decompresses, so the archive buys decode time with encode time and size — the
# measured trade-off is in docs/research/build-tools/zstd.md. `fast` (-12) is
# only for pull-request verification builds, whose archives are installed and
# exercised but never published.
zstd_preset_flags() {
    # Unset means release; an explicit empty value is a mistake, not a default.
    case "${HCTL2_ZSTD_PRESET-release}" in
        release) printf '%s\n' --ultra -22 --long=27 ;;
        fast) printf '%s\n' -12 ;;
        *) die "unsupported HCTL2_ZSTD_PRESET: '${HCTL2_ZSTD_PRESET-}'" ;;
    esac
}

compress_archive() {
    local preset
    local flags=()
    # A bad preset must fail here, not fall through to zstd's default level: the
    # command substitution's status is returned explicitly because errexit stays
    # off when a caller uses this function inside a condition.
    preset="$(zstd_preset_flags)" || return 1
    # The preset is a fixed, whitespace-separated flag list: split it on purpose.
    # shellcheck disable=SC2206
    flags=($preset)
    run_zstd "${flags[@]}" -T0 -c
}

hash_file() {
    local path="$1"

    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$path" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$path" | awk '{print $1}'
    else
        die "required SHA-256 tool not found: sha256sum or shasum"
    fi
}

verify_sha256() {
    local path="$1"
    local expected="$2"
    local actual

    actual="$(hash_file "$path")"
    [[ "$actual" == "$expected" ]] || \
        die "checksum mismatch for $path: expected $expected, got $actual"
}

init_build_environment() {
    [[ -n "${HCTL2_TARGET_ID:-}" ]] || die "Buck-generated target metadata must be loaded first"
    [[ -n "${HCTL2_BUILD_CACHE:-}" ]] || die "HCTL2_BUILD_CACHE must be set by the Buck action"
    P0_ROOT="$HCTL2_BUILD_CACHE/$HCTL2_TARGET_ID"
    [[ "$P0_ROOT" == /* && "$P0_ROOT" != "/" ]] || \
        die "HCTL2_BUILD_CACHE must be an absolute, non-root path"

    P0_BIN_DIR="$P0_ROOT/bin"
    P0_TUWUNEL_ROOT="${HCTL2_TUWUNEL_CACHE:-$HCTL2_BUILD_CACHE}/$HCTL2_TARGET_ID"
    [[ "$P0_TUWUNEL_ROOT" == /* && "$P0_TUWUNEL_ROOT" != "/" ]] || \
        die "HCTL2_TUWUNEL_CACHE must be an absolute, non-root path"
    P0_TUWUNEL_BIN_DIR="$P0_TUWUNEL_ROOT/bin"
    P0_TUWUNEL_LIBRARY_DIR="$P0_TUWUNEL_ROOT/lib/tuwunel"
    P0_TUWUNEL_MANIFEST_DIR="$P0_TUWUNEL_ROOT/manifests"
    P0_DOWNLOAD_DIR="${HCTL2_DOWNLOAD_ROOT:-$P0_ROOT/downloads}"
    [[ "$P0_DOWNLOAD_DIR" == /* && -d "$P0_DOWNLOAD_DIR" ]] || \
        die "Buck did not provide an existing absolute download directory"
    P0_MANIFEST_DIR="$P0_ROOT/manifests"
    P0_TMP_DIR="$P0_ROOT/tmp"
    P0_VENDOR_DIR="$P0_ROOT/vendor"
    readonly P0_ROOT P0_BIN_DIR P0_TUWUNEL_ROOT P0_TUWUNEL_BIN_DIR
    readonly P0_TUWUNEL_LIBRARY_DIR P0_TUWUNEL_MANIFEST_DIR P0_DOWNLOAD_DIR
    readonly P0_MANIFEST_DIR P0_TMP_DIR P0_VENDOR_DIR

    mkdir -p "$P0_BIN_DIR" "$P0_DOWNLOAD_DIR" "$P0_MANIFEST_DIR" "$P0_TMP_DIR" "$P0_VENDOR_DIR"
}

require_target_host() {
    local actual_system
    local actual_machine

    actual_system="$(uname -s)"
    actual_machine="$(uname -m)"
    [[ "$actual_system" == "$HCTL2_TARGET_UNAME_S" ]] || \
        die "$HCTL2_TARGET_ID must be built on $HCTL2_TARGET_UNAME_S, not $actual_system"
    [[ "$actual_machine" == "$HCTL2_TARGET_UNAME_M" ]] || \
        die "$HCTL2_TARGET_ID must be built natively on $HCTL2_TARGET_UNAME_M, not $actual_machine"
}

prepare_source_tree() {
    local archive="$1"
    local expected="$2"
    local destination="$3"

    [[ -n "$expected" ]] || die "Buck source archive digest is missing for $archive"

    case "$destination" in
        "$P0_VENDOR_DIR"/*) ;;
        *) die "refusing to replace source directory outside the build cache: $destination" ;;
    esac
    mkdir -p "$destination"
    tar -xf "$archive" -C "$destination" --strip-components=1
}
