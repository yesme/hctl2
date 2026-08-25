#!/usr/bin/env bash
# Compatibility view of the common locks plus the current native target.

P0_VERSIONS_WRAPPER_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly P0_VERSIONS_WRAPPER_DIR
source "$P0_VERSIONS_WRAPPER_DIR/common/versions.sh"

case "$(uname -s):$(uname -m)" in
    Linux:x86_64) source "$P0_VERSIONS_WRAPPER_DIR/targets/linux-x86_64.sh" ;;
    Darwin:arm64) source "$P0_VERSIONS_WRAPPER_DIR/targets/macos-aarch64.sh" ;;
    Darwin:x86_64) source "$P0_VERSIONS_WRAPPER_DIR/targets/macos-x86_64.sh" ;;
    *) printf 'error: unsupported native target: %s %s\n' "$(uname -s)" "$(uname -m)" >&2; return 1 ;;
esac
