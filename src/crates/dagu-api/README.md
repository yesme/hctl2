# `hctl2-dagu-api`

Dagu v1 OpenAPI generated client (`progenitor 0.14.0`), controlled workflow engine port models (`Run–Engine Binding`, `CheckpointObservation`, `DivergenceRecord`), and the loopback-only Dagu adapter/driver.

## OpenAPI Input & License

- **Upstream Specification**: [`api/v1/api.yaml`](./api/v1/api.yaml) from [`dagucloud/dagu` `v2.15.1` (`532c512944b2e5eb8991b5bc7cbeafa74fd5b47a`)](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/api/v1/api.yaml), OpenAPI `3.0.0`, `info.version: "1.0.0"`.
- **License**: `api/v1/api.yaml` retains its upstream `GPL-3.0` declaration. Per the 2026-09-04 owner ruling recorded in [`docs/research/sdk/dagu.md`](../../../docs/research/sdk/dagu.md), client code generated from an interface specification does not constitute a derivative work and is checked into the repository (`src/generated.rs`) with reproducible build verification.
- **Generator**: [`progenitor 0.14.0`](https://crates.io/crates/progenitor) (`MPL-2.0`), invoked by [`codegen/main.rs`](./codegen/main.rs).

## Reproducing `src/generated.rs`

To regenerate `src/generated.rs` from `api/v1/api.yaml` and verify byte-for-byte reproducibility:

```bash
cd src
./buck2 build root//crates/dagu-api:generated_check
```
