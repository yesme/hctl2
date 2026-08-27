#!/usr/bin/env bash
# Verify first-party and external subsystem contracts, then assemble one release.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR
PRODUCT_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd -P)"
readonly PRODUCT_ROOT

# shellcheck source=../dependencies/common/build.sh
source "$SCRIPT_DIR/../dependencies/common/build.sh"

usage() {
    printf '%s\n' \
        'usage: assemble.sh --first-party DIR --dependencies ARCHIVE --sources ARCHIVE --output DIR'
}

verify_archive_sidecar() {
    local archive="$1"
    local sidecar="$archive.sha256"
    local expected
    local recorded_name

    [[ -f "$sidecar" ]] || die "archive checksum sidecar is missing: $sidecar"
    read -r expected recorded_name <"$sidecar"
    recorded_name="${recorded_name#\*}"
    [[ "$expected" =~ ^[0-9a-f]{64}$ ]] || die "invalid checksum in $sidecar"
    [[ "$recorded_name" == "$(basename -- "$archive")" ]] || \
        die "checksum sidecar names the wrong archive: $recorded_name"
    verify_sha256 "$archive" "$expected"
}

safe_relative_path() {
    case "$1" in
        "" | /* | ".." | ../* | */../* | */..) return 1 ;;
        *) return 0 ;;
    esac
}

verify_tree_manifest() {
    local root="$1"
    local manifest="$2"
    local expected
    local relative

    [[ -f "$root/$manifest" ]] || die "manifest is missing: $manifest"
    while read -r expected relative; do
        relative="${relative#\*}"
        [[ "$expected" =~ ^[0-9a-f]{64}$ ]] || die "invalid checksum in $manifest"
        safe_relative_path "$relative" || die "unsafe manifest path: $relative"
        [[ -f "$root/$relative" ]] || die "manifest file is missing: $relative"
        verify_sha256 "$root/$relative" "$expected"
    done <"$root/$manifest"
}

write_checksum_manifest() {
    local root="$1"
    local manifest="$2"
    shift 2

    (
        cd "$root"
        find "$@" -type f ! -path "$manifest" -print | LC_ALL=C sort | while IFS= read -r path; do
            printf '%s  %s\n' "$(hash_file "$path")" "$path"
        done >"$manifest"
    )
}

validate_archive_layout() {
    local archive="$1"
    local expected_root="$2"
    local entry

    while IFS= read -r entry; do
        entry="${entry%/}"
        [[ -n "$entry" ]] || continue
        safe_relative_path "$entry" || die "unsafe archive path in $archive: $entry"
        case "$entry" in
            "$expected_root" | "$expected_root"/*) ;;
            *) die "archive entry is outside $expected_root: $entry" ;;
        esac
    done < <(tar -tzf "$archive")
}

spdx_license_for() {
    case "$1" in
        hctl2 | hctl2-agentd | hctl2-tool | tuwunel) printf 'Apache-2.0\n' ;;
        vikunja | cinny) printf 'AGPL-3.0-only\n' ;;
        dagu) printf 'GPL-3.0-only\n' ;;
        tmux) printf 'ISC\n' ;;
        static-web-server) printf '(Apache-2.0 OR MIT)\n' ;;
        *) printf 'NOASSERTION\n' ;;
    esac
}

spdx_id_for() {
    printf '%s' "$1" | tr -c 'A-Za-z0-9.-' '-'
}

write_spdx_package() {
    local output="$1"
    local component="$2"
    local version="$3"
    local checksum="$4"
    local spdx_id

    spdx_id="$(spdx_id_for "$component")"
    {
        printf '\nPackageName: %s\n' "$component"
        printf 'SPDXID: SPDXRef-Package-%s\n' "$spdx_id"
        printf 'PackageVersion: %s\n' "$version"
        printf 'PackageDownloadLocation: NOASSERTION\n'
        printf 'FilesAnalyzed: false\n'
        [[ -z "$checksum" ]] || printf 'PackageChecksum: SHA256: %s\n' "$checksum"
        printf 'PackageLicenseConcluded: NOASSERTION\n'
        printf 'PackageLicenseDeclared: %s\n' "$(spdx_license_for "$component")"
        printf 'PackageCopyrightText: NOASSERTION\n'
        printf 'Relationship: SPDXRef-Package-hctl2 DEPENDS_ON SPDXRef-Package-%s\n' "$spdx_id"
    } >>"$output"
}

format_spdx_time() {
    local epoch="$1"

    if date -u -d "@$epoch" '+%Y-%m-%dT%H:%M:%SZ' >/dev/null 2>&1; then
        date -u -d "@$epoch" '+%Y-%m-%dT%H:%M:%SZ'
    else
        date -u -r "$epoch" '+%Y-%m-%dT%H:%M:%SZ'
    fi
}

write_spdx_sbom() {
    local output="$1"
    local package_id="$2"
    local version="$3"
    local created="$4"
    local dependencies="$5"
    local first_party="$6"
    local component
    local component_version
    local commit
    local build_input_sha256
    local source_sha256
    local binary_sha256
    local target

    {
        printf 'SPDXVersion: SPDX-2.3\n'
        printf 'DataLicense: CC0-1.0\n'
        printf 'SPDXID: SPDXRef-DOCUMENT\n'
        printf 'DocumentName: %s\n' "$package_id"
        printf 'DocumentNamespace: https://github.com/yesme/hctl2/releases/%s\n' "$package_id"
        printf 'Creator: Tool: hctl2-release-assembler\n'
        printf 'Created: %s\n' "$created"
        printf '\nPackageName: hctl2\n'
        printf 'SPDXID: SPDXRef-Package-hctl2\n'
        printf 'PackageVersion: %s\n' "$version"
        printf 'PackageDownloadLocation: NOASSERTION\n'
        printf 'FilesAnalyzed: false\n'
        printf 'PackageLicenseConcluded: NOASSERTION\n'
        printf 'PackageLicenseDeclared: Apache-2.0\n'
        printf 'PackageCopyrightText: NOASSERTION\n'
        printf 'Relationship: SPDXRef-DOCUMENT DESCRIBES SPDXRef-Package-hctl2\n'
    } >"$output"

    while IFS=$'\t' read -r component component_version commit build_input_sha256 \
        source_sha256 binary_sha256; do
        [[ "$component" == "component" ]] && continue
        write_spdx_package "$output" "$component" "$component_version" "$binary_sha256"
    done <"$dependencies"

    while IFS=$'\t' read -r component component_version target binary_sha256; do
        [[ "$component" == "component" ]] && continue
        write_spdx_package "$output" "$component" "$component_version" "$binary_sha256"
    done <"$first_party"
}

create_archive() {
    local build_dir="$1"
    local package_id="$2"
    local archive="$3"
    local source_date_epoch="$4"
    local timestamp
    local file_list

    if tar --version 2>/dev/null | grep -q 'GNU tar'; then
        tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$source_date_epoch" \
            -C "$build_dir" -cf - "$package_id" | gzip -n >"$archive"
        return
    fi

    timestamp="$(date -u -r "$source_date_epoch" '+%Y%m%d%H%M.%S')"
    file_list="$build_dir/archive-list"
    find "$build_dir/$package_id" -exec touch -h -t "$timestamp" {} +
    (
        cd "$build_dir"
        find "$package_id" -print | LC_ALL=C sort >"$file_list"
        COPYFILE_DISABLE=1 tar \
            --no-recursion \
            --uid 0 --gid 0 --uname root --gname wheel --numeric-owner \
            -cf - -T "$file_list"
    ) | gzip -n >"$archive"
    rm -f -- "$file_list"
}

first_party=""
dependencies_archive=""
sources_archive=""
output_dir=""
while (($# > 0)); do
    case "$1" in
        --first-party) first_party="${2:-}"; shift 2 ;;
        --dependencies) dependencies_archive="${2:-}"; shift 2 ;;
        --sources) sources_archive="${2:-}"; shift 2 ;;
        --output) output_dir="${2:-}"; shift 2 ;;
        --help | -h) usage; exit 0 ;;
        *) die "unknown argument: $1" ;;
    esac
done

for required in "$first_party" "$dependencies_archive" "$sources_archive" "$output_dir"; do
    [[ -n "$required" ]] || { usage >&2; exit 2; }
done
[[ -d "$first_party" ]] || die "first-party export is missing: $first_party"
[[ -f "$dependencies_archive" ]] || die "dependency archive is missing: $dependencies_archive"
[[ -f "$sources_archive" ]] || die "source archive is missing: $sources_archive"
mkdir -p "$output_dir"
output_dir="$(cd -- "$output_dir" && pwd -P)"
[[ "$output_dir" != "/" ]] || die "output directory cannot be the filesystem root"

verify_archive_sidecar "$dependencies_archive"
verify_archive_sidecar "$sources_archive"

target="$(awk -F '\t' 'NR == 2 { print $1; exit }' "$first_party/target.tsv")"
case "$target" in
    linux-x86_64 | macos-x86_64 | macos-aarch64) ;;
    *) die "unsupported first-party release target: $target" ;;
esac
hctl2_version="$(read_hctl2_version "$PRODUCT_ROOT/Cargo.toml")"
[[ -n "$hctl2_version" ]] || die "could not read HCTL2 workspace version"
package_id="hctl2-$hctl2_version-$target"
source_package_id="$package_id-sources"
[[ "$(basename -- "$dependencies_archive")" == "$package_id.tar.gz" ]] || \
    die "dependency archive does not match first-party target: $dependencies_archive"
[[ "$(basename -- "$sources_archive")" == "$source_package_id.tar.gz" ]] || \
    die "source archive does not match first-party target: $sources_archive"
validate_archive_layout "$dependencies_archive" "$package_id"
validate_archive_layout "$sources_archive" "$source_package_id"

build_dir="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-release.XXXXXX")"
case "$build_dir" in
    /*/hctl2-release.*) ;;
    *) die "unsafe release build directory: $build_dir" ;;
esac
trap 'find "${build_dir:?}" -depth -delete' EXIT
tar -xzf "$dependencies_archive" -C "$build_dir"
package_root="$build_dir/$package_id"
payload_root="$package_root/payload"

verify_tree_manifest "$package_root" MANIFEST.sha256
verify_tree_manifest "$payload_root" share/hctl2/PAYLOAD.sha256
[[ "$(<"$payload_root/share/hctl2/package-id")" == "$package_id" ]] || \
    die "dependency payload has the wrong package id"
[[ "$(awk -F '\t' 'NR == 2 { print $1; exit }' "$payload_root/share/hctl2/target.tsv")" == "$target" ]] || \
    die "dependency payload has the wrong target"

agentd_seen=0
tool_seen=0
while IFS=$'\t' read -r component version component_target binary_sha256; do
    [[ "$component" == "component" ]] && continue
    case "$component" in
        hctl2-agentd) agentd_seen=$((agentd_seen + 1)) ;;
        hctl2-tool) tool_seen=$((tool_seen + 1)) ;;
        *) die "unexpected first-party component: $component" ;;
    esac
    [[ "$version" == "$hctl2_version" ]] || die "$component has the wrong version: $version"
    [[ "$component_target" == "$target" ]] || die "$component has the wrong target: $component_target"
    [[ "$binary_sha256" =~ ^[0-9a-f]{64}$ ]] || die "$component has an invalid SHA-256"
    [[ -x "$first_party/bin/$component" ]] || die "$component is missing from first-party export"
    verify_sha256 "$first_party/bin/$component" "$binary_sha256"
    install -m 0755 "$first_party/bin/$component" "$payload_root/bin/$component"
done <"$first_party/binaries.tsv"
[[ "$agentd_seen" -eq 1 && "$tool_seen" -eq 1 ]] || die "first-party export is incomplete"

install -m 0644 "$first_party/binaries.tsv" "$payload_root/share/hctl2/first-party.tsv"
install -m 0755 "$SCRIPT_DIR/install.sh" "$package_root/install.sh"
install -m 0644 "$SCRIPT_DIR/PACKAGE-README.md" "$package_root/README.md"

source_date_epoch="${SOURCE_DATE_EPOCH:-$(git -C "$PRODUCT_ROOT" log -1 --format=%ct)}"
[[ "$source_date_epoch" =~ ^[0-9]+$ ]] || die "SOURCE_DATE_EPOCH must be an integer"
sbom_name="$package_id.spdx"
write_spdx_sbom \
    "$payload_root/share/hctl2/SBOM.spdx" \
    "$package_id" \
    "$hctl2_version" \
    "$(format_spdx_time "$source_date_epoch")" \
    "$payload_root/share/hctl2/dependencies.tsv" \
    "$payload_root/share/hctl2/first-party.tsv"

write_checksum_manifest "$payload_root" share/hctl2/PAYLOAD.sha256 bin lib libexec share
write_checksum_manifest "$package_root" MANIFEST.sha256 \
    README.md SOURCES.md USAGE.md install.sh payload

runtime_output="$output_dir/$package_id.tar.gz"
sources_output="$output_dir/$source_package_id.tar.gz"
sbom_output="$output_dir/$sbom_name"
release_manifest="$output_dir/$package_id.release.tsv"
checksum_manifest="$output_dir/$package_id.SHA256SUMS"
create_archive "$build_dir" "$package_id" "$runtime_output" "$source_date_epoch"
install -m 0644 "$sources_archive" "$sources_output"
install -m 0644 "$payload_root/share/hctl2/SBOM.spdx" "$sbom_output"
printf '%s  %s\n' "$(hash_file "$runtime_output")" "$(basename -- "$runtime_output")" \
    >"$runtime_output.sha256"
printf '%s  %s\n' "$(hash_file "$sources_output")" "$(basename -- "$sources_output")" \
    >"$sources_output.sha256"
{
    printf 'artifact\tsha256\trole\n'
    printf '%s\t%s\tuser-installation\n' \
        "$(basename -- "$runtime_output")" "$(hash_file "$runtime_output")"
    printf '%s\t%s\tcorresponding-source\n' \
        "$(basename -- "$sources_output")" "$(hash_file "$sources_output")"
    printf '%s\t%s\tsbom\n' "$(basename -- "$sbom_output")" "$(hash_file "$sbom_output")"
} >"$release_manifest"
(
    cd "$output_dir"
    for artifact in \
        "$(basename -- "$runtime_output")" \
        "$(basename -- "$sources_output")" \
        "$(basename -- "$sbom_output")" \
        "$(basename -- "$release_manifest")"; do
        printf '%s  %s\n' "$(hash_file "$artifact")" "$artifact"
    done
) >"$checksum_manifest"

find "$build_dir" -depth -delete
trap - EXIT
note "assembled release $runtime_output"
