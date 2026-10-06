# Go SDK

The Go module is a pure Go client for the Financial Modeling Prep API. It
mirrors the Rust crate `libfmp` one-for-one: the same client options, the same
authentication modes, the same error categories, and the same transport rules.
Endpoint models and methods are generated from the Rust contract by
`cargo run -p fmp-py-gen --bin gen_go`, so the two SDKs decode the same
fixtures; the generated files carry a `DO NOT EDIT` header. It requires **Go 1.27** or later and builds with
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

While this repository is private, the public module proxy cannot serve the
module, so tell Go to fetch it directly: set `GOPRIVATE=github.com/bitbrew-dev/*`
(which also disables the checksum database for that path) and give git
credentials for github.com: a token over https as below, or ssh with
`git config --global url."git@github.com:".insteadOf "https://github.com/"`.
With that environment `go get` clones the repository directly, and
`go list -m -versions` needs the same environment:

```sh
export GOPRIVATE='github.com/bitbrew-dev/*'
git config --global url."https://x-access-token:TOKEN@github.com/".insteadOf "https://github.com/"
go get github.com/bitbrew-dev/libfmp/sdk/go@vX.Y.Z
```

Because the version is shared, a `feat(sdk/go)` commit minor-bumps the
workspace and republishes `libfmp` and `fmp-py` with no Rust change; ADR 0030
accepts that consequence. Every release tags `sdk/go/vX.Y.Z`. Check which
versions have been published with:

```sh
go list -m -versions github.com/bitbrew-dev/libfmp/sdk/go
```

## Building a client

`NewClient` validates every option before any request is made. Direct access
to the FMP origin requires explicit authentication; the client never reads the
environment on its own, so opt in with `FMPHeaderFromEnv`:

```go
auth, ok := fmp.FMPHeaderFromEnv()
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

Header modes (`FMPHeader`, `Bearer`, `CustomHeader`, `CustomHeaderWithPrefix`)
never place a credential in the URL. Query modes (`FMPQuery`, `CustomQuery`)
append the secret as the last query pair of every request. `WithBaseURL` and
`WithPathPrefix` route the client through a proxy; `WithHTTPClient` injects a
caller-owned `*http.Client` for tests and custom transports.

### Using a proxy

A proxy that issues its own keys and forwards to FMP, such as
[valet](https://valet.bitbrew.app), needs only `FMP_API_KEY=vk_...` and
`FMP_BASE_URL=https://valet.bitbrew.app/fmp` in the environment:

```go
client, err := fmp.NewClientFromEnv(fmp.WithTimeout(20 * time.Second))
if err != nil {
	return err
}
```

`NewClientFromEnv` sends the key in the `apikey` header and keeps the base
URL's path, so the quote endpoint goes to
`https://valet.bitbrew.app/fmp/stable/quote`. Without `FMP_BASE_URL` it
targets the FMP origin; without `FMP_API_KEY` it returns
`ConfigurationKindMissingCredential`. Options are applied after the
environment, so `WithBaseURL` wins, but `WithAuthentication` conflicts with
the environment key (use `NewClient` to pick another mode). Plaintext
`http://` to a non-loopback host still needs
`WithDangerAllowInsecureAuthentication`. `NewClient` never reads either
variable.

`client.CloseIdleConnections()` closes the idle keep-alive connections of the
transport `NewClient` built; the client stays usable. It is a no-op with
`WithHTTPClient`, whose transport stays with the caller. There is no `Close`:
a `Client` owns nothing beyond that pool.

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

The environment names match the Rust crate (`Client::from_env`) and its live
opt-in tests:

| Variable | Read by |
| --- | --- |
| `FMP_API_KEY` | `FMPHeaderFromEnv`, `APIKeyFromEnv`, and `NewClientFromEnv` |
| `FMP_BASE_URL` | `BaseURLFromEnv` and `NewClientFromEnv` |
| `FMP_LIVE_TESTS` | live tests (`live_test.go`), which run only when it is `1` |
| `FMP_PROXY_BASE_URL`, `FMP_PROXY_TOKEN`, `FMP_PROXY_PATH_PREFIX`, `FMP_TENANT` | live tests against a proxy |

## Names

Endpoint methods, query types, and models are generated from the same
registry as the Rust and Python SDKs, so each endpoint has one name in all
three ([ADR 0032](../../docs/adr/0032-naming-policy.md)). Go spells
initialisms in all caps, generated and hand-written alike:

| Group | Initialisms |
| --- | --- |
| Web and data | `ID`, `UID`, `URL`, `JSON`, `API`, `HTTP` |
| Provider and regulators | `FMP`, `SEC`, `US`, `COT` |
| Instruments and identifiers | `ETF`, `CIK`, `CUSIP`, `ISIN`, `IPO`, `SP500` |
| Metrics | `ESG`, `DCF`, `EPS`, `TTM` |
| Form names | `8K`, `13F` |

So the Python `sec_filings.latest_8k` is `client.SECFilings.Latest8K`,
`statements.metrics.key_metrics_ttm` returns `[]fmp.KeyMetricsTTM`, and the
namespaces are `TipRanks`, `DCF`, `ESG`, and `SECFilings`. Struct tags keep
the wire names (`json:"cik"`).

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
provider body, redacted and capped. A `Status` error can carry `Status` 200
when FMP answers a success status with its own error message (the plain-text
`Invalid name`, or an object whose only member is `"Error Message"`) instead
of the documented payload. A `Decode` error also carries `Path`, the
JSON pointer of the failing member (`/37/beta`), and `DecodeKind`, the coarse
reason (`DecodeKindNull`, `DecodeKindWrongType`, ...); neither ever includes
the member value. The client never retries, wraps every
transport failure before it escapes, and formats `Authentication`, `Client`,
and `Error` without any secret value, so an error is safe to log.

## Endpoint metadata

Every endpoint method whose Rust descriptor attaches advisory metadata has an
entry in the generated `metadata_table.go`, keyed by the Go call path without
the client (`Quote.Full`, `Statements.Growth.Income`):

```go
metadata, ok := fmp.EndpointMetadataFor("TipRanks.SearchRatings")
if ok {
	log.Printf("access %s, plan %s, bounds %s", metadata.Access, metadata.ConditionalPlan, metadata.Bounds)
	if metadata.Bounds.Limit != nil && limit > *metadata.Bounds.Limit {
		limit = *metadata.Bounds.Limit
	}
}
```

`EndpointMetadata` mirrors the Rust type one-for-one (`Geography`, `Access`,
`ConditionalPlan`, `Realtime`, `Bounds`), every zero value means unspecified,
and each type has a short `String()` that is safe to log. The lookup reports
`false` with the zero value for a method without metadata and for an unknown
key. The metadata is advisory: the client never validates a request against
it, exactly as the Rust crate does not.

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

### Live tests

The opt-in live tests mirror `crates/libfmp/tests/live_opt_in.rs` and skip
unless `FMP_LIVE_TESTS` is `1` (after trimming), so the gate above never
opens a socket. Run them by hand from `sdk/go`:

```sh
FMP_LIVE_TESTS=1 FMP_API_KEY=... go test -run 'TestLive' -count=1 ./...
```

The proxy test needs `FMP_PROXY_BASE_URL` and `FMP_PROXY_TOKEN` (sent as
`X-Proxy-Token: Bearer <token>`); `FMP_PROXY_PATH_PREFIX` overrides the
`router/stable` prefix and `FMP_TENANT` adds an `X-Tenant` header. A test
skips naming the missing variable and never prints a value.

Endpoint coverage across the SDKs is tracked in
[docs/endpoint-coverage.md](../../docs/endpoint-coverage.md).
