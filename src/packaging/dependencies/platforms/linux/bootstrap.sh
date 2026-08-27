#!/usr/bin/env bash
# Linux dependency acquisition and compilation.

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

prepare_tmux_linux_build_deps() {
    local apt_download_dir="$P0_VENDOR_DIR/tmux-build-debs"
    local sysroot="$P0_VENDOR_DIR/tmux-sysroot"
    local package_file
    local package
    local package_version
    local candidate
    local selected
    local packages=(
        bison
        libevent-dev
        libevent-2.1-7t64
        libevent-core-2.1-7t64
        libevent-extra-2.1-7t64
        libevent-openssl-2.1-7t64
        libevent-pthreads-2.1-7t64
        libncurses-dev
        libncurses6
        libncursesw6
        ncurses-base
    )
    local selected_packages=()

    mkdir -p "$apt_download_dir" "$sysroot"
    note "ensuring Ubuntu build dependencies for tmux without installing system packages"
    for package in "${packages[@]}"; do
        case "$package" in
            bison) package_version="$TMUX_BUILD_BISON_APT_VERSION" ;;
            libevent*) package_version="$TMUX_BUILD_LIBEVENT_APT_VERSION" ;;
            libncurses* | ncurses-base) package_version="$TMUX_BUILD_NCURSES_APT_VERSION" ;;
            *) die "no pinned apt version for tmux build dependency $package" ;;
        esac

        selected=""
        for candidate in "$apt_download_dir/${package}_"*.deb; do
            [[ -f "$candidate" ]] || continue
            if [[ "$(dpkg-deb --field "$candidate" Package)" == "$package" ]] &&
                [[ "$(dpkg-deb --field "$candidate" Version)" == "$package_version" ]]; then
                selected="$candidate"
                break
            fi
        done
        if [[ -z "$selected" ]]; then
            (
                cd "$apt_download_dir"
                apt-get download "$package=$package_version"
            )
            for candidate in "$apt_download_dir/${package}_"*.deb; do
                [[ -f "$candidate" ]] || continue
                if [[ "$(dpkg-deb --field "$candidate" Package)" == "$package" ]] &&
                    [[ "$(dpkg-deb --field "$candidate" Version)" == "$package_version" ]]; then
                    selected="$candidate"
                    break
                fi
            done
        fi
        [[ -n "$selected" ]] || die "apt did not provide $package=$package_version"
        selected_packages+=("$selected")
    done

    find "$sysroot" -mindepth 1 -depth -delete
    : >"$P0_MANIFEST_DIR/tmux-build-debs.txt"
    for package_file in "${selected_packages[@]}"; do
        dpkg-deb --extract "$package_file" "$sysroot"
        dpkg-deb --field "$package_file" Package Version >>"$P0_MANIFEST_DIR/tmux-build-debs.txt"
    done

    {
        printf 'tool\tversion\n'
        printf 'gcc\t%s\n' "$(gcc -dumpfullversion -dumpversion)"
        printf 'binutils\t%s\n' "$(ld --version | sed -n '1p')"
        printf 'glibc\t%s\n' "$(getconf GNU_LIBC_VERSION)"
        printf 'make\t%s\n' "$(make --version | sed -n '1p')"
        printf 'pkg-config\t%s\n' "$(pkg-config --version)"
    } >"$P0_MANIFEST_DIR/tmux-build-environment.tsv"

    export PKG_CONFIG_PATH="$sysroot/usr/lib/x86_64-linux-gnu/pkgconfig"
    export CPPFLAGS="-I$sysroot/usr/include"
    export LDFLAGS="-L$sysroot/usr/lib/x86_64-linux-gnu -Wl,-rpath,'\$\$ORIGIN/../../lib/hctl2/vendor'"
    export PATH="$sysroot/usr/bin:$PATH"
}

install_tmux_linux() {
    local build_dir
    local install_dir="$P0_VENDOR_DIR/tmux-$TMUX_VERSION"
    local library_dir="$P0_VENDOR_DIR/tmux-sysroot/usr/lib/x86_64-linux-gnu"

    prepare_tmux_linux_build_deps
    if [[ -x "$P0_BIN_DIR/tmux" ]] &&
        [[ "$(LD_LIBRARY_PATH="$library_dir" "$P0_BIN_DIR/tmux" -V)" == "tmux $TMUX_VERSION" ]] &&
        readelf -d "$P0_BIN_DIR/tmux" | grep -F "$TMUX_RELATIVE_RUNPATH" >/dev/null; then
        note "tmux $TMUX_VERSION already built"
        return
    fi

    build_dir="$(mktemp -d "$P0_TMP_DIR/tmux-build.XXXXXX")"
    TMUX_BUILD_DIR_TO_CLEAN="$build_dir"
    trap 'find "${TMUX_BUILD_DIR_TO_CLEAN:?}" -depth -delete' EXIT
    tar -xzf "$P0_DOWNLOAD_DIR/$TMUX_ASSET" -C "$build_dir" --strip-components=1

    (
        cd "$build_dir"
        ./configure --prefix="$install_dir"
        make -j"$(nproc)"
        make install
    )

    install -m 0755 "$install_dir/bin/tmux" "$P0_BIN_DIR/tmux"
    find "$build_dir" -depth -delete
    TMUX_BUILD_DIR_TO_CLEAN=""
    trap - EXIT
}

bootstrap_dependencies() {
    local tmux_library_dir

    require_target_host
    require_command apt-get
    require_command dpkg-deb
    require_command gcc
    require_command getconf
    require_command ld
    require_command make
    require_command pkg-config
    require_command readelf
    require_command tar
    require_command unzip

    download_locked_inputs
    install_tuwunel_linux
    install_vikunja_linux
    install_dagu_linux
    install_tmux_linux
    prepare_cinny
    install_static_web_server
    write_installed_manifest

    note "installed $HCTL2_TARGET_ID binaries in $P0_BIN_DIR"
    "$P0_BIN_DIR/tuwunel" --version
    "$P0_BIN_DIR/vikunja" version
    "$P0_BIN_DIR/dagu" version
    tmux_library_dir="$P0_VENDOR_DIR/tmux-sysroot/usr/lib/x86_64-linux-gnu"
    LD_LIBRARY_PATH="$tmux_library_dir" "$P0_BIN_DIR/tmux" -V
    "$P0_BIN_DIR/static-web-server" --version
}
