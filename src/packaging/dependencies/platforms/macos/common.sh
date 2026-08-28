#!/usr/bin/env bash
# Shared macOS Mach-O inspection and compatibility helpers.

macos_dependency_paths() {
    otool -L "$1" | sed -n '2,$p' | awk '{print $1}'
}

macos_dependency_is_system() {
    case "$1" in
        /System/* | /usr/lib/* | @loader_path/*) return 0 ;;
        *) return 1 ;;
    esac
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
