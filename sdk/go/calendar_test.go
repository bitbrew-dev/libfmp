package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// calendarRoutes is the exact path and wire query of every calendar endpoint
// state the Rust endpoint tests drive (crates/libfmp/tests/calendar_*
// _endpoints.rs), keyed by "path?query" under the test client's
// "/router/stable" prefix so one path can carry several query states.
var calendarRoutes = map[string]string{
	"/router/stable/dividends?symbol=AAPL&limit=1001":                                "dividends.json",
	"/router/stable/dividends-calendar?from=2026-01-27&to=2026-04-27&page=0":         "dividends_calendar.json",
	"/router/stable/earnings?symbol=AAPL&includeReportTimes=true":                    "earnings.json",
	"/router/stable/earnings-calendar?to=2026-07-26&page=0&includeReportTimes=false": "earnings_calendar.json",
	"/router/stable/ipos-calendar":                                                   "ipos_calendar.json",
	"/router/stable/ipos-calendar?from=2026-03-06":                                   "ipos_calendar.json",
	"/router/stable/ipos-disclosure?to=2026-06-06":                                   "ipos_disclosure.json",
	"/router/stable/ipos-prospectus?from=2026-03-06&to=2026-06-06":                   "ipos_prospectus.json",
	"/router/stable/splits?symbol=AAPL&limit=0":                                      "stock_splits.json",
	"/router/stable/splits-calendar?from=2026-01-27&to=2026-04-27&page=0":            "stock_splits_calendar.json",
}

func calendarRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		key := r.URL.Path
		if r.URL.RawQuery != "" {
			key += "?" + r.URL.RawQuery
		}
		fixture, ok := calendarRoutes[key]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", key)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

// Every calendar query state the Rust endpoint tests drive: required symbol
// with a limit (including limit 0), the report-time flag set to true and to
// false, independent from and to omission, and the empty query.
func TestCalendarMethodsUseExactPathsAndWireQueryOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, calendarRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	jan27, apr27 := mustParseDate(t, "2026-01-27"), mustParseDate(t, "2026-04-27")
	mar6, jun6 := mustParseDate(t, "2026-03-06"), mustParseDate(t, "2026-06-06")

	dividends, err := client.Calendar.Dividends(ctx, NewDividendsQuery("AAPL").WithLimit(1001))
	if err != nil || len(dividends) != 1 || dividends[0].Dividend != 0.27 {
		t.Fatalf("Dividends = %+v, %v", dividends, err)
	}
	dividendCalendar, err := client.Calendar.DividendsCalendar(ctx,
		NewDividendsCalendarQuery().WithFrom(jan27).WithTo(apr27).WithPage(0))
	if err != nil || len(dividendCalendar) != 1 || dividendCalendar[0].DeclarationDate != nil {
		t.Fatalf("DividendsCalendar = %+v, %v", dividendCalendar, err)
	}
	earnings, err := client.Calendar.Earnings(ctx, NewEarningsQuery("AAPL").WithIncludeReportTimes(true))
	if err != nil || len(earnings) != 1 || earnings[0].EPSActual != nil {
		t.Fatalf("Earnings = %+v, %v", earnings, err)
	}
	earningsCalendar, err := client.Calendar.EarningsCalendar(ctx,
		NewEarningsCalendarQuery().WithTo(mustParseDate(t, "2026-07-26")).WithPage(0).WithIncludeReportTimes(false))
	if err != nil || len(earningsCalendar) != 1 || earningsCalendar[0].Symbol != "GRG.L" {
		t.Fatalf("EarningsCalendar = %+v, %v", earningsCalendar, err)
	}
	ipos, err := client.Calendar.IPOCalendar(ctx, NewIPOCalendarQuery())
	if err != nil || len(ipos) != 1 || ipos[0].Symbol != "IMC" {
		t.Fatalf("IPOCalendar (no query) = %+v, %v", ipos, err)
	}
	if _, err := client.Calendar.IPOCalendar(ctx, NewIPOCalendarQuery().WithFrom(mar6)); err != nil {
		t.Fatalf("IPOCalendar (from only): %v", err)
	}
	disclosures, err := client.Calendar.IPODisclosure(ctx, NewIPODisclosureQuery().WithTo(jun6))
	if err != nil || len(disclosures) != 1 || disclosures[0].CIK != "0001415726" {
		t.Fatalf("IPODisclosure = %+v, %v", disclosures, err)
	}
	prospectuses, err := client.Calendar.IPOProspectus(ctx, NewIPOProspectusQuery().WithFrom(mar6).WithTo(jun6))
	if err != nil || len(prospectuses) != 1 || prospectuses[0].PricePublicTotal != 434 {
		t.Fatalf("IPOProspectus = %+v, %v", prospectuses, err)
	}
	splits, err := client.Calendar.StockSplits(ctx, NewStockSplitsQuery("AAPL").WithLimit(0))
	if err != nil || len(splits) != 1 || splits[0].Numerator != 4 {
		t.Fatalf("StockSplits = %+v, %v", splits, err)
	}
	splitCalendar, err := client.Calendar.StockSplitsCalendar(ctx,
		NewStockSplitsCalendarQuery().WithFrom(jan27).WithTo(apr27).WithPage(0))
	if err != nil || len(splitCalendar) != 1 || splitCalendar[0].Denominator != 5 {
		t.Fatalf("StockSplitsCalendar = %+v, %v", splitCalendar, err)
	}

	requests := rec.all()
	if len(requests) != len(calendarRoutes) {
		t.Fatalf("requests = %d, want exactly one per route", len(requests))
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

// Unset optional parameters are absent from the wire, and the getters report
// nil rather than a zero value.
func TestCalendarOptionalParametersAreOmittedUntilSet(t *testing.T) {
	t.Parallel()
	query := NewEarningsCalendarQuery()
	if query.From() != nil || query.To() != nil || query.Page() != nil || query.IncludeReportTimes() != nil {
		t.Fatalf("fresh query reports a set parameter: %+v", query)
	}
	params, err := query.params()
	if err != nil || len(params) != 0 {
		t.Fatalf("params = %v, %v, want none", params, err)
	}
	flagged := query.WithIncludeReportTimes(false)
	if params, err = flagged.params(); err != nil || len(params) != 1 || params[0].Name != "includeReportTimes" ||
		params[0].Value != "false" {
		t.Fatalf("params = %v, %v, want includeReportTimes=false", params, err)
	}
	if query.IncludeReportTimes() != nil {
		t.Fatal("WithIncludeReportTimes mutated the receiver")
	}
}

func TestCalendarQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, calendarRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	_, err := client.Calendar.Dividends(ctx, NewDividendsQuery("AAPL,MSFT"))
	typed := assertQuoteError(t, err, CategoryValidation, 0, "")
	if !errors.Is(err, ErrCommaInTicker) || typed.Message != "symbol: "+ErrCommaInTicker.Error() {
		t.Fatalf("comma ticker: %v", err)
	}
	_, err = client.Calendar.IPOCalendar(ctx, NewIPOCalendarQuery().WithFrom(Date{}))
	typed = assertQuoteError(t, err, CategoryValidation, 0, "")
	if !errors.Is(err, ErrZeroTemporalValue) || typed.Message != "from: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero from date: %v", err)
	}
	_, err = client.Calendar.StockSplitsCalendar(ctx, NewStockSplitsCalendarQuery().WithTo(Date{}))
	if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); typed.Message != "to: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero to date: %v", err)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}
}

func TestCalendarMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, jsonHandler(`[{"symbol":"AAPL"}]`))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Calendar.StockSplits(context.Background(), NewStockSplitsQuery("AAPL"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "splits")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"date"`) ||
		!strings.Contains(cause.Error(), "StockSplitEvent") {
		t.Fatalf("cause = %v, want it to name the missing member date of StockSplitEvent", cause)
	}
	if strings.Contains(err.Error(), "route-secret") {
		t.Fatalf("decode error leaked the credential: %s", err)
	}
}
