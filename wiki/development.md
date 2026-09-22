# Development

[Project index](../README.md)

## Toolchains

| Toolchain | Pin | Where |
|-----------|-----|-------|
| Rust | 1.96.0 | [`rust-toolchain.toml`](../rust-toolchain.toml); plain `cargo` picks it up |
| Python | CPython 3.10 or newer | `abi3-py310` wheels; Rust is only needed to build from source |
| Go | 1.27.1 | [`sdk/go/go.mod`](../sdk/go/go.mod); the checks pin it exactly |
| golangci-lint | v2.13.2 | the `go-sdk.yml` workflow; `scripts/check_go_sdk.sh` requires a v2 release |

## Build and test

```sh
cargo build                  # whole workspace
cargo test                   # Rust unit, fixture, and integration tests
cargo fmt
cargo clippy --all-targets
```

The repository hooks in `.pre-commit-config.yaml` run through `prek`. They
cover Rust formatting, check, clippy, and tests, the generic file hooks,
the Go SDK gate, and the commit-message check:

```sh
prek run --all-files
```

The Go SDK gate on its own, from the repository root:

```sh
bash scripts/check_go_sdk.sh
```

It pins Go 1.27.1 and golangci-lint v2, then runs `gofmt`, `go vet`,
`golangci-lint run`, `go test -race -count=1`, and every example under
`sdk/go/examples/`, all with `CGO_ENABLED=0`.

## Registry and generators

The Python namespaces, the Python models, and the Go module are generated
from the registry under `crates/fmp-py-gen/registry/` (one TOML file per
domain) and the `libfmp` sources. Validate the registry against the real
`libfmp` signatures first:

```sh
cargo run -p fmp-py-gen --bin registry_check
```

A clean run ends with these three lines:

```
registry ok: 271 verified, 0 trusted
wire ok: 271 resolved, 0 unresolved
metadata ok: 251 descriptors carry advisory metadata
```

Regenerate the Go module after a registry or response-model change:

```sh
cargo run -p fmp-py-gen --bin gen_go
```

The Python generators (`gen_models`, `gen_namespaces`, `stub_gen`) and the
`gen_go` flags are documented in the fmp-py README, from
[Regenerating the response models](../crates/fmp-py/README.md#regenerating-the-response-models)
through
[Regenerating the stubs and public packages](../crates/fmp-py/README.md#regenerating-the-stubs-and-public-packages).
Every generator is idempotent: a second run leaves `git status` clean.

## Project layout

```
crates/libfmp/                     the async Rust core: client, transport, endpoints, models
crates/libfmp/tests/fixtures/      recorded wire fixtures shared by every decoder
crates/fmp-py/                     pyo3 extension and the `fmp` package         (on PyPI as fmp-py-sdk)
crates/fmp-py-gen/                 registry_check, gen_models, gen_namespaces, gen_go
crates/fmp-py-gen/registry/        one TOML file per domain: the endpoint registry
sdk/go/                            pure Go module generated from the registry
docs/                              ADRs, endpoint coverage, contract ambiguities, releasing
scripts/                           release, coverage, and Go SDK gate scripts
artifacts/documentations/outer.md  captured FMP documentation oracle              (git-ignored)
```

The `artifacts/` tree is git-ignored: the oracle it holds is the captured
provider documentation that the registry, the coverage table, and the
[contract ambiguities register](../docs/contract-ambiguities.md) refer to.
