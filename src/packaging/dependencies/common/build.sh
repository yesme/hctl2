#!/usr/bin/env bash
# Shared preparation helpers for prebuilt external components.

set -euo pipefail

source "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)/action.sh"

prepare_cinny() {
    local destination="$P0_VENDOR_DIR/cinny-$CINNY_VERSION"

    prepare_source_tree "$P0_DOWNLOAD_DIR/$CINNY_ASSET" \
        "$CINNY_SHA256" "$destination"
    install -m 0644 "$P0_DEPENDENCY_SOURCE_ROOT/cinny-config.json" \
        "$destination/config.json"
}

prepare_cinny_dependency() {
    require_target_host
    prepare_cinny
    note "prepared Cinny for $HCTL2_TARGET_ID"
}

install_static_web_server() {
    local destination="$P0_VENDOR_DIR/static-web-server-$STATIC_WEB_SERVER_VERSION"
    local binary="$destination/static-web-server"

    prepare_source_tree "$P0_DOWNLOAD_DIR/$STATIC_WEB_SERVER_ASSET" \
        "$STATIC_WEB_SERVER_SHA256" "$destination"
    [[ -x "$binary" ]] || \
        die "Static Web Server archive did not contain an executable named static-web-server"
    install -m 0755 "$binary" "$P0_BIN_DIR/static-web-server"
}

install_tmux_release() {
    local destination="$P0_VENDOR_DIR/tmux-release-$TMUX_VERSION-$HCTL2_TARGET_ID"
    local archive="$P0_DOWNLOAD_DIR/$TMUX_ASSET"
    local entry

    while IFS= read -r entry; do
        entry="${entry#./}"
        entry="${entry%/}"
        [[ -z "$entry" || "$entry" == "tmux" ]] || \
            die "unexpected path in official tmux archive: $entry"
    done < <(tar -tzf "$archive")

    mkdir -p "$destination"
    tar -xzf "$archive" -C "$destination"

    [[ -f "$destination/tmux" && ! -L "$destination/tmux" && -x "$destination/tmux" ]] || \
        die "official tmux archive did not contain an executable named tmux"
    install -m 0755 "$destination/tmux" "$P0_BIN_DIR/tmux"
}

read_hctl2_version() {
    local cargo_toml="$1"

    awk '
        $0 == "[workspace.package]" { in_package = 1; next }
        in_package && $1 == "version" { gsub(/\"/, "", $3); print $3; exit }
    ' "$cargo_toml"
}
