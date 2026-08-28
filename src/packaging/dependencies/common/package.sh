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
    local source_root
    local license_file
    local usage_file
    local build_metadata
    local hctl2_version
    local package_id
    local source_package_id
    local dist_dir
    local archive
    local source_archive
    local build_dir
    local package_root
    local payload_root
    local source_package_root
    local source_files_root
    local source_date_epoch
    local component
    local script

    require_target_host
    require_command gzip
    require_command install
    require_command sed
    require_command tar

    source_root="${HCTL2_PRODUCT_ROOT:?Buck must declare the product root}"
    license_file="${HCTL2_LICENSE_FILE:?Buck must declare the HCTL2 license}"
    usage_file="${HCTL2_USAGE_FILE:?Buck must declare the usage guide}"
    build_metadata="${HCTL2_BUILD_METADATA:?Buck must declare generated build metadata}"
    [[ -f "$source_root/Cargo.toml" ]] || die "Buck product root is missing Cargo.toml"
    [[ -f "$license_file" ]] || die "Buck HCTL2 license input is missing"
    [[ -f "$usage_file" ]] || die "Buck usage guide input is missing"
    [[ -f "$build_metadata" ]] || die "Buck build metadata input is missing"
    hctl2_version="$(read_hctl2_version "$source_root/Cargo.toml")"
    [[ -n "$hctl2_version" ]] || die "could not read workspace package version"

    package_id="hctl2-$hctl2_version-$HCTL2_TARGET_ID"
    source_package_id="$package_id-sources"
    dist_dir="${HCTL2_DIST_DIR:?Buck must declare the package output directory}"
    archive="$dist_dir/$package_id.tar.gz"
    source_archive="$dist_dir/$source_package_id.tar.gz"
    build_dir="$(mktemp -d "$P0_TMP_DIR/package.XXXXXX")"
    package_root="$build_dir/$package_id"
    payload_root="$package_root/payload"
    source_package_root="$build_dir/$source_package_id"
    source_files_root="$source_package_root/sources"

    PACKAGE_ID="$package_id"
    SOURCE_PACKAGE_ID="$source_package_id"
    PACKAGE_ROOT="$package_root"
    PAYLOAD_ROOT="$payload_root"
    SOURCE_PACKAGE_ROOT="$source_package_root"
    ARCHIVE="$archive"
    SOURCE_ARCHIVE="$source_archive"
    readonly PACKAGE_ID SOURCE_PACKAGE_ID PACKAGE_ROOT PAYLOAD_ROOT SOURCE_PACKAGE_ROOT
    readonly ARCHIVE SOURCE_ARCHIVE

    PACKAGE_BUILD_DIR_TO_CLEAN="$build_dir"
    readonly PACKAGE_BUILD_DIR_TO_CLEAN
    trap 'find "${PACKAGE_BUILD_DIR_TO_CLEAN:?}" -depth -delete' EXIT

    for component in tuwunel vikunja dagu tmux static-web-server; do
        [[ -x "$P0_BIN_DIR/$component" ]] || die "$component is missing from the prepared action"
    done
    [[ -f "$P0_VENDOR_DIR/cinny-$CINNY_VERSION/index.html" ]] || \
        die "Cinny is missing from the prepared action"

    mkdir -p \
        "$payload_root/bin" \
        "$payload_root/lib/hctl2/services" \
        "$payload_root/lib/hctl2/vendor" \
        "$payload_root/libexec/hctl2" \
        "$payload_root/share/hctl2/chatroom/cinny" \
        "$payload_root/share/hctl2/licenses" \
        "$source_files_root"

    for component in tuwunel vikunja dagu tmux static-web-server; do
        install -m 0755 "$P0_BIN_DIR/$component" "$payload_root/libexec/hctl2/$component"
    done
    cp -a "$P0_VENDOR_DIR/cinny-$CINNY_VERSION/." \
        "$payload_root/share/hctl2/chatroom/cinny/"
    find "$payload_root/share/hctl2/chatroom/cinny" \
        -name .hctl2-source-sha256 -delete
    install -m 0755 "$P0_DEPENDENCY_SOURCE_ROOT/hctl2-services" "$payload_root/bin/hctl2-services"
    for script in smoke.sh start.sh status.sh stop.sh; do
        install -m 0755 "$P0_DEPENDENCY_SOURCE_ROOT/$script" "$payload_root/lib/hctl2/services/$script"
    done
    install -m 0644 "$P0_COMMON_DIR/runtime.sh" "$payload_root/lib/hctl2/services/lib.sh"
    install -m 0644 "$build_metadata" "$payload_root/lib/hctl2/services/versions.sh"
    install -m 0644 \
        "$P0_DEPENDENCY_SOURCE_ROOT/platforms/$HCTL2_TARGET_OS/runtime.sh" \
        "$payload_root/lib/hctl2/services/platform.sh"

    platform_stage_payload

    install -m 0644 "$license_file" \
        "$payload_root/share/hctl2/licenses/HCTL2-Apache-2.0.txt"
    tar -xOf "$P0_DOWNLOAD_DIR/$CINNY_SOURCE_ASSET" \
        "cinny-$CINNY_SOURCE_COMMIT/LICENSE" \
        >"$payload_root/share/hctl2/licenses/Cinny-AGPL-3.0-only.txt"
    install -m 0644 \
        "$P0_VENDOR_DIR/static-web-server-$STATIC_WEB_SERVER_VERSION/LICENSE-APACHE" \
        "$payload_root/share/hctl2/licenses/Static-Web-Server-Apache-2.0.txt"
    install -m 0644 \
        "$P0_VENDOR_DIR/static-web-server-$STATIC_WEB_SERVER_VERSION/LICENSE-MIT" \
        "$payload_root/share/hctl2/licenses/Static-Web-Server-MIT.txt"
    tar -xOf "$P0_DOWNLOAD_DIR/$TMUX_LICENSES_ASSET" ./COPYING.tmux \
        >"$payload_root/share/hctl2/licenses/tmux-ISC.txt"
    tar -xOf "$P0_DOWNLOAD_DIR/$TMUX_LICENSES_ASSET" ./COPYING.ncurses \
        >"$payload_root/share/hctl2/licenses/tmux-builds-ncurses.txt"
    tar -xOf "$P0_DOWNLOAD_DIR/$TMUX_LICENSES_ASSET" ./LICENSE.libevent \
        >"$payload_root/share/hctl2/licenses/tmux-builds-libevent.txt"
    tar -xOf "$P0_DOWNLOAD_DIR/$TMUX_LICENSES_ASSET" ./LICENSE.utf8proc \
        >"$payload_root/share/hctl2/licenses/tmux-builds-utf8proc.txt"
    if [[ "$HCTL2_TARGET_OS" == "linux" ]]; then
        tar -xOf "$P0_DOWNLOAD_DIR/$TMUX_LICENSES_ASSET" ./COPYRIGHT.musl \
            >"$payload_root/share/hctl2/licenses/tmux-builds-musl.txt"
    fi
    platform_stage_licenses

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
            "$TMUX_VERSION" "$TMUX_SOURCE_COMMIT" "$TMUX_BUILD_INPUT_SHA256" \
            "$TMUX_SOURCE_SHA256" \
            "$(hash_file "$payload_root/libexec/hctl2/tmux")"
        printf 'cinny\t%s\t%s\t%s\t%s\t%s\n' \
            "$CINNY_VERSION" "$CINNY_SOURCE_COMMIT" "$CINNY_SHA256" \
            "$CINNY_SOURCE_SHA256" "$CINNY_SHA256"
        printf 'static-web-server\t%s\t%s\t%s\t%s\t%s\n' \
            "$STATIC_WEB_SERVER_VERSION" "$STATIC_WEB_SERVER_SOURCE_COMMIT" \
            "$STATIC_WEB_SERVER_BUILD_INPUT_SHA256" "$STATIC_WEB_SERVER_SOURCE_SHA256" \
            "$(hash_file "$payload_root/libexec/hctl2/static-web-server")"
    } >"$payload_root/share/hctl2/dependencies.tsv"

    write_checksum_manifest "$payload_root" share/hctl2/PAYLOAD.sha256 bin lib libexec share

    install -m 0755 "$P0_DEPENDENCY_SOURCE_ROOT/install-package.sh" "$package_root/install.sh"
    install -m 0644 "$P0_DEPENDENCY_SOURCE_ROOT/PACKAGE-README.md" "$package_root/README.md"
    install -m 0644 "$usage_file" "$package_root/USAGE.md"
    sed "s/@SOURCE_PACKAGE_ID@/$source_package_id/g" \
        "$P0_DEPENDENCY_SOURCE_ROOT/SOURCE-INFO.md.in" >"$package_root/SOURCES.md"
    write_checksum_manifest "$package_root" MANIFEST.sha256 \
        README.md SOURCES.md USAGE.md install.sh payload

    install -m 0644 "$P0_DOWNLOAD_DIR/$VIKUNJA_SOURCE_ASSET" \
        "$source_files_root/$VIKUNJA_SOURCE_ASSET"
    install -m 0644 "$P0_DOWNLOAD_DIR/$DAGU_SOURCE_ASSET" \
        "$source_files_root/$DAGU_SOURCE_ASSET"
    install -m 0644 "$P0_DOWNLOAD_DIR/$TUWUNEL_SOURCE_ASSET" \
        "$source_files_root/$TUWUNEL_SOURCE_ASSET"
    install -m 0644 "$P0_DOWNLOAD_DIR/$TMUX_SOURCE_ASSET" \
        "$source_files_root/$TMUX_SOURCE_ASSET"
    install -m 0644 "$P0_DOWNLOAD_DIR/$CINNY_SOURCE_ASSET" \
        "$source_files_root/$CINNY_SOURCE_ASSET"
    install -m 0644 "$P0_DOWNLOAD_DIR/$STATIC_WEB_SERVER_SOURCE_ASSET" \
        "$source_files_root/$STATIC_WEB_SERVER_SOURCE_ASSET"
    {
        printf 'component\tversion\tcommit\tarchive\tsha256\trole\n'
        printf 'tuwunel\t%s\t%s\t%s\t%s\treproducibility\n' \
            "$TUWUNEL_VERSION" "$TUWUNEL_SOURCE_COMMIT" \
            "$TUWUNEL_SOURCE_ASSET" "$TUWUNEL_SOURCE_SHA256"
        printf 'vikunja\t%s\t%s\t%s\t%s\tcorresponding-source\n' \
            "$VIKUNJA_VERSION" "$VIKUNJA_SOURCE_COMMIT" \
            "$VIKUNJA_SOURCE_ASSET" "$VIKUNJA_SOURCE_SHA256"
        printf 'dagu\t%s\t%s\t%s\t%s\tcorresponding-source\n' \
            "$DAGU_VERSION" "$DAGU_SOURCE_COMMIT" \
            "$DAGU_SOURCE_ASSET" "$DAGU_SOURCE_SHA256"
        printf 'tmux\t%s\t%s\t%s\t%s\treproducibility\n' \
            "$TMUX_VERSION" "$TMUX_SOURCE_COMMIT" \
            "$TMUX_SOURCE_ASSET" "$TMUX_SOURCE_SHA256"
        printf 'cinny\t%s\t%s\t%s\t%s\tcorresponding-source\n' \
            "$CINNY_VERSION" "$CINNY_SOURCE_COMMIT" \
            "$CINNY_SOURCE_ASSET" "$CINNY_SOURCE_SHA256"
        printf 'static-web-server\t%s\t%s\t%s\t%s\treproducibility\n' \
            "$STATIC_WEB_SERVER_VERSION" "$STATIC_WEB_SERVER_SOURCE_COMMIT" \
            "$STATIC_WEB_SERVER_SOURCE_ASSET" "$STATIC_WEB_SERVER_SOURCE_SHA256"
    } >"$source_package_root/sources.tsv"
    platform_stage_sources "$source_files_root" "$source_package_root/sources.tsv"
    {
        printf 'target\tos\tarch\trust_target\n'
        printf '%s\t%s\t%s\t%s\n' \
            "$HCTL2_TARGET_ID" "$HCTL2_TARGET_OS" "$HCTL2_TARGET_ARCH" "$HCTL2_RUST_TARGET"
    } >"$source_package_root/target.tsv"
    sed "s/@PACKAGE_ID@/$package_id/g" \
        "$P0_DEPENDENCY_SOURCE_ROOT/SOURCE-PACKAGE-README.md.in" \
        >"$source_package_root/README.md"
    write_checksum_manifest "$source_package_root" SOURCE-MANIFEST.sha256 \
        README.md sources.tsv target.tsv sources

    mkdir -p "$dist_dir"
    source_date_epoch="${SOURCE_DATE_EPOCH:?Buck must declare SOURCE_DATE_EPOCH}"
    [[ "$source_date_epoch" =~ ^[0-9]+$ ]] || die "SOURCE_DATE_EPOCH must be an integer"
    platform_create_archive "$build_dir" "$package_id" "$archive" "$source_date_epoch"
    platform_create_archive \
        "$build_dir" "$source_package_id" "$source_archive" "$source_date_epoch"
    printf '%s  %s\n' "$(hash_file "$archive")" "$(basename -- "$archive")" >"$archive.sha256"
    printf '%s  %s\n' "$(hash_file "$source_archive")" "$(basename -- "$source_archive")" \
        >"$source_archive.sha256"

    find "$build_dir" -depth -delete
    trap - EXIT

    note "built offline installation package $archive"
    note "package checksum: $(hash_file "$archive")"
    note "built corresponding source package $source_archive"
    note "source package checksum: $(hash_file "$source_archive")"
}
