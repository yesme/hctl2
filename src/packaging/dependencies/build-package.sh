#!/usr/bin/env bash
# Assemble the offline, versioned HCTL2 payload.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

require_linux_amd64
ensure_layout
require_command gzip
require_command ldd
require_command tar

REPOSITORY_ROOT="$(cd -- "$SCRIPT_DIR/../../.." && pwd -P)"
readonly REPOSITORY_ROOT
readonly SOURCE_ROOT="$REPOSITORY_ROOT/src"
HCTL2_VERSION="$(awk '
    $0 == "[workspace.package]" { in_package = 1; next }
    in_package && $1 == "version" { gsub(/\"/, "", $3); print $3; exit }
' "$SOURCE_ROOT/Cargo.toml")"
readonly HCTL2_VERSION
[[ -n "$HCTL2_VERSION" ]] || die "could not read workspace package version"

readonly TARGET="linux-x86_64"
readonly PACKAGE_ID="hctl2-$HCTL2_VERSION-$TARGET"
readonly DIST_DIR="${HCTL2_DIST_DIR:-$SOURCE_ROOT/dist}"
readonly ARCHIVE="$DIST_DIR/$PACKAGE_ID.tar.gz"
BUILD_DIR="$(mktemp -d "$P0_TMP_DIR/package.XXXXXX")"
readonly BUILD_DIR
readonly PACKAGE_ROOT="$BUILD_DIR/$PACKAGE_ID"
readonly PAYLOAD_ROOT="$PACKAGE_ROOT/payload"
trap 'rm -rf -- "$BUILD_DIR"' EXIT

if [[ "${HCTL2_SKIP_BOOTSTRAP:-0}" != "1" ]]; then
    "$SCRIPT_DIR/bootstrap.sh"
fi

for component in tuwunel vikunja dagu tmux; do
    [[ -x "$P0_BIN_DIR/$component" ]] || die "$component is missing after bootstrap"
done

mkdir -p \
    "$PAYLOAD_ROOT/bin" \
    "$PAYLOAD_ROOT/lib/hctl2/services" \
    "$PAYLOAD_ROOT/lib/hctl2/vendor" \
    "$PAYLOAD_ROOT/libexec/hctl2" \
    "$PAYLOAD_ROOT/share/hctl2/licenses" \
    "$PAYLOAD_ROOT/share/hctl2/sources"

for component in tuwunel vikunja dagu tmux; do
    install -m 0755 "$P0_BIN_DIR/$component" "$PAYLOAD_ROOT/libexec/hctl2/$component"
done
install -m 0755 "$SCRIPT_DIR/hctl2-services" "$PAYLOAD_ROOT/bin/hctl2-services"
for script in smoke.sh start.sh status.sh stop.sh; do
    install -m 0755 "$SCRIPT_DIR/$script" "$PAYLOAD_ROOT/lib/hctl2/services/$script"
done
for script in lib.sh versions.sh; do
    install -m 0644 "$SCRIPT_DIR/$script" "$PAYLOAD_ROOT/lib/hctl2/services/$script"
done

copy_tmux_libraries() {
    local library
    local resolved
    local link_name
    local resolved_name

    while read -r library; do
        [[ "$library" == "$P0_VENDOR_DIR/tmux-sysroot/"* ]] || continue
        resolved="$(readlink -f -- "$library")"
        link_name="$(basename -- "$library")"
        resolved_name="$(basename -- "$resolved")"
        install -m 0755 "$resolved" "$PAYLOAD_ROOT/lib/hctl2/vendor/$resolved_name"
        if [[ "$link_name" != "$resolved_name" ]]; then
            ln -sfn "$resolved_name" "$PAYLOAD_ROOT/lib/hctl2/vendor/$link_name"
        fi
    done < <(ldd "$P0_BIN_DIR/tmux" | awk '/=> \// { print $3 } /^\// { print $1 }')
}
copy_tmux_libraries
[[ -e "$PAYLOAD_ROOT/lib/hctl2/vendor/libevent_core-2.1.so.7" ]] || \
    die "tmux's bundled libevent runtime was not resolved from the build sysroot"

install -m 0644 "$REPOSITORY_ROOT/LICENSE" "$PAYLOAD_ROOT/share/hctl2/licenses/HCTL2-Apache-2.0.txt"
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

install -m 0644 "$P0_DOWNLOAD_DIR/$VIKUNJA_SOURCE_ASSET" "$PAYLOAD_ROOT/share/hctl2/sources/$VIKUNJA_SOURCE_ASSET"
install -m 0644 "$P0_DOWNLOAD_DIR/$DAGU_SOURCE_ASSET" "$PAYLOAD_ROOT/share/hctl2/sources/$DAGU_SOURCE_ASSET"
install -m 0644 "$P0_DOWNLOAD_DIR/$TUWUNEL_SOURCE_ASSET" "$PAYLOAD_ROOT/share/hctl2/sources/$TUWUNEL_SOURCE_ASSET"
install -m 0644 "$P0_DOWNLOAD_DIR/$TMUX_ASSET" "$PAYLOAD_ROOT/share/hctl2/sources/$TMUX_ASSET"
install -m 0644 "$P0_MANIFEST_DIR/tmux-build-debs.txt" "$PAYLOAD_ROOT/share/hctl2/tmux-build-debs.txt"
install -m 0644 "$P0_MANIFEST_DIR/tmux-build-environment.tsv" "$PAYLOAD_ROOT/share/hctl2/tmux-build-environment.tsv"

printf '%s\n' "$PACKAGE_ID" >"$PAYLOAD_ROOT/share/hctl2/package-id"
{
    printf 'component\tversion\tcommit\tasset_sha256\tsource_sha256\n'
    printf 'tuwunel\t%s\t%s\t%s\t%s\n' \
        "$TUWUNEL_VERSION" "$TUWUNEL_SOURCE_COMMIT" "$TUWUNEL_SHA256" "$TUWUNEL_SOURCE_SHA256"
    printf 'vikunja\t%s\t%s\t%s\t%s\n' \
        "$VIKUNJA_VERSION" "$VIKUNJA_SOURCE_COMMIT" "$VIKUNJA_SHA256" "$VIKUNJA_SOURCE_SHA256"
    printf 'dagu\t%s\t%s\t%s\t%s\n' \
        "$DAGU_VERSION" "$DAGU_SOURCE_COMMIT" "$DAGU_SHA256" "$DAGU_SOURCE_SHA256"
    printf 'tmux\t%s\t%s\t%s\t%s\n' \
        "$TMUX_VERSION" "$TMUX_SOURCE_COMMIT" "$TMUX_SHA256" "$TMUX_SHA256"
} >"$PAYLOAD_ROOT/share/hctl2/dependencies.tsv"

(
    cd "$PAYLOAD_ROOT"
    find bin lib libexec share -type f ! -name PAYLOAD.sha256 -print0 |
        sort -z |
        xargs -0 sha256sum >share/hctl2/PAYLOAD.sha256
)

install -m 0755 "$SCRIPT_DIR/install-package.sh" "$PACKAGE_ROOT/install.sh"
install -m 0644 "$SCRIPT_DIR/PACKAGE-README.md" "$PACKAGE_ROOT/README.md"
install -m 0644 "$REPOSITORY_ROOT/docs/usage.md" "$PACKAGE_ROOT/USAGE.md"
(
    cd "$PACKAGE_ROOT"
    find README.md USAGE.md install.sh payload -type f -print0 | sort -z | xargs -0 sha256sum >MANIFEST.sha256
)

mkdir -p "$DIST_DIR"
readonly SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git -C "$REPOSITORY_ROOT" log -1 --format=%ct)}"
tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$SOURCE_DATE_EPOCH" \
    -C "$BUILD_DIR" -cf - "$PACKAGE_ID" | gzip -n >"$ARCHIVE"
sha256sum "$ARCHIVE" >"$ARCHIVE.sha256"

note "built offline installation package $ARCHIVE"
note "package checksum: $(awk '{print $1}' "$ARCHIVE.sha256")"
