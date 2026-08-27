#!/usr/bin/env bash
# Offline installer shipped at the release package root.

set -euo pipefail

PACKAGE_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly PACKAGE_ROOT
readonly PAYLOAD_ROOT="$PACKAGE_ROOT/payload"

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

usage() {
    printf 'usage: ./install.sh [--prefix ABSOLUTE_PATH]\n'
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

verify_manifest() {
    local root="$1"
    local manifest="$2"
    local expected
    local relative
    local actual

    while read -r expected relative; do
        [[ -n "$expected" && -n "$relative" ]] || die "invalid checksum line in $manifest"
        relative="${relative#\*}"
        case "$relative" in
            /* | *"../"* | "..") die "unsafe manifest path: $relative" ;;
        esac
        [[ -f "$root/$relative" ]] || die "manifest file is missing: $relative"
        actual="$(hash_file "$root/$relative")"
        [[ "$actual" == "$expected" ]] || die "checksum mismatch for $relative"
    done <"$root/$manifest"
}

prefix="${HCTL2_INSTALL_PREFIX:-${HOME:?HOME must be set}/.local}"
while (($# > 0)); do
    case "$1" in
        --prefix)
            (($# >= 2)) || die "--prefix requires a path"
            prefix="$2"
            shift 2
            ;;
        --help | -h)
            usage
            exit 0
            ;;
        *) die "unknown argument: $1" ;;
    esac
done

[[ "$prefix" == /* && "$prefix" != "/" ]] || die "install prefix must be an absolute, non-root path"
[[ -f "$PAYLOAD_ROOT/share/hctl2/package-id" ]] || die "package payload is incomplete"

verify_manifest "$PACKAGE_ROOT" MANIFEST.sha256

read -r package_id <"$PAYLOAD_ROOT/share/hctl2/package-id"
[[ "$package_id" =~ ^[a-zA-Z0-9._-]+$ ]] || die "invalid package id"

readonly RELEASES_DIR="$prefix/lib/hctl2"
readonly DESTINATION="$RELEASES_DIR/$package_id"
readonly BIN_DIR="$prefix/bin"

verify_payload() {
    verify_manifest "$1" share/hctl2/PAYLOAD.sha256
}

mkdir -p "$RELEASES_DIR" "$BIN_DIR"
if [[ ! -d "$DESTINATION" ]]; then
    temporary="$(mktemp -d "$RELEASES_DIR/.install.XXXXXX")"
    trap 'rm -rf -- "$temporary"' EXIT
    cp -a "$PAYLOAD_ROOT/." "$temporary/"
    verify_payload "$temporary" || die "installed payload verification failed"
    mv -- "$temporary" "$DESTINATION"
    trap - EXIT
else
    verify_payload "$DESTINATION" || die "existing installation at $DESTINATION is not intact"
    printf 'hctl2: %s is already installed at %s\n' "$package_id" "$DESTINATION"
fi

link_command() {
    local command="$1"
    local command_link="$BIN_DIR/$command"
    local target="../lib/hctl2/$package_id/bin/$command"

    [[ -x "$DESTINATION/bin/$command" ]] || return
    if [[ -e "$command_link" && ! -L "$command_link" ]]; then
        die "refusing to replace non-symlink $command_link"
    fi
    if [[ -L "$command_link" ]]; then
        case "$(readlink "$command_link")" in
            ../lib/hctl2/*/bin/"$command") ;;
            *) die "refusing to replace foreign symlink $command_link" ;;
        esac
    fi
    ln -sfn "$target" "$command_link"
}

for command in hctl2-agentd hctl2-tool hctl2-services; do
    link_command "$command"
done

printf 'hctl2: installed %s at %s\n' "$package_id" "$DESTINATION"
printf 'hctl2: start bundled services with %s/bin/hctl2-services start\n' "$prefix"
