#!/usr/bin/env bash
# macOS payload staging, Mach-O dependency relocation, and archive hooks.

relocate_macos_consumer() {
    local consumer="$1"
    local consumer_kind="$2"
    local staged_dependencies="${3:-}"
    local dependency
    local dependency_name
    local destination
    local replacement
    local source

    while IFS= read -r dependency; do
        macos_dependency_is_system "$dependency" && continue
        [[ "$dependency" == /* ]] || die "unsupported Mach-O dependency in $consumer: $dependency"

        dependency_name="$(basename -- "$dependency")"
        source="$dependency"
        if [[ -n "$staged_dependencies" && -f "$staged_dependencies/$dependency_name" ]]; then
            source="$staged_dependencies/$dependency_name"
        fi
        [[ -f "$source" ]] || die "Mach-O dependency is missing from declared inputs: $dependency"
        destination="$PAYLOAD_ROOT/lib/hctl2/vendor/$dependency_name"
        if [[ -f "$destination" ]]; then
            [[ "$(hash_file "$destination")" == "$(hash_file "$source")" ]] || \
                die "different Mach-O dependencies share the filename $dependency_name"
        else
            install -m 0755 "$source" "$destination"
        fi

        if [[ "$consumer_kind" == "binary" ]]; then
            replacement="@loader_path/../../lib/hctl2/vendor/$dependency_name"
        else
            replacement="@loader_path/$dependency_name"
        fi
        install_name_tool -change "$dependency" "$replacement" "$consumer"
    done < <(macos_dependency_paths "$consumer")
}

assert_macos_dependencies_relocatable() {
    local consumer="$1"
    local dependency

    while IFS= read -r dependency; do
        macos_dependency_is_system "$dependency" || \
            die "unbundled Mach-O dependency remains in $consumer: $dependency"
    done < <(macos_dependency_paths "$consumer")
}

platform_stage_payload() {
    local component
    local library
    local before
    local after
    local pass

    require_command codesign
    require_command install_name_tool
    require_command otool
    require_command vtool

    relocate_macos_consumer \
        "$PAYLOAD_ROOT/libexec/hctl2/tuwunel" binary "$P0_TUWUNEL_LIBRARY_DIR"
    for component in vikunja dagu tmux static-web-server; do
        relocate_macos_consumer "$PAYLOAD_ROOT/libexec/hctl2/$component" binary
    done

    for pass in 1 2 3 4 5 6 7 8; do
        before="$(find "$PAYLOAD_ROOT/lib/hctl2/vendor" -type f | wc -l | tr -d ' ')"
        while IFS= read -r library; do
            relocate_macos_consumer "$library" library "$P0_TUWUNEL_LIBRARY_DIR"
            install_name_tool -id "@loader_path/$(basename -- "$library")" "$library"
        done < <(find "$PAYLOAD_ROOT/lib/hctl2/vendor" -type f -print | LC_ALL=C sort)
        after="$(find "$PAYLOAD_ROOT/lib/hctl2/vendor" -type f | wc -l | tr -d ' ')"
        [[ "$before" == "$after" ]] && break
    done
    [[ "$pass" -lt 8 || "$before" == "$after" ]] || die "Mach-O dependency closure did not converge"

    while IFS= read -r library; do
        assert_macos_dependencies_relocatable "$library"
        verify_macos_binary_compatibility "$(basename -- "$library")" "$library"
        codesign --force --sign - "$library"
    done < <(find "$PAYLOAD_ROOT/lib/hctl2/vendor" -type f -print | LC_ALL=C sort)
    for component in tuwunel vikunja dagu tmux static-web-server; do
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
