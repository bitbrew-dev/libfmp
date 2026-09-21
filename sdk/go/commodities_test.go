package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// commoditiesRoutes is the exact request-URI table of the commodities
// endpoints, copied from crates/libfmp/tests/asset_catalog_quote_endpoints.rs
// and asset_history_endpoints.rs. Keying on the full URI lets one chart path
// carry its both-dates, from-only, and bare variants: the wire order is
// symbol, from, to, and an unset optional is omitted.
var commoditiesRoutes = map[string]string{
	"/router/stable/commodities-list":                                                      "commodities_list.json",
	"/router/stable/quote?symbol=GCUSD":                                                    "commodities_quote.json",
	"/router/stable/quote-short?symbol=GCUSD":                                              "commodities_quote_short.json",
	"/router/stable/historical-price-eod/light?symbol=GCUSD&from=2026-01-27&to=2026-04-27": "commodity_chart_light.json",
	"/router/stable/historical-price-eod/light?symbol=GCUSD&from=2026-01-27":               "commodity_chart_light.json",
	"/router/stable/historical-price-eod/full?symbol=GCUSD&from=2026-01-27&to=2026-04-27":  "commodity_chart_full.json",
	"/router/stable/historical-price-eod/full?symbol=GCUSD&to=2026-04-27":                  "commodity_chart_full.json",
	"/router/stable/historical-chart/1min?symbol=GCUSD&from=2024-01-01&to=2024-03-01":      "commodity_chart_one_minute.json",
	"/router/stable/historical-chart/1min?symbol=GCUSD":                                    "commodity_chart_one_minute.json",
	"/router/stable/historical-chart/5min?symbol=GCUSD&from=2024-01-01&to=2024-03-01":      "commodity_chart_five_minutes.json",
	"/router/stable/historical-chart/1hour?symbol=GCUSD&from=2024-01-01&to=2024-03-01":     "commodity_chart_one_hour.json",
}

func commoditiesRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := commoditiesRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestCommoditiesMethodsUseExactPathsAndQueryOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, commoditiesRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	eodFrom, eodTo := mustParseDate(t, "2026-01-27"), mustParseDate(t, "2026-04-27")
	from, to := mustParseDate(t, "2024-01-01"), mustParseDate(t, "2024-03-01")

	listing, err := client.Commodities.List(ctx)
	if err != nil || len(listing) != 1 || listing[0].Symbol != "ZMUSD" || listing[0].Exchange != nil {
		t.Fatalf("List = %+v, %v", listing, err)
	}
	quote, err := client.Commodities.Quote(ctx, NewQuoteQuery("GCUSD"))
	if err != nil || len(quote) != 1 || quote[0].Symbol != "GCUSD" || quote[0].MarketCap != nil {
		t.Fatalf("Quote = %+v, %v", quote, err)
	}
	short, err := client.Commodities.QuoteShort(ctx, NewQuoteShortQuery("GCUSD"))
	if err != nil || len(short) != 1 || short[0].Symbol != "GCUSD" || short[0].Volume != 125_925 {
		t.Fatalf("QuoteShort = %+v, %v", short, err)
	}

	light, err := client.Commodities.ChartLight(ctx, NewAssetChartQuery("GCUSD").WithFrom(eodFrom).WithTo(eodTo))
	if err != nil || len(light) != 1 || light[0].Volume != 126_573 {
		t.Fatalf("ChartLight = %+v, %v", light, err)
	}
	if _, err := client.Commodities.ChartLight(ctx, NewAssetChartQuery("GCUSD").WithFrom(eodFrom)); err != nil {
		t.Fatalf("ChartLight from-only: %v", err)
	}
	full, err := client.Commodities.ChartFull(ctx, NewAssetChartQuery("GCUSD").WithFrom(eodFrom).WithTo(eodTo))
	if err != nil || len(full) != 1 || full[0].Change != 43.3 {
		t.Fatalf("ChartFull = %+v, %v", full, err)
	}
	if _, err := client.Commodities.ChartFull(ctx, NewAssetChartQuery("GCUSD").WithTo(eodTo)); err != nil {
		t.Fatalf("ChartFull to-only: %v", err)
	}

	intraday := []struct {
		name   string
		call   func(context.Context, AssetChartQuery) ([]StockChartIntradayBar, error)
		query  AssetChartQuery
		volume uint64
	}{
		{"OneMinute", client.Commodities.ChartOneMinute, NewAssetChartQuery("GCUSD").WithFrom(from).WithTo(to), 59},
		{"OneMinute bare", client.Commodities.ChartOneMinute, NewAssetChartQuery("GCUSD"), 59},
		{"FiveMinutes", client.Commodities.ChartFiveMinutes, NewAssetChartQuery("GCUSD").WithFrom(from).WithTo(to), 103},
		{"OneHour", client.Commodities.ChartOneHour, NewAssetChartQuery("GCUSD").WithFrom(from).WithTo(to), 690},
	}
	for _, tc := range intraday {
		rows, err := tc.call(ctx, tc.query)
		if err != nil || len(rows) != 1 || rows[0].Volume != tc.volume {
			t.Fatalf("%s = %+v, %v, want volume %d", tc.name, rows, err, tc.volume)
		}
	}

	requests := rec.all()
	if len(requests) != 11 {
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

func TestAssetChartQueryExposesItsArgumentsAsGiven(t *testing.T) {
	t.Parallel()
	from, to := mustParseDate(t, "2024-01-01"), mustParseDate(t, "2024-03-01")
	q := NewAssetChartQuery(" GCUSD ")
	if q.Symbol() != " GCUSD " || q.From() != nil || q.To() != nil {
		t.Fatalf("query normalized or defaulted an argument: %+v", q)
	}
	dated := q.WithFrom(from).WithTo(to)
	if got := dated.From(); got == nil || *got != from {
		t.Fatalf("From() = %v, want %v", got, from)
	}
	if got := dated.To(); got == nil || *got != to {
		t.Fatalf("To() = %v, want %v", got, to)
	}
	if q.From() != nil || q.To() != nil {
		t.Fatal("WithFrom or WithTo mutated the receiver instead of returning a copy")
	}
}

func TestCommoditiesQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, commoditiesRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	for _, tc := range []struct {
		name   string
		symbol string
		reason error
	}{
		{"empty", "", ErrEmptyValue},
		{"control", "GC\nUSD", ErrControlCharacterValue},
		{"comma", "GCUSD,SIUSD", ErrCommaInTicker},
	} {
		_, err := client.Commodities.ChartLight(ctx, NewAssetChartQuery(tc.symbol))
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != "symbol: "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want symbol: %v", tc.name, err, tc.reason)
		}
		_, err = client.Commodities.Quote(ctx, NewQuoteQuery(tc.symbol))
		if quote := assertQuoteError(t, err, CategoryValidation, 0, ""); quote.Message != typed.Message {
			t.Fatalf("%s: Quote message %q differs from ChartLight message %q", tc.name, quote.Message, typed.Message)
		}
	}
	_, err := client.Commodities.ChartOneHour(ctx, NewAssetChartQuery("GCUSD").WithTo(Date{}))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrZeroTemporalValue) ||
		typed.Message != "to: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero to: %v", err)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}
}

// Mirrors malformed_non_arrays_keep_shared_path_endpoint_identity in
// asset_history_endpoints.rs: a shared chart path reports its own endpoint id.
func TestCommoditiesMethodsReportMalformedBodiesAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, jsonHandler(`{}`))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))

	_, err := client.Commodities.ChartLight(context.Background(), NewAssetChartQuery("GCUSD"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "historical-price-eod/light")
	if typed.Unwrap() == nil {
		t.Fatal("decode error carries no cause")
	}
	if strings.Contains(err.Error(), "route-secret") {
		t.Fatalf("decode error leaked the credential: %s", err)
	}
	_, err = client.Commodities.List(context.Background())
	if typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "commodities-list"); typed.Unwrap() == nil {
		t.Fatal("decode error carries no cause")
	}
}
