package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// indexesRoutes is the exact request-URI table of the indexes endpoints,
// copied from crates/libfmp/tests/indexes_directory_quote_endpoints.rs,
// indexes_history_endpoints.rs and indexes_constituent_endpoints.rs. The
// caret in an index symbol is percent-encoded on the wire; the chart wire
// order is symbol, from, to, and an unset optional is omitted. The six
// constituent routes carry no query at all, and the Dow Jones paths spell
// "dowjones" without a hyphen.
var indexesRoutes = map[string]string{
	"/router/stable/index-list":                "indexes_list.json",
	"/router/stable/quote?symbol=%5EVIX":       "indexes_quote.json",
	"/router/stable/quote-short?symbol=%5EVIX": "indexes_quote_short.json",
	"/router/stable/historical-price-eod/light?symbol=%5EVIX&from=2026-01-27&to=2026-04-27": "indexes_chart_light.json",
	"/router/stable/historical-price-eod/light?symbol=%5EVIX&from=2026-01-27":               "indexes_chart_light.json",
	"/router/stable/historical-price-eod/full?symbol=%5EVIX&from=2026-01-27&to=2026-04-27":  "indexes_chart_full.json",
	"/router/stable/historical-chart/1min?symbol=%5EVIX&from=2024-01-01&to=2024-03-01":      "indexes_chart_one_minute.json",
	"/router/stable/historical-chart/1min?symbol=%5EVIX":                                    "indexes_chart_one_minute.json",
	"/router/stable/historical-chart/5min?symbol=%5EVIX&from=2024-01-01&to=2024-03-01":      "indexes_chart_five_minutes.json",
	"/router/stable/historical-chart/1hour?symbol=%5EVIX&from=2024-01-01&to=2024-03-01":     "indexes_chart_one_hour.json",
	"/router/stable/historical-chart/1hour?symbol=%5EVIX&to=2024-03-01":                     "indexes_chart_one_hour.json",
	"/router/stable/sp500-constituent":                                                      "indexes_sp500_constituents.json",
	"/router/stable/nasdaq-constituent":                                                     "indexes_nasdaq_constituents.json",
	"/router/stable/dowjones-constituent":                                                   "indexes_dow_jones_constituents.json",
	"/router/stable/historical-sp500-constituent":                                           "indexes_historical_sp500_constituents.json",
	"/router/stable/historical-nasdaq-constituent":                                          "indexes_historical_nasdaq_constituents.json",
	"/router/stable/historical-dowjones-constituent":                                        "indexes_historical_dow_jones_constituents.json",
}

func indexesRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := indexesRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestIndexesMethodsUseExactPathsAndQueryOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, indexesRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	eodFrom, eodTo := mustParseDate(t, "2026-01-27"), mustParseDate(t, "2026-04-27")
	from, to := mustParseDate(t, "2024-01-01"), mustParseDate(t, "2024-03-01")

	listing, err := client.Indexes.List(ctx)
	if err != nil || len(listing) != 1 || listing[0].Symbol != "^TTIN" || listing[0].Exchange != "TSX" {
		t.Fatalf("List = %+v, %v", listing, err)
	}
	quote, err := client.Indexes.Quote(ctx, NewQuoteQuery("^VIX"))
	if err != nil || len(quote) != 1 || quote[0].Symbol != "^VIX" || quote[0].MarketCap == nil || *quote[0].MarketCap != 0 {
		t.Fatalf("Quote = %+v, %v", quote, err)
	}
	short, err := client.Indexes.QuoteShort(ctx, NewQuoteShortQuery("^VIX"))
	if err != nil || len(short) != 1 || short[0].Symbol != "^VIX" || short[0].Price != 18.1 {
		t.Fatalf("QuoteShort = %+v, %v", short, err)
	}

	light, err := client.Indexes.ChartLight(ctx, NewIndexChartQuery("^VIX").WithFrom(eodFrom).WithTo(eodTo))
	if err != nil || len(light) != 1 || light[0].Price != 17.9 {
		t.Fatalf("ChartLight = %+v, %v", light, err)
	}
	if _, err := client.Indexes.ChartLight(ctx, NewIndexChartQuery("^VIX").WithFrom(eodFrom)); err != nil {
		t.Fatalf("ChartLight from-only: %v", err)
	}
	full, err := client.Indexes.ChartFull(ctx, NewIndexChartQuery("^VIX").WithFrom(eodFrom).WithTo(eodTo))
	if err != nil || len(full) != 1 || full[0].ChangePercent != -8.48671 {
		t.Fatalf("ChartFull = %+v, %v", full, err)
	}

	intraday := []struct {
		name  string
		call  func(context.Context, IndexChartQuery) ([]StockChartIntradayBar, error)
		query IndexChartQuery
		date  string
	}{
		{"OneMinute", client.Indexes.ChartOneMinute, NewIndexChartQuery("^VIX").WithFrom(from).WithTo(to),
			"2026-07-30 13:17:00"},
		{"OneMinute bare", client.Indexes.ChartOneMinute, NewIndexChartQuery("^VIX"), "2026-07-30 13:17:00"},
		{"FiveMinutes", client.Indexes.ChartFiveMinutes, NewIndexChartQuery("^VIX").WithFrom(from).WithTo(to),
			"2026-07-30 13:15:00"},
		{"OneHour", client.Indexes.ChartOneHour, NewIndexChartQuery("^VIX").WithFrom(from).WithTo(to),
			"2026-07-30 12:30:00"},
		{"OneHour to-only", client.Indexes.ChartOneHour, NewIndexChartQuery("^VIX").WithTo(to),
			"2026-07-30 12:30:00"},
	}
	for _, tc := range intraday {
		rows, err := tc.call(ctx, tc.query)
		if err != nil || len(rows) != 1 || rows[0].Date.String() != tc.date {
			t.Fatalf("%s = %+v, %v, want date %s", tc.name, rows, err, tc.date)
		}
	}

	current := []struct {
		name   string
		call   func(context.Context) ([]IndexConstituent, error)
		symbol string
	}{
		{"Sp500Constituents", client.Indexes.Sp500Constituents, "HONA"},
		{"NasdaqConstituents", client.Indexes.NasdaqConstituents, "ADBE"},
		{"DowJonesConstituents", client.Indexes.DowJonesConstituents, "GOOGL"},
	}
	for _, tc := range current {
		rows, err := tc.call(ctx)
		if err != nil || len(rows) != 1 || rows[0].Symbol != tc.symbol {
			t.Fatalf("%s = %+v, %v, want symbol %s", tc.name, rows, err, tc.symbol)
		}
	}
	historical := []struct {
		name   string
		call   func(context.Context) ([]HistoricalIndexConstituent, error)
		symbol string
	}{
		{"HistoricalSp500Constituents", client.Indexes.HistoricalSp500Constituents, "HONA"},
		{"HistoricalNasdaqConstituents", client.Indexes.HistoricalNasdaqConstituents, "SPCX"},
		{"HistoricalDowJonesConstituents", client.Indexes.HistoricalDowJonesConstituents, "GOOGL"},
	}
	for _, tc := range historical {
		rows, err := tc.call(ctx)
		if err != nil || len(rows) != 1 || rows[0].Symbol != tc.symbol {
			t.Fatalf("%s = %+v, %v, want symbol %s", tc.name, rows, err, tc.symbol)
		}
	}

	requests := rec.all()
	if len(requests) != 17 {
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

func TestIndexChartQueryExposesItsArgumentsAsGiven(t *testing.T) {
	t.Parallel()
	from, to := mustParseDate(t, "2024-01-01"), mustParseDate(t, "2024-03-01")
	q := NewIndexChartQuery(" ^VIX ")
	if q.Symbol() != " ^VIX " || q.From() != nil || q.To() != nil {
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

func TestIndexesQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, indexesRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	for _, tc := range []struct {
		name   string
		symbol string
		reason error
	}{
		{"empty", "", ErrEmptyValue},
		{"control", "^V\nIX", ErrControlCharacterValue},
		{"comma", "^VIX,^GSPC", ErrCommaInTicker},
	} {
		_, err := client.Indexes.ChartFull(ctx, NewIndexChartQuery(tc.symbol))
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != "symbol: "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want symbol: %v", tc.name, err, tc.reason)
		}
		_, err = client.Indexes.QuoteShort(ctx, NewQuoteShortQuery(tc.symbol))
		if short := assertQuoteError(t, err, CategoryValidation, 0, ""); short.Message != typed.Message {
			t.Fatalf("%s: QuoteShort message %q differs from ChartFull message %q", tc.name, short.Message, typed.Message)
		}
	}
	_, err := client.Indexes.ChartOneMinute(ctx, NewIndexChartQuery("^VIX").WithFrom(Date{}))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrZeroTemporalValue) ||
		typed.Message != "from: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero from: %v", err)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}
}

// Mirrors the malformed-body tests in indexes_constituent_endpoints.rs and
// indexes_history_endpoints.rs: a constituent route and a shared chart path
// each report their own endpoint id.
func TestIndexesMethodsReportMalformedBodiesAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, jsonHandler(`{}`))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Indexes.Sp500Constituents(context.Background())
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "sp500-constituent")
	if typed.Unwrap() == nil {
		t.Fatal("decode error carries no cause")
	}
	if strings.Contains(err.Error(), "route-secret") {
		t.Fatalf("decode error leaked the credential: %s", err)
	}
	_, err = client.Indexes.ChartLight(context.Background(), NewIndexChartQuery("^VIX"))
	if typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "historical-price-eod/light"); typed.Unwrap() == nil {
		t.Fatal("decode error carries no cause")
	}
}
