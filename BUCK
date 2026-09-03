# The `repo` cell: repository-level inputs that the product workspace (`root`
# cell at src/) consumes — release documents and the doc-lint tree. No product
# code lives here.

oncall("hctl2")

export_file(
    name = "LICENSE",
    src = "LICENSE",
    visibility = ["PUBLIC"],
)

export_file(
    name = "usage",
    src = "docs/usage.md",
    visibility = ["PUBLIC"],
)

# Everything the doc checks lint: root-level documents, docs/ and .memo/.
filegroup(
    name = "docs_tree",
    srcs = glob([
        "README.md",
        "AGENTS.md",
        "CLAUDE.md",
        "CONSTRAINTS.md",
        "WRITING-GUIDE.md",
        "LICENSE",
        "docs/**",
        ".memo/**",
    ]),
    visibility = ["PUBLIC"],
)

export_file(
    name = "pr_contract_yml",
    src = ".github/workflows/pr-contract.yml",
    visibility = ["PUBLIC"],
)
