#!/usr/bin/env bash
# macOS dependency acquisition and Tuwunel native compilation.

readonly TUWUNEL_MACOS_FEATURES="brotli_compression,direct_tls,element_hacks,gzip_compression,media_thumbnail,release_max_log_level,url_preview,zstd_compression"

macos_libclang_path() {
    local clang_path
    local toolchain_root
    local library_dir

    clang_path="$(xcrun --find clang)"
    toolchain_root="$(cd -- "$(dirname -- "$clang_path")/.." && pwd -P)"
    library_dir="$toolchain_root/lib"
    [[ -f "$library_dir/libclang.dylib" ]] || \
        die "libclang.dylib was not found beside the active Apple toolchain: $library_dir"
    printf '%s\n' "$library_dir"
}

verify_macos_binary_arch() {
    local name="$1"
    local binary="$2"
    local architectures

    architectures="$(lipo -archs "$binary")"
    [[ "$architectures" == "$HCTL2_TARGET_UNAME_M" ]] || \
        die "$name binary architecture is '$architectures', expected '$HCTL2_TARGET_UNAME_M'"
}

macos_version_at_most() {
    awk -v actual="$1" -v maximum="$2" 'BEGIN {
        split(actual, a, ".")
        split(maximum, m, ".")
        if ((a[1] + 0) < (m[1] + 0)) exit 0
        if ((a[1] + 0) > (m[1] + 0)) exit 1
        exit !((a[2] + 0) <= (m[2] + 0))
    }'
}

verify_macos_binary_compatibility() {
    local name="$1"
    local binary="$2"
    local minimum

    verify_macos_binary_arch "$name" "$binary"
    minimum="$(vtool -show-build "$binary" | awk '$1 == "minos" { print $2; exit }')"
    if [[ -z "$minimum" ]]; then
        minimum="$(otool -l "$binary" | awk '
            $1 == "cmd" {
                in_version_command = ($2 == "LC_VERSION_MIN_MACOSX")
                next
            }
            in_version_command && $1 == "version" { print $2; exit }
        ')"
    fi
    [[ -n "$minimum" ]] || die "$name does not declare a macOS deployment target"
    macos_version_at_most "$minimum" "$MACOS_DEPLOYMENT_TARGET" || \
        die "$name requires macOS $minimum, above the $MACOS_DEPLOYMENT_TARGET release baseline"
}

install_tuwunel_macos() {
    local source_dir="$P0_VENDOR_DIR/tuwunel-source-$TUWUNEL_SOURCE_COMMIT"
    local target_dir="$P0_VENDOR_DIR/tuwunel-target-$HCTL2_TARGET_ID"
    local binary="$target_dir/$HCTL2_RUST_TARGET/release/tuwunel"
    local jobs="${HCTL2_BUILD_JOBS:-$(sysctl -n hw.ncpu)}"
    local toolchain_bin

    prepare_source_tree \
        "$P0_DOWNLOAD_DIR/$TUWUNEL_SOURCE_ASSET" \
        "$TUWUNEL_SOURCE_SHA256" \
        "$source_dir"

    toolchain_bin="$(tuwunel_toolchain_bin)"

    (
        cd "$source_dir"
        export LIBCLANG_PATH
        LIBCLANG_PATH="$(macos_libclang_path)"
        export CARGO_TARGET_DIR="$target_dir"
        export CARGO_BUILD_JOBS="$jobs"
        if [[ -n "${HCTL2_CARGO_HOME:-}" ]]; then
            export CARGO_HOME="$HCTL2_CARGO_HOME"
        fi
        export MACOSX_DEPLOYMENT_TARGET="$MACOS_DEPLOYMENT_TARGET"
        export PATH="$toolchain_bin:$PATH"
        "$toolchain_bin/cargo" build \
            --release \
            --locked \
            --target "$HCTL2_RUST_TARGET" \
            -p tuwunel \
            --no-default-features \
            --features "$TUWUNEL_MACOS_FEATURES"
    )

    [[ -x "$binary" ]] || die "Tuwunel native build did not produce $binary"
    verify_macos_binary_compatibility Tuwunel "$binary"
    install -m 0755 "$binary" "$P0_BIN_DIR/tuwunel"
}

tuwunel_toolchain_bin() {
    local toolchain_root="${HCTL2_TUWUNEL_TOOLCHAIN_ROOT:-}"

    [[ "$toolchain_root" == /* ]] || \
        die "Buck did not provide an absolute Tuwunel Rust toolchain"
    [[ -x "$toolchain_root/bin/cargo" && -x "$toolchain_root/bin/rustc" ]] || \
        die "Tuwunel Rust toolchain is incomplete: $toolchain_root"
    printf '%s\n' "$toolchain_root/bin"
}

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

write_macos_build_environment() {
    local toolchain_bin

    toolchain_bin="$(tuwunel_toolchain_bin)"
    {
        printf 'tool\tversion\n'
        printf 'target\t%s\n' "$HCTL2_RUST_TARGET"
        printf 'deployment-target\t%s\n' "$MACOS_DEPLOYMENT_TARGET"
        printf 'rustc\t%s\n' "$("$toolchain_bin/rustc" --version)"
        printf 'clang\t%s\n' "$(clang --version | sed -n '1p')"
        printf 'make\t%s\n' "$(make --version | sed -n '1p')"
        printf 'macos\t%s\n' "$(sw_vers -productVersion)"
    } >"$P0_MANIFEST_DIR/macos-build-environment.tsv"
    printf '%s\n' "$TUWUNEL_MACOS_FEATURES" >"$P0_MANIFEST_DIR/tuwunel-features.txt"
}

bootstrap_dependencies() {
    require_target_host
    require_command clang
    require_command lipo
    require_command make
    require_command otool
    require_command sysctl
    require_command tar
    require_command unzip
    require_command vtool
    require_command xcrun

    install_tuwunel_macos
    install_vikunja_macos
    install_dagu_macos
    install_tmux_release
    verify_macos_binary_compatibility tmux "$P0_BIN_DIR/tmux"
    prepare_cinny
    install_static_web_server
    verify_macos_binary_compatibility static-web-server "$P0_BIN_DIR/static-web-server"
    write_macos_build_environment
    write_installed_manifest

    note "installed $HCTL2_TARGET_ID binaries in $P0_BIN_DIR"
    "$P0_BIN_DIR/tuwunel" --version
    "$P0_BIN_DIR/vikunja" version
    "$P0_BIN_DIR/dagu" version
    "$P0_BIN_DIR/tmux" -V
    "$P0_BIN_DIR/static-web-server" --version
}
