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

archives=("$RELEASE_OUTPUT_DIRECTORY"/hctl2-*-"$HCTL2_TARGET_ID".tar.xz)
if [[ "${#archives[@]}" -ne 1 || ! -f "${archives[0]}" ]]; then
    die "expected exactly one complete release archive for $HCTL2_TARGET_ID"
fi
ARCHIVE="${archives[0]}"
PACKAGE_ID="$(basename -- "$ARCHIVE" .tar.xz)"
SOURCE_PACKAGE_ID="$PACKAGE_ID-sources"
SOURCE_ARCHIVE="$RELEASE_OUTPUT_DIRECTORY/$SOURCE_PACKAGE_ID.tar.xz"
readonly ARCHIVE PACKAGE_ID SOURCE_ARCHIVE SOURCE_PACKAGE_ID

[[ -f "$SOURCE_ARCHIVE" ]] || die "source package is missing: $SOURCE_ARCHIVE"

test_root="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-release-contract.XXXXXX")"
case "$test_root" in
    /*/hctl2-release-contract.*) ;;
    *) die "unsafe release test directory: $test_root" ;;
esac
trap 'find "${test_root:?}" -depth -delete' EXIT
tar -xJf "$ARCHIVE" -C "$test_root"
release_root="$test_root/$PACKAGE_ID"

[[ -x "$release_root/payload/bin/hctl2-tool" ]] || die "release is missing hctl2-tool"
[[ -x "$release_root/payload/bin/hctl2" ]] || die "release is missing hctl2"
[[ -x "$release_root/payload/bin/hctl2-control" ]] || die "release is missing hctl2-control"
[[ -x "$release_root/payload/libexec/hctl2/herdr" ]] || die "release is missing Herdr"
[[ -x "$release_root/payload/libexec/hctl2/process-compose" ]] || \
    die "release is missing Process Compose"
[[ -x "$release_root/payload/libexec/hctl2/gitea" ]] || die "release is missing Gitea"
[[ -x "$release_root/payload/libexec/hctl2/tea" ]] || die "release is missing tea"
[[ -x "$release_root/payload/libexec/hctl2/gh" ]] || die "release is missing GitHub CLI"
[[ ! -e "$release_root/payload/bin/hctl2-agentd" ]] || die "release still contains hctl2-agentd"
[[ -f "$release_root/payload/share/hctl2/first-party.tsv" ]] || \
    die "release is missing the first-party manifest"
[[ -f "$release_root/payload/share/hctl2/SBOM.spdx" ]] || die "release is missing its SBOM"
[[ -f "$release_root/payload/share/hctl2/agency/skills/hctl2-shaping/SKILL.md" ]] || \
    die "release is missing the local Agency skills"
[[ -f "$release_root/payload/share/hctl2/agency/skills/hctl2-shaping/LICENSE-mattpocock-skills" ]] || \
    die "release is missing the skill license notice"
grep -F "  share/hctl2/agency/skills/hctl2-shaping/SKILL.md" "$release_root/payload/share/hctl2/PAYLOAD.sha256" >/dev/null || \
    die "payload manifest does not cover the local Agency skills"
grep -F 'SPDXVersion: SPDX-2.3' "$release_root/payload/share/hctl2/SBOM.spdx" >/dev/null
grep -F 'Creator: Tool: syft-1.51.1' "$release_root/payload/share/hctl2/SBOM.spdx" >/dev/null
grep -F "PackageName: $PACKAGE_ID" "$release_root/payload/share/hctl2/SBOM.spdx" >/dev/null
grep -F 'FileName: libexec/hctl2/herdr' "$release_root/payload/share/hctl2/SBOM.spdx" >/dev/null
grep -F 'FileName: libexec/hctl2/tuwunel' "$release_root/payload/share/hctl2/SBOM.spdx" >/dev/null
grep -F 'FileName: libexec/hctl2/process-compose' "$release_root/payload/share/hctl2/SBOM.spdx" >/dev/null
grep -F 'FileName: libexec/hctl2/gh' "$release_root/payload/share/hctl2/SBOM.spdx" >/dev/null
"$release_root/payload/libexec/hctl2/herdr" --version | grep -F 'herdr ' >/dev/null
"$release_root/payload/libexec/hctl2/process-compose" version | \
    grep -F 'v1.122.0' >/dev/null
"$release_root/payload/libexec/hctl2/gh" --version | grep -F 'gh version 2.99.0' >/dev/null
"$release_root/payload/bin/hctl2-tool" --version | grep -F 'hctl2-tool ' >/dev/null
contract_prefix="$test_root/prefix"
"$release_root/install.sh" --prefix "$contract_prefix"
for command in hctl2-tool hctl2 hctl2-control hctl2-services; do
    [[ -L "$contract_prefix/bin/$command" ]] || die "installer did not link $command"
done
"$contract_prefix/bin/hctl2-tool" --version | grep -F 'hctl2-tool ' >/dev/null

: "${HCTL2_TOOLBOX_TEST:?Buck must provide HCTL2_TOOLBOX_TEST}"
# shellcheck source=test-toolbox.sh
source "$HCTL2_TOOLBOX_TEST"
test_packaged_toolbox "$contract_prefix/bin/hctl2-tool"

# B0: control + consumed services restart without changing storage identity.
# Must run before $test_root is deleted; the installer prefix lives under it.
b0_root="$test_root/b0-control"
"$contract_prefix/bin/hctl2" --json --root "$b0_root" init >/dev/null
"$contract_prefix/bin/hctl2" --json --root "$b0_root" start >/dev/null
b0_status="$("$contract_prefix/bin/hctl2" --json --root "$b0_root" status)"
printf '%s\n' "$b0_status" | grep -F '"ready":true' >/dev/null || \
    grep -F '"ready": true' <<<"$b0_status" >/dev/null || \
    die "hctl2 start did not report a ready control: $b0_status"
b0_id="$(printf '%s\n' "$b0_status" | sed -n 's/.*"control_id":"\([^"]*\)".*/\1/p')"
[[ -n "$b0_id" ]] || die "could not read control_id from status: $b0_status"
wait_consumed_available() {
    local name="$1"
    local attempt
    local snapshot=""
    local needle
    needle="\"available\":true,\"consumed\":true,\"name\":\"${name}\""
    for attempt in $(seq 1 60); do
        snapshot="$("$contract_prefix/bin/hctl2" --json --root "$b0_root" services status 2>/dev/null || true)"
        if printf '%s\n' "$snapshot" | grep -F "$needle" >/dev/null; then
            note "B0 $name available"
            return 0
        fi
        sleep 1
    done
    die "hctl2 start did not bring $name to available: $snapshot"
}
wait_consumed_available tuwunel
# Gitea is hosted but only consumed by the first purely local repository (or an
# explicit local-platform choice); a plain start must leave it down.
b0_services="$("$contract_prefix/bin/hctl2" --json --root "$b0_root" services status)"
printf '%s\n' "$b0_services" | grep -F '"available":false,"consumed":false,"name":"gitea"' >/dev/null || \
    die "hctl2 start brought up Gitea before any consumption: $b0_services"
printf '%s\n' "$b0_services" | grep -F '"name":"gitea","pid":null,"ready":false,"running":false' >/dev/null || \
    die "hctl2 start left an unconsumed Gitea process running: $b0_services"
"$contract_prefix/bin/hctl2" --json --root "$b0_root" services consume gitea >/dev/null
wait_consumed_available gitea
"$contract_prefix/bin/hctl2" --json --root "$b0_root" stop >/dev/null || true
sleep 2
if HCTL2_INSTALL_ROOT="$contract_prefix/lib/hctl2/$PACKAGE_ID" \
    HCTL2_STATE_ROOT="$b0_root/services" \
    "$contract_prefix/bin/hctl2-services" status >/dev/null 2>&1
then
    die "hctl2-services status succeeded after hctl2 stop"
fi
"$contract_prefix/bin/hctl2" --json --root "$b0_root" start >/dev/null
# Consumption is remembered: the restart brings Gitea back without a new consume.
wait_consumed_available tuwunel
wait_consumed_available gitea
b0_again="$("$contract_prefix/bin/hctl2" --json --root "$b0_root" status)"
b0_id2="$(printf '%s\n' "$b0_again" | sed -n 's/.*"control_id":"\([^"]*\)".*/\1/p')"
[[ "$b0_id" == "$b0_id2" ]] || die "restart changed control identity: $b0_id -> $b0_id2"
"$contract_prefix/bin/hctl2" --json --root "$b0_root" stop >/dev/null || true
sleep 2

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
