# Build-only prebuilt tool; provenance and layout: docs/research/build-tools/xz.md.
XZ_VERSION = "5.8.4"
XZ_DIRECTORY = "v" + XZ_VERSION

_ASSETS = {
    "linux_x86_64": ("linux/x86-64", "9a6341f4993aeef3365b700858983fe504d5ac9a0b94126d76d71084f8b7d720"),
    "macos_arm64": ("darwin/aarch64", "45bac764305accc818c9cea5340361a120464a715d8022bd6293ef44f5d71120"),
    "macos_x86_64": ("darwin/x86-64", "d93afe9dbb1b19e4e60c336d6f0893e90be3815c025d482a42e6bdd72bb4b63f"),
}

def declare_xz():
    for target, (platform, sha256) in _ASSETS.items():
        native.http_archive(
            name = "xz-" + target,
            urls = ["https://dist.pkgx.dev/tukaani.org/xz/{}/{}.tar.xz".format(platform, XZ_DIRECTORY)],
            sha256 = sha256,
            strip_prefix = "tukaani.org/xz",
            type = "tar.xz",
        )

    native.alias(
        name = "xz",
        actual = select({
            "prelude//os:linux": select({
                "prelude//cpu:x86_64": ":xz-linux_x86_64",
            }),
            "prelude//os:macos": select({
                "prelude//cpu:arm64": ":xz-macos_arm64",
                "prelude//cpu:x86_64": ":xz-macos_x86_64",
            }),
        }),
        visibility = ["PUBLIC"],
    )
