load("//build/rules:rust.bzl", "HCTL2_VERSION")

def _export_command(target: str) -> str:
    return " ".join([
        "bash $(location :export-first-party.sh)",
        '"$OUT"',
        HCTL2_VERSION,
        target,
        "$(location root//apps/hctl2-agentd:hctl2-agentd)",
        "$(location root//apps/hctl2-tool:hctl2-tool)",
    ])

def first_party_release(name: str):
    native.genrule(
        name = name,
        out = "hctl2-first-party",
        cmd = select({
            "prelude//os:linux": select({
                "prelude//cpu:x86_64": _export_command("linux-x86_64"),
            }),
            "prelude//os:macos": select({
                "prelude//cpu:arm64": _export_command("macos-aarch64"),
                "prelude//cpu:x86_64": _export_command("macos-x86_64"),
            }),
        }),
        visibility = ["PUBLIC"],
        cacheable = True,
        labels = ["large_copy"],
    )

def _complete_release_command(target: str) -> str:
    package_id = "hctl2-{}-{}".format(HCTL2_VERSION, target)
    return """
set -euo pipefail
source_root="$PWD/$SRCDIR"
output_root="$PWD/$OUT"
mkdir -p "$output_root"

source "$source_root/build-metadata.sh"
export HCTL2_PRODUCT_ROOT="$source_root/product"
export HCTL2_DEPENDENCY_SOURCE_ROOT="$source_root/packaging/dependencies"
export SOURCE_DATE_EPOCH="$HCTL2_SOURCE_DATE_EPOCH"

bash "$source_root/packaging/release/assemble.sh" \
  --first-party "$source_root/first-party" \
  --dependencies "$source_root/dependencies/{package_id}.tar.gz" \
  --sources "$source_root/dependencies/{package_id}-sources.tar.gz" \
  --output "$output_root"
""".format(package_id = package_id)

def _platform_select(values: dict):
    return select({
        "prelude//os:linux": select({
            "prelude//cpu:x86_64": values["linux-x86_64"],
        }),
        "prelude//os:macos": select({
            "prelude//cpu:arm64": values["macos-aarch64"],
            "prelude//cpu:x86_64": values["macos-x86_64"],
        }),
    })

def complete_release(name: str):
    native.genrule(
        name = name,
        srcs = {
            "build-metadata.sh": "root//packaging/dependencies:metadata",
            "dependencies": "root//packaging/dependencies:package",
            "first-party": ":first-party",
            "packaging/dependencies": "root//packaging/dependencies:test-support",
            "packaging/release/PACKAGE-README.md": "PACKAGE-README.md",
            "packaging/release/assemble.sh": "assemble.sh",
            "packaging/release/install.sh": "install.sh",
            "product/Cargo.toml": "root//:Cargo.toml",
        },
        bash = _platform_select({
            target: _complete_release_command(target)
            for target in ["linux-x86_64", "macos-aarch64", "macos-x86_64"]
        }),
        out = "release",
        cacheable = True,
        labels = ["large_copy"],
        tests = [":{}-test".format(name)],
        visibility = ["PUBLIC"],
    )

    native.sh_test(
        name = "{}-test".format(name),
        test = "test-package.sh",
        args = ["$(location :{})".format(name)],
        env = {
            "HCTL2_BUILD_METADATA": "$(location root//packaging/dependencies:metadata)",
            "HCTL2_DEPENDENCY_SOURCE_ROOT": "$(location root//packaging/dependencies:test-support)",
        },
        resources = [
            ":{}".format(name),
            "root//packaging/dependencies:metadata",
            "root//packaging/dependencies:test-support",
        ],
        run_test_separately = True,
        test_rule_timeout_ms = 600000,
        visibility = ["PUBLIC"],
    )
