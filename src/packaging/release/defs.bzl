load("//build/rules:ci.bzl", "CI_INTEGRATION", "CI_PLATFORM", "CI_RELEASE")
load("//build/rules:rust.bzl", "HCTL2_VERSION")

_SYFT_VERSION = "1.51.1"
_SYFT_ASSETS = {
    "linux-x86_64": {
        "archive_platform": "linux_amd64",
        "sha256": "8fcb33017a0dc1058298c923c436d19dfa68ae93968e0b423248542e3afb9fc3",
    },
    "macos-aarch64": {
        "archive_platform": "darwin_arm64",
        "sha256": "ac063af3b9874769deb7ea1e6d76841e68f9e3bb50cd654226fc977de65532c1",
    },
    "macos-x86_64": {
        "archive_platform": "darwin_amd64",
        "sha256": "0e186ce1d4351ec276126851ca3ff258ed070e93e73574ed64858d4fc2339867",
    },
}

def _export_command(target: str) -> str:
    return " ".join([
        "bash $(location :export-first-party.sh)",
        '"$OUT"',
        HCTL2_VERSION,
        target,
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
        labels = ["large_copy"] + CI_PLATFORM,
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
export HCTL2_SYFT="$source_root/tools/syft/syft"
export SOURCE_DATE_EPOCH="$HCTL2_SOURCE_DATE_EPOCH"

bash "$source_root/packaging/release/assemble.sh" \
  --first-party "$source_root/first-party" \
  --agency-skills "$source_root/agency/skills" \
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

def _declare_syft():
    for target, asset in _SYFT_ASSETS.items():
        archive = "syft_{}_{}.tar.gz".format(_SYFT_VERSION, asset["archive_platform"])
        native.http_archive(
            name = "syft-{}".format(target),
            urls = [
                "https://github.com/anchore/syft/releases/download/v{}/{}".format(
                    _SYFT_VERSION,
                    archive,
                ),
            ],
            sha256 = asset["sha256"],
            type = "tar.gz",
        )

def complete_release(name: str):
    _declare_syft()

    native.genrule(
        name = name,
        srcs = {
            "build-metadata.sh": "root//packaging/dependencies:metadata",
            "dependencies": "root//packaging/dependencies:package",
            "first-party": ":first-party",
            "agency": "root//agency:skills",
            "packaging/dependencies": "root//packaging/dependencies:test-support",
            "packaging/release/PACKAGE-README.md": "PACKAGE-README.md",
            "packaging/release/assemble.sh": "assemble.sh",
            "packaging/release/install.sh": "install.sh",
            "product/Cargo.toml": "root//:Cargo.toml",
            "tools/syft": _platform_select({
                target: ":syft-{}".format(target)
                for target in _SYFT_ASSETS
            }),
        },
        bash = _platform_select({
            target: _complete_release_command(target)
            for target in ["linux-x86_64", "macos-aarch64", "macos-x86_64"]
        }),
        out = "release",
        cacheable = True,
        labels = ["large_copy"] + CI_RELEASE,
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
        labels = CI_INTEGRATION,
        run_test_separately = True,
        test_rule_timeout_ms = 600000,
        visibility = ["PUBLIC"],
    )
