#!/usr/bin/env bash
# Bootstrap the Linux x86_64 dependency build cache.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR
source "$SCRIPT_DIR/common/build.sh"
source "$SCRIPT_DIR/targets/linux-x86_64.sh"
source "$SCRIPT_DIR/platforms/linux/bootstrap.sh"

init_build_environment
bootstrap_dependencies
