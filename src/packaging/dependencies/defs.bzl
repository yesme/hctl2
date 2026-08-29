load(":lock.json", LOCK = "value")

_MACOS_SDK_VERSION = read_config("hctl2", "macos_sdk_version", "unavailable")
_MACOS_XCODE_BUILD = read_config("hctl2", "macos_xcode_build", "unavailable")
_MACOS_XCODE_VERSION = read_config("hctl2", "macos_xcode_version", "unavailable")

_COMPONENT_PREFIXES = {
    "cinny": "CINNY",
    "dagu": "DAGU",
    "herdr": "HERDR",
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
        labels = ["large_copy"],
        target_compatible_with = ["prelude//os/constraints:os[macos]"],
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

    target_components = ["tuwunel", "vikunja", "dagu", "herdr", "static_web_server"]
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

def _component_load_paths(target: str, component: str) -> list[str]:
    platform = LOCK["targets"][target]["os"]
    paths = ["common/action.sh"] if component == "tuwunel" else ["common/build.sh"]
    if platform == "macos":
        paths.append("platforms/macos/common.sh")
        if component == "tuwunel":
            paths.append("platforms/macos/tuwunel.sh")
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
    elif component == "tuwunel" and spec["os"] == "macos":
        source_name = LOCK["metadata"]["components"]["tuwunel"]["source_asset"]
        asset = LOCK["common"][source_name]
        sources["downloads/{}".format(asset["filename"])] = ":{}".format(
            _asset_target_name("common", source_name),
        )
        sources["tuwunel-rust-toolchain"] = ":tuwunel-rust-toolchain"
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
scratch_root="$TMP"
mkdir -p "$output_root"

source "$source_root/build-metadata.sh"
{source_scripts}

export HCTL2_BUILD_CACHE="$output_root"
export HCTL2_DOWNLOAD_ROOT="$source_root/downloads"
if [[ -d "$source_root/tuwunel-rust-toolchain" ]]; then
  export HCTL2_TUWUNEL_TOOLCHAIN_ROOT="$source_root/tuwunel-rust-toolchain"
  export HCTL2_CARGO_HOME="$scratch_root/cargo-home"
fi

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

def _package_sources(package_sources: dict) -> dict:
    sources = dict(package_sources)
    sources.update({
        "build-metadata.sh": ":metadata",
        "product/Cargo.toml": "root//:Cargo.toml",
        "release/LICENSE": "root//packaging/release:license",
        "release/USAGE.md": "root//packaging/release:usage",
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
            labels = ["network_access"] if component == "tuwunel" else ["large_copy"],
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
        labels = ["large_copy"],
        tests = [":package-test"],
        visibility = ["PUBLIC"],
    )

    native.genrule(
        name = "test-support",
        srcs = test_sources,
        bash = "mkdir -p \"$OUT\" && cp -aL \"$SRCDIR/.\" \"$OUT/\"",
        out = "dependency-test-support",
        cacheable = True,
        labels = ["large_copy"],
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
        run_test_separately = True,
        test_rule_timeout_ms = 600000,
        visibility = ["PUBLIC"],
    )
