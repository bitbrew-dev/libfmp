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

## Stability and semver

libfmp is pre-1.0. Breaking changes still ship as minor versions until the
maintainer cuts 1.0; each one carries a `BREAKING CHANGE:` note in the
release notes. From 1.0 on, the rules below apply as written.

### What is public API

| SDK | Public | Not public |
|-----|--------|------------|
| Rust | items reachable from the `libfmp` crate root: `Client`, `ClientBuilder`, endpoint functions, query and response types, `Error` and its categories | `#[doc(hidden)]` items, private modules, `Debug` output |
| Python | names exported from `fmp` and its domain packages, as typed in the shipped `.pyi` stubs | `fmp._native`, `repr` output |
| Go | exported identifiers of `github.com/bitbrew-dev/libfmp/sdk/go` | unexported identifiers, `String()` output |

In every SDK, error message text is not stable: match on the category
(`Validation`, `Configuration`, `Transport`, `Status`, `Decode`) and the
structured fields, not on the message.

### Change classes

| Change | Release |
|--------|---------|
| New endpoint, new method, new option | minor |
| New field on a response struct (Rust response structs are `#[non_exhaustive]`) | minor |
| Widening a response field type (`i64` to `f64`) or `T` to `Option<T>` to match observed provider data | minor, with a changelog notice naming the field |
| Raising the Rust MSRV, the Python floor, or the Go version | minor |
| Bug fix that keeps the public signature | patch |
| Removing or renaming a public item, or narrowing a type | major |
| Changing the meaning of a field or the error category of a condition | major |

A field widening is a minor because the provider, not the SDK, changed the
contract: the old type already rejected real responses. The decisions behind
these rules are in
[ADR 0031](../docs/adr/0031-pre-1.0-contract-decisions.md).

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
