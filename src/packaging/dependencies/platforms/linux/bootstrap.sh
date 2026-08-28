#!/usr/bin/env bash
# Linux dependency acquisition.

install_tuwunel_linux() {
    local destination="$P0_VENDOR_DIR/tuwunel-$TUWUNEL_VERSION"
    local binary="$destination/usr/sbin/tuwunel"

    mkdir -p "$destination"
    dpkg-deb --extract "$P0_DOWNLOAD_DIR/$TUWUNEL_ASSET" "$destination"
    [[ -x "$binary" ]] || die "Tuwunel package did not contain usr/sbin/tuwunel"
    install -m 0755 "$binary" "$P0_BIN_DIR/tuwunel"
}

install_vikunja_linux() {
    local destination="$P0_VENDOR_DIR/vikunja-$VIKUNJA_VERSION"
    local binary

    mkdir -p "$destination"
    unzip -q -o "$P0_DOWNLOAD_DIR/$VIKUNJA_ASSET" -d "$destination"
    binary="$(find "$destination" -type f -name 'vikunja-v*-linux-amd64' -perm -u+x -print -quit)"
    [[ -n "$binary" ]] || die "Vikunja archive did not contain its Linux x86_64 executable"
    install -m 0755 "$binary" "$P0_BIN_DIR/vikunja"
}

install_dagu_linux() {
    local destination="$P0_VENDOR_DIR/dagu-$DAGU_VERSION"
    local binary

    mkdir -p "$destination"
    tar -xzf "$P0_DOWNLOAD_DIR/$DAGU_ASSET" -C "$destination"
    binary="$(find "$destination" -type f -name dagu -perm -u+x -print -quit)"
    [[ -n "$binary" ]] || die "Dagu archive did not contain an executable named dagu"
    install -m 0755 "$binary" "$P0_BIN_DIR/dagu"
}

bootstrap_dependencies() {
    require_target_host
    require_command dpkg-deb
    require_command readelf
    require_command tar
    require_command unzip

    download_locked_inputs
    install_tuwunel_linux
    install_vikunja_linux
    install_dagu_linux
    install_tmux_release
    if readelf -d "$P0_BIN_DIR/tmux" 2>/dev/null | grep -F '(NEEDED)' >/dev/null; then
        die "official Linux tmux binary unexpectedly has dynamic dependencies"
    fi
    prepare_cinny
    install_static_web_server
    write_installed_manifest

    note "installed $HCTL2_TARGET_ID binaries in $P0_BIN_DIR"
    "$P0_BIN_DIR/tuwunel" --version
    "$P0_BIN_DIR/vikunja" version
    "$P0_BIN_DIR/dagu" version
    "$P0_BIN_DIR/tmux" -V
    "$P0_BIN_DIR/static-web-server" --version
}
