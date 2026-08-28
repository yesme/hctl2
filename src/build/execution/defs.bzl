def _local_cache_platform_impl(ctx: AnalysisContext) -> list[Provider]:
    constraints = dict()
    constraints.update(ctx.attrs.cpu_configuration[ConfigurationInfo].constraints)
    constraints.update(ctx.attrs.os_configuration[ConfigurationInfo].constraints)
    configuration = ConfigurationInfo(constraints = constraints, values = {})

    platform = ExecutionPlatformInfo(
        label = ctx.label.raw_target(),
        configuration = configuration,
        executor_config = CommandExecutorConfig(
            local_enabled = True,
            remote_enabled = False,
            remote_cache_enabled = True,
            allow_cache_uploads = True,
            max_cache_upload_mebibytes = 2048,
        ),
    )
    return [
        DefaultInfo(),
        ExecutionPlatformRegistrationInfo(platforms = [platform]),
    ]

_local_cache_platform = rule(
    impl = _local_cache_platform_impl,
    attrs = {
        "cpu_configuration": attrs.dep(providers = [ConfigurationInfo]),
        "os_configuration": attrs.dep(providers = [ConfigurationInfo]),
    },
)

def hctl2_local_cache_platforms(name: str):
    host = host_info()
    if host.os.is_macos:
        os_configuration = "prelude//os:macos"
    elif host.os.is_linux:
        os_configuration = "prelude//os:linux"
    else:
        fail("Unsupported HCTL2 build host OS: {}".format(host.os))

    if host.arch.is_aarch64:
        cpu_configuration = "prelude//cpu:arm64"
    elif host.arch.is_x86_64:
        cpu_configuration = "prelude//cpu:x86_64"
    else:
        fail("Unsupported HCTL2 build host architecture: {}".format(host.arch))

    _local_cache_platform(
        name = name,
        cpu_configuration = cpu_configuration,
        os_configuration = os_configuration,
        visibility = ["PUBLIC"],
    )
