PLATFORM_SPECS = {
    "linux_x86_64_gnu": struct(
        os = "linux",
        cpu = "x86_64",
        abi = "gnu",
        rust_target = "x86_64-unknown-linux-gnu",
        goos = "linux",
        goarch = "amd64",
        uname_os = "Linux",
        uname_arch = "x86_64",
    ),
    "macos_x86_64": struct(
        os = "macos",
        cpu = "x86_64",
        abi = None,
        rust_target = "x86_64-apple-darwin",
        goos = "darwin",
        goarch = "amd64",
        uname_os = "Darwin",
        uname_arch = "x86_64",
    ),
    "macos_arm64": struct(
        os = "macos",
        cpu = "arm64",
        abi = None,
        rust_target = "aarch64-apple-darwin",
        goos = "darwin",
        goarch = "arm64",
        uname_os = "Darwin",
        uname_arch = "arm64",
    ),
}

def hctl2_platform(name: str):
    spec = PLATFORM_SPECS[name]
    constraints = [
        "prelude//os/constraints:os[{}]".format(spec.os),
        "prelude//cpu/constraints:cpu[{}]".format(spec.cpu),
    ]
    if spec.abi != None:
        constraints.append("prelude//abi/constraints:abi[{}]".format(spec.abi))

    native.platform(
        name = name,
        constraint_values = constraints,
        visibility = ["PUBLIC"],
    )

def hctl2_host_platform(name: str):
    os = host_info().os
    arch = host_info().arch

    if os.is_linux and arch.is_x86_64:
        actual = ":linux_x86_64_gnu"
    elif os.is_macos and arch.is_x86_64:
        actual = ":macos_x86_64"
    elif os.is_macos and arch.is_aarch64:
        actual = ":macos_arm64"
    else:
        fail("Unsupported HCTL2 build host: {} {}".format(os, arch))

    native.platform(
        name = name,
        deps = [actual],
        visibility = ["PUBLIC"],
    )
