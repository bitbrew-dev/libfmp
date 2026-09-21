package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// cryptoRoutes is the exact path and query table of the crypto endpoints,
// copied from crates/libfmp/tests/asset_catalog_quote_endpoints.rs and
// asset_history_endpoints.rs, keyed by request path under the test client's
// "/router/stable" prefix. Wire order is symbol, from, to; an unset optional
// is omitted. The five chart paths spread the AssetChartQuery shapes (both
// dates, from only, to only, neither) across the domain.
var cryptoRoutes = map[string]struct {
	query   string
	fixture string
}{
	"/router/stable/cryptocurrency-list":        {"", "cryptocurrency_list.json"},
	"/router/stable/quote":                      {"symbol=BTCUSD", "cryptocurrency_quote.json"},
	"/router/stable/quote-short":                {"symbol=BTCUSD", "cryptocurrency_quote_short.json"},
	"/router/stable/historical-price-eod/light": {"symbol=BTCUSD&from=2026-01-27&to=2026-04-27", "crypto_chart_light.json"},
	"/router/stable/historical-price-eod/full":  {"symbol=BTCUSD&from=2026-01-27", "crypto_chart_full.json"},
	"/router/stable/historical-chart/1min":      {"symbol=BTCUSD&to=2024-03-01", "crypto_chart_one_minute.json"},
	"/router/stable/historical-chart/5min":      {"symbol=BTC+%2F+USD", "crypto_chart_five_minutes.json"},
	"/router/stable/historical-chart/1hour":     {"symbol=BTCUSD&from=2024-01-01&to=2024-03-01", "crypto_chart_one_hour.json"},
}

func cryptoRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		route, ok := cryptoRoutes[r.URL.Path]
		if !ok || r.URL.RawQuery != route.query || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s?%s", r.URL.Path, r.URL.RawQuery)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, route.fixture))
	}
}

func TestCryptoMethodsUseExactPathsAndQueryOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, cryptoRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	eodFrom, eodTo := mustParseDate(t, "2026-01-27"), mustParseDate(t, "2026-04-27")
	from, to := mustParseDate(t, "2024-01-01"), mustParseDate(t, "2024-03-01")

	listing, err := client.Crypto.List(ctx)
	if err != nil || len(listing) != 1 || listing[0].Symbol != "MIOTAUSD" || listing[0].TotalSupply != 4_788_606_639 {
		t.Fatalf("List = %+v, %v", listing, err)
	}
	quote, err := client.Crypto.Quote(ctx, NewQuoteQuery("BTCUSD"))
	if err != nil || len(quote) != 1 || quote[0].MarketCap == nil || *quote[0].MarketCap != 1_293_361_815_015 {
		t.Fatalf("Quote = %+v, %v", quote, err)
	}
	short, err := client.Crypto.QuoteShort(ctx, NewQuoteShortQuery("BTCUSD"))
	if err != nil || len(short) != 1 || short[0].Volume != 32_030_003_200 {
		t.Fatalf("QuoteShort = %+v, %v", short, err)
	}
	light, err := client.Crypto.ChartLight(ctx, NewAssetChartQuery("BTCUSD").WithFrom(eodFrom).WithTo(eodTo))
	if err != nil || len(light) != 1 || light[0].Volume != 32_030_003_200 {
		t.Fatalf("ChartLight = %+v, %v", light, err)
	}
	full, err := client.Crypto.ChartFull(ctx, NewAssetChartQuery("BTCUSD").WithFrom(eodFrom))
	if err != nil || len(full) != 1 || full[0].Volume != 32_030_003_200 {
		t.Fatalf("ChartFull = %+v, %v", full, err)
	}

	intraday := []struct {
		name  string
		call  func(context.Context, AssetChartQuery) ([]StockChartIntradayBar, error)
		query AssetChartQuery
		date  string
	}{
		{"ChartOneMinute", client.Crypto.ChartOneMinute, NewAssetChartQuery("BTCUSD").WithTo(to), "2026-07-30 13:16:00"},
		{"ChartFiveMinutes", client.Crypto.ChartFiveMinutes, NewAssetChartQuery("BTC / USD"), "2026-07-30 13:15:00"},
		{"ChartOneHour", client.Crypto.ChartOneHour, NewAssetChartQuery("BTCUSD").WithFrom(from).WithTo(to),
			"2026-07-30 13:00:00"},
	}
	for _, tc := range intraday {
		rows, err := tc.call(ctx, tc.query)
		if err != nil || len(rows) != 1 || rows[0].Volume != 0 || rows[0].Date.String() != tc.date {
			t.Fatalf("%s = %+v, %v, want date %s", tc.name, rows, err, tc.date)
		}
	}

	requests := rec.all()
	if len(requests) != 8 {
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

func TestCryptoAssetChartQueryExposesItsArgumentsAsGiven(t *testing.T) {
	t.Parallel()
	from, to := mustParseDate(t, "2024-01-01"), mustParseDate(t, "2024-03-01")
	query := NewAssetChartQuery(" BTCUSD ")
	if query.Symbol() != " BTCUSD " || query.From() != nil || query.To() != nil {
		t.Fatalf("query normalized or defaulted an argument: %+v", query)
	}
	dated := query.WithFrom(from).WithTo(to)
	if got := dated.From(); got == nil || *got != from {
		t.Fatalf("From() = %v, want %v", got, from)
	}
	if got := dated.To(); got == nil || *got != to {
		t.Fatalf("To() = %v, want %v", got, to)
	}
	if query.From() != nil || query.To() != nil {
		t.Fatal("WithFrom or WithTo mutated the receiver instead of returning a copy")
	}
}

func TestCryptoQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, cryptoRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	for _, tc := range []struct {
		name   string
		symbol string
		reason error
	}{
		{"empty", "", ErrEmptyValue},
		{"control", "BTC\nUSD", ErrControlCharacterValue},
		{"comma", "BTCUSD,ETHUSD", ErrCommaInTicker},
	} {
		_, err := client.Crypto.ChartLight(ctx, NewAssetChartQuery(tc.symbol))
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != "symbol: "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want symbol: %v", tc.name, err, tc.reason)
		}
		_, err = client.Crypto.Quote(ctx, NewQuoteQuery(tc.symbol))
		if quote := assertQuoteError(t, err, CategoryValidation, 0, ""); quote.Message != typed.Message {
			t.Fatalf("%s: Quote message %q differs from ChartLight message %q", tc.name, quote.Message, typed.Message)
		}
		_, err = client.Crypto.QuoteShort(ctx, NewQuoteShortQuery(tc.symbol))
		if short := assertQuoteError(t, err, CategoryValidation, 0, ""); short.Message != typed.Message {
			t.Fatalf("%s: QuoteShort message %q differs from ChartLight message %q", tc.name, short.Message, typed.Message)
		}
	}

	// A zero Date has no wire form: dateParam rejects it on either setter.
	_, err := client.Crypto.ChartFull(ctx, NewAssetChartQuery("BTCUSD").WithFrom(Date{}))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrZeroTemporalValue) ||
		typed.Message != "from: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero from: %v", err)
	}
	_, err = client.Crypto.ChartOneHour(ctx, NewAssetChartQuery("BTCUSD").WithTo(Date{}))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrZeroTemporalValue) ||
		typed.Message != "to: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero to: %v", err)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}
}

// Mirrors malformed_non_arrays_keep_shared_path_endpoint_identity and the
// catalog decode-error test in the Rust endpoint suites: a decode failure
// names the endpoint, keeps the 200 status, and never carries the credential.
func TestCryptoMethodsReportDecodeErrorsWithEndpointIdentity(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, jsonHandler(`[{"symbol":"MIOTAUSD","name":"IOTA USD","exchange":"CCC",`+
		`"icoDate":"2017-11-09","circulatingSupply":4232705124}]`))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()

	_, err := client.Crypto.List(ctx)
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "cryptocurrency-list")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"totalSupply"`) {
		t.Fatalf("cause = %v, want it to name the missing member totalSupply", cause)
	}
	if strings.Contains(err.Error(), "route-secret") {
		t.Fatalf("decode error leaked the credential: %s", err)
	}

	object, _ := newServer(t, jsonHandler(`{}`))
	client = newClient(t, object, WithAuthentication(FmpHeader("route-secret")))
	_, err = client.Crypto.ChartOneMinute(ctx, NewAssetChartQuery("BTCUSD"))
	_ = assertQuoteError(t, err, CategoryDecode, http.StatusOK, "historical-chart/1min")
	_, err = client.Crypto.Quote(ctx, NewQuoteQuery("BTCUSD"))
	_ = assertQuoteError(t, err, CategoryDecode, http.StatusOK, "quote")
}
