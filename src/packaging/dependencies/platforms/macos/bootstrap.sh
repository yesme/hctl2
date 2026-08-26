#!/usr/bin/env bash
# macOS dependency acquisition and native compilation.

P0_MACOS_PLATFORM_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly P0_MACOS_PLATFORM_DIR
source "$P0_MACOS_PLATFORM_DIR/versions.sh"

readonly TUWUNEL_MACOS_FEATURES="brotli_compression,direct_tls,element_hacks,gzip_compression,media_thumbnail,release_max_log_level,url_preview,zstd_compression"

expected_brew_formula_version() {
    case "$1" in
        bison) printf '%s\n' "$MACOS_BISON_VERSION" ;;
        pkgconf) printf '%s\n' "$MACOS_PKGCONF_VERSION" ;;
        *) die "no pinned Homebrew version for $1" ;;
    esac
}

verify_brew_formula_version() {
    local formula="$1"
    local expected
    local actual

    expected="$(expected_brew_formula_version "$formula")"
    actual="$(brew list --versions "$formula" 2>/dev/null | awk '{print $2}')" || true
    [[ "$actual" == "$expected" ]] || \
        die "Homebrew $formula must be $expected for $HCTL2_TARGET_ID; found '${actual:-not installed}'"
}

brew_formula_prefix() {
    local formula="$1"
    local prefix

    verify_brew_formula_version "$formula"
    prefix="$(brew --prefix "$formula" 2>/dev/null)" || \
        die "missing macOS build dependency: brew install $formula"
    [[ -d "$prefix" ]] || die "Homebrew formula has no installation prefix: $formula"
    printf '%s\n' "$prefix"
}

download_macos_linked_sources() {
    download_verified "libevent source $MACOS_LIBEVENT_VERSION" \
        "$MACOS_LIBEVENT_SOURCE_URL" "$MACOS_LIBEVENT_SOURCE_SHA256" \
        "$P0_DOWNLOAD_DIR/$MACOS_LIBEVENT_SOURCE_ASSET"
    download_verified "ncurses source $MACOS_NCURSES_VERSION" \
        "$MACOS_NCURSES_SOURCE_URL" "$MACOS_NCURSES_SOURCE_SHA256" \
        "$P0_DOWNLOAD_DIR/$MACOS_NCURSES_SOURCE_ASSET"
    download_verified "utf8proc source $MACOS_UTF8PROC_VERSION" \
        "$MACOS_UTF8PROC_SOURCE_URL" "$MACOS_UTF8PROC_SOURCE_SHA256" \
        "$P0_DOWNLOAD_DIR/$MACOS_UTF8PROC_SOURCE_ASSET"
}

prepare_macos_linked_source_trees() {
    prepare_source_tree "$P0_DOWNLOAD_DIR/$MACOS_LIBEVENT_SOURCE_ASSET" \
        "$MACOS_LIBEVENT_SOURCE_SHA256" "$P0_VENDOR_DIR/libevent-source-$MACOS_LIBEVENT_VERSION"
    prepare_source_tree "$P0_DOWNLOAD_DIR/$MACOS_NCURSES_SOURCE_ASSET" \
        "$MACOS_NCURSES_SOURCE_SHA256" "$P0_VENDOR_DIR/ncurses-source-$MACOS_NCURSES_VERSION"
    prepare_source_tree "$P0_DOWNLOAD_DIR/$MACOS_UTF8PROC_SOURCE_ASSET" \
        "$MACOS_UTF8PROC_SOURCE_SHA256" "$P0_VENDOR_DIR/utf8proc-source-$MACOS_UTF8PROC_VERSION"
}

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
    [[ -n "$minimum" ]] || die "$name does not declare a macOS deployment target"
    macos_version_at_most "$minimum" "$MACOS_DEPLOYMENT_TARGET" || \
        die "$name requires macOS $minimum, above the $MACOS_DEPLOYMENT_TARGET release baseline"
}

reset_macos_build_directory() {
    local directory="$1"

    case "$directory" in
        "$P0_VENDOR_DIR"/*) ;;
        *) die "refusing to reset build directory outside the build cache: $directory" ;;
    esac
    mkdir -p "$directory"
    find "$directory" -mindepth 1 -depth -delete
}

extract_macos_build_tree() {
    local archive="$1"
    local destination="$2"

    reset_macos_build_directory "$destination"
    tar -xf "$archive" -C "$destination" --strip-components=1
}

build_macos_tmux_dependencies() {
    local sysroot="$P0_VENDOR_DIR/macos-tmux-sysroot-$HCTL2_TARGET_ID"
    local marker="$sysroot/.hctl2-build-recipe"
    local expected_recipe
    local jobs="${HCTL2_BUILD_JOBS:-$(sysctl -n hw.ncpu)}"
    local build_dir
    local library

    expected_recipe="$MACOS_TMUX_BUILD_RECIPE:$HCTL2_TARGET_ID:$MACOS_DEPLOYMENT_TARGET:$MACOS_LIBEVENT_SOURCE_SHA256:$MACOS_NCURSES_SOURCE_SHA256:$MACOS_UTF8PROC_SOURCE_SHA256"
    if [[ -f "$marker" ]] && [[ "$(<"$marker")" == "$expected_recipe" ]]; then
        for library in \
            "$sysroot/lib/libevent_core-2.1.7.dylib" \
            "$sysroot/lib/libncursesw.6.dylib" \
            "$sysroot/lib/libutf8proc.3.dylib"; do
            [[ -f "$library" ]] || die "cached macOS tmux dependency is missing: $library"
            verify_macos_binary_compatibility "$(basename -- "$library")" "$library"
        done
        return
    fi

    reset_macos_build_directory "$sysroot"

    build_dir="$P0_VENDOR_DIR/libevent-build-$HCTL2_TARGET_ID"
    extract_macos_build_tree "$P0_DOWNLOAD_DIR/$MACOS_LIBEVENT_SOURCE_ASSET" "$build_dir"
    (
        cd "$build_dir"
        export MACOSX_DEPLOYMENT_TARGET="$MACOS_DEPLOYMENT_TARGET"
        ./configure \
            --prefix="$sysroot" \
            --disable-static \
            --enable-shared \
            --disable-openssl \
            --disable-libevent-regress \
            --disable-samples
        make -s -j"$jobs"
        make -s install
    )

    build_dir="$P0_VENDOR_DIR/ncurses-build-$HCTL2_TARGET_ID"
    extract_macos_build_tree "$P0_DOWNLOAD_DIR/$MACOS_NCURSES_SOURCE_ASSET" "$build_dir"
    (
        cd "$build_dir"
        export MACOSX_DEPLOYMENT_TARGET="$MACOS_DEPLOYMENT_TARGET"
        ./configure \
            --prefix="$sysroot" \
            --with-shared \
            --without-debug \
            --without-ada \
            --without-cxx \
            --without-cxx-binding \
            --without-manpages \
            --without-progs \
            --without-tests \
            --disable-db-install \
            --enable-widec \
            --enable-pc-files \
            --with-pkg-config-libdir="$sysroot/lib/pkgconfig"
        make -s -j"$jobs"
        make -s install
    )

    build_dir="$P0_VENDOR_DIR/utf8proc-build-$HCTL2_TARGET_ID"
    extract_macos_build_tree "$P0_DOWNLOAD_DIR/$MACOS_UTF8PROC_SOURCE_ASSET" "$build_dir"
    (
        cd "$build_dir"
        export MACOSX_DEPLOYMENT_TARGET="$MACOS_DEPLOYMENT_TARGET"
        make -s -j"$jobs" prefix="$sysroot"
        make -s install prefix="$sysroot"
    )

    for library in \
        "$sysroot/lib/libevent_core-2.1.7.dylib" \
        "$sysroot/lib/libncursesw.6.dylib" \
        "$sysroot/lib/libutf8proc.3.dylib"; do
        [[ -f "$library" ]] || die "macOS tmux dependency build did not produce $library"
        verify_macos_binary_compatibility "$(basename -- "$library")" "$library"
    done
    printf '%s\n' "$expected_recipe" >"$marker"
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

    rustup toolchain install "$TUWUNEL_RUST_TOOLCHAIN" --profile minimal
    rustup target add --toolchain "$TUWUNEL_RUST_TOOLCHAIN" "$HCTL2_RUST_TARGET"
    toolchain_bin="$(dirname -- "$(rustup which --toolchain "$TUWUNEL_RUST_TOOLCHAIN" rustc)")"

    (
        cd "$source_dir"
        export LIBCLANG_PATH
        LIBCLANG_PATH="$(macos_libclang_path)"
        export CARGO_TARGET_DIR="$target_dir"
        export CARGO_BUILD_JOBS="$jobs"
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

install_tmux_macos() {
    local source_dir="$P0_VENDOR_DIR/tmux-source-$TMUX_VERSION"
    local install_dir="$P0_VENDOR_DIR/tmux-$TMUX_VERSION"
    local build_dir="$P0_VENDOR_DIR/tmux-build-$HCTL2_TARGET_ID"
    local sysroot="$P0_VENDOR_DIR/macos-tmux-sysroot-$HCTL2_TARGET_ID"
    local bison_prefix
    local marker="$install_dir/.hctl2-build-recipe"
    local expected_recipe
    local jobs="${HCTL2_BUILD_JOBS:-$(sysctl -n hw.ncpu)}"

    expected_recipe="$MACOS_TMUX_BUILD_RECIPE:$HCTL2_TARGET_ID:$MACOS_DEPLOYMENT_TARGET:$TMUX_SHA256"
    build_macos_tmux_dependencies
    if [[ -x "$P0_BIN_DIR/tmux" ]] && \
        [[ "$("$P0_BIN_DIR/tmux" -V)" == "tmux $TMUX_VERSION" ]] && \
        [[ -f "$marker" ]] && [[ "$(<"$marker")" == "$expected_recipe" ]]; then
        verify_macos_binary_compatibility tmux "$P0_BIN_DIR/tmux"
        note "tmux $TMUX_VERSION already built"
        return
    fi

    bison_prefix="$(brew_formula_prefix bison)"
    prepare_source_tree "$P0_DOWNLOAD_DIR/$TMUX_ASSET" "$TMUX_SHA256" "$source_dir"
    extract_macos_build_tree "$P0_DOWNLOAD_DIR/$TMUX_ASSET" "$build_dir"
    reset_macos_build_directory "$install_dir"

    (
        cd "$build_dir"
        export MACOSX_DEPLOYMENT_TARGET="$MACOS_DEPLOYMENT_TARGET"
        export PATH="$bison_prefix/bin:$PATH"
        export PKG_CONFIG_LIBDIR="$sysroot/lib/pkgconfig"
        export PKG_CONFIG_PATH=""
        export CPPFLAGS="-I$sysroot/include"
        export CFLAGS="-mmacosx-version-min=$MACOS_DEPLOYMENT_TARGET"
        export LDFLAGS="-mmacosx-version-min=$MACOS_DEPLOYMENT_TARGET -L$sysroot/lib"
        ./configure --prefix="$install_dir" --enable-utf8proc --disable-jemalloc
        make -s -j"$jobs"
        make -s install
    )

    verify_macos_binary_compatibility tmux "$install_dir/bin/tmux"
    printf '%s\n' "$expected_recipe" >"$marker"
    install -m 0755 "$install_dir/bin/tmux" "$P0_BIN_DIR/tmux"
}

write_macos_build_environment() {
    local toolchain_bin

    toolchain_bin="$(dirname -- "$(rustup which --toolchain "$TUWUNEL_RUST_TOOLCHAIN" rustc)")"
    {
        printf 'tool\tversion\n'
        printf 'target\t%s\n' "$HCTL2_RUST_TARGET"
        printf 'deployment-target\t%s\n' "$MACOS_DEPLOYMENT_TARGET"
        printf 'rustc\t%s\n' "$("$toolchain_bin/rustc" --version)"
        printf 'hctl-rustc\t%s\n' \
            "$(cd "$P0_DEPENDENCY_SOURCE_ROOT/../.." && rustc --version)"
        printf 'clang\t%s\n' "$(clang --version | sed -n '1p')"
        printf 'make\t%s\n' "$(make --version | sed -n '1p')"
        printf 'pkg-config\t%s\n' "$(pkg-config --version)"
        printf 'macos\t%s\n' "$(sw_vers -productVersion)"
        printf 'brew-bison\t%s\n' "$MACOS_BISON_VERSION"
        printf 'source-libevent\t%s\n' "$MACOS_LIBEVENT_VERSION"
        printf 'source-ncurses\t%s\n' "$MACOS_NCURSES_VERSION"
        printf 'source-utf8proc\t%s\n' "$MACOS_UTF8PROC_VERSION"
    } >"$P0_MANIFEST_DIR/macos-build-environment.tsv"
    printf '%s\n' "$TUWUNEL_MACOS_FEATURES" >"$P0_MANIFEST_DIR/tuwunel-features.txt"
}

bootstrap_dependencies() {
    require_target_host
    require_command brew
    require_command clang
    require_command lipo
    require_command make
    require_command pkg-config
    require_command rustup
    require_command sysctl
    require_command tar
    require_command unzip
    require_command vtool
    require_command xcrun

    verify_brew_formula_version bison
    verify_brew_formula_version pkgconf
    download_locked_inputs
    download_macos_linked_sources
    prepare_macos_linked_source_trees
    install_tuwunel_macos
    install_vikunja_macos
    install_dagu_macos
    install_tmux_macos
    prepare_element_web
    build_hctl2_web_server
    write_macos_build_environment
    write_installed_manifest

    note "installed $HCTL2_TARGET_ID binaries in $P0_BIN_DIR"
    "$P0_BIN_DIR/tuwunel" --version
    "$P0_BIN_DIR/vikunja" version
    "$P0_BIN_DIR/dagu" version
    "$P0_BIN_DIR/tmux" -V
    "$P0_BIN_DIR/hctl2-web-server" --version
}
