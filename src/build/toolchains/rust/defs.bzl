load("@prelude//rust:rust_toolchain.bzl", "PanicRuntime", "RustToolchainInfo")

RUST_VERSION = "1.98.0"
RUST_RELEASE_DATE = "2026-08-20"

RUST_RELEASES = {
    "x86_64-unknown-linux-gnu": struct(
        rustc_sha256 = "0e37cb339f447fc44d6d781073bacacebfdc5612f2600e4c7e84c266f5f3aced",
        std_sha256 = "f5022e6c95a5ad23cca2513dc8281200f585fa188de6370aa37b128a43f876a3",
        clippy_sha256 = "646c6bd2450ea4c32a3c78ad7a3727294eea1f1519718a3f1487424021695ffa",
        rustfmt_sha256 = "8922a03e68265a74a8040590e28b173fb5d9d49655bb6eacfd880fbd3bef0dc3",
    ),
    "x86_64-apple-darwin": struct(
        rustc_sha256 = "c82d8f536955a9d6fc4465637fce5dcacf1d3913a98b5eb7edd9ead5a8b3f509",
        std_sha256 = "8923fa9d0e0407b8a492e59d568e7aceff75949e1eee2e3e1b60e174373890cb",
        clippy_sha256 = "e4c976879ad68e093522c1079a6c45beb1cdad059fe5b564f8c3950e1a4e0758",
        rustfmt_sha256 = "0945a6319424dfb3fb711477de73b4f3f913b2c4a549a4b5a959086f1f19ecd4",
    ),
    "aarch64-apple-darwin": struct(
        rustc_sha256 = "287edbc2e285b9c23ef7b085413b90cb8539909eda9c4b49f2a55ec0b52819d4",
        std_sha256 = "48c05269ed36fb5f0a8438065156891fe59fcb868d90cfed9b7209540d54b2ce",
        clippy_sha256 = "5916330cbbf3a46c172123287f370810cf28b9a0013b971c764054846e7f920a",
        rustfmt_sha256 = "cec1b153c2331a94a6c8344ac54887917d89a0eef935d9e827bb553713b2f9fa",
    ),
}

def _component_url(component: str, triple: str) -> str:
    return "https://static.rust-lang.org/dist/{}/{}-{}-{}.tar.xz".format(
        RUST_RELEASE_DATE,
        component,
        RUST_VERSION,
        triple,
    )

def _declare_rust_archives(name: str, triple: str):
    release = RUST_RELEASES[triple]

    native.http_archive(
        name = "rustc_dist_{}".format(name),
        urls = [_component_url("rustc", triple)],
        sha256 = release.rustc_sha256,
        strip_prefix = "rustc-{}-{}".format(RUST_VERSION, triple),
        type = "tar.xz",
    )
    native.http_archive(
        name = "rust_std_dist_{}".format(name),
        urls = [_component_url("rust-std", triple)],
        sha256 = release.std_sha256,
        strip_prefix = "rust-std-{}-{}".format(RUST_VERSION, triple),
        type = "tar.xz",
    )
    native.http_archive(
        name = "clippy_dist_{}".format(name),
        urls = [_component_url("clippy", triple)],
        sha256 = release.clippy_sha256,
        strip_prefix = "clippy-{}-{}".format(RUST_VERSION, triple),
        type = "tar.xz",
    )
    native.http_archive(
        name = "rustfmt_dist_{}".format(name),
        urls = [_component_url("rustfmt", triple)],
        sha256 = release.rustfmt_sha256,
        strip_prefix = "rustfmt-{}-{}".format(RUST_VERSION, triple),
        type = "tar.xz",
    )

def _rust_toolchain_impl(ctx: AnalysisContext) -> list[Provider]:
    rustc_dist = ctx.attrs.rustc_distribution[DefaultInfo].default_outputs[0]
    std_dist = ctx.attrs.standard_library_distribution[DefaultInfo].default_outputs[0]
    clippy_dist = ctx.attrs.clippy_distribution[DefaultInfo].default_outputs[0]
    triple = ctx.attrs.target_triple

    compiler_path = rustc_dist.project("rustc/bin/rustc")
    rustdoc_path = rustc_dist.project("rustc/bin/rustdoc")
    clippy_path = clippy_dist.project("clippy-preview/bin/clippy-driver")
    compiler_lib = rustc_dist.project("rustc/lib")

    clippy_wrapper, _ = ctx.actions.write(
        "clippy-driver",
        [
            "#!/bin/sh",
            cmd_args(compiler_lib, format = "export DYLD_LIBRARY_PATH=\"{}\""),
            cmd_args(compiler_lib, format = "export LD_LIBRARY_PATH=\"{}\""),
            cmd_args(clippy_path, format = "exec {} \"$@\""),
        ],
        allow_args = True,
        is_executable = True,
    )

    sysroot = ctx.actions.declare_output("rust-sysroot", dir = True)
    ctx.actions.run(
        cmd_args(
            "/bin/sh",
            ctx.attrs.merge_sysroot,
            rustc_dist,
            std_dist,
            sysroot.as_output(),
            triple,
        ),
        category = "rust_sysroot",
        identifier = triple,
    )

    return [
        DefaultInfo(),
        RustToolchainInfo(
            compiler = RunInfo(args = cmd_args(compiler_path, hidden = [rustc_dist, std_dist])),
            rustdoc = RunInfo(args = cmd_args(rustdoc_path, hidden = [rustc_dist, std_dist])),
            clippy_driver = RunInfo(args = cmd_args(clippy_wrapper, hidden = [clippy_path, compiler_lib, std_dist])),
            default_edition = "2024",
            rustc_target_triple = triple,
            panic_runtime = PanicRuntime("unwind"),
            sysroot_path = sysroot,
            nightly_features = False,
            deny_lints = ["unsafe_code"],
        ),
    ]

_rust_toolchain = rule(
    impl = _rust_toolchain_impl,
    attrs = {
        "rustc_distribution": attrs.exec_dep(providers = [DefaultInfo]),
        "standard_library_distribution": attrs.exec_dep(providers = [DefaultInfo]),
        "clippy_distribution": attrs.exec_dep(providers = [DefaultInfo]),
        "merge_sysroot": attrs.source(),
        "target_triple": attrs.string(),
    },
    is_toolchain_rule = True,
)

def _rustc_impl(ctx: AnalysisContext) -> list[Provider]:
    rustc_dist = ctx.attrs.rustc_distribution[DefaultInfo].default_outputs[0]
    compiler_path = rustc_dist.project("rustc/bin/rustc")
    return [
        DefaultInfo(default_output = compiler_path),
        RunInfo(args = cmd_args(compiler_path, hidden = [rustc_dist])),
    ]

_rustc = rule(
    impl = _rustc_impl,
    attrs = {
        "rustc_distribution": attrs.exec_dep(providers = [DefaultInfo]),
    },
)

def _rustfmt_impl(ctx: AnalysisContext) -> list[Provider]:
    rustc_dist = ctx.attrs.rustc_distribution[DefaultInfo].default_outputs[0]
    rustfmt_dist = ctx.attrs.rustfmt_distribution[DefaultInfo].default_outputs[0]
    rustfmt_path = rustfmt_dist.project("rustfmt-preview/bin/rustfmt")
    compiler_lib = rustc_dist.project("rustc/lib")

    wrapper, _ = ctx.actions.write(
        "rustfmt",
        [
            "#!/bin/sh",
            cmd_args(compiler_lib, format = "export DYLD_LIBRARY_PATH=\"{}\""),
            cmd_args(compiler_lib, format = "export LD_LIBRARY_PATH=\"{}\""),
            cmd_args(rustfmt_path, format = "exec {} \"$@\""),
        ],
        allow_args = True,
        is_executable = True,
    )

    return [
        DefaultInfo(default_output = wrapper),
        RunInfo(args = cmd_args(wrapper, hidden = [rustc_dist, rustfmt_dist])),
    ]

_rustfmt = rule(
    impl = _rustfmt_impl,
    attrs = {
        "rustc_distribution": attrs.exec_dep(providers = [DefaultInfo]),
        "rustfmt_distribution": attrs.exec_dep(providers = [DefaultInfo]),
    },
)

def declare_rust_toolchains():
    triples = {
        "linux_x86_64_gnu": "x86_64-unknown-linux-gnu",
        "macos_x86_64": "x86_64-apple-darwin",
        "macos_arm64": "aarch64-apple-darwin",
    }

    for name, triple in triples.items():
        _declare_rust_archives(name, triple)
        _rust_toolchain(
            name = "rust_{}".format(name),
            rustc_distribution = ":rustc_dist_{}".format(name),
            standard_library_distribution = ":rust_std_dist_{}".format(name),
            clippy_distribution = ":clippy_dist_{}".format(name),
            merge_sysroot = "//rust:merge_sysroot.sh",
            target_triple = triple,
        )
        _rustc(
            name = "rustc_{}".format(name),
            rustc_distribution = ":rustc_dist_{}".format(name),
        )
        _rustfmt(
            name = "rustfmt_{}".format(name),
            rustc_distribution = ":rustc_dist_{}".format(name),
            rustfmt_distribution = ":rustfmt_dist_{}".format(name),
        )

    native.toolchain_alias(
        name = "rust",
        actual = select({
            "prelude//os:linux": select({
                "prelude//cpu:x86_64": ":rust_linux_x86_64_gnu",
            }),
            "prelude//os:macos": select({
                "prelude//cpu:arm64": ":rust_macos_arm64",
                "prelude//cpu:x86_64": ":rust_macos_x86_64",
            }),
        }),
        visibility = ["PUBLIC"],
    )

def host_rustc(name: str):
    os = host_info().os
    arch = host_info().arch

    if os.is_linux and arch.is_x86_64:
        actual = ":rustc_linux_x86_64_gnu"
    elif os.is_macos and arch.is_x86_64:
        actual = ":rustc_macos_x86_64"
    elif os.is_macos and arch.is_aarch64:
        actual = ":rustc_macos_arm64"
    else:
        fail("Unsupported Rust build host: {} {}".format(os, arch))

    native.alias(
        name = name,
        actual = actual,
        visibility = ["PUBLIC"],
    )

def host_rustfmt(name: str):
    os = host_info().os
    arch = host_info().arch

    if os.is_linux and arch.is_x86_64:
        actual = ":rustfmt_linux_x86_64_gnu"
    elif os.is_macos and arch.is_x86_64:
        actual = ":rustfmt_macos_x86_64"
    elif os.is_macos and arch.is_aarch64:
        actual = ":rustfmt_macos_arm64"
    else:
        fail("Unsupported rustfmt host: {} {}".format(os, arch))

    native.alias(
        name = name,
        actual = actual,
        visibility = ["PUBLIC"],
    )
