#!/usr/bin/env bash
# Shared build-host helpers. Target descriptors and platform implementations
# contain every value or operation that differs by release target.

set -euo pipefail

P0_COMMON_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly P0_COMMON_DIR
P0_DEPENDENCY_SOURCE_ROOT="$(cd -- "$P0_COMMON_DIR/.." && pwd -P)"
readonly P0_DEPENDENCY_SOURCE_ROOT
# shellcheck source=versions.sh
source "$P0_COMMON_DIR/versions.sh"

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

note() {
    printf 'hctl2: %s\n' "$*"
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

hash_file() {
    local path="$1"

    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$path" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$path" | awk '{print $1}'
    else
        die "required SHA-256 tool not found: sha256sum or shasum"
    fi
}

verify_sha256() {
    local path="$1"
    local expected="$2"
    local actual

    actual="$(hash_file "$path")"
    [[ "$actual" == "$expected" ]] || die "checksum mismatch for $path: expected $expected, got $actual"
}

download_verified() {
    local name="$1"
    local url="$2"
    local expected="$3"
    local destination="$4"
    local partial="$destination.partial"

    if [[ -f "$destination" ]]; then
        verify_sha256 "$destination" "$expected"
        note "$name asset already verified"
        return
    fi

    note "downloading $name"
    curl --fail --location --retry 3 --continue-at - --output "$partial" "$url"
    verify_sha256 "$partial" "$expected"
    mv -- "$partial" "$destination"
}

init_build_environment() {
    [[ -n "${HCTL2_TARGET_ID:-}" ]] || die "a target descriptor must be loaded first"

    if [[ -n "${HCTL2_BUILD_CACHE:-}" ]]; then
        P0_ROOT="$HCTL2_BUILD_CACHE/$HCTL2_TARGET_ID"
    elif [[ -n "${XDG_CACHE_HOME:-}" ]]; then
        P0_ROOT="$XDG_CACHE_HOME/hctl2/dependencies/$HCTL2_TARGET_ID"
    else
        P0_ROOT="${HOME:?HOME must be set}/.cache/hctl2/dependencies/$HCTL2_TARGET_ID"
    fi
    [[ "$P0_ROOT" == /* && "$P0_ROOT" != "/" ]] || \
        die "HCTL2_BUILD_CACHE must be an absolute, non-root path"

    P0_BIN_DIR="$P0_ROOT/bin"
    if [[ -n "${HCTL2_DOWNLOAD_ROOT:-}" ]]; then
        P0_DOWNLOAD_DIR="$HCTL2_DOWNLOAD_ROOT"
        [[ "$P0_DOWNLOAD_DIR" == /* && -d "$P0_DOWNLOAD_DIR" ]] || \
            die "HCTL2_DOWNLOAD_ROOT must be an existing absolute directory"
    else
        P0_DOWNLOAD_DIR="$P0_ROOT/downloads"
    fi
    P0_MANIFEST_DIR="$P0_ROOT/manifests"
    P0_TMP_DIR="$P0_ROOT/tmp"
    P0_VENDOR_DIR="$P0_ROOT/vendor"
    readonly P0_ROOT P0_BIN_DIR P0_DOWNLOAD_DIR P0_MANIFEST_DIR P0_TMP_DIR P0_VENDOR_DIR

    mkdir -p "$P0_BIN_DIR" "$P0_DOWNLOAD_DIR" "$P0_MANIFEST_DIR" "$P0_TMP_DIR" "$P0_VENDOR_DIR"
}

require_target_host() {
    local actual_system
    local actual_machine

    actual_system="$(uname -s)"
    actual_machine="$(uname -m)"
    [[ "$actual_system" == "$HCTL2_TARGET_UNAME_S" ]] || \
        die "$HCTL2_TARGET_ID must be built on $HCTL2_TARGET_UNAME_S, not $actual_system"
    [[ "$actual_machine" == "$HCTL2_TARGET_UNAME_M" ]] || \
        die "$HCTL2_TARGET_ID must be built natively on $HCTL2_TARGET_UNAME_M, not $actual_machine"
}

download_locked_inputs() {
    require_command curl

    if [[ -n "$TUWUNEL_ASSET" ]]; then
        download_verified "Tuwunel $TUWUNEL_VERSION" \
            "$TUWUNEL_URL" "$TUWUNEL_SHA256" "$P0_DOWNLOAD_DIR/$TUWUNEL_ASSET"
    fi
    download_verified "Vikunja $VIKUNJA_VERSION" \
        "$VIKUNJA_URL" "$VIKUNJA_SHA256" "$P0_DOWNLOAD_DIR/$VIKUNJA_ASSET"
    download_verified "Dagu $DAGU_VERSION" \
        "$DAGU_URL" "$DAGU_SHA256" "$P0_DOWNLOAD_DIR/$DAGU_ASSET"
    download_verified "tmux $TMUX_VERSION" \
        "$TMUX_URL" "$TMUX_SHA256" "$P0_DOWNLOAD_DIR/$TMUX_ASSET"
    download_verified "tmux-builds licenses $TMUX_VERSION" \
        "$TMUX_LICENSES_URL" "$TMUX_LICENSES_SHA256" \
        "$P0_DOWNLOAD_DIR/$TMUX_LICENSES_ASSET"
    download_verified "Cinny $CINNY_VERSION" \
        "$CINNY_URL" "$CINNY_SHA256" "$P0_DOWNLOAD_DIR/$CINNY_ASSET"
    download_verified "Static Web Server $STATIC_WEB_SERVER_VERSION" \
        "$STATIC_WEB_SERVER_URL" "$STATIC_WEB_SERVER_SHA256" \
        "$P0_DOWNLOAD_DIR/$STATIC_WEB_SERVER_ASSET"
    download_verified "Tuwunel source $TUWUNEL_SOURCE_COMMIT" \
        "$TUWUNEL_SOURCE_URL" "$TUWUNEL_SOURCE_SHA256" "$P0_DOWNLOAD_DIR/$TUWUNEL_SOURCE_ASSET"
    download_verified "Vikunja corresponding source $VIKUNJA_SOURCE_COMMIT" \
        "$VIKUNJA_SOURCE_URL" "$VIKUNJA_SOURCE_SHA256" "$P0_DOWNLOAD_DIR/$VIKUNJA_SOURCE_ASSET"
    download_verified "Dagu corresponding source $DAGU_SOURCE_COMMIT" \
        "$DAGU_SOURCE_URL" "$DAGU_SOURCE_SHA256" "$P0_DOWNLOAD_DIR/$DAGU_SOURCE_ASSET"
    download_verified "tmux source $TMUX_SOURCE_COMMIT" \
        "$TMUX_SOURCE_URL" "$TMUX_SOURCE_SHA256" "$P0_DOWNLOAD_DIR/$TMUX_SOURCE_ASSET"
    download_verified "Cinny corresponding source $CINNY_SOURCE_COMMIT" \
        "$CINNY_SOURCE_URL" "$CINNY_SOURCE_SHA256" \
        "$P0_DOWNLOAD_DIR/$CINNY_SOURCE_ASSET"
    download_verified "Static Web Server source $STATIC_WEB_SERVER_SOURCE_COMMIT" \
        "$STATIC_WEB_SERVER_SOURCE_URL" "$STATIC_WEB_SERVER_SOURCE_SHA256" \
        "$P0_DOWNLOAD_DIR/$STATIC_WEB_SERVER_SOURCE_ASSET"
}

prepare_cinny() {
    local destination="$P0_VENDOR_DIR/cinny-$CINNY_VERSION"

    prepare_source_tree "$P0_DOWNLOAD_DIR/$CINNY_ASSET" \
        "$CINNY_SHA256" "$destination"
    install -m 0644 "$P0_DEPENDENCY_SOURCE_ROOT/cinny-config.json" \
        "$destination/config.json"
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
    local marker="$destination/.hctl2-archive-sha256"
    local recorded=""
    local entry

    if [[ -f "$marker" ]]; then
        read -r recorded <"$marker"
    fi
    if [[ "$recorded" != "$TMUX_SHA256" ]]; then
        while IFS= read -r entry; do
            entry="${entry#./}"
            entry="${entry%/}"
            [[ -z "$entry" || "$entry" == "tmux" ]] || \
                die "unexpected path in official tmux archive: $entry"
        done < <(tar -tzf "$archive")

        case "$destination" in
            "$P0_VENDOR_DIR"/*) ;;
            *) die "refusing to replace tmux directory outside the build cache: $destination" ;;
        esac
        mkdir -p "$destination"
        find "$destination" -mindepth 1 -depth -delete
        tar -xzf "$archive" -C "$destination"
        printf '%s\n' "$TMUX_SHA256" >"$marker"
    fi

    [[ -f "$destination/tmux" && ! -L "$destination/tmux" && -x "$destination/tmux" ]] || \
        die "official tmux archive did not contain an executable named tmux"
    install -m 0755 "$destination/tmux" "$P0_BIN_DIR/tmux"
}

prepare_source_tree() {
    local archive="$1"
    local expected="$2"
    local destination="$3"
    local marker="$destination/.hctl2-source-sha256"
    local recorded=""

    if [[ -f "$marker" ]]; then
        read -r recorded <"$marker"
    fi
    if [[ "$recorded" == "$expected" ]]; then
        return
    fi

    case "$destination" in
        "$P0_VENDOR_DIR"/*) ;;
        *) die "refusing to replace source directory outside the build cache: $destination" ;;
    esac
    mkdir -p "$destination"
    find "$destination" -mindepth 1 -depth -delete
    tar -xf "$archive" -C "$destination" --strip-components=1
    printf '%s\n' "$expected" >"$marker"
}

read_hctl2_version() {
    local cargo_toml="$1"

    awk '
        $0 == "[workspace.package]" { in_package = 1; next }
        in_package && $1 == "version" { gsub(/\"/, "", $3); print $3; exit }
    ' "$cargo_toml"
}

write_installed_manifest() {
    {
        printf 'component\tversion\tbinary_sha256\n'
        printf 'tuwunel\t%s\t%s\n' "$TUWUNEL_VERSION" "$(hash_file "$P0_BIN_DIR/tuwunel")"
        printf 'vikunja\t%s\t%s\n' "$VIKUNJA_VERSION" "$(hash_file "$P0_BIN_DIR/vikunja")"
        printf 'dagu\t%s\t%s\n' "$DAGU_VERSION" "$(hash_file "$P0_BIN_DIR/dagu")"
        printf 'tmux\t%s\t%s\n' "$TMUX_VERSION" "$(hash_file "$P0_BIN_DIR/tmux")"
        printf 'cinny\t%s\t%s\n' "$CINNY_VERSION" "$CINNY_SHA256"
        printf 'static-web-server\t%s\t%s\n' "$STATIC_WEB_SERVER_VERSION" \
            "$(hash_file "$P0_BIN_DIR/static-web-server")"
    } >"$P0_MANIFEST_DIR/installed.tsv"
}
