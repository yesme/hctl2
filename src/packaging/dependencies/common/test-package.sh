#!/usr/bin/env bash
# Shared offline install and lifecycle verification.

verify_test_manifest() {
    local root="$1"
    local manifest="$2"
    local expected
    local relative

    while read -r expected relative; do
        [[ -n "$expected" && -n "$relative" ]] || die "invalid checksum line in $manifest"
        relative="${relative#\*}"
        [[ -f "$root/$relative" ]] || die "manifest file is missing: $relative"
        [[ "$(hash_file "$root/$relative")" == "$expected" ]] || \
            die "checksum mismatch for $relative"
    done <"$root/$manifest"
}

verify_archive_sidecar() {
    local archive="$1"
    local expected
    local recorded_name

    read -r expected recorded_name <"$archive.sha256"
    recorded_name="${recorded_name#\*}"
    [[ "$recorded_name" == "$(basename -- "$archive")" ]] || \
        die "checksum sidecar names the wrong archive: $recorded_name"
    [[ "$(hash_file "$archive")" == "$expected" ]] || \
        die "archive checksum mismatch: $archive"
}

test_dependency_package() {
    local test_root
    local prefix
    local state_root
    local services
    local source_component
    local source_version
    local source_commit
    local source_asset
    local source_sha256
    local source_role
    local cinny_headers
    local cinny_range_status
    local retired_script

    [[ -n "${PACKAGE_ID:-}" && -n "${ARCHIVE:-}" && \
        -n "${SOURCE_PACKAGE_ID:-}" && -n "${SOURCE_ARCHIVE:-}" ]] || \
        die "assemble_dependency_package must run before test_dependency_package"
    require_command curl

    test_root="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-package-test.XXXXXX")"
    prefix="$test_root/prefix"
    state_root="$test_root/state"
    services="$prefix/bin/hctl2-services"

    case "$test_root" in
        /*/hctl2-package-test.*) ;;
        *) die "unsafe package test directory: $test_root" ;;
    esac

    PACKAGE_TEST_ROOT_TO_CLEAN="$test_root"
    PACKAGE_TEST_SERVICES="$services"
    PACKAGE_TEST_STATE_ROOT="$state_root"
    readonly PACKAGE_TEST_ROOT_TO_CLEAN PACKAGE_TEST_SERVICES PACKAGE_TEST_STATE_ROOT
    cleanup_package_test() {
        local test_exit=$?

        if [[ -x "$PACKAGE_TEST_SERVICES" ]] &&
            ! HCTL2_STATE_ROOT="$PACKAGE_TEST_STATE_ROOT" "$PACKAGE_TEST_SERVICES" stop >/dev/null 2>&1; then
            printf 'warning: preserving failed package test at %s because managed services did not stop\n' \
                "$PACKAGE_TEST_ROOT_TO_CLEAN" >&2
            return "$test_exit"
        fi
        find "$PACKAGE_TEST_ROOT_TO_CLEAN" -depth -delete
        return "$test_exit"
    }
    trap cleanup_package_test EXIT

    verify_archive_sidecar "$ARCHIVE"
    verify_archive_sidecar "$SOURCE_ARCHIVE"
    tar -xzf "$ARCHIVE" -C "$test_root"
    tar -xzf "$SOURCE_ARCHIVE" -C "$test_root"
    grep -F '# HCTL2 使用说明' "$test_root/$PACKAGE_ID/USAGE.md" >/dev/null
    grep -F "$SOURCE_PACKAGE_ID.tar.gz" "$test_root/$PACKAGE_ID/SOURCES.md" >/dev/null
    [[ -f "$test_root/$PACKAGE_ID/payload/share/hctl2/chatroom/cinny/index.html" ]] || \
        die "runtime package does not contain Cinny"
    [[ -x "$test_root/$PACKAGE_ID/payload/libexec/hctl2/static-web-server" ]] || \
        die "runtime package does not contain static-web-server"
    [[ -x "$test_root/$PACKAGE_ID/payload/libexec/hctl2/process-compose" ]] || \
        die "runtime package does not contain Process Compose"
    [[ -x "$test_root/$PACKAGE_ID/payload/libexec/hctl2/gh" ]] || \
        die "runtime package does not contain GitHub CLI"
    [[ ! -e "$test_root/$PACKAGE_ID/payload/libexec/hctl2/hctl2-web-server" ]] || \
        die "runtime package still contains hctl2-web-server"
    for retired_script in start.sh stop.sh status.sh; do
        [[ ! -e "$test_root/$PACKAGE_ID/payload/lib/hctl2/services/$retired_script" ]] || \
            die "runtime package still contains retired lifecycle script $retired_script"
    done
    [[ ! -e "$test_root/$PACKAGE_ID/payload/share/hctl2/chatroom/element-web" ]] || \
        die "runtime package still contains Element Web"
    grep -F $'cinny\t' "$test_root/$PACKAGE_ID/payload/share/hctl2/dependencies.tsv" >/dev/null
    grep -F $'static-web-server\t' \
        "$test_root/$PACKAGE_ID/payload/share/hctl2/dependencies.tsv" >/dev/null
    grep -F $'process-compose\t' \
        "$test_root/$PACKAGE_ID/payload/share/hctl2/dependencies.tsv" >/dev/null
    grep -F $'gh\t' "$test_root/$PACKAGE_ID/payload/share/hctl2/dependencies.tsv" >/dev/null
    awk -F '\t' \
        -v build_input="$HERDR_BUILD_INPUT_SHA256" \
        -v source="$HERDR_SOURCE_SHA256" \
        '$1 == "herdr" && $4 == build_input && $5 == source { found = 1 } END { exit !found }' \
        "$test_root/$PACKAGE_ID/payload/share/hctl2/dependencies.tsv" || \
        die "runtime package records the wrong Herdr build input or source digest"
    [[ -s "$test_root/$PACKAGE_ID/payload/share/hctl2/licenses/Herdr-Apache-2.0.txt" ]] || \
        die "runtime package is missing the Herdr license"
    [[ -s "$test_root/$PACKAGE_ID/payload/share/hctl2/licenses/Process-Compose-Apache-2.0.txt" ]] || \
        die "runtime package is missing the Process Compose license"
    ! grep -F $'element-web\t' \
        "$test_root/$PACKAGE_ID/payload/share/hctl2/dependencies.tsv" >/dev/null
    [[ ! -e "$test_root/$PACKAGE_ID/payload/share/hctl2/sources" ]] || \
        die "runtime package still contains upstream source archives"
    [[ ! -e "$test_root/$SOURCE_PACKAGE_ID/install.sh" ]] || \
        die "source package unexpectedly contains an installer"
    verify_test_manifest "$test_root/$SOURCE_PACKAGE_ID" SOURCE-MANIFEST.sha256
    grep -F "$HCTL2_TARGET_ID" "$test_root/$SOURCE_PACKAGE_ID/target.tsv" >/dev/null
    grep -F $'cinny\t' "$test_root/$SOURCE_PACKAGE_ID/sources.tsv" >/dev/null
    grep -F $'static-web-server\t' "$test_root/$SOURCE_PACKAGE_ID/sources.tsv" >/dev/null
    grep -F $'process-compose\t' "$test_root/$SOURCE_PACKAGE_ID/sources.tsv" >/dev/null
    grep -F $'gh\t' "$test_root/$SOURCE_PACKAGE_ID/sources.tsv" >/dev/null
    grep -F $'herdr\t' "$test_root/$SOURCE_PACKAGE_ID/sources.tsv" | \
        grep -F "$HERDR_SOURCE_ASSET" >/dev/null || \
        die "source package does not contain the locked Herdr source"
    ! grep -F $'element-web\t' "$test_root/$SOURCE_PACKAGE_ID/sources.tsv" >/dev/null
    while IFS=$'\t' read -r source_component source_version source_commit \
        source_asset source_sha256 source_role; do
        [[ "$source_asset" != "archive" ]] || continue
        verify_sha256 "$test_root/$SOURCE_PACKAGE_ID/sources/$source_asset" "$source_sha256"
    done <"$test_root/$SOURCE_PACKAGE_ID/sources.tsv"
    "$test_root/$PACKAGE_ID/install.sh" --prefix "$prefix"
    "$test_root/$PACKAGE_ID/install.sh" --prefix "$prefix"
    "$services" --help | grep -F 'Usage:' >/dev/null
    "$services" --help | grep -F 'tuwunel  cinny  vikunja  dagu  herdr' >/dev/null
    if "$services" status unexpected >/dev/null 2>&1; then
        die "hctl2-services accepted an argument for status"
    fi
    HCTL2_STATE_ROOT="$state_root" "$services" start
    HCTL2_STATE_ROOT="$state_root" "$services" start
    HCTL2_STATE_ROOT="$state_root" "$services" status
    HCTL2_STATE_ROOT="$state_root" "$services" stop tuwunel
    HCTL2_STATE_ROOT="$state_root" "$services" start cinny
    HCTL2_STATE_ROOT="$state_root" "$services" status
    HCTL2_STATE_ROOT="$state_root" "$services" restart herdr
    HCTL2_STATE_ROOT="$state_root" "$services" stop herdr
    if HCTL2_STATE_ROOT="$state_root" "$services" status >/dev/null 2>&1; then
        die "hctl2-services status accepted a stopped component as ready"
    fi
    HCTL2_STATE_ROOT="$state_root" "$services" start herdr
    HCTL2_STATE_ROOT="$state_root" "$services" smoke
    cinny_headers="$(curl --fail --silent --show-error --head \
        http://127.0.0.1:$CINNY_PORT/config.json | tr -d '\r')"
    grep -Fi 'content-type: application/json' <<<"$cinny_headers" >/dev/null || \
        die "static-web-server returned the wrong MIME type for Cinny config.json"
    cinny_range_status="$(curl --silent --show-error --output /dev/null \
        --header 'Range: bytes=0-9' --write-out '%{http_code}' \
        http://127.0.0.1:$CINNY_PORT/index.html)"
    [[ "$cinny_range_status" == "206" ]] || \
        die "static-web-server did not honor a Cinny byte-range request"
    HCTL2_STATE_ROOT="$state_root" "$services" stop

    note "$HCTL2_TARGET_ID runtime and source packages passed integrity and lifecycle tests"
}
