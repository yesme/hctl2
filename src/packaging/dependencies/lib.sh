#!/usr/bin/env bash
# Source-tree compatibility wrapper for the platform-neutral runtime library.

P0_LIB_WRAPPER_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly P0_LIB_WRAPPER_DIR

case "$(uname -s)" in
    Linux) P0_RUNTIME_PLATFORM="$P0_LIB_WRAPPER_DIR/platforms/linux/runtime.sh" ;;
    Darwin) P0_RUNTIME_PLATFORM="$P0_LIB_WRAPPER_DIR/platforms/macos/runtime.sh" ;;
    *) printf 'error: unsupported runtime platform: %s\n' "$(uname -s)" >&2; return 1 ;;
esac
readonly P0_RUNTIME_PLATFORM

HCTL2_RUNTIME_SOURCE_ROOT="$P0_LIB_WRAPPER_DIR"
HCTL2_RUNTIME_VERSIONS_FILE="$P0_LIB_WRAPPER_DIR/common/versions.sh"
HCTL2_RUNTIME_PLATFORM_FILE="$P0_RUNTIME_PLATFORM"
export HCTL2_RUNTIME_SOURCE_ROOT HCTL2_RUNTIME_VERSIONS_FILE HCTL2_RUNTIME_PLATFORM_FILE
source "$P0_LIB_WRAPPER_DIR/common/runtime.sh"
