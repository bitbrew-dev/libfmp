package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// forexRoutes is the exact path and query table of the eight forex
// endpoints (crates/libfmp/tests/asset_catalog_quote_endpoints.rs and
// asset_history_endpoints.rs), keyed by "path?rawquery" under the test
// client's "/router/stable" prefix. The catalog sends no query, the quotes
// send symbol, and AssetChartQuery sends symbol, from, to in that order with
// an unset optional omitted.
var forexRoutes = map[string]string{
	"/router/stable/forex-list?":               "forex_list.json",
	"/router/stable/quote?symbol=EURUSD":       "forex_quote.json",
	"/router/stable/quote-short?symbol=EURUSD": "forex_quote_short.json",
	"/router/stable/historical-price-eod/light?symbol=EURUSD&from=2026-01-27&to=2026-04-27":     "forex_chart_light.json",
	"/router/stable/historical-price-eod/full?symbol=EUR+%2F+USD&from=2026-01-27&to=2026-04-27": "forex_chart_full.json",
	"/router/stable/historical-chart/1min?symbol=EURUSD&from=2024-01-01&to=2024-03-01":          "forex_chart_one_minute.json",
	"/router/stable/historical-chart/5min?symbol=EURUSD&from=2024-01-01":                        "forex_chart_five_minutes.json",
	"/router/stable/historical-chart/1hour?symbol=EURUSD&to=2024-03-01":                         "forex_chart_one_hour.json",
}

func forexRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := forexRoutes[r.URL.Path+"?"+r.URL.RawQuery]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s?%s", r.URL.Path, r.URL.RawQuery)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestForexMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, forexRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	eodFrom, eodTo := mustParseDate(t, "2026-01-27"), mustParseDate(t, "2026-04-27")
	from, to := mustParseDate(t, "2024-01-01"), mustParseDate(t, "2024-03-01")

	pairs, err := client.Forex.List(ctx)
	if err != nil || len(pairs) != 1 || pairs[0].FromCurrency != "ARS" {
		t.Fatalf("List = %+v, %v", pairs, err)
	}
	quotes, err := client.Forex.Quote(ctx, NewQuoteQuery("EURUSD"))
	if err != nil || len(quotes) != 1 || quotes[0].MarketCap != nil || quotes[0].Exchange != "FOREX" {
		t.Fatalf("Quote = %+v, %v", quotes, err)
	}
	short, err := client.Forex.QuoteShort(ctx, NewQuoteShortQuery("EURUSD"))
	if err != nil || len(short) != 1 || short[0].Volume != 146_872 {
		t.Fatalf("QuoteShort = %+v, %v", short, err)
	}
	light, err := client.Forex.ChartLight(ctx, NewAssetChartQuery("EURUSD").WithFrom(eodFrom).WithTo(eodTo))
	if err != nil || len(light) != 1 || light[0].Price != 1.15258 {
		t.Fatalf("ChartLight = %+v, %v", light, err)
	}
	full, err := client.Forex.ChartFull(ctx, NewAssetChartQuery("EUR / USD").WithFrom(eodFrom).WithTo(eodTo))
	if err != nil || len(full) != 1 || full[0].ChangePercent != 0.51628207 {
		t.Fatalf("ChartFull = %+v, %v", full, err)
	}

	intraday := []struct {
		name   string
		call   func(context.Context, AssetChartQuery) ([]StockChartIntradayBar, error)
		query  AssetChartQuery
		volume float64
	}{
		{"ChartOneMinute", client.Forex.ChartOneMinute, NewAssetChartQuery("EURUSD").WithFrom(from).WithTo(to), 76},
		{"ChartFiveMinutes", client.Forex.ChartFiveMinutes, NewAssetChartQuery("EURUSD").WithFrom(from), 91},
		{"ChartOneHour", client.Forex.ChartOneHour, NewAssetChartQuery("EURUSD").WithTo(to), 1_420},
	}
	for _, tc := range intraday {
		rows, err := tc.call(ctx, tc.query)
		if err != nil || len(rows) != 1 || rows[0].Volume != tc.volume {
			t.Fatalf("%s = %+v, %v, want volume %.0f", tc.name, rows, err, tc.volume)
		}
	}

	requests := rec.all()
	if len(requests) != len(forexRoutes) {
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

func TestForexAssetChartQueryExposesItsArgumentsAsGiven(t *testing.T) {
	t.Parallel()
	from := mustParseDate(t, "2026-01-27")
	q := NewAssetChartQuery(" EURUSD ")
	if q.Symbol() != " EURUSD " || q.From() != nil || q.To() != nil {
		t.Fatalf("query normalized or defaulted an argument: %+v", q)
	}
	if got := q.WithFrom(from).From(); got == nil || *got != from {
		t.Fatalf("From() = %v, want %v", got, from)
	}
	if q.From() != nil {
		t.Fatal("WithFrom mutated the receiver instead of returning a copy")
	}
	if got := q.WithTo(from).To(); got == nil || *got != from {
		t.Fatalf("To() = %v, want %v", got, from)
	}
}

func TestForexQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, forexRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	for _, tc := range []struct {
		name   string
		symbol string
		reason error
	}{
		{"empty", "", ErrEmptyValue},
		{"whitespace", " \t", ErrEmptyValue},
		{"control", "EUR\nUSD", ErrControlCharacterValue},
		{"comma", "EURUSD,GBPUSD", ErrCommaInTicker},
	} {
		_, err := client.Forex.ChartLight(ctx, NewAssetChartQuery(tc.symbol))
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != "symbol: "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want symbol: %v", tc.name, err, tc.reason)
		}
		_, err = client.Forex.Quote(ctx, NewQuoteQuery(tc.symbol))
		if quote := assertQuoteError(t, err, CategoryValidation, 0, ""); quote.Message != typed.Message {
			t.Fatalf("%s: Quote message %q differs from ChartLight message %q", tc.name, quote.Message, typed.Message)
		}
	}

	// A zero Date has no wire form: dateParam rejects it for from and to.
	_, err := client.Forex.ChartFull(ctx, NewAssetChartQuery("EURUSD").WithFrom(Date{}))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrZeroTemporalValue) ||
		typed.Message != "from: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero from: %v", err)
	}
	_, err = client.Forex.ChartOneMinute(ctx, NewAssetChartQuery("EURUSD").WithTo(Date{}))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrZeroTemporalValue) ||
		typed.Message != "to: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero to: %v", err)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}
}

// Mirrors malformed_non_array_responses_keep_catalog_quote_and_batch_endpoint_identities
// in crates/libfmp/tests/asset_catalog_quote_endpoints.rs, plus the
// missing-member path through a method.
func TestForexMethodsReportMissingMembersAndObjectRootsAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, jsonHandler(`[{"symbol":"ARSMXN","fromCurrency":"ARS","toCurrency":"MXN",`+
		`"fromName":"Argentine Peso"}]`))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))

	_, err := client.Forex.List(context.Background())
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "forex-list")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"toName"`) {
		t.Fatalf("cause = %v, want it to name the missing member toName", cause)
	}
	if strings.Contains(err.Error(), "route-secret") {
		t.Fatalf("decode error leaked the credential: %s", err)
	}

	objectRoot, _ := newServer(t, jsonHandler(`{}`))
	client = newClient(t, objectRoot, WithAuthentication(FmpHeader("route-secret")))
	_, err = client.Forex.Quote(context.Background(), NewQuoteQuery("EURUSD"))
	if typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "quote"); typed.Unwrap() == nil {
		t.Fatalf("object root error %v carries no cause", typed)
	}
}
