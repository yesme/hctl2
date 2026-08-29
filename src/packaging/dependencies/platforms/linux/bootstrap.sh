#!/usr/bin/env bash
# Linux dependency acquisition.

install_tuwunel_linux() {
    local destination="$P0_VENDOR_DIR/tuwunel-$TUWUNEL_VERSION"
    local binary="$destination/usr/sbin/tuwunel"

    mkdir -p "$destination"
    dpkg-deb --extract "$P0_DOWNLOAD_DIR/$TUWUNEL_ASSET" "$destination"
    [[ -x "$binary" ]] || die "Tuwunel package did not contain usr/sbin/tuwunel"
    install -m 0755 "$binary" "$P0_BIN_DIR/tuwunel"
    install -m 0644 "$destination/usr/share/doc/tuwunel/copyright" \
        "$P0_MANIFEST_DIR/tuwunel-license"
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

prepare_tuwunel_dependency() {
    require_target_host
    require_command dpkg-deb

    install_tuwunel_linux

    note "installed Tuwunel for $HCTL2_TARGET_ID"
    "$P0_BIN_DIR/tuwunel" --version
    find "$P0_VENDOR_DIR" -depth -delete
}

prepare_vikunja_dependency() {
    require_target_host
    require_command unzip

    install_vikunja_linux
    find "$P0_VENDOR_DIR/vikunja-$VIKUNJA_VERSION" \
        -type f -name 'vikunja-v*-linux-amd64' -delete

    note "prepared Vikunja for $HCTL2_TARGET_ID"
    "$P0_BIN_DIR/vikunja" version
}

prepare_dagu_dependency() {
    require_target_host
    require_command tar

    install_dagu_linux
    find "$P0_VENDOR_DIR/dagu-$DAGU_VERSION" -type f -name dagu -delete

    note "prepared Dagu for $HCTL2_TARGET_ID"
    "$P0_BIN_DIR/dagu" version
}

prepare_herdr_dependency() {
    require_target_host
    require_command readelf

    install_herdr_release
    if readelf -d "$P0_BIN_DIR/herdr" 2>/dev/null | grep -F '(NEEDED)' >/dev/null; then
        die "official Linux Herdr binary unexpectedly has dynamic dependencies"
    fi
    find "$P0_VENDOR_DIR" -depth -delete

    note "prepared Herdr for $HCTL2_TARGET_ID"
    "$P0_BIN_DIR/herdr" --version
}

prepare_static_web_server_dependency() {
    require_target_host
    require_command tar

    install_static_web_server
    find "$P0_VENDOR_DIR/static-web-server-$STATIC_WEB_SERVER_VERSION" \
        -type f -name static-web-server -delete

    note "prepared Static Web Server for $HCTL2_TARGET_ID"
    "$P0_BIN_DIR/static-web-server" --version
}
