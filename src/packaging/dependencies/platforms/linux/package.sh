#!/usr/bin/env bash
# Linux payload staging and archive hooks.

platform_stage_payload() {
    local library
    local resolved
    local link_name

    require_command ldd
    require_command readlink

    while IFS= read -r library; do
        [[ "$library" == "$P0_VENDOR_DIR/tmux-sysroot/"* ]] || continue
        resolved="$(readlink -f -- "$library")"
        link_name="$(basename -- "$library")"
        # Install the resolved bytes under the SONAME requested by tmux. This
        # keeps the payload free of unchecked symlinks.
        install -m 0755 "$resolved" "$PAYLOAD_ROOT/lib/hctl2/vendor/$link_name"
    done < <(ldd "$P0_BIN_DIR/tmux" | awk '/=> \// { print $3 } /^\// { print $1 }')

    [[ -e "$PAYLOAD_ROOT/lib/hctl2/vendor/libevent_core-2.1.so.7" ]] || \
        die "tmux's bundled libevent runtime was not resolved from the build sysroot"
}

platform_stage_licenses() {
    install -m 0644 "$P0_VENDOR_DIR/tuwunel-$TUWUNEL_VERSION/usr/share/doc/tuwunel/copyright" \
        "$PAYLOAD_ROOT/share/hctl2/licenses/Tuwunel-copyright.txt"
    install -m 0644 "$P0_VENDOR_DIR/vikunja-$VIKUNJA_VERSION/LICENSE" \
        "$PAYLOAD_ROOT/share/hctl2/licenses/Vikunja-AGPL-3.0.txt"
    install -m 0644 "$P0_VENDOR_DIR/dagu-$DAGU_VERSION/LICENSE" \
        "$PAYLOAD_ROOT/share/hctl2/licenses/Dagu-GPL-3.0.txt"
    install -m 0644 "$P0_VENDOR_DIR/tmux-sysroot/usr/share/doc/libevent-core-2.1-7t64/copyright" \
        "$PAYLOAD_ROOT/share/hctl2/licenses/libevent-copyright.txt"
    install -m 0644 "$P0_VENDOR_DIR/tmux-sysroot/usr/share/doc/ncurses-base/copyright" \
        "$PAYLOAD_ROOT/share/hctl2/licenses/ncurses-copyright.txt"
    tar -xOf "$P0_DOWNLOAD_DIR/$TMUX_ASSET" "tmux-$TMUX_VERSION/COPYING" \
        >"$PAYLOAD_ROOT/share/hctl2/licenses/tmux-ISC.txt"
}

platform_stage_build_metadata() {
    install -m 0644 "$P0_MANIFEST_DIR/tmux-build-debs.txt" \
        "$PAYLOAD_ROOT/share/hctl2/tmux-build-debs.txt"
    install -m 0644 "$P0_MANIFEST_DIR/tmux-build-environment.tsv" \
        "$PAYLOAD_ROOT/share/hctl2/build-environment.tsv"
}

platform_create_archive() {
    local build_dir="$1"
    local package_id="$2"
    local archive="$3"
    local source_date_epoch="$4"

    tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$source_date_epoch" \
        -C "$build_dir" -cf - "$package_id" | gzip -n >"$archive"
}
