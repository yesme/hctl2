#!/usr/bin/env bash
# Isolated macOS Tuwunel native build action.

readonly TUWUNEL_MACOS_FEATURES="brotli_compression,direct_tls,element_hacks,gzip_compression,media_thumbnail,release_max_log_level,url_preview,zstd_compression"

verify_macos_toolchain_identity() {
    local actual_sdk
    local actual_xcode_build
    local actual_xcode_version

    [[ "$HCTL2_MACOS_XCODE_VERSION" != unavailable ]] || \
        die "invoke Buck through ./buck2 so Xcode identity participates in the action key"
    actual_xcode_version="$(xcodebuild -version | awk 'NR == 1 { print $2 }')"
    actual_xcode_build="$(xcodebuild -version | awk 'NR == 2 { print $3 }')"
    actual_sdk="$(xcrun --sdk macosx --show-sdk-version)"
    [[ "$actual_xcode_version" == "$HCTL2_MACOS_XCODE_VERSION" ]] || \
        die "Xcode changed during the build: expected $HCTL2_MACOS_XCODE_VERSION, got $actual_xcode_version"
    [[ "$actual_xcode_build" == "$HCTL2_MACOS_XCODE_BUILD" ]] || \
        die "Xcode build changed during the build: expected $HCTL2_MACOS_XCODE_BUILD, got $actual_xcode_build"
    [[ "$actual_sdk" == "$HCTL2_MACOS_SDK_VERSION" ]] || \
        die "macOS SDK changed during the build: expected $HCTL2_MACOS_SDK_VERSION, got $actual_sdk"
}

stage_tuwunel_macos_dependency_closure() {
    local destination_dir="$P0_ROOT/lib/tuwunel"
    local consumer
    local dependency
    local destination
    local before
    local after
    local pass

    mkdir -p "$destination_dir"
    for pass in 1 2 3 4 5 6 7 8; do
        before="$(find "$destination_dir" -type f | wc -l | tr -d ' ')"
        while IFS= read -r consumer; do
            while IFS= read -r dependency; do
                macos_dependency_is_system "$dependency" && continue
                [[ "$dependency" == /* && -f "$dependency" ]] || \
                    die "unsupported Tuwunel Mach-O dependency: $dependency"
                destination="$destination_dir/$(basename -- "$dependency")"
                if [[ -f "$destination" ]]; then
                    [[ "$(hash_file "$destination")" == "$(hash_file "$dependency")" ]] || \
                        die "different Tuwunel dependencies share the filename $(basename -- "$dependency")"
                else
                    install -m 0755 "$dependency" "$destination"
                fi
            done < <(macos_dependency_paths "$consumer")
        done < <(
            printf '%s\n' "$P0_BIN_DIR/tuwunel"
            find "$destination_dir" -type f -print | LC_ALL=C sort
        )
        after="$(find "$destination_dir" -type f | wc -l | tr -d ' ')"
        [[ "$before" == "$after" ]] && break
    done
    [[ "$pass" -lt 8 || "$before" == "$after" ]] || \
        die "Tuwunel Mach-O dependency closure did not converge"
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

tuwunel_toolchain_bin() {
    local toolchain_root="${HCTL2_TUWUNEL_TOOLCHAIN_ROOT:-}"

    [[ "$toolchain_root" == /* ]] || \
        die "Buck did not provide an absolute Tuwunel Rust toolchain"
    [[ -x "$toolchain_root/bin/cargo" && -x "$toolchain_root/bin/rustc" ]] || \
        die "Tuwunel Rust toolchain is incomplete: $toolchain_root"
    printf '%s\n' "$toolchain_root/bin"
}

install_tuwunel_macos() {
    local source_dir="$P0_VENDOR_DIR/tuwunel-source-$TUWUNEL_SOURCE_COMMIT"
    local target_dir="$P0_VENDOR_DIR/tuwunel-target-$HCTL2_TARGET_ID"
    local binary="$target_dir/$HCTL2_RUST_TARGET/release/tuwunel"
    local jobs="${HCTL2_BUILD_JOBS:-$(sysctl -n hw.ncpu)}"
    local license
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
    license="$(find "$source_dir" -maxdepth 2 -type f \
        \( -iname 'LICENSE' -o -iname 'LICENSE.*' -o -iname 'COPYING' \) -print -quit)"
    [[ -n "$license" ]] || die "Tuwunel source did not contain a license file"
    install -m 0644 "$license" "$P0_MANIFEST_DIR/tuwunel-license"
}

write_macos_build_environment() {
    local toolchain_bin

    toolchain_bin="$(tuwunel_toolchain_bin)"
    {
        printf 'tool\tversion\n'
        printf 'target\t%s\n' "$HCTL2_RUST_TARGET"
        printf 'deployment-target\t%s\n' "$MACOS_DEPLOYMENT_TARGET"
        printf 'xcode\t%s (%s)\n' "$HCTL2_MACOS_XCODE_VERSION" "$HCTL2_MACOS_XCODE_BUILD"
        printf 'macos-sdk\t%s\n' "$HCTL2_MACOS_SDK_VERSION"
        printf 'rustc\t%s\n' "$("$toolchain_bin/rustc" --version)"
        printf 'clang\t%s\n' "$(clang --version | sed -n '1p')"
        printf 'make\t%s\n' "$(make --version | sed -n '1p')"
        printf 'macos\t%s\n' "$(sw_vers -productVersion)"
    } >"$P0_MANIFEST_DIR/macos-build-environment.tsv"
    printf '%s\n' "$TUWUNEL_MACOS_FEATURES" >"$P0_MANIFEST_DIR/tuwunel-features.txt"
}

prepare_tuwunel_dependency() {
    require_target_host
    require_command clang
    require_command lipo
    require_command make
    require_command otool
    require_command sysctl
    require_command tar
    require_command vtool
    require_command xcodebuild
    require_command xcrun

    verify_macos_toolchain_identity
    install_tuwunel_macos
    stage_tuwunel_macos_dependency_closure
    write_macos_build_environment

    note "built Tuwunel for $HCTL2_TARGET_ID"
    "$P0_BIN_DIR/tuwunel" --version
    find "$P0_VENDOR_DIR" -depth -delete
}
