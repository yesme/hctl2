#!/usr/bin/env bash
# Compatibility dispatcher for the current native release target test.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR

case "$(uname -s):$(uname -m)" in
    Linux:x86_64) exec "$SCRIPT_DIR/test-package-linux-x86_64.sh" "$@" ;;
    Darwin:arm64) exec "$SCRIPT_DIR/test-package-macos-aarch64.sh" "$@" ;;
    Darwin:x86_64) exec "$SCRIPT_DIR/test-package-macos-x86_64.sh" "$@" ;;
    *) printf 'error: unsupported native test host: %s %s\n' "$(uname -s)" "$(uname -m)" >&2; exit 1 ;;
esac
