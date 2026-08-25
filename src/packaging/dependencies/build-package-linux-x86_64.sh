#!/usr/bin/env bash
# Build the Linux x86_64 offline dependency package.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR
source "$SCRIPT_DIR/common/build.sh"
source "$SCRIPT_DIR/targets/linux-x86_64.sh"
source "$SCRIPT_DIR/platforms/linux/bootstrap.sh"
source "$SCRIPT_DIR/platforms/linux/package.sh"
source "$SCRIPT_DIR/common/package.sh"

init_build_environment
assemble_dependency_package
