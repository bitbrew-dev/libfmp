// Command quote fetches one detailed quote through the Go SDK against an
// in-process TLS server, so it runs offline and prints no credential. Replace
// the server with the real origin by building the client from
// fmp.FMPHeaderFromEnv and dropping WithBaseURL and WithHTTPClient.
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

// quoteFixture is a copy of crates/libfmp/tests/fixtures/quote.json, the
// documented provider response the Rust and Go parity tests both decode.
const quoteFixture = `[
  {
    "change": -6.33498,
    "changePercentage": -1.8732,
    "dayHigh": 334.48,
    "dayLow": 329.59,
    "exchange": "NASDAQ",
    "marketCap": 4874072686740,
    "name": "Apple Inc.",
    "open": 333.13,
    "previousClose": 338.18999,
    "price": 331.85501,
    "priceAvg200": 277.21344,
    "priceAvg50": 308.5888,
    "symbol": "AAPL",
    "timestamp": 1785430812,
    "volume": 28718014,
    "yearHigh": 344.57,
    "yearLow": 201.5
  }
]`

func main() {
	if err := run(); err != nil {
		log.Fatal(err)
	}
}

func run() error {
	server := httptest.NewTLSServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/stable/quote" || r.URL.Query().Get("symbol") != "AAPL" {
			http.NotFound(w, r)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = fmt.Fprint(w, quoteFixture)
	}))
	defer server.Close()

	client, err := fmp.NewClient(
		fmp.WithBaseURL(server.URL),
		fmp.WithHTTPClient(server.Client()),
		fmp.WithAuthentication(fmp.FMPHeader("example-key")),
		fmp.WithTimeout(10*time.Second),
	)
	if err != nil {
		return err
	}

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	quotes, err := client.Quote.Full(ctx, fmp.NewQuoteQuery("AAPL"))
	if err != nil {
		return err
	}
	if len(quotes) != 1 {
		return fmt.Errorf("expected one quote, got %d", len(quotes))
	}

	quote := quotes[0]
	fmt.Printf("%s (%s) on %s\n", quote.Symbol, quote.Name, quote.Exchange)
	fmt.Printf("  price %.5f, change %.5f (%.4f%%), volume %.0f\n",
		quote.Price, quote.Change, quote.ChangePercentage, quote.Volume)
	if quote.MarketCap != nil {
		fmt.Printf("  market cap %.0f\n", *quote.MarketCap)
	}
	fmt.Printf("  as of %s\n", quote.Timestamp.Time().Format(time.RFC3339))
	return nil
}
