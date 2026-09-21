package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// quoteRoutes is the exact path and query table of the three hand-written
// endpoints (crates/libfmp/tests/quote_*_endpoints.rs), keyed by request
// path under the test client's "/router/stable" prefix.
var quoteRoutes = map[string]struct {
	query   string
	fixture string
}{
	"/router/stable/quote":                {"symbol=%5EVIX", "quote.json"},
	"/router/stable/quote-short":          {"symbol=%5EVIX", "quote_short.json"},
	"/router/stable/batch-etf-quotes":     {"short=true", "quote_etf_short.json"},
	"/router/stable/batch-quote-short":    {"symbols=AAPL%2C%5EVIX", "quote_short_multiple.json"},
	"/router/stable/batch-exchange-quote": {"exchange=NASDAQ&short=true", "quote_exchange_short.json"},
	"/router/stable/aftermarket-trade":    {"symbol=AAPL", "aftermarket_trade.json"},
}

func quoteRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		route, ok := quoteRoutes[r.URL.Path]
		if !ok || r.URL.RawQuery != route.query || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s?%s", r.URL.Path, r.URL.RawQuery)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, route.fixture))
	}
}

func assertQuoteError(t *testing.T, err error, category ErrorCategory, status int, endpoint string) *Error {
	t.Helper()
	var typed *Error
	if !errors.As(err, &typed) {
		t.Fatalf("error %v (%T) is not *Error", err, err)
	}
	if typed.Category != category || typed.Status != status || typed.Endpoint != endpoint {
		t.Fatalf("error = %+v, want category %v status %d endpoint %q", typed, category, status, endpoint)
	}
	return typed
}

func TestQuoteMethodsUseExactPathsQueriesAndHeaderAuthentication(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, quoteRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()

	full, err := client.Quote.Full(ctx, NewQuoteQuery("^VIX"))
	if err != nil || len(full) != 1 || full[0].Symbol != "AAPL" || full[0].Price != 331.85501 {
		t.Fatalf("Full = %+v, %v", full, err)
	}
	short, err := client.Quote.Short(ctx, NewQuoteShortQuery("^VIX"))
	if err != nil || len(short) != 1 || short[0].Volume != 28_718_014 {
		t.Fatalf("Short = %+v, %v", short, err)
	}
	etfs, err := client.Quote.Etfs(ctx)
	if err != nil || len(etfs) != 1 || etfs[0].Symbol != "P60.SI" || etfs[0].Volume != 1 {
		t.Fatalf("Etfs = %+v, %v", etfs, err)
	}

	batch, err := client.Quote.BatchQuoteShort(ctx, NewBatchQuoteShortQuery([]string{"AAPL", "^VIX"}))
	if err != nil || len(batch) != 2 || batch[1].Symbol != "^VIX" {
		t.Fatalf("BatchQuoteShort = %+v, %v", batch, err)
	}
	exchange, err := client.Quote.Exchange(ctx, NewExchangeQuotesQuery("NASDAQ"))
	if err != nil || len(exchange) == 0 {
		t.Fatalf("Exchange = %+v, %v", exchange, err)
	}
	trades, err := client.Quote.AftermarketTrade(ctx, NewAftermarketTradeQuery("AAPL"))
	if err != nil || len(trades) != 1 || trades[0].TradeSize != 16 {
		t.Fatalf("AftermarketTrade = %+v, %v", trades, err)
	}

	requests := rec.all()
	if len(requests) != 6 {
		t.Fatalf("requests = %d, want exactly one per call", len(requests))
	}
	for _, req := range requests {
		if req.Method != http.MethodGet {
			t.Fatalf("%s used %s, want GET", req.URL.Path, req.Method)
		}
		if strings.Contains(req.URL.RawQuery, "apikey") {
			t.Fatalf("%s carried the credential in the query: %s", req.URL.Path, req.URL.RawQuery)
		}
	}
}

func TestQuoteMethodsSurfaceStatusFamiliesAfterOneRequest(t *testing.T) {
	t.Parallel()
	for _, status := range []int{401, 402, 403, 429, 500, 503} {
		t.Run(fmt.Sprint(status), func(t *testing.T) {
			t.Parallel()
			server, rec := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				w.WriteHeader(status)
				_, _ = fmt.Fprint(w, `{"Error Message":"denied"}`)
			})
			client := newClient(t, server, WithAuthentication(FmpHeader("status-secret")))

			_, err := client.Quote.Full(context.Background(), NewQuoteQuery("AAPL"))
			typed := assertQuoteError(t, err, CategoryStatus, status, "quote")
			if typed.Body == nil || typed.Body.Text != `{"Error Message":"denied"}` {
				t.Fatalf("body = %+v", typed.Body)
			}
			if strings.Contains(err.Error(), "status-secret") {
				t.Fatalf("status %d leaked the credential: %s", status, err)
			}
			if rec.count() != 1 {
				t.Fatalf("status %d was retried: %d requests", status, rec.count())
			}
		})
	}
}

func TestQuoteQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, quoteRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	cases := []struct {
		name   string
		symbol string
		reason error
	}{
		{"empty", "", ErrEmptyValue},
		{"whitespace", " \t", ErrEmptyValue},
		{"control", "AA\nPL", ErrControlCharacterValue},
		{"comma", "AAPL,MSFT", ErrCommaInTicker},
	}
	for _, tc := range cases {
		_, err := client.Quote.Full(context.Background(), NewQuoteQuery(tc.symbol))
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != "symbol: "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want symbol: %v", tc.name, err, tc.reason)
		}
		_, err = client.Quote.Short(context.Background(), NewQuoteShortQuery(tc.symbol))
		if short := assertQuoteError(t, err, CategoryValidation, 0, ""); short.Message != typed.Message {
			t.Fatalf("%s: Short message %q differs from Full message %q", tc.name, short.Message, typed.Message)
		}
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}
	if q := NewQuoteQuery(" AAPL "); q.Symbol() != " AAPL " {
		t.Fatalf("Symbol() normalized the ticker: %q", q.Symbol())
	}

	_, err := client.Quote.BatchQuoteShort(context.Background(), NewBatchQuoteShortQuery(nil))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrEmptyTickerList) ||
		typed.Message != "symbols: "+ErrEmptyTickerList.Error() {
		t.Fatalf("empty ticker list: %v", err)
	}
	symbols := []string{"AAPL", "MSFT"}
	q := NewBatchQuoteShortQuery(symbols)
	symbols[0] = "GOOG"
	if got := q.Symbols(); got[0] != "AAPL" {
		t.Fatalf("Symbols() aliased the caller's slice: %v", got)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}
}

func TestQuoteMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, jsonHandler(`[{"symbol":"AAPL","price":1.5,"volume":1}]`))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))

	_, err := client.Quote.Short(context.Background(), NewQuoteShortQuery("AAPL"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "quote-short")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"change"`) {
		t.Fatalf("cause = %v, want it to name the missing member change", cause)
	}
}
