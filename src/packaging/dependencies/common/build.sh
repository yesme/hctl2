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

install_herdr_release() {
    local source="$P0_DOWNLOAD_DIR/$HERDR_ASSET"

    [[ -f "$source" ]] || die "official Herdr release is missing"
    verify_sha256 "$source" "$HERDR_SHA256"
    install -m 0755 "$source" "$P0_BIN_DIR/herdr"
}

install_gh_release() {
    local destination="$P0_VENDOR_DIR/gh-$GH_VERSION"
    local binary

    prepare_source_tree "$P0_DOWNLOAD_DIR/$GH_ASSET" "$GH_SHA256" "$destination"
    binary="$(find "$destination" -type f -path '*/bin/gh' -print -quit)"
    [[ -n "$binary" ]] || die "GitHub CLI archive did not contain bin/gh"
    install -m 0755 "$binary" "$P0_BIN_DIR/gh"
}

install_process_compose() {
    local destination="$P0_VENDOR_DIR/process-compose-$PROCESS_COMPOSE_VERSION"
    local binary="$destination/process-compose"

    mkdir -p "$destination"
    tar -xf "$P0_DOWNLOAD_DIR/$PROCESS_COMPOSE_ASSET" -C "$destination"
    [[ -x "$binary" ]] || \
        die "Process Compose archive did not contain an executable named process-compose"
    install -m 0755 "$binary" "$P0_BIN_DIR/process-compose"
}

read_hctl2_version() {
    local cargo_toml="$1"

    awk '
        $0 == "[workspace.package]" { in_package = 1; next }
        in_package && $1 == "version" { gsub(/\"/, "", $3); print $3; exit }
    ' "$cargo_toml"
}
