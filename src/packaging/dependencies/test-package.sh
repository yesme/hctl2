#!/usr/bin/env bash
# Verify one Buck-built dependency package and exercise its offline lifecycle.

set -euo pipefail

usage() {
    printf 'usage: test-package.sh PACKAGE_OUTPUT_DIRECTORY\n'
}

if [[ "$#" -ne 1 ]]; then
    usage >&2
    exit 2
fi

readonly PACKAGE_OUTPUT_DIRECTORY="$1"
: "${HCTL2_BUILD_METADATA:?Buck must provide HCTL2_BUILD_METADATA}"
: "${HCTL2_DEPENDENCY_SOURCE_ROOT:?Buck must provide HCTL2_DEPENDENCY_SOURCE_ROOT}"
[[ -d "$PACKAGE_OUTPUT_DIRECTORY" ]] || {
    printf 'package output directory is missing: %s\n' "$PACKAGE_OUTPUT_DIRECTORY" >&2
    exit 1
}
[[ -f "$HCTL2_BUILD_METADATA" ]] || {
    printf 'build metadata is missing: %s\n' "$HCTL2_BUILD_METADATA" >&2
    exit 1
}

# shellcheck source=/dev/null
source "$HCTL2_BUILD_METADATA"
# shellcheck source=common/build.sh
source "$HCTL2_DEPENDENCY_SOURCE_ROOT/common/build.sh"

archives=("$PACKAGE_OUTPUT_DIRECTORY"/hctl2-*-"$HCTL2_TARGET_ID".tar.gz)
if [[ "${#archives[@]}" -ne 1 || ! -f "${archives[0]}" ]]; then
    die "expected exactly one runtime archive for $HCTL2_TARGET_ID"
fi
ARCHIVE="${archives[0]}"
PACKAGE_ID="$(basename -- "$ARCHIVE" .tar.gz)"
SOURCE_PACKAGE_ID="$PACKAGE_ID-sources"
SOURCE_ARCHIVE="$PACKAGE_OUTPUT_DIRECTORY/$SOURCE_PACKAGE_ID.tar.gz"
readonly ARCHIVE PACKAGE_ID SOURCE_ARCHIVE SOURCE_PACKAGE_ID

[[ -f "$SOURCE_ARCHIVE" ]] || die "source archive is missing: $SOURCE_ARCHIVE"

# shellcheck source=common/test-package.sh
source "$HCTL2_DEPENDENCY_SOURCE_ROOT/common/test-package.sh"
test_dependency_package

note "$HCTL2_TARGET_ID dependency package passed offline lifecycle tests"
