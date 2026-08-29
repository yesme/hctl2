#!/usr/bin/env bash
# HCTL2-hosted Tuwunel macOS release acquisition.

install_tuwunel_macos_release() {
    local destination="$P0_VENDOR_DIR/tuwunel-$TUWUNEL_VERSION-$HCTL2_TARGET_ID"
    local binary="$destination/bin/tuwunel"
    local manifest_dir="$destination/manifests"

    mkdir -p "$destination"
    tar -xzf "$P0_DOWNLOAD_DIR/$TUWUNEL_ASSET" -C "$destination"

    [[ -x "$binary" ]] || die "Tuwunel archive did not contain bin/tuwunel"
    for manifest in tuwunel-license tuwunel-features.txt macos-build-environment.tsv; do
        [[ -f "$manifest_dir/$manifest" ]] || \
            die "Tuwunel archive did not contain manifests/$manifest"
    done

    verify_macos_binary_compatibility Tuwunel "$binary"
    install -m 0755 "$binary" "$P0_BIN_DIR/tuwunel"
    install -m 0644 "$manifest_dir/tuwunel-license" \
        "$P0_MANIFEST_DIR/tuwunel-license"
    install -m 0644 "$manifest_dir/tuwunel-features.txt" \
        "$P0_MANIFEST_DIR/tuwunel-features.txt"
    install -m 0644 "$manifest_dir/macos-build-environment.tsv" \
        "$P0_MANIFEST_DIR/macos-build-environment.tsv"

    if [[ -d "$destination/lib/tuwunel" ]]; then
        mkdir -p "$P0_ROOT/lib/tuwunel"
        cp -a "$destination/lib/tuwunel/." "$P0_ROOT/lib/tuwunel/"
    fi
}

prepare_tuwunel_dependency() {
    require_target_host
    require_command lipo
    require_command otool
    require_command tar
    require_command vtool

    install_tuwunel_macos_release
    find "$P0_VENDOR_DIR" -depth -delete

    note "installed HCTL2-hosted Tuwunel for $HCTL2_TARGET_ID"
    "$P0_BIN_DIR/tuwunel" --version
}
