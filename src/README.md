# HCTL2 product workspace

This directory contains all HCTL2 product code and product tests. The repository root remains
the home of product and design documentation.

The initial P1 workspace contains only the standalone mechanical components:

- `hctl2-tool`: Git/SCM and repository mechanics;
- `hctl2-agentd`: harness discovery, runtime ownership, PTY, and host observation;
- `hctl2-protocol`: transport envelopes shared by HCTL2 processes.

`hctl2-control`, the public `hctl2` CLI, and Workbench enter in P2/P3. Neither P1 executable is a
governance command entry.

Run the workspace checks from this directory:

```bash
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo build --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
```
