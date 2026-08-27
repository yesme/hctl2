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
    )
