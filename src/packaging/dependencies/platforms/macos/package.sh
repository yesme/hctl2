#!/usr/bin/env bash
# macOS payload staging, Mach-O dependency relocation, and archive hooks.

platform_stage_payload() {
    local component
    local library
    local dependency_dir="$PAYLOAD_ROOT/lib/hctl2/vendor"

    require_command codesign
    require_command install_name_tool
    require_command otool
    require_command vtool

    stage_macos_dependency_closure \
        "$dependency_dir" \
        "$P0_TUWUNEL_LIBRARY_DIR" \
        "$PAYLOAD_ROOT/libexec/hctl2/tuwunel" \
        "$PAYLOAD_ROOT/libexec/hctl2/vikunja" \
        "$PAYLOAD_ROOT/libexec/hctl2/dagu" \
        "$PAYLOAD_ROOT/libexec/hctl2/herdr" \
        "$PAYLOAD_ROOT/libexec/hctl2/static-web-server"

    for component in vikunja dagu herdr static-web-server; do
        relocate_macos_consumer \
            "$PAYLOAD_ROOT/libexec/hctl2/$component" binary "$dependency_dir"
    done
    relocate_macos_consumer \
        "$PAYLOAD_ROOT/libexec/hctl2/tuwunel" binary "$dependency_dir"

    while IFS= read -r library; do
        relocate_macos_consumer "$library" library "$dependency_dir"
        install_name_tool -id "@loader_path/$(basename -- "$library")" "$library"
    done < <(find "$dependency_dir" -type f -print | LC_ALL=C sort)

    while IFS= read -r library; do
        assert_macos_dependencies_relocatable "$library"
        verify_macos_binary_compatibility "$(basename -- "$library")" "$library"
        codesign --force --sign - "$library"
    done < <(find "$dependency_dir" -type f -print | LC_ALL=C sort)
    for component in tuwunel vikunja dagu herdr static-web-server; do
        assert_macos_dependencies_relocatable "$PAYLOAD_ROOT/libexec/hctl2/$component"
        verify_macos_binary_compatibility "$component" "$PAYLOAD_ROOT/libexec/hctl2/$component"
        codesign --force --sign - "$PAYLOAD_ROOT/libexec/hctl2/$component"
    done
}

find_license_file() {
    local root="$1"
    find "$root" -type f \( -iname 'LICENSE' -o -iname 'LICENSE.*' -o -iname 'COPYING' \) -print -quit
}

install_named_license() {
    local root="$1"
    local destination="$2"
    local license

    license="$(find_license_file "$root")"
    [[ -n "$license" ]] || die "no license file found below $root"
    install -m 0644 "$license" "$PAYLOAD_ROOT/share/hctl2/licenses/$destination"
}

platform_stage_licenses() {
    install -m 0644 "$P0_TUWUNEL_MANIFEST_DIR/tuwunel-license" \
        "$PAYLOAD_ROOT/share/hctl2/licenses/Tuwunel-Apache-2.0.txt"
    install_named_license "$P0_VENDOR_DIR/vikunja-$VIKUNJA_VERSION" \
        Vikunja-AGPL-3.0.txt
    install_named_license "$P0_VENDOR_DIR/dagu-$DAGU_VERSION" \
        Dagu-GPL-3.0.txt
}

platform_stage_build_metadata() {
    install -m 0644 "$P0_TUWUNEL_MANIFEST_DIR/macos-build-environment.tsv" \
        "$PAYLOAD_ROOT/share/hctl2/build-environment.tsv"
    install -m 0644 "$P0_TUWUNEL_MANIFEST_DIR/tuwunel-features.txt" \
        "$PAYLOAD_ROOT/share/hctl2/tuwunel-features.txt"
}

platform_stage_sources() {
    :
}

platform_create_archive() {
    local build_dir="$1"
    local package_id="$2"
    local archive="$3"
    local source_date_epoch="$4"
    local timestamp
    local file_list="$P0_TMP_DIR/archive-list.$$"

    timestamp="$(date -u -r "$source_date_epoch" '+%Y%m%d%H%M.%S')"
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
