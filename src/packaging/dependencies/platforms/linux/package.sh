#!/usr/bin/env bash
# Linux payload staging and archive hooks.

platform_stage_payload() {
    if readelf -d "$PAYLOAD_ROOT/libexec/hctl2/tmux" 2>/dev/null | \
        grep -F '(NEEDED)' >/dev/null; then
        die "official Linux tmux binary unexpectedly has dynamic dependencies"
    fi
}

platform_stage_licenses() {
    install -m 0644 "$P0_VENDOR_DIR/tuwunel-$TUWUNEL_VERSION/usr/share/doc/tuwunel/copyright" \
        "$PAYLOAD_ROOT/share/hctl2/licenses/Tuwunel-copyright.txt"
    install -m 0644 "$P0_VENDOR_DIR/vikunja-$VIKUNJA_VERSION/LICENSE" \
        "$PAYLOAD_ROOT/share/hctl2/licenses/Vikunja-AGPL-3.0.txt"
    install -m 0644 "$P0_VENDOR_DIR/dagu-$DAGU_VERSION/LICENSE" \
        "$PAYLOAD_ROOT/share/hctl2/licenses/Dagu-GPL-3.0.txt"
}

platform_stage_build_metadata() {
    :
}

platform_stage_sources() {
    :
}

platform_create_archive() {
    local build_dir="$1"
    local package_id="$2"
    local archive="$3"
    local source_date_epoch="$4"

    tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$source_date_epoch" \
        -C "$build_dir" -cf - "$package_id" | gzip -n >"$archive"
}
