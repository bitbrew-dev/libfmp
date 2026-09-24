# Go guide

[README](../README.md) | [Python](python.md) | [Rust](rust.md) | Go

The Go module is a pure Go client for the same API. It mirrors the Rust crate
one-for-one: the same client options, authentication modes, error categories,
and transport rules, with all 271 methods across 30 namespaces generated from
the Rust contract. It requires Go 1.27 or later (CI builds with toolchain 1.27.1) and builds with `CGO_ENABLED=0`:
there is no cgo and no native library. The module path ends in `go`, so
import it with an alias:

```go
import fmp "github.com/bitbrew-dev/libfmp/sdk/go"
```

Start with [installation](../README.md#install). The module shares the
libfmp workspace version and every release tags `sdk/go/vX.Y.Z`; the
[Installation](../sdk/go/README.md#installation) section of the Go README
lists the published versions and the private-repository setup.

## Building a client

`NewClient` takes functional options (`WithAuthentication`, `WithTimeout`,
`WithBaseURL`, `WithPathPrefix`, `WithDefaultHeader`, `WithHTTPClient`) and
validates every one before any request is made. The client never reads the
environment on its own: `FmpHeaderFromEnv` is the explicit opt-in for
`FMP_API_KEY`. Header modes never place a credential in the URL; query modes
append the secret as the last query pair. See
[Building a client](../sdk/go/README.md#building-a-client) for the option
list and the environment variables the live tests read.

## Custom routers and proxies

`WithBaseURL` and `WithPathPrefix` route the client through a proxy, and
`CustomHeaderWithPrefix("X-Proxy-Token", "Bearer ", secret)` sends a prefixed
proxy token without a separator of its own. The full proxy example, with a
`router/stable` path prefix and an optional `X-Tenant` header, is in the Go
README under [Building a client](../sdk/go/README.md#building-a-client).

## Errors

Every client operation returns a `*fmp.Error`; match it with `errors.As`.
`Category` is one of `Validation`, `Configuration`, `Transport`, `Status`, or
`Decode`. The client never retries, and it formats `Authentication`, `Client`,
and `Error` without any secret value, so an error is safe to log. See
[Errors](../sdk/go/README.md#errors).

## Endpoint metadata

`EndpointMetadataFor("Quote.Full")` returns the advisory metadata a Rust
descriptor attaches to an endpoint (geography, access, conditional plan,
realtime, bounds), keyed by the Go call path without the client. The client
never validates a request against it, exactly as the Rust crate does not.
`EndpointMetadataByID(err.Endpoint)` resolves the endpoint id an `*Error`
carries to one entry per method that sends it, since a shared id such as
`quote` can carry different metadata per namespace. See
[Endpoint metadata](../sdk/go/README.md#endpoint-metadata).

## Examples and tests

- [`examples/quote`](../sdk/go/examples/quote/main.go) fetches one detailed quote and [`examples/proxy`](../sdk/go/examples/proxy/main.go) routes through a proxy; both run offline against an in-process TLS server. See [Examples](../sdk/go/README.md#examples).
- [`example_client_test.go`](../sdk/go/example_client_test.go) and [`example_error_test.go`](../sdk/go/example_error_test.go) hold the godoc `Example*` functions shown on pkg.go.dev.
- The opt-in live tests skip unless `FMP_LIVE_TESTS` is `1`, so the checks never open a socket by default. See [Live tests](../sdk/go/README.md#live-tests).
- `bash scripts/check_go_sdk.sh` from the repository root is the full gate: `gofmt`, `go vet`, `golangci-lint`, `go test -race`, and every example.

## Reference

- [Go API reference](https://pkg.go.dev/github.com/bitbrew-dev/libfmp/sdk/go) on pkg.go.dev.
- [Go SDK README](../sdk/go/README.md): the full guide, packaged with the module.
- [ADR 0030](../docs/adr/0030-go-sdk-architecture.md): the Go SDK design record.
