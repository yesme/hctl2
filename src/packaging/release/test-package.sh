#!/usr/bin/env bash
# Verify a complete release package, then exercise its offline service lifecycle.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR
DEPENDENCY_ROOT="$(cd -- "$SCRIPT_DIR/../dependencies" && pwd -P)"
readonly DEPENDENCY_ROOT

usage() {
    printf 'usage: test-package.sh RUNTIME_ARCHIVE SOURCE_ARCHIVE\n'
}

if [[ "$#" -ne 2 ]]; then
    usage >&2
    exit 2
fi

ARCHIVE="$(cd -- "$(dirname -- "$1")" && pwd -P)/$(basename -- "$1")"
SOURCE_ARCHIVE="$(cd -- "$(dirname -- "$2")" && pwd -P)/$(basename -- "$2")"
readonly ARCHIVE SOURCE_ARCHIVE

# shellcheck source=../dependencies/common/build.sh
source "$DEPENDENCY_ROOT/common/build.sh"

archive_name="$(basename -- "$ARCHIVE")"
PACKAGE_ID="${archive_name%.tar.gz}"
SOURCE_PACKAGE_ID="$PACKAGE_ID-sources"
case "$PACKAGE_ID" in
    *-linux-x86_64)
        # shellcheck source=../dependencies/targets/linux-x86_64.sh
        source "$DEPENDENCY_ROOT/targets/linux-x86_64.sh"
        ;;
    *-macos-x86_64)
        # shellcheck source=../dependencies/targets/macos-x86_64.sh
        source "$DEPENDENCY_ROOT/targets/macos-x86_64.sh"
        ;;
    *-macos-aarch64)
        # shellcheck source=../dependencies/targets/macos-aarch64.sh
        source "$DEPENDENCY_ROOT/targets/macos-aarch64.sh"
        ;;
    *) die "unsupported release package: $PACKAGE_ID" ;;
esac
readonly PACKAGE_ID SOURCE_PACKAGE_ID

[[ "$(basename -- "$SOURCE_ARCHIVE")" == "$SOURCE_PACKAGE_ID.tar.gz" ]] || \
    die "source package does not match runtime package"

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
source "$DEPENDENCY_ROOT/common/test-package.sh"
test_dependency_package

note "$HCTL2_TARGET_ID complete release passed first-party and service lifecycle tests"
