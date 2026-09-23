// Command proxy fetches one compact quote through a proxy-shaped client: a
// base URL with its own path, a custom path prefix, a tenant header on every
// request, and a proxy token in a caller-selected header. It runs offline
// against an in-process TLS server and prints no credential. To use a real
// proxy, build the options from FMP_PROXY_BASE_URL, FMP_PROXY_PATH_PREFIX,
// FMP_TENANT, and FMP_PROXY_TOKEN, and drop WithHTTPClient.
package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"net/http/httptest"
	"time"

	fmp "github.com/bitbrew-dev/libfmp/sdk/go"
)

const (
	// proxyToken is a placeholder, never a real credential.
	proxyToken = "example-key"
	tenant     = "acme"
	pathPrefix = "v1"
)

// quoteShortFixture is a copy of crates/libfmp/tests/fixtures/quote_short.json,
// the documented provider response the Rust and Go parity tests both decode.
const quoteShortFixture = `[
  {
    "change": -6.33498,
    "price": 331.85501,
    "symbol": "AAPL",
    "volume": 28718014
  }
]`

func main() {
	if err := run(); err != nil {
		log.Fatal(err)
	}
}

func run() error {
	server := httptest.NewTLSServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		switch {
		case r.URL.Path != "/router/"+pathPrefix+"/quote-short":
			http.NotFound(w, r)
		case r.Header.Get("X-Proxy-Token") != proxyToken:
			w.WriteHeader(http.StatusUnauthorized)
			_, _ = fmt.Fprint(w, `{"error":"proxy token rejected"}`)
		case r.Header.Get("X-Tenant") != tenant:
			w.WriteHeader(http.StatusForbidden)
			_, _ = fmt.Fprint(w, `{"error":"unknown tenant"}`)
		default:
			_, _ = fmt.Fprint(w, quoteShortFixture)
		}
	}))
	defer server.Close()

	client, err := fmp.NewClient(
		fmp.WithBaseURL(server.URL+"/router"),
		fmp.WithPathPrefix(pathPrefix),
		fmp.WithDefaultHeader("X-Tenant", tenant),
		fmp.WithAuthentication(fmp.CustomHeader("X-Proxy-Token", proxyToken)),
		fmp.WithHTTPClient(server.Client()),
		fmp.WithTimeout(10*time.Second),
	)
	if err != nil {
		return err
	}

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	quotes, err := client.Quote.Short(ctx, fmp.NewQuoteShortQuery("AAPL"))
	if err != nil {
		return err
	}
	if len(quotes) != 1 {
		return fmt.Errorf("expected one quote, got %d", len(quotes))
	}

	quote := quotes[0]
	fmt.Printf("client: %v\n", client)
	fmt.Printf("%s via tenant %s: price %.5f, change %.5f, volume %.0f\n",
		quote.Symbol, tenant, quote.Price, quote.Change, quote.Volume)
	return nil
}
