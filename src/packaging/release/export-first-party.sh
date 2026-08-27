#!/usr/bin/env bash
# Assemble the configured Buck2 first-party outputs and their content manifest.

set -euo pipefail

if [[ "$#" -ne 5 ]]; then
    printf 'usage: export-first-party.sh OUTPUT_DIR VERSION TARGET AGENTD TOOL\n' >&2
    exit 2
fi

readonly OUTPUT_DIR="$1"
readonly HCTL2_VERSION="$2"
readonly HCTL2_TARGET="$3"
readonly AGENTD="$4"
readonly TOOL="$5"

hash_file() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

mkdir -p "$OUTPUT_DIR/bin"
install -m 0755 "$AGENTD" "$OUTPUT_DIR/bin/hctl2-agentd"
install -m 0755 "$TOOL" "$OUTPUT_DIR/bin/hctl2-tool"

case "$HCTL2_TARGET" in
    macos-*)
        command -v codesign >/dev/null 2>&1 || {
            printf 'error: codesign is required for macOS first-party exports\n' >&2
            exit 1
        }
        codesign --force --sign - --timestamp=none "$OUTPUT_DIR/bin/hctl2-agentd"
        codesign --force --sign - --timestamp=none "$OUTPUT_DIR/bin/hctl2-tool"
        ;;
esac

{
    printf 'target\n'
    printf '%s\n' "$HCTL2_TARGET"
} >"$OUTPUT_DIR/target.tsv"

{
    printf 'component\tversion\ttarget\tbinary_sha256\n'
    printf 'hctl2-agentd\t%s\t%s\t%s\n' \
        "$HCTL2_VERSION" "$HCTL2_TARGET" \
        "$(hash_file "$OUTPUT_DIR/bin/hctl2-agentd")"
    printf 'hctl2-tool\t%s\t%s\t%s\n' \
        "$HCTL2_VERSION" "$HCTL2_TARGET" \
        "$(hash_file "$OUTPUT_DIR/bin/hctl2-tool")"
} >"$OUTPUT_DIR/binaries.tsv"
