#!/usr/bin/env bash
# macOS prebuilt dependency acquisition. Tuwunel lives in tuwunel.sh so
# unrelated component changes cannot invalidate its expensive native action.

install_vikunja_macos() {
    local destination="$P0_VENDOR_DIR/vikunja-$VIKUNJA_VERSION"
    local binary

    mkdir -p "$destination"
    unzip -q -o "$P0_DOWNLOAD_DIR/$VIKUNJA_ASSET" -d "$destination"
    binary="$(find "$destination" -type f -name 'vikunja-v*-darwin-*' ! -name '*.sha256' -perm -u+x -print -quit)"
    [[ -n "$binary" ]] || die "Vikunja archive did not contain its Darwin executable"
    verify_macos_binary_compatibility Vikunja "$binary"
    install -m 0755 "$binary" "$P0_BIN_DIR/vikunja"
}

install_dagu_macos() {
    local destination="$P0_VENDOR_DIR/dagu-$DAGU_VERSION"
    local binary

    mkdir -p "$destination"
    tar -xzf "$P0_DOWNLOAD_DIR/$DAGU_ASSET" -C "$destination"
    binary="$(find "$destination" -type f -name dagu -print -quit)"
    [[ -n "$binary" ]] || die "Dagu archive did not contain an executable named dagu"
    verify_macos_binary_compatibility Dagu "$binary"
    install -m 0755 "$binary" "$P0_BIN_DIR/dagu"
}

prepare_vikunja_dependency() {
    require_target_host
    require_command lipo
    require_command otool
    require_command unzip
    require_command vtool

    install_vikunja_macos
    find "$P0_VENDOR_DIR/vikunja-$VIKUNJA_VERSION" \
        -type f -name 'vikunja-v*-darwin-*' -delete

    note "prepared Vikunja for $HCTL2_TARGET_ID"
    "$P0_BIN_DIR/vikunja" version
}

prepare_dagu_dependency() {
    require_target_host
    require_command lipo
    require_command otool
    require_command tar
    require_command vtool

    install_dagu_macos
    find "$P0_VENDOR_DIR/dagu-$DAGU_VERSION" -type f -name dagu -delete

    note "prepared Dagu for $HCTL2_TARGET_ID"
    "$P0_BIN_DIR/dagu" version
}

prepare_tmux_dependency() {
    require_target_host
    require_command lipo
    require_command otool
    require_command tar
    require_command vtool

    install_tmux_release
    verify_macos_binary_compatibility tmux "$P0_BIN_DIR/tmux"

    find "$P0_VENDOR_DIR" -depth -delete
    note "prepared tmux for $HCTL2_TARGET_ID"
    "$P0_BIN_DIR/tmux" -V
}

prepare_static_web_server_dependency() {
    require_target_host
    require_command lipo
    require_command otool
    require_command tar
    require_command vtool

    install_static_web_server
    verify_macos_binary_compatibility static-web-server "$P0_BIN_DIR/static-web-server"
    find "$P0_VENDOR_DIR/static-web-server-$STATIC_WEB_SERVER_VERSION" \
        -type f -name static-web-server -delete

    note "prepared Static Web Server for $HCTL2_TARGET_ID"
    "$P0_BIN_DIR/static-web-server" --version
}
