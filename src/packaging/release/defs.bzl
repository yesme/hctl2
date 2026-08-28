load("//build/rules:rust.bzl", "HCTL2_VERSION")

def _release_inputs_impl(ctx):
    first_party = ctx.attrs.first_party[DefaultInfo].default_outputs
    dependencies = ctx.attrs.dependencies[DefaultInfo].default_outputs
    return [
        DefaultInfo(
            other_outputs = first_party + dependencies,
            sub_targets = {
                "first-party": [DefaultInfo(other_outputs = first_party)],
                "dependencies": [DefaultInfo(other_outputs = dependencies)],
            },
        ),
    ]

_release_inputs = rule(
    impl = _release_inputs_impl,
    attrs = {
        "dependencies": attrs.dep(),
        "first_party": attrs.dep(),
    },
)

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
    )

def complete_release_inputs(name: str):
    _release_inputs(
        name = name,
        dependencies = "root//packaging/dependencies:prepared",
        first_party = ":first-party",
        visibility = ["PUBLIC"],
    )
