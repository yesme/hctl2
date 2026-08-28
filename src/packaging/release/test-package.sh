#!/usr/bin/env bash
# Verify a complete release package, then exercise its offline service lifecycle.

set -euo pipefail

usage() {
    printf 'usage: test-package.sh RELEASE_OUTPUT_DIRECTORY\n'
}

if [[ "$#" -ne 1 ]]; then
    usage >&2
    exit 2
fi

: "${HCTL2_BUILD_METADATA:?Buck must provide HCTL2_BUILD_METADATA}"
: "${HCTL2_DEPENDENCY_SOURCE_ROOT:?Buck must provide HCTL2_DEPENDENCY_SOURCE_ROOT}"
readonly RELEASE_OUTPUT_DIRECTORY="$1"
[[ -d "$RELEASE_OUTPUT_DIRECTORY" ]] || {
    printf 'release output directory is missing: %s\n' "$RELEASE_OUTPUT_DIRECTORY" >&2
    exit 1
}
[[ -f "$HCTL2_BUILD_METADATA" ]] || {
    printf 'build metadata is missing: %s\n' "$HCTL2_BUILD_METADATA" >&2
    exit 1
}

# shellcheck source=/dev/null
source "$HCTL2_BUILD_METADATA"
# shellcheck source=../dependencies/common/build.sh
source "$HCTL2_DEPENDENCY_SOURCE_ROOT/common/build.sh"

archives=("$RELEASE_OUTPUT_DIRECTORY"/hctl2-*-"$HCTL2_TARGET_ID".tar.gz)
if [[ "${#archives[@]}" -ne 1 || ! -f "${archives[0]}" ]]; then
    die "expected exactly one complete release archive for $HCTL2_TARGET_ID"
fi
ARCHIVE="${archives[0]}"
PACKAGE_ID="$(basename -- "$ARCHIVE" .tar.gz)"
SOURCE_PACKAGE_ID="$PACKAGE_ID-sources"
SOURCE_ARCHIVE="$RELEASE_OUTPUT_DIRECTORY/$SOURCE_PACKAGE_ID.tar.gz"
readonly ARCHIVE PACKAGE_ID SOURCE_ARCHIVE SOURCE_PACKAGE_ID

[[ -f "$SOURCE_ARCHIVE" ]] || die "source package is missing: $SOURCE_ARCHIVE"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-release-contract.XXXXXX")"
case "$test_root" in
    /*/hctl2-release-contract.*) ;;
    *) die "unsafe release test directory: $test_root" ;;
esac
trap 'find "${test_root:?}" -depth -delete' EXIT
tar -xzf "$ARCHIVE" -C "$test_root"
release_root="$test_root/$PACKAGE_ID"

[[ -x "$release_root/payload/bin/hctl2-agentd" ]] || die "release is missing hctl2-agentd"
[[ -x "$release_root/payload/bin/hctl2-tool" ]] || die "release is missing hctl2-tool"
[[ -f "$release_root/payload/share/hctl2/first-party.tsv" ]] || \
    die "release is missing the first-party manifest"
[[ -f "$release_root/payload/share/hctl2/SBOM.spdx" ]] || die "release is missing its SBOM"
grep -F 'SPDXVersion: SPDX-2.3' "$release_root/payload/share/hctl2/SBOM.spdx" >/dev/null
grep -F 'PackageName: hctl2-agentd' "$release_root/payload/share/hctl2/SBOM.spdx" >/dev/null
grep -F 'PackageName: tuwunel' "$release_root/payload/share/hctl2/SBOM.spdx" >/dev/null
"$release_root/payload/bin/hctl2-agentd" --version | grep -F 'hctl2-agentd ' >/dev/null
"$release_root/payload/bin/hctl2-tool" --version | grep -F 'hctl2-tool ' >/dev/null
contract_prefix="$test_root/prefix"
"$release_root/install.sh" --prefix "$contract_prefix"
for command in hctl2-agentd hctl2-tool hctl2-services; do
    [[ -L "$contract_prefix/bin/$command" ]] || die "installer did not link $command"
done
"$contract_prefix/bin/hctl2-agentd" --version | grep -F 'hctl2-agentd ' >/dev/null
"$contract_prefix/bin/hctl2-tool" --version | grep -F 'hctl2-tool ' >/dev/null

find "$test_root" -depth -delete
trap - EXIT

# The dependency lifecycle verifier also checks both archive sidecars, all
# manifests, licenses/source correspondence, idempotent installation, browser
# assets, service health, byte ranges, and clean shutdown. Running it against
# the assembled archive proves that the final user package preserves that
# contract rather than merely testing the intermediate dependency package.
# shellcheck source=../dependencies/common/test-package.sh
source "$HCTL2_DEPENDENCY_SOURCE_ROOT/common/test-package.sh"
test_dependency_package

note "$HCTL2_TARGET_ID complete release passed first-party and service lifecycle tests"
