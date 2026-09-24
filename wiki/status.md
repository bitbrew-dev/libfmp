# Status and support

[Project index](../README.md)

## Support policy

| Language | Supported | Pinned for development and release |
|----------|-----------|------------------------------------|
| Rust | 1.96 or newer | 1.96.0 (`rust-toolchain.toml`) |
| Python | CPython 3.10 or newer, stable ABI from 3.10 (`abi3-py310`) | one wheel per platform |
| Go | 1.27 or newer, pure Go, `CGO_ENABLED=0` | toolchain 1.27.1; golangci-lint v2.13.2 for the lint gate |

## Versions and releases

The Cargo workspace version in `Cargo.toml` is the single source of truth.
The `libfmp` crate on crates.io, the `fmp-py-sdk` wheel on PyPI, and the Go
module all carry that same version: `scripts/set-version.sh` writes it to
`Cargo.toml`, `Cargo.lock`, and `sdk/go/version.go` on every release.

Releases are user-dispatched. `semantic-release` derives the version and the
GitHub release notes from the conventional commit messages on `main`, then
the same workflow publishes the crate and the wheels and tags the Go module
as `sdk/go/vX.Y.Z` at the release commit. Because the version is shared, a
`feat(sdk/go)` commit minor-bumps the workspace and republishes the crate and
the wheel with no Rust change.
[docs/releasing.md](../docs/releasing.md) holds the artifact checks, the
registry name checks, the release-notes checklist, and the Go module
checklist.

## Endpoint coverage

| Count | Value |
|-------|-------|
| Documented oracle entries | 276 |
| Rust `Client` methods | 271 |
| Python methods | 271 across 30 namespaces |
| Go methods | 271 across 30 namespaces |

This is coverage of the pinned oracle in the repository's captured API
documentation, not a claim that every endpoint currently or historically
offered by the provider is covered.
[docs/endpoint-coverage.md](../docs/endpoint-coverage.md) lists every entry
with its supported, raw, gated, or deferred state, and is regenerated from
the registry before each release.

## License

This project is available under the [MIT License](../LICENSE).
