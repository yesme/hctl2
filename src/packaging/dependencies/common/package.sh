#!/usr/bin/env bash
# Platform-neutral offline payload assembly.

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

assemble_dependency_package() {
    local repository_root
    local source_root
    local hctl2_version
    local package_id
    local dist_dir
    local archive
    local build_dir
    local package_root
    local payload_root
    local source_date_epoch
    local component
    local script

    require_target_host
    require_command gzip
    require_command install
    require_command tar

    repository_root="$(cd -- "$P0_DEPENDENCY_SOURCE_ROOT/../../.." && pwd -P)"
    source_root="$repository_root/src"
    hctl2_version="$(read_hctl2_version "$source_root/Cargo.toml")"
    [[ -n "$hctl2_version" ]] || die "could not read workspace package version"

    package_id="hctl2-$hctl2_version-$HCTL2_TARGET_ID"
    dist_dir="${HCTL2_DIST_DIR:-$source_root/dist}"
    archive="$dist_dir/$package_id.tar.gz"
    build_dir="$(mktemp -d "$P0_TMP_DIR/package.XXXXXX")"
    package_root="$build_dir/$package_id"
    payload_root="$package_root/payload"

    PACKAGE_ID="$package_id"
    PACKAGE_ROOT="$package_root"
    PAYLOAD_ROOT="$payload_root"
    ARCHIVE="$archive"
    readonly PACKAGE_ID PACKAGE_ROOT PAYLOAD_ROOT ARCHIVE

    PACKAGE_BUILD_DIR_TO_CLEAN="$build_dir"
    readonly PACKAGE_BUILD_DIR_TO_CLEAN
    trap 'find "${PACKAGE_BUILD_DIR_TO_CLEAN:?}" -depth -delete' EXIT

    if [[ "${HCTL2_SKIP_BOOTSTRAP:-0}" != "1" ]]; then
        bootstrap_dependencies
    fi
    for component in tuwunel vikunja dagu tmux; do
        [[ -x "$P0_BIN_DIR/$component" ]] || die "$component is missing after bootstrap"
    done

    mkdir -p \
        "$payload_root/bin" \
        "$payload_root/lib/hctl2/services" \
        "$payload_root/lib/hctl2/vendor" \
        "$payload_root/libexec/hctl2" \
        "$payload_root/share/hctl2/licenses" \
        "$payload_root/share/hctl2/sources"

    for component in tuwunel vikunja dagu tmux; do
        install -m 0755 "$P0_BIN_DIR/$component" "$payload_root/libexec/hctl2/$component"
    done
    install -m 0755 "$P0_DEPENDENCY_SOURCE_ROOT/hctl2-services" "$payload_root/bin/hctl2-services"
    for script in smoke.sh start.sh status.sh stop.sh; do
        install -m 0755 "$P0_DEPENDENCY_SOURCE_ROOT/$script" "$payload_root/lib/hctl2/services/$script"
    done
    install -m 0644 "$P0_COMMON_DIR/runtime.sh" "$payload_root/lib/hctl2/services/lib.sh"
    install -m 0644 "$P0_COMMON_DIR/versions.sh" "$payload_root/lib/hctl2/services/versions.sh"
    install -m 0644 \
        "$P0_DEPENDENCY_SOURCE_ROOT/platforms/$HCTL2_TARGET_OS/runtime.sh" \
        "$payload_root/lib/hctl2/services/platform.sh"

    platform_stage_payload

    install -m 0644 "$repository_root/LICENSE" \
        "$payload_root/share/hctl2/licenses/HCTL2-Apache-2.0.txt"
    platform_stage_licenses

    install -m 0644 "$P0_DOWNLOAD_DIR/$VIKUNJA_SOURCE_ASSET" \
        "$payload_root/share/hctl2/sources/$VIKUNJA_SOURCE_ASSET"
    install -m 0644 "$P0_DOWNLOAD_DIR/$DAGU_SOURCE_ASSET" \
        "$payload_root/share/hctl2/sources/$DAGU_SOURCE_ASSET"
    install -m 0644 "$P0_DOWNLOAD_DIR/$TUWUNEL_SOURCE_ASSET" \
        "$payload_root/share/hctl2/sources/$TUWUNEL_SOURCE_ASSET"
    install -m 0644 "$P0_DOWNLOAD_DIR/$TMUX_ASSET" \
        "$payload_root/share/hctl2/sources/$TMUX_ASSET"
    platform_stage_build_metadata

    printf '%s\n' "$package_id" >"$payload_root/share/hctl2/package-id"
    {
        printf 'target\tos\tarch\trust_target\n'
        printf '%s\t%s\t%s\t%s\n' \
            "$HCTL2_TARGET_ID" "$HCTL2_TARGET_OS" "$HCTL2_TARGET_ARCH" "$HCTL2_RUST_TARGET"
    } >"$payload_root/share/hctl2/target.tsv"
    {
        printf 'component\tversion\tcommit\tbuild_input_sha256\tsource_sha256\tbinary_sha256\n'
        printf 'tuwunel\t%s\t%s\t%s\t%s\t%s\n' \
            "$TUWUNEL_VERSION" "$TUWUNEL_SOURCE_COMMIT" "$TUWUNEL_BUILD_INPUT_SHA256" \
            "$TUWUNEL_SOURCE_SHA256" "$(hash_file "$payload_root/libexec/hctl2/tuwunel")"
        printf 'vikunja\t%s\t%s\t%s\t%s\t%s\n' \
            "$VIKUNJA_VERSION" "$VIKUNJA_SOURCE_COMMIT" "$VIKUNJA_BUILD_INPUT_SHA256" \
            "$VIKUNJA_SOURCE_SHA256" "$(hash_file "$payload_root/libexec/hctl2/vikunja")"
        printf 'dagu\t%s\t%s\t%s\t%s\t%s\n' \
            "$DAGU_VERSION" "$DAGU_SOURCE_COMMIT" "$DAGU_BUILD_INPUT_SHA256" \
            "$DAGU_SOURCE_SHA256" "$(hash_file "$payload_root/libexec/hctl2/dagu")"
        printf 'tmux\t%s\t%s\t%s\t%s\t%s\n' \
            "$TMUX_VERSION" "$TMUX_SOURCE_COMMIT" "$TMUX_SHA256" "$TMUX_SHA256" \
            "$(hash_file "$payload_root/libexec/hctl2/tmux")"
    } >"$payload_root/share/hctl2/dependencies.tsv"

    write_checksum_manifest "$payload_root" share/hctl2/PAYLOAD.sha256 bin lib libexec share

    install -m 0755 "$P0_DEPENDENCY_SOURCE_ROOT/install-package.sh" "$package_root/install.sh"
    install -m 0644 "$P0_DEPENDENCY_SOURCE_ROOT/PACKAGE-README.md" "$package_root/README.md"
    install -m 0644 "$repository_root/docs/usage.md" "$package_root/USAGE.md"
    write_checksum_manifest "$package_root" MANIFEST.sha256 README.md USAGE.md install.sh payload

    mkdir -p "$dist_dir"
    source_date_epoch="${SOURCE_DATE_EPOCH:-$(git -C "$repository_root" log -1 --format=%ct)}"
    platform_create_archive "$build_dir" "$package_id" "$archive" "$source_date_epoch"
    printf '%s  %s\n' "$(hash_file "$archive")" "$(basename -- "$archive")" >"$archive.sha256"

    find "$build_dir" -depth -delete
    trap - EXIT

    note "built offline installation package $archive"
    note "package checksum: $(hash_file "$archive")"
}
