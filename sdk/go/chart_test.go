package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// chartRoutes is the exact path and query table of the chart endpoints,
// copied from crates/libfmp/tests/chart_eod_core_endpoints.rs,
// chart_eod_adjusted_endpoints.rs and chart_intraday_endpoints.rs, keyed by
// request path under the test client's "/router/stable" prefix. Wire order
// is symbol, from, to, nonadjusted, extended; an unset optional is omitted.
var chartRoutes = map[string]struct {
	query   string
	fixture string
}{
	"/router/stable/historical-price-eod/light": {"symbol=BRK.B+%2F+Class+A&from=2026-04-30&to=2026-07-30",
		"stock_chart_light.json"},
	"/router/stable/historical-price-eod/full":               {"symbol=AAPL&from=2026-04-30", "stock_chart_full.json"},
	"/router/stable/historical-price-eod/non-split-adjusted": {"symbol=AAPL&to=2026-07-30", "stock_chart_non_split_adjusted.json"},
	"/router/stable/historical-price-eod/dividend-adjusted":  {"symbol=AAPL", "stock_chart_dividend_adjusted.json"},
	"/router/stable/historical-chart/1min":                   {"symbol=BRK.B+%2F+Class+A", "stock_chart_one_minute.json"},
	"/router/stable/historical-chart/5min": {"symbol=BRK.B+%2F+Class+A&from=2024-01-01&nonadjusted=false",
		"stock_chart_five_minutes.json"},
	"/router/stable/historical-chart/15min": {"symbol=BRK.B+%2F+Class+A&to=2024-03-01&extended=false",
		"stock_chart_fifteen_minutes.json"},
	"/router/stable/historical-chart/30min": {
		"symbol=BRK.B+%2F+Class+A&from=2024-01-01&to=2024-03-01&nonadjusted=false&extended=false",
		"stock_chart_thirty_minutes.json"},
	"/router/stable/historical-chart/1hour": {
		"symbol=BRK.B+%2F+Class+A&from=2024-01-01&to=2024-03-01&nonadjusted=true&extended=true",
		"stock_chart_one_hour.json"},
	"/router/stable/historical-chart/4hour": {
		"symbol=BRK.B+%2F+Class+A&from=2024-01-01&to=2024-03-01&nonadjusted=false&extended=true",
		"stock_chart_four_hours.json"},
}

func chartRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		route, ok := chartRoutes[r.URL.Path]
		if !ok || r.URL.RawQuery != route.query || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s?%s", r.URL.Path, r.URL.RawQuery)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, route.fixture))
	}
}

func TestChartMethodsUseExactPathsAndQueryOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, chartRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	eodFrom, eodTo := mustParseDate(t, "2026-04-30"), mustParseDate(t, "2026-07-30")
	from, to := mustParseDate(t, "2024-01-01"), mustParseDate(t, "2024-03-01")
	share := "BRK.B / Class A"

	light, err := client.Chart.Light(ctx, NewStockChartEodQuery(share).WithFrom(eodFrom).WithTo(eodTo))
	if err != nil || len(light) != 1 || light[0].Symbol != "AAPL" || light[0].Price != 332.39 {
		t.Fatalf("Light = %+v, %v", light, err)
	}
	full, err := client.Chart.Full(ctx, NewStockChartEodQuery("AAPL").WithFrom(eodFrom))
	if err != nil || len(full) != 1 || full[0].Vwap != 332.15 {
		t.Fatalf("Full = %+v, %v", full, err)
	}
	nonSplit, err := client.Chart.NonSplitAdjusted(ctx, NewStockChartEodQuery("AAPL").WithTo(eodTo))
	if err != nil || len(nonSplit) != 1 || nonSplit[0].AdjClose != 332.39 {
		t.Fatalf("NonSplitAdjusted = %+v, %v", nonSplit, err)
	}
	dividend, err := client.Chart.DividendAdjusted(ctx, NewStockChartEodQuery("AAPL"))
	if err != nil || len(dividend) != 1 || dividend[0].Volume != 29_207_295 {
		t.Fatalf("DividendAdjusted = %+v, %v", dividend, err)
	}

	intraday := []struct {
		name  string
		call  func(context.Context, StockChartIntradayQuery) ([]StockChartIntradayBar, error)
		query StockChartIntradayQuery
		date  string
	}{
		{"OneMinute", client.Chart.OneMinute, NewStockChartIntradayQuery(share), "2026-07-30 13:16:00"},
		{"FiveMinutes", client.Chart.FiveMinutes,
			NewStockChartIntradayQuery(share).WithFrom(from).WithNonadjusted(false), "2026-07-30 13:15:00"},
		{"FifteenMinutes", client.Chart.FifteenMinutes,
			NewStockChartIntradayQuery(share).WithTo(to).WithExtended(false), "2026-07-30 13:15:00"},
		{"ThirtyMinutes", client.Chart.ThirtyMinutes, NewStockChartIntradayQuery(share).WithFrom(from).WithTo(to).
			WithNonadjusted(false).WithExtended(false), "2026-07-30 13:00:00"},
		{"OneHour", client.Chart.OneHour, NewStockChartIntradayQuery(share).WithFrom(from).WithTo(to).
			WithNonadjusted(true).WithExtended(true), "2026-07-30 12:30:00"},
		{"FourHours", client.Chart.FourHours, NewStockChartIntradayQuery(share).WithFrom(from).WithTo(to).
			WithNonadjusted(false).WithExtended(true), "2026-07-30 09:30:00"},
	}
	for _, tc := range intraday {
		rows, err := tc.call(ctx, tc.query)
		if err != nil || len(rows) != 1 || rows[0].Date.String() != tc.date {
			t.Fatalf("%s = %+v, %v, want date %s", tc.name, rows, err, tc.date)
		}
	}

	requests := rec.all()
	if len(requests) != 10 {
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

func TestChartQueriesExposeTheirArgumentsAsGiven(t *testing.T) {
	t.Parallel()
	from := mustParseDate(t, "2026-04-30")
	eod := NewStockChartEodQuery(" AAPL ")
	if eod.Symbol() != " AAPL " || eod.From() != nil || eod.To() != nil {
		t.Fatalf("EOD query normalized or defaulted an argument: %+v", eod)
	}
	if got := eod.WithFrom(from).From(); got == nil || *got != from {
		t.Fatalf("From() = %v, want %v", got, from)
	}
	if eod.From() != nil {
		t.Fatal("WithFrom mutated the receiver instead of returning a copy")
	}
	intraday := NewStockChartIntradayQuery("AAPL").WithNonadjusted(false)
	if got := intraday.Nonadjusted(); got == nil || *got {
		t.Fatalf("Nonadjusted() = %v, want false (distinct from unset)", got)
	}
	if intraday.Extended() != nil {
		t.Fatalf("Extended() = %v, want nil when unset", *intraday.Extended())
	}
}

func TestChartQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, chartRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	for _, tc := range []struct {
		name   string
		symbol string
		reason error
	}{
		{"empty", "", ErrEmptyValue},
		{"control", "AA\nPL", ErrControlCharacterValue},
		{"comma", "AAPL,MSFT", ErrCommaInTicker},
	} {
		_, err := client.Chart.Light(ctx, NewStockChartEodQuery(tc.symbol))
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != "symbol: "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want symbol: %v", tc.name, err, tc.reason)
		}
		_, err = client.Chart.OneMinute(ctx, NewStockChartIntradayQuery(tc.symbol))
		if intraday := assertQuoteError(t, err, CategoryValidation, 0, ""); intraday.Message != typed.Message {
			t.Fatalf("%s: OneMinute message %q differs from Light message %q", tc.name, intraday.Message, typed.Message)
		}
	}

	// A zero Date has no wire form: dateParam rejects it for both query types.
	_, err := client.Chart.Full(ctx, NewStockChartEodQuery("AAPL").WithFrom(Date{}))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrZeroTemporalValue) ||
		typed.Message != "from: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero from: %v", err)
	}
	_, err = client.Chart.FourHours(ctx, NewStockChartIntradayQuery("AAPL").WithTo(Date{}))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrZeroTemporalValue) ||
		typed.Message != "to: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero to: %v", err)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}
}

func TestChartMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, jsonHandler(`[{"date":"2026-07-30 13:16:00","open":1,"low":1,"high":1,"volume":1}]`))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))

	_, err := client.Chart.OneHour(context.Background(), NewStockChartIntradayQuery("AAPL"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "historical-chart/1hour")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"close"`) {
		t.Fatalf("cause = %v, want it to name the missing member close", cause)
	}
	if strings.Contains(err.Error(), "route-secret") {
		t.Fatalf("decode error leaked the credential: %s", err)
	}
}
