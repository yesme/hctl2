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
