# libfmp

[![Crates.io](https://img.shields.io/crates/v/libfmp)](https://crates.io/crates/libfmp)
[![PyPI](https://img.shields.io/pypi/v/fmp-py-sdk)](https://pypi.org/project/fmp-py-sdk/)
[![docs.rs](https://img.shields.io/docsrs/libfmp)](https://docs.rs/libfmp)
[![License: MIT](https://img.shields.io/crates/l/libfmp)](LICENSE)

A Rust-first client for the
[Financial Modeling Prep (FMP)](https://financialmodelingprep.com/) data API,
with synchronous Python bindings distributed as `fmp-py-sdk` and imported as
`fmp`, and a pure Go module generated from the same contract. All three cover
the 271 `Client` methods across 30 namespaces of the pinned documentation
oracle. See [status and support](wiki/status.md) for the support policy,
the [stability and semver policy](wiki/status.md#stability-and-semver), and
the coverage caveat.

## Install

### Rust

```sh
cargo add libfmp
```

[Rust guide and examples](wiki/rust.md) · [API reference](https://docs.rs/libfmp)

### Python

Requires CPython 3.10 or newer.

```sh
python -m pip install fmp-py-sdk
```

[Python guide and examples](wiki/python.md) · [Package README](crates/fmp-py/README.md)

### Go

Requires Go 1.27.1; the module is pure Go with no cgo and no native library.

```sh
go get github.com/bitbrew-dev/libfmp/sdk/go@vX.Y.Z
```

Releases are user-triggered, and the first `sdk/go/v*` tag ships with the
first release after the release plumbing landed.
[Go guide and examples](wiki/go.md) · [API reference](https://pkg.go.dev/github.com/bitbrew-dev/libfmp/sdk/go)

## Documentation

| Topic | Start here |
|-------|------------|
| Language guides and examples | [Rust](wiki/rust.md) · [Python](wiki/python.md) · [Go](wiki/go.md) |
| Endpoint coverage and status | [Status and support](wiki/status.md) · [Endpoint coverage](docs/endpoint-coverage.md) · [Issues](https://github.com/bitbrew-dev/libfmp/issues) |
| Build, test, and repository layout | [Development](wiki/development.md) |
| Architecture and decisions | [Design](wiki/design.md) · [ADRs](docs/adr/) |
| Contract ambiguities | [Register](docs/contract-ambiguities.md) |
| Releasing | [Release validation](docs/releasing.md) |

The tracked `wiki/` pages hold the detailed guides.

## License

This project is available under the [MIT License](LICENSE).
