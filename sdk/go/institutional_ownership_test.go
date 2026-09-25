package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// institutionalOwnershipRoutes is the exact path and query table of the eight
// institutional_ownership endpoints, one per query shape: no query at all
// (latest with both setters unset), cik plus year and quarter, cik alone,
// symbol plus year and quarter with both setters, cik plus a page, symbol
// plus year and quarter, and year plus quarter. The wire text is copied from
// the query tests in crates/libfmp/src/endpoints/institutional_ownership.rs.
var institutionalOwnershipRoutes = map[string]struct {
	query   string
	fixture string
}{
	"/router/stable/institutional-ownership/latest":                     {"", "latest_institutional_ownership_filings.json"},
	"/router/stable/institutional-ownership/extract":                    {"cik=0001388838&year=2023&quarter=3", "institutional_ownership_extract.json"},
	"/router/stable/institutional-ownership/dates":                      {"cik=0001067983", "form_13f_filing_dates.json"},
	"/router/stable/institutional-ownership/extract-analytics/holder":   {"symbol=AAPL&year=2023&quarter=3&page=0&limit=10", "institutional_holder_analytics.json"},
	"/router/stable/institutional-ownership/holder-performance-summary": {"cik=0001067983&page=0", "holder_performance_summary.json"},
	"/router/stable/institutional-ownership/holder-industry-breakdown":  {"cik=0001067983&year=2023&quarter=3", "holder_industry_breakdown.json"},
	"/router/stable/institutional-ownership/symbol-positions-summary":   {"symbol=AAPL&year=2023&quarter=3", "institutional_positions_summary.json"},
	"/router/stable/institutional-ownership/industry-summary":           {"year=2023&quarter=3", "institutional_industry_summary.json"},
}

func institutionalOwnershipRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		route, ok := institutionalOwnershipRoutes[r.URL.Path]
		if !ok || r.URL.RawQuery != route.query || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s?%s", r.URL.Path, r.URL.RawQuery)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, route.fixture))
	}
}

func TestInstitutionalOwnershipMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, institutionalOwnershipRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	ns := client.InstitutionalOwnership

	filings, err := ns.LatestFilings(ctx, NewLatestInstitutionalOwnershipFilingsQuery())
	if err != nil || len(filings) != 1 || filings[0].CIK != "0001803005" {
		t.Fatalf("LatestFilings = %+v, %v", filings, err)
	}
	holdings, err := ns.Extract(ctx, NewInstitutionalOwnershipExtractQuery("0001388838", 2023, QuarterQ3))
	if err != nil || len(holdings) != 1 || holdings[0].SecurityCusip != "674215207" {
		t.Fatalf("Extract = %+v, %v", holdings, err)
	}
	dates, err := ns.Form13FFilingDates(ctx, NewForm13FFilingDatesQuery("0001067983"))
	if err != nil || len(dates) != 1 || dates[0].Quarter != 1 {
		t.Fatalf("Form13FFilingDates = %+v, %v", dates, err)
	}
	analytics, err := ns.HolderAnalytics(ctx,
		NewInstitutionalHolderAnalyticsQuery("AAPL", 2023, QuarterQ3).WithPage(0).WithLimit(10))
	if err != nil || len(analytics) != 1 || analytics[0].ChangeInPerformance != -67_750_129_670 {
		t.Fatalf("HolderAnalytics = %+v, %v", analytics, err)
	}
	performance, err := ns.HolderPerformanceSummary(ctx, NewHolderPerformanceSummaryQuery("0001067983").WithPage(0))
	if err != nil || len(performance) != 1 || performance[0].ChangeInPerformance != -14_398_745_159 {
		t.Fatalf("HolderPerformanceSummary = %+v, %v", performance, err)
	}
	breakdown, err := ns.HolderIndustryBreakdown(ctx, NewHolderIndustryBreakdownQuery("0001067983", 2023, QuarterQ3))
	if err != nil || len(breakdown) != 1 || breakdown[0].IndustryTitle != "ELECTRONIC COMPUTERS" {
		t.Fatalf("HolderIndustryBreakdown = %+v, %v", breakdown, err)
	}
	positions, err := ns.PositionsSummary(ctx, NewInstitutionalPositionsSummaryQuery("AAPL", 2023, QuarterQ3))
	if err != nil || len(positions) != 1 || positions[0].TotalInvestedChange != -245_052_087_186 {
		t.Fatalf("PositionsSummary = %+v, %v", positions, err)
	}
	industry, err := ns.IndustrySummary(ctx, NewInstitutionalIndustrySummaryQuery(2023, QuarterQ3))
	if err != nil || len(industry) != 1 || industry[0].IndustryValue != 11_088_059_691 {
		t.Fatalf("IndustrySummary = %+v, %v", industry, err)
	}

	requests := rec.all()
	if len(requests) != len(institutionalOwnershipRoutes) {
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

func TestInstitutionalOwnershipQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, institutionalOwnershipRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	ns := client.InstitutionalOwnership

	for _, quarter := range []Quarter{"5", "Q3", "", "3 "} {
		_, err := ns.IndustrySummary(ctx, NewInstitutionalIndustrySummaryQuery(2023, quarter))
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, ErrUnknownWireValue) ||
			typed.Message != "quarter: "+ErrUnknownWireValue.Error()+", expected one of 1, 2, 3, 4" {
			t.Fatalf("quarter %q: error = %v", quarter, err)
		}
	}
	_, err := ns.Extract(ctx, NewInstitutionalOwnershipExtractQuery(" ", 2023, QuarterQ1))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrEmptyValue) ||
		typed.Message != "cik: "+ErrEmptyValue.Error() {
		t.Fatalf("blank cik: error = %v", err)
	}
	_, err = ns.PositionsSummary(ctx, NewInstitutionalPositionsSummaryQuery("AAPL,MSFT", 2023, QuarterQ4))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrCommaInTicker) ||
		typed.Message != "symbol: "+ErrCommaInTicker.Error() {
		t.Fatalf("comma ticker: error = %v", err)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	q := NewInstitutionalHolderAnalyticsQuery(" AAPL ", 2023, QuarterQ2)
	if q.Symbol() != " AAPL " || q.Year() != 2023 || q.Quarter() != QuarterQ2 || q.Page() != nil || q.Limit() != nil {
		t.Fatalf("getters = %q %d %q %v %v", q.Symbol(), q.Year(), q.Quarter(), q.Page(), q.Limit())
	}
	if page := q.WithPage(7).Page(); page == nil || *page != 7 || q.Page() != nil {
		t.Fatalf("WithPage mutated the receiver or lost the value: %v %v", page, q.Page())
	}
	latest := NewLatestInstitutionalOwnershipFilingsQuery()
	if latest.Page() != nil || latest.Limit() != nil {
		t.Fatalf("latest getters = %v %v, want both unset", latest.Page(), latest.Limit())
	}
}

func TestInstitutionalOwnershipMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[{"date":"2026-03-31","year":2026}]`))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.InstitutionalOwnership.Form13FFilingDates(context.Background(), NewForm13FFilingDatesQuery("0001067983"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "institutional-ownership/dates")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"quarter"`) {
		t.Fatalf("cause = %v, want it to name the missing member quarter", cause)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}
