load(":lock.json", LOCK = "value")

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
        target_compatible_with = ["prelude//os/constraints:os[macos]"],
        visibility = ["PUBLIC"],
    )

def _asset_sources(target: str, build_sources: dict) -> dict:
    sources = dict(build_sources)
    for name, asset in LOCK["common"].items():
        sources["downloads/{}".format(asset["filename"])] = ":{}".format(
            _asset_target_name("common", name),
        )
    for name, asset in LOCK["targets"][target]["assets"].items():
        sources["downloads/{}".format(asset["filename"])] = ":{}".format(
            _asset_target_name(target, name),
        )
    if target.startswith("macos_"):
        sources["tuwunel-rust-toolchain"] = ":tuwunel-rust-toolchain"
    return sources

def _prepared_command(target: str) -> str:
    spec = LOCK["targets"][target]
    return """
set -euo pipefail
output_root="$PWD/$OUT"
source_root="$PWD/$SRCDIR"
scratch_root="$TMP"
mkdir -p "$output_root"

export HCTL2_BUILD_CACHE="$output_root"
export HCTL2_DOWNLOAD_ROOT="$source_root/downloads"
if [[ -d "$source_root/tuwunel-rust-toolchain" ]]; then
  export HCTL2_TUWUNEL_TOOLCHAIN_ROOT="$source_root/tuwunel-rust-toolchain"
  export HCTL2_CARGO_HOME="$scratch_root/cargo-home"
fi

bash "$source_root/{bootstrap}"

target_root="$output_root/{package_target}"
mkdir -p "$target_root/downloads"
cp -aL "$source_root/downloads/." "$target_root/downloads/"
if [[ -d "$target_root/tmp" ]]; then
  find "$target_root/tmp" -mindepth 1 -depth -delete
fi
for cargo_target in "$target_root"/vendor/tuwunel-target-*; do
  if [[ -d "$cargo_target" ]]; then
    find "$cargo_target" -depth -delete
  fi
done
""".format(
        bootstrap = spec["bootstrap"],
        package_target = spec["package_target"],
    )

def declare_external_dependencies(build_sources: dict):
    for name, asset in LOCK["common"].items():
        _declare_http_file(_asset_target_name("common", name), asset)
    for target, spec in LOCK["targets"].items():
        for name, asset in spec["assets"].items():
            _declare_http_file(_asset_target_name(target, name), asset)

    _declare_tuwunel_rust_components()

    native.genrule(
        name = "prepared",
        srcs = select({
            "prelude//os:linux": select({
                "prelude//cpu:x86_64": _asset_sources("linux_x86_64", build_sources),
            }),
            "prelude//os:macos": select({
                "prelude//cpu:arm64": _asset_sources("macos_arm64", build_sources),
                "prelude//cpu:x86_64": _asset_sources("macos_x86_64", build_sources),
            }),
        }),
        bash = select({
            "prelude//os:linux": select({
                "prelude//cpu:x86_64": _prepared_command("linux_x86_64"),
            }),
            "prelude//os:macos": select({
                "prelude//cpu:arm64": _prepared_command("macos_arm64"),
                "prelude//cpu:x86_64": _prepared_command("macos_x86_64"),
            }),
        }),
        out = "hctl2-build-cache",
        cacheable = True,
        visibility = ["PUBLIC"],
    )
