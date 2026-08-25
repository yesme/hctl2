#!/usr/bin/env bash
# Offline installer shipped at the package root.

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

(
    cd "$PACKAGE_ROOT"
    sha256sum --check --quiet MANIFEST.sha256
) || die "package integrity check failed"

read -r package_id <"$PAYLOAD_ROOT/share/hctl2/package-id"
[[ "$package_id" =~ ^[a-zA-Z0-9._-]+$ ]] || die "invalid package id"

readonly RELEASES_DIR="$prefix/lib/hctl2"
readonly DESTINATION="$RELEASES_DIR/$package_id"
readonly BIN_DIR="$prefix/bin"

verify_payload() {
    local root="$1"

    (
        cd "$root"
        sha256sum --check --quiet share/hctl2/PAYLOAD.sha256
    )
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

readonly COMMAND_LINK="$BIN_DIR/hctl2-services"
if [[ -e "$COMMAND_LINK" && ! -L "$COMMAND_LINK" ]]; then
    die "refusing to replace non-symlink $COMMAND_LINK"
fi
if [[ -L "$COMMAND_LINK" ]]; then
    case "$(readlink -- "$COMMAND_LINK")" in
        ../lib/hctl2/*/bin/hctl2-services) ;;
        *) die "refusing to replace foreign symlink $COMMAND_LINK" ;;
    esac
fi
ln -sfn "../lib/hctl2/$package_id/bin/hctl2-services" "$COMMAND_LINK"

printf 'hctl2: installed %s at %s\n' "$package_id" "$DESTINATION"
printf 'hctl2: start bundled services with %s start\n' "$COMMAND_LINK"
