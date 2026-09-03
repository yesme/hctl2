load(":lock.json", LOCK = "value")
load("//build/rules:ci.bzl", "CI_INTEGRATION")

_MACOS_SDK_VERSION = read_config("hctl2", "macos_sdk_version", "unavailable")
_MACOS_XCODE_BUILD = read_config("hctl2", "macos_xcode_build", "unavailable")
_MACOS_XCODE_VERSION = read_config("hctl2", "macos_xcode_version", "unavailable")
_TUWUNEL_NATIVE_BUILD = read_config("hctl2", "tuwunel_native_build", "0")

_COMPONENT_PREFIXES = {
    "cinny": "CINNY",
    "dagu": "DAGU",
    "gh": "GH",
    "herdr": "HERDR",
    "process_compose": "PROCESS_COMPOSE",
    "static_web_server": "STATIC_WEB_SERVER",
    "tuwunel": "TUWUNEL",
    "vikunja": "VIKUNJA",
}

def _asset_target_name(scope: str, name: str) -> str:
    return "asset_{}_{}".format(scope, name)

def _declare_http_file(name: str, asset: dict):
    native.http_file(
        name = name,
        out = asset["filename"],
        urls = [asset["url"]],
        sha256 = asset["sha256"],
    )

def _rust_component_url(component: str, triple: str) -> str:
    rust = LOCK["tuwunel_rust"]
    return "https://static.rust-lang.org/dist/{}/{}-{}-{}.tar.xz".format(
        rust["date"],
        component,
        rust["version"],
        triple,
    )

def _tuwunel_native_compatibility() -> list[str]:
    if _TUWUNEL_NATIVE_BUILD == "1":
        return ["prelude//os/constraints:os[macos]"]
    return ["prelude//os/constraints:os[windows]"]

def _declare_tuwunel_rust_components():
    rust = LOCK["tuwunel_rust"]
    for target, spec in rust["targets"].items():
        triple = spec["triple"]
        for component, hash_key in [
            ("cargo", "cargo_sha256"),
            ("rustc", "rustc_sha256"),
            ("rust-std", "std_sha256"),
        ]:
            native.http_archive(
                name = "tuwunel_rust_{}_{}".format(component.replace("-", "_"), target),
                urls = [_rust_component_url(component, triple)],
                sha256 = spec[hash_key],
                strip_prefix = "{}-{}-{}".format(component, rust["version"], triple),
                type = "tar.xz",
                target_compatible_with = _tuwunel_native_compatibility(),
            )

    native.genrule(
        name = "tuwunel-rust-toolchain",
        srcs = select({
            "prelude//os:macos": select({
                "prelude//cpu:arm64": {
                    "cargo": ":tuwunel_rust_cargo_macos_arm64",
                    "rustc": ":tuwunel_rust_rustc_macos_arm64",
                    "rust-std": ":tuwunel_rust_rust_std_macos_arm64",
                },
                "prelude//cpu:x86_64": {
                    "cargo": ":tuwunel_rust_cargo_macos_x86_64",
                    "rustc": ":tuwunel_rust_rustc_macos_x86_64",
                    "rust-std": ":tuwunel_rust_rust_std_macos_x86_64",
                },
            }),
            "DEFAULT": {},
        }),
        bash = """
set -euo pipefail
output_root="$PWD/$OUT"
mkdir -p "$output_root"
for component in cargo rustc rust-std; do
  "$SRCDIR/$component/install.sh" \
    --prefix="$output_root" \
    --disable-ldconfig
done
"$output_root/bin/cargo" --version
"$output_root/bin/rustc" --version
        """,
        out = "tuwunel-rust-toolchain",
        cacheable = True,
        labels = ["large_copy"] + CI_INTEGRATION,
        target_compatible_with = _tuwunel_native_compatibility(),
        visibility = ["PUBLIC"],
    )

def _readonly(name: str, value: str) -> str:
    return "readonly {}=\"{}\"".format(name, value)

def _metadata_lines(target: str, component = None) -> list[str]:
    metadata = LOCK["metadata"]
    target_spec = LOCK["targets"][target]
    common_assets = LOCK["common"]
    target_assets = target_spec["assets"]
    lines = [
        "#!/usr/bin/env bash",
        "# Generated from packaging/dependencies/lock.json by Buck2.",
        "",
        _readonly("HCTL2_TARGET_ID", target_spec["package_target"]),
        _readonly("HCTL2_TARGET_OS", target_spec["os"]),
        _readonly("HCTL2_TARGET_ARCH", target_spec["arch"]),
        _readonly("HCTL2_TARGET_UNAME_S", target_spec["uname_s"]),
        _readonly("HCTL2_TARGET_UNAME_M", target_spec["uname_m"]),
        _readonly("HCTL2_RUST_TARGET", target_spec["rust_target"]),
        _readonly("MACOS_DEPLOYMENT_TARGET", metadata["macos_deployment_target"]),
        _readonly("HCTL2_MACOS_SDK_VERSION", _MACOS_SDK_VERSION),
        _readonly("HCTL2_MACOS_XCODE_BUILD", _MACOS_XCODE_BUILD),
        _readonly("HCTL2_MACOS_XCODE_VERSION", _MACOS_XCODE_VERSION),
        _readonly("HCTL2_SOURCE_DATE_EPOCH", metadata["source_date_epoch"]),
        "",
    ]

    if component == None or component == "tuwunel":
        lines.append(_readonly("TUWUNEL_RUST_TOOLCHAIN", LOCK["tuwunel_rust"]["version"]))

    selected_components = _COMPONENT_PREFIXES.keys() if component == None else [component]
    for selected_component in selected_components:
        prefix = _COMPONENT_PREFIXES[selected_component]
        component_metadata = metadata["components"][selected_component]
        source_asset = common_assets[component_metadata["source_asset"]]
        lines.extend([
            _readonly("{}_VERSION".format(prefix), component_metadata["version"]),
            _readonly("{}_SOURCE_COMMIT".format(prefix), component_metadata["source_commit"]),
            _readonly("{}_SOURCE_ASSET".format(prefix), source_asset["filename"]),
            _readonly("{}_SOURCE_SHA256".format(prefix), source_asset["sha256"]),
        ])

    if component == None or component == "cinny":
        cinny = common_assets["cinny"]
        lines.extend([
            _readonly("CINNY_ASSET", cinny["filename"]),
            _readonly("CINNY_SHA256", cinny["sha256"]),
            "",
        ])

    target_components = ["tuwunel", "vikunja", "dagu", "gh", "herdr", "static_web_server", "process_compose"]
    selected_target_components = target_components if component == None else [component]
    for selected_component in selected_target_components:
        if selected_component == "cinny":
            continue
        prefix = _COMPONENT_PREFIXES[selected_component]
        asset = target_assets.get(selected_component)
        if asset == None:
            lines.extend([
                _readonly("{}_ASSET".format(prefix), ""),
                _readonly("{}_SHA256".format(prefix), ""),
                _readonly(
                    "{}_BUILD_INPUT_SHA256".format(prefix),
                    common_assets[metadata["components"][selected_component]["source_asset"]]["sha256"],
                ),
            ])
        else:
            lines.extend([
                _readonly("{}_ASSET".format(prefix), asset["filename"]),
                _readonly("{}_SHA256".format(prefix), asset["sha256"]),
                _readonly("{}_BUILD_INPUT_SHA256".format(prefix), asset["sha256"]),
            ])

    if component == None:
        for name, value in metadata["runtime"].items():
            lines.append(_readonly(name.upper(), value))
    return lines

def _metadata_command(target: str, component = None) -> str:
    return "cat > \"$OUT\" <<'HCTL2_METADATA'\n{}\nHCTL2_METADATA\n".format(
        "\n".join(_metadata_lines(target, component)),
    )

def _platform_select(values: dict):
    return select({
        "prelude//os:linux": select({
            "prelude//cpu:x86_64": values["linux_x86_64"],
        }),
        "prelude//os:macos": select({
            "prelude//cpu:arm64": values["macos_arm64"],
            "prelude//cpu:x86_64": values["macos_x86_64"],
        }),
    })

def _macos_select(values: dict):
    return select({
        "prelude//os:macos": select({
            "prelude//cpu:arm64": values["macos_arm64"],
            "prelude//cpu:x86_64": values["macos_x86_64"],
        }),
        "DEFAULT": {},
    })

def _macos_command_select(values: dict):
    return select({
        "prelude//os:macos": select({
            "prelude//cpu:arm64": values["macos_arm64"],
            "prelude//cpu:x86_64": values["macos_x86_64"],
        }),
        "DEFAULT": "exit 1",
    })

def _component_load_paths(target: str, component: str) -> list[str]:
    platform = LOCK["targets"][target]["os"]
    paths = ["common/action.sh"] if component == "tuwunel" else ["common/build.sh"]
    if platform == "macos":
        paths.append("platforms/macos/common.sh")
        if component == "tuwunel":
            paths.append("platforms/macos/tuwunel-prebuilt.sh")
        else:
            paths.append("platforms/macos/bootstrap.sh")
    else:
        paths.append("platforms/linux/bootstrap.sh")
    return paths

def _action_script_sources(target: str, component: str, build_sources: dict) -> dict:
    sources = {"build-metadata.sh": ":metadata-{}".format(_component_target_name(component))}
    paths = _component_load_paths(target, component)
    if component != "tuwunel":
        paths.append("common/action.sh")
    for path in paths:
        sources[path] = build_sources[path]
    return sources

def _component_target_name(component: str) -> str:
    return component.replace("_", "-")

def _component_sources(target: str, component: str, build_sources: dict) -> dict:
    sources = _action_script_sources(target, component, build_sources)
    spec = LOCK["targets"][target]
    if component == "cinny":
        sources["cinny-config.json"] = build_sources["cinny-config.json"]
        asset = LOCK["common"]["cinny"]
        sources["downloads/{}".format(asset["filename"])] = ":{}".format(
            _asset_target_name("common", "cinny"),
        )
    else:
        asset = spec["assets"][component]
        sources["downloads/{}".format(asset["filename"])] = ":{}".format(
            _asset_target_name(target, component),
        )
    return sources

def _build_action_command(target: str, component: str) -> str:
    spec = LOCK["targets"][target]
    platform = spec["os"]
    return """
set -euo pipefail
output_root="$PWD/$OUT"
source_root="$PWD/$SRCDIR"
mkdir -p "$output_root"

source "$source_root/build-metadata.sh"
{source_scripts}

export HCTL2_BUILD_CACHE="$output_root"
export HCTL2_DOWNLOAD_ROOT="$source_root/downloads"

init_build_environment
prepare_{component}_dependency

target_root="$output_root/{package_target}"
if [[ -d "$target_root/tmp" ]]; then
  find "$target_root/tmp" -mindepth 1 -depth -delete
fi
""".format(
        component = component,
        package_target = spec["package_target"],
        platform = platform,
        source_scripts = "\n".join([
            "source \"$source_root/{}\"".format(path)
            for path in _component_load_paths(target, component)
        ]),
    )

def _tuwunel_native_sources(target: str, build_sources: dict) -> dict:
    source_name = LOCK["metadata"]["components"]["tuwunel"]["source_asset"]
    asset = LOCK["common"][source_name]
    return {
        "build-metadata.sh": ":metadata-tuwunel",
        "common/action.sh": build_sources["common/action.sh"],
        "downloads/{}".format(asset["filename"]): ":{}".format(
            _asset_target_name("common", source_name),
        ),
        "platforms/macos/common.sh": build_sources["platforms/macos/common.sh"],
        "platforms/macos/tuwunel.sh": build_sources["platforms/macos/tuwunel.sh"],
        "tuwunel-rust-toolchain": ":tuwunel-rust-toolchain",
    }

def _tuwunel_native_command(target: str) -> str:
    spec = LOCK["targets"][target]
    return """
set -euo pipefail
output_root="$PWD/$OUT"
source_root="$PWD/$SRCDIR"
mkdir -p "$output_root"

source "$source_root/build-metadata.sh"
source "$source_root/common/action.sh"
source "$source_root/platforms/macos/common.sh"
source "$source_root/platforms/macos/tuwunel.sh"

export HCTL2_BUILD_CACHE="$output_root"
export HCTL2_DOWNLOAD_ROOT="$source_root/downloads"
export HCTL2_TUWUNEL_TOOLCHAIN_ROOT="$source_root/tuwunel-rust-toolchain"
export HCTL2_CARGO_HOME="$TMP/cargo-home"

init_build_environment
prepare_tuwunel_dependency

target_root="$output_root/{package_target}"
if [[ -d "$target_root/tmp" ]]; then
  find "$target_root/tmp" -mindepth 1 -depth -delete
fi
""".format(package_target = spec["package_target"])

def _tuwunel_native_archive_command(target: str) -> str:
    spec = LOCK["targets"][target]
    asset = spec["assets"]["tuwunel"]
    return """
set -euo pipefail
source_root="$PWD/$SRCDIR"
output_root="$PWD/$OUT"
component_root="$source_root/component/{package_target}"
stage_root="$TMP/stage"
file_list="$TMP/archive-files"
archive="$output_root/{filename}"

source "$source_root/common/action.sh"
mkdir -p "$output_root" "$stage_root/bin" "$stage_root/manifests"
install -m 0755 "$component_root/bin/tuwunel" "$stage_root/bin/tuwunel"
for manifest in tuwunel-license tuwunel-features.txt macos-build-environment.tsv; do
  install -m 0644 "$component_root/manifests/$manifest" "$stage_root/manifests/$manifest"
done
if [[ -d "$component_root/lib/tuwunel" ]]; then
  mkdir -p "$stage_root/lib/tuwunel"
  cp -aL "$component_root/lib/tuwunel/." "$stage_root/lib/tuwunel/"
fi

find "$stage_root" -exec touch -h -t 200001010000.00 {{}} +
(
  cd "$stage_root"
  find . -mindepth 1 -print | sed 's#^./##' | LC_ALL=C sort >"$file_list"
  COPYFILE_DISABLE=1 tar \
    --no-recursion \
    --uid 0 --gid 0 --uname root --gname wheel --numeric-owner \
    -cf - -T "$file_list"
) | gzip -n >"$archive"
printf '%s  %s\n' "$$(hash_file "$archive")" "{filename}" >"$archive.sha256"
""".format(
        filename = asset["filename"],
        package_target = spec["package_target"],
    )

def _package_sources(package_sources: dict) -> dict:
    sources = dict(package_sources)
    sources.update({
        "build-metadata.sh": ":metadata",
        "product/Cargo.toml": "root//:Cargo.toml",
        "release/LICENSE": "repo//:LICENSE",
        "release/USAGE.md": "repo//:usage",
    })
    for component in _COMPONENT_PREFIXES:
        target_name = _component_target_name(component)
        sources["components/{}".format(target_name)] = ":{}".format(target_name)
    common_assets = LOCK["common"]
    for component_metadata in LOCK["metadata"]["components"].values():
        name = component_metadata["source_asset"]
        asset = common_assets[name]
        sources["downloads/{}".format(asset["filename"])] = ":{}".format(
            _asset_target_name("common", name),
        )
    return sources

def _package_command(target: str) -> str:
    spec = LOCK["targets"][target]
    return """
set -euo pipefail
source_root="$PWD/$SRCDIR"
output_root="$PWD/$OUT"
combined_root="$TMP/combined"
mkdir -p "$output_root"

mkdir -p "$combined_root/{package_target}"
for component_root in "$source_root"/components/*; do
  cp -aL "$component_root/{package_target}/." "$combined_root/{package_target}/"
done

source "$source_root/build-metadata.sh"
source "$source_root/common/build.sh"
if [[ -f "$source_root/platforms/{platform}/common.sh" ]]; then
  source "$source_root/platforms/{platform}/common.sh"
fi
source "$source_root/platforms/{platform}/bootstrap.sh"
source "$source_root/platforms/{platform}/package.sh"
source "$source_root/common/package.sh"

export HCTL2_BUILD_METADATA="$source_root/build-metadata.sh"
export HCTL2_BUILD_CACHE="$combined_root"
export HCTL2_DOWNLOAD_ROOT="$source_root/downloads"
export HCTL2_DIST_DIR="$output_root"
export HCTL2_PRODUCT_ROOT="$source_root/product"
export HCTL2_LICENSE_FILE="$source_root/release/LICENSE"
export HCTL2_USAGE_FILE="$source_root/release/USAGE.md"
export SOURCE_DATE_EPOCH="$HCTL2_SOURCE_DATE_EPOCH"

init_build_environment
assemble_dependency_package
""".format(
        package_target = spec["package_target"],
        platform = spec["os"],
    )

def declare_external_dependencies(build_sources: dict, package_sources: dict, test_sources: dict):
    for name, asset in LOCK["common"].items():
        _declare_http_file(_asset_target_name("common", name), asset)
    for target, spec in LOCK["targets"].items():
        for name, asset in spec["assets"].items():
            _declare_http_file(_asset_target_name(target, name), asset)

    _declare_tuwunel_rust_components()

    native.genrule(
        name = "metadata",
        bash = _platform_select({
            target: _metadata_command(target)
            for target in LOCK["targets"]
        }),
        out = "build-metadata.sh",
        cacheable = True,
        visibility = ["PUBLIC"],
    )

    for component in _COMPONENT_PREFIXES:
        target_name = _component_target_name(component)
        native.genrule(
            name = "metadata-{}".format(target_name),
            bash = _platform_select({
                target: _metadata_command(target, component)
                for target in LOCK["targets"]
            }),
            out = "build-metadata-{}.sh".format(target_name),
            cacheable = True,
            visibility = ["PUBLIC"],
        )

    native.genrule(
        name = "tuwunel-native-build",
        srcs = _macos_select({
            target: _tuwunel_native_sources(target, build_sources)
            for target in ["macos_arm64", "macos_x86_64"]
        }),
        bash = _macos_command_select({
            target: _tuwunel_native_command(target)
            for target in ["macos_arm64", "macos_x86_64"]
        }),
        out = "hctl2-tuwunel-native-build",
        cacheable = True,
        labels = ["network_access"] + CI_INTEGRATION,
        target_compatible_with = _tuwunel_native_compatibility(),
        visibility = ["PUBLIC"],
    )

    native.genrule(
        name = "tuwunel-native-archive",
        srcs = {
            "common/action.sh": build_sources["common/action.sh"],
            "component": ":tuwunel-native-build",
        },
        bash = _macos_command_select({
            target: _tuwunel_native_archive_command(target)
            for target in ["macos_arm64", "macos_x86_64"]
        }),
        out = "tuwunel-native-archive",
        cacheable = True,
        labels = ["large_copy"] + CI_INTEGRATION,
        target_compatible_with = _tuwunel_native_compatibility(),
        visibility = ["PUBLIC"],
    )

    for component in _COMPONENT_PREFIXES:
        target_name = _component_target_name(component)
        native.genrule(
            name = target_name,
            srcs = _platform_select({
                target: _component_sources(target, component, build_sources)
                for target in LOCK["targets"]
            }),
            bash = _platform_select({
                target: _build_action_command(target, component)
                for target in LOCK["targets"]
            }),
            out = "hctl2-{}-cache".format(target_name),
            cacheable = True,
            labels = ["large_copy"] + CI_INTEGRATION,
            visibility = ["PUBLIC"],
        )

    native.genrule(
        name = "package",
        srcs = _package_sources(package_sources),
        bash = _platform_select({
            target: _package_command(target)
            for target in LOCK["targets"]
        }),
        out = "dependency-packages",
        cacheable = True,
        labels = ["large_copy"] + CI_INTEGRATION,
        tests = [":package-test"],
        visibility = ["PUBLIC"],
    )

    native.genrule(
        name = "test-support",
        srcs = test_sources,
        bash = "mkdir -p \"$OUT\" && cp -aL \"$SRCDIR/.\" \"$OUT/\"",
        out = "dependency-test-support",
        cacheable = True,
        labels = ["large_copy"] + CI_INTEGRATION,
        visibility = ["PUBLIC"],
    )

    native.sh_test(
        name = "package-test",
        test = "test-package.sh",
        args = ["$(location :package)"],
        env = {
            "HCTL2_BUILD_METADATA": "$(location :metadata)",
            "HCTL2_DEPENDENCY_SOURCE_ROOT": "$(location :test-support)",
        },
        resources = [
            ":metadata",
            ":package",
            ":test-support",
        ],
        labels = CI_INTEGRATION,
        run_test_separately = True,
        test_rule_timeout_ms = 600000,
        visibility = ["PUBLIC"],
    )
