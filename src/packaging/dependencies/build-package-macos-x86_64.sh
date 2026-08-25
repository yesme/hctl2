#!/usr/bin/env bash
# Build the macOS Intel offline dependency package.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR
source "$SCRIPT_DIR/common/build.sh"
source "$SCRIPT_DIR/targets/macos-x86_64.sh"
source "$SCRIPT_DIR/platforms/macos/bootstrap.sh"
source "$SCRIPT_DIR/platforms/macos/package.sh"
source "$SCRIPT_DIR/common/package.sh"

init_build_environment
assemble_dependency_package
