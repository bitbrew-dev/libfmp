# Go SDK

The Go module is a pure Go client for the Financial Modeling Prep API. It
mirrors the Rust crate `libfmp` one-for-one: the same client options, the same
authentication modes, the same error categories, and the same transport rules.
Endpoint models and methods are generated from the Rust contract by
`cargo run -p fmp-py-gen --bin gen_go`, so the two SDKs decode the same
fixtures; the generated files carry a `DO NOT EDIT` header. It requires **Go 1.27.1** and builds with
`CGO_ENABLED=0`; there is no cgo and no native library. The design is recorded
in [ADR 0030](../../docs/adr/0030-go-sdk-architecture.md).

## Installation

The module path ends in `go`, so import it with an alias:

```go
import fmp "github.com/bitbrew-dev/libfmp/sdk/go"
```

The Go module shares the libfmp workspace version: every release tags
`sdk/go/vX.Y.Z` at the release commit (the tag carries the subdirectory
prefix that Go requires for a nested module, while the module version stays
`vX.Y.Z`), so install a release with:

```sh
go get github.com/bitbrew-dev/libfmp/sdk/go@vX.Y.Z
```

Because the version is shared, a `feat(sdk/go)` commit minor-bumps the
workspace and republishes `libfmp` and `fmp-py` with no Rust change; ADR 0030
accepts that consequence. Releases are user-triggered, and the first
`sdk/go/v*` tag ships with the first release after the release plumbing
landed. Check which versions have been published with:

```sh
go list -m -versions github.com/bitbrew-dev/libfmp/sdk/go
```

While that list is empty, use a source checkout of this repository and point
your module at it with a `replace` directive:

```
require github.com/bitbrew-dev/libfmp/sdk/go v0.0.0

replace github.com/bitbrew-dev/libfmp/sdk/go => ../libfmp/sdk/go
```

## Building a client

`NewClient` validates every option before any request is made. Direct access
to the FMP origin requires explicit authentication; the client never reads the
environment on its own, so opt in with `FmpHeaderFromEnv`:

```go
auth, ok := fmp.FmpHeaderFromEnv()
if !ok {
	return errors.New("FMP_API_KEY is not set")
}
client, err := fmp.NewClient(
	fmp.WithAuthentication(auth),
	fmp.WithTimeout(20*time.Second),
)
if err != nil {
	return err
}
```

Header modes (`FmpHeader`, `Bearer`, `CustomHeader`, `CustomHeaderWithPrefix`)
never place a credential in the URL. Query modes (`FmpQuery`, `CustomQuery`)
append the secret as the last query pair of every request. `WithBaseURL` and
`WithPathPrefix` route the client through a proxy; `WithHTTPClient` injects a
caller-owned `*http.Client` for tests and custom transports.

`CustomHeaderWithPrefix(name, prefix, secret)` places non-secret text
immediately before the secret, byte for byte: no separator is inserted, so a
`"Bearer "` prefix carries its own trailing space. It mirrors the Rust
`Authentication::custom_header(name, Some(prefix), secret)` form; an empty
prefix is the same value as `CustomHeader`. Only the secret is redacted from
diagnostics, never the prefix, and formatting the value reports `has_prefix`
without the prefix text. A custom router or proxy that expects
`X-Proxy-Token: Bearer <token>` is configured like this (the same shape as
the Rust crate's live proxy test: the path prefix defaults to `router/stable`
and `X-Tenant` is sent only when `FMP_TENANT` is set, since an empty default
header value is accepted and sent as an empty field):

```go
pathPrefix := os.Getenv("FMP_PROXY_PATH_PREFIX")
if pathPrefix == "" {
	pathPrefix = "router/stable"
}
opts := []fmp.Option{
	fmp.WithBaseURL(os.Getenv("FMP_PROXY_BASE_URL")),
	fmp.WithPathPrefix(pathPrefix),
	fmp.WithAuthentication(fmp.CustomHeaderWithPrefix(
		"X-Proxy-Token", "Bearer ", os.Getenv("FMP_PROXY_TOKEN"),
	)),
}
if tenant := os.Getenv("FMP_TENANT"); tenant != "" {
	opts = append(opts, fmp.WithDefaultHeader("X-Tenant", tenant))
}
client, err := fmp.NewClient(opts...)
if err != nil {
	return err
}
```

The environment names match the Rust crate's live opt-in tests:

| Variable | Read by |
| --- | --- |
| `FMP_API_KEY` | `FmpHeaderFromEnv` and `FmpAPIKeyFromEnv` |
| `FMP_LIVE_TESTS` | live tests, which run only when it is `1` |
| `FMP_PROXY_BASE_URL`, `FMP_PROXY_TOKEN`, `FMP_PROXY_PATH_PREFIX`, `FMP_TENANT` | live tests against a proxy |

## Errors

Every client operation returns a `*fmp.Error`. Use `errors.As` rather than a
type assertion so wrapped errors still match:

```go
client, err := fmp.NewClient()
var fmpErr *fmp.Error
if errors.As(err, &fmpErr) {
	switch fmpErr.Category {
	case fmp.CategoryConfiguration:
		log.Printf("configuration rejected: %v", fmpErr)
	case fmp.CategoryStatus:
		log.Printf("endpoint %s returned %d: %s", fmpErr.Endpoint, fmpErr.Status, fmpErr.Body)
	default:
		log.Printf("%s: %v", fmpErr.Category, fmpErr)
	}
}
```

`Category` is one of `Validation`, `Configuration`, `Transport`, `Status`, or
`Decode`. `Endpoint` is a logical endpoint id, never a URL. `Body` is the
provider body, redacted and capped. The client never retries, wraps every
transport failure before it escapes, and formats `Authentication`, `Client`,
and `Error` without any secret value, so an error is safe to log.

## Examples

Two runnable programs and the package's godoc examples all run offline against
an in-process TLS server with placeholder credentials:
[`examples/quote`](examples/quote/main.go) fetches one detailed quote,
[`examples/proxy`](examples/proxy/main.go) routes through a proxy with a path
prefix, a tenant header, and a proxy token, and
[`example_client_test.go`](example_client_test.go) and
[`example_error_test.go`](example_error_test.go) hold the `Example*` functions
shown on pkg.go.dev (`cd sdk/go && go test -run Example ./...`).

## Checks

Run the full gate from the repository root:

```sh
bash scripts/check_go_sdk.sh
```

It pins Go 1.27.1 and golangci-lint v2, then runs `gofmt`, `go vet`,
`golangci-lint run`, `go test -race -count=1`, and every example under
`sdk/go/examples/`, all with `CGO_ENABLED=0`. The lint set lives in
`.golangci.yml` next to this file. The same script backs the `go-sdk-tests`
pre-commit hook, which is scoped by `files:` so Rust-only commits do not pay
for Go checks, and the `go-sdk.yml` workflow, which runs on Ubuntu and macOS
for changes under `sdk/go/`. Run the hook by hand with:

```sh
prek run go-sdk-tests --all-files
```

Endpoint coverage across the SDKs is tracked in
[docs/endpoint-coverage.md](../../docs/endpoint-coverage.md).
