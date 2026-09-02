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

stage_macos_dependency_closure() {
    local destination_dir="$1"
    local staged_dependencies="$2"
    shift 2
    local -a root_consumers=("$@")
    local before
    local after
    local consumer
    local dependency
    local dependency_name
    local destination
    local pass
    local source

    mkdir -p "$destination_dir"
    for pass in 1 2 3 4 5 6 7 8; do
        before="$(find "$destination_dir" -type f | wc -l | tr -d ' ')"
        while IFS= read -r consumer; do
            while IFS= read -r dependency; do
                macos_dependency_is_system "$dependency" && continue
                [[ "$dependency" == /* ]] || \
                    die "unsupported Mach-O dependency in $consumer: $dependency"

                dependency_name="$(basename -- "$dependency")"
                source="$dependency"
                if [[ -n "$staged_dependencies" && \
                    -f "$staged_dependencies/$dependency_name" ]]; then
                    source="$staged_dependencies/$dependency_name"
                fi
                [[ -f "$source" ]] || \
                    die "Mach-O dependency is missing from declared inputs: $dependency"
                destination="$destination_dir/$dependency_name"
                if [[ -f "$destination" ]]; then
                    [[ "$(hash_file "$destination")" == "$(hash_file "$source")" ]] || \
                        die "different Mach-O dependencies share the filename $dependency_name"
                else
                    install -m 0755 "$source" "$destination"
                fi
            done < <(macos_dependency_paths "$consumer")
        done < <(
            printf '%s\n' "${root_consumers[@]}"
            find "$destination_dir" -type f -print | LC_ALL=C sort
        )
        after="$(find "$destination_dir" -type f | wc -l | tr -d ' ')"
        [[ "$before" == "$after" ]] && break
    done
    [[ "$pass" -lt 8 || "$before" == "$after" ]] || \
        die "Mach-O dependency closure did not converge"
}

relocate_macos_consumer() {
    local consumer="$1"
    local consumer_kind="$2"
    local dependency_dir="$3"
    local dependency
    local dependency_name
    local replacement

    while IFS= read -r dependency; do
        macos_dependency_is_system "$dependency" && continue
        [[ "$dependency" == /* ]] || \
            die "unsupported Mach-O dependency in $consumer: $dependency"
        dependency_name="$(basename -- "$dependency")"
        [[ -f "$dependency_dir/$dependency_name" ]] || \
            die "bundled Mach-O dependency is missing: $dependency_name"

        if [[ "$consumer_kind" == "binary" ]]; then
            replacement="@loader_path/../../lib/hctl2/vendor/$dependency_name"
        else
            replacement="@loader_path/$dependency_name"
        fi
        install_name_tool -change "$dependency" "$replacement" "$consumer"
    done < <(macos_dependency_paths "$consumer")
}

assert_macos_dependencies_relocatable() {
    local consumer="$1"
    local dependency

    while IFS= read -r dependency; do
        macos_dependency_is_system "$dependency" || \
            die "unbundled Mach-O dependency remains in $consumer: $dependency"
    done < <(macos_dependency_paths "$consumer")
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
