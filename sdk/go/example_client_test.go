package fmp_test

import (
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"

	fmp "github.com/bitbrew-dev/libfmp/sdk/go"
)

// exampleKey is the placeholder credential every example sends. It is not a
// real key, and no example prints it.
const exampleKey = "example-key"

// quoteShortFixture is a copy of crates/libfmp/tests/fixtures/quote_short.json,
// the documented provider response the Rust and Go parity tests both decode.
// It is inlined so the examples stay self-contained on pkg.go.dev and when the
// module is tested outside this repository.
const quoteShortFixture = `[
  {
    "change": -6.33498,
    "price": 331.85501,
    "symbol": "AAPL",
    "volume": 28718014
  }
]`

// serveQuoteShort starts an in-process TLS server that answers GET
// <prefix>/quote-short with the quote_short.json fixture, after check has
// accepted the request. Any other path, or a rejected request, is a 404.
func serveQuoteShort(prefix string, check func(*http.Request) bool) *httptest.Server {
	return httptest.NewTLSServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != prefix+"/quote-short" || !check(r) {
			http.NotFound(w, r)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = fmt.Fprint(w, quoteShortFixture)
	}))
}

// printQuoteShort prints the first compact quote with fixed formats so the
// output is stable across platforms.
func printQuoteShort(quotes []fmp.QuoteShort) {
	if len(quotes) == 0 {
		fmt.Println("no quotes")
		return
	}
	q := quotes[0]
	fmt.Printf("%s price %.5f change %.5f volume %.0f\n", q.Symbol, q.Price, q.Change, q.Volume)
}

// ExampleNewClient builds a client against an in-process server. Drop
// WithBaseURL and WithHTTPClient to talk to the FMP origin instead.
func ExampleNewClient() {
	server := serveQuoteShort("/stable", func(r *http.Request) bool {
		return r.Header.Get("apikey") == exampleKey
	})
	defer server.Close()

	client, err := fmp.NewClient(
		fmp.WithBaseURL(server.URL),
		fmp.WithHTTPClient(server.Client()),
		fmp.WithAuthentication(fmp.FmpHeader(exampleKey)),
	)
	if err != nil {
		fmt.Println("configuration rejected:", err)
		return
	}

	quotes, err := client.Quote.Short(context.Background(), fmp.NewQuoteShortQuery("AAPL"))
	if err != nil {
		fmt.Println("request failed:", err)
		return
	}
	printQuoteShort(quotes)
	// Output:
	// AAPL price 331.85501 change -6.33498 volume 28718014
}

// ExampleFmpHeaderFromEnv reads FMP_API_KEY and sends it in the apikey
// header. NewClient never reads the environment on its own, so the opt-in is
// explicit. Formatting the Authentication never reveals the key.
func ExampleFmpHeaderFromEnv() {
	previous, had := os.LookupEnv(fmp.EnvAPIKey)
	if err := os.Setenv(fmp.EnvAPIKey, exampleKey); err != nil {
		fmt.Println("setenv failed:", err)
		return
	}
	defer func() {
		if had {
			_ = os.Setenv(fmp.EnvAPIKey, previous)
		} else {
			_ = os.Unsetenv(fmp.EnvAPIKey)
		}
	}()

	auth, ok := fmp.FmpHeaderFromEnv()
	if !ok {
		fmt.Println(fmp.EnvAPIKey, "is not set")
		return
	}
	fmt.Println("authentication:", auth)

	server := serveQuoteShort("/stable", func(r *http.Request) bool {
		return r.Header.Get("apikey") == exampleKey
	})
	defer server.Close()

	client, err := fmp.NewClient(
		fmp.WithBaseURL(server.URL),
		fmp.WithHTTPClient(server.Client()),
		fmp.WithAuthentication(auth),
	)
	if err != nil {
		fmt.Println("configuration rejected:", err)
		return
	}
	quotes, err := client.Quote.Short(context.Background(), fmp.NewQuoteShortQuery("AAPL"))
	if err != nil {
		fmt.Println("request failed:", err)
		return
	}
	printQuoteShort(quotes)
	// Output:
	// authentication: FmpHeader([REDACTED])
	// AAPL price 331.85501 change -6.33498 volume 28718014
}

// ExampleWithPathPrefix routes the client through a proxy that mounts the
// FMP endpoints under its own base path and prefix, identifies the tenant
// with a default header, and authenticates with a proxy-specific header.
func ExampleWithPathPrefix() {
	server := serveQuoteShort("/router/v1", func(r *http.Request) bool {
		return r.Header.Get("X-Tenant") == "acme" && r.Header.Get("X-Proxy-Token") == exampleKey
	})
	defer server.Close()

	client, err := fmp.NewClient(
		fmp.WithBaseURL(server.URL+"/router"),
		fmp.WithPathPrefix("v1"),
		fmp.WithDefaultHeader("X-Tenant", "acme"),
		fmp.WithAuthentication(fmp.CustomHeader("X-Proxy-Token", exampleKey)),
		fmp.WithHTTPClient(server.Client()),
	)
	if err != nil {
		fmt.Println("configuration rejected:", err)
		return
	}

	quotes, err := client.Quote.Short(context.Background(), fmp.NewQuoteShortQuery("AAPL"))
	if err != nil {
		fmt.Println("request failed:", err)
		return
	}
	printQuoteShort(quotes)
	// Output:
	// AAPL price 331.85501 change -6.33498 volume 28718014
}
