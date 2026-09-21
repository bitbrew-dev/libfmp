package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// cotRoutes is the exact path and query table of the three COT endpoints
// (crates/libfmp/tests/cot_endpoints.rs), keyed by "path?rawquery" under the
// test client's "/router/stable" prefix. The same path appears with two
// queries because every CotQuery parameter is independently optional.
var cotRoutes = map[string]string{
	"/router/stable/commitment-of-traders-report?symbol=VX+%2F+Index&from=2024-01-01&to=2024-03-01":   "cot_report.json",
	"/router/stable/commitment-of-traders-analysis?symbol=VX+%2F+Index&from=2024-01-01&to=2024-03-01": "cot_analysis.json",
	"/router/stable/commitment-of-traders-report?to=2024-03-01":                                       "cot_report.json",
	"/router/stable/commitment-of-traders-analysis?symbol=PA":                                         "cot_analysis.json",
	"/router/stable/commitment-of-traders-list?":                                                      "cot_report_list.json",
}

func cotRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := cotRoutes[r.URL.Path+"?"+r.URL.RawQuery]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s?%s", r.URL.Path, r.URL.RawQuery)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestCommitmentOfTradersMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, cotRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	full := NewCotQuery().WithSymbol("VX / Index").WithFrom(mustParseDate(t, "2024-01-01")).
		WithTo(mustParseDate(t, "2024-03-01"))

	reports, err := client.CommitmentOfTraders.Report(ctx, full)
	if err != nil || len(reports) != 1 || reports[0].Name != "CBOE VIX (VX)" {
		t.Fatalf("Report = %+v, %v", reports, err)
	}
	analyses, err := client.CommitmentOfTraders.Analysis(ctx, full)
	if err != nil || len(analyses) != 1 || analyses[0].NetPosition != -12_315 {
		t.Fatalf("Analysis = %+v, %v", analyses, err)
	}
	listings, err := client.CommitmentOfTraders.ReportList(ctx)
	if err != nil || len(listings) != 1 || listings[0].Symbol != "NG" {
		t.Fatalf("ReportList = %+v, %v", listings, err)
	}
	if _, err := client.CommitmentOfTraders.Report(ctx, NewCotQuery().WithTo(mustParseDate(t, "2024-03-01"))); err != nil {
		t.Fatalf("Report with only to: %v", err)
	}
	if _, err := client.CommitmentOfTraders.Analysis(ctx, NewCotQuery().WithSymbol("PA")); err != nil {
		t.Fatalf("Analysis with only symbol: %v", err)
	}

	want := []string{
		"/router/stable/commitment-of-traders-report?symbol=VX+%2F+Index&from=2024-01-01&to=2024-03-01",
		"/router/stable/commitment-of-traders-analysis?symbol=VX+%2F+Index&from=2024-01-01&to=2024-03-01",
		"/router/stable/commitment-of-traders-list?",
		"/router/stable/commitment-of-traders-report?to=2024-03-01",
		"/router/stable/commitment-of-traders-analysis?symbol=PA",
	}
	requests := rec.all()
	if len(requests) != len(want) {
		t.Fatalf("requests = %d, want exactly one per call", len(requests))
	}
	for index, req := range requests {
		if got := req.URL.Path + "?" + req.URL.RawQuery; got != want[index] {
			t.Fatalf("request %d = %s, want %s", index, got, want[index])
		}
		if req.Method != http.MethodGet {
			t.Fatalf("%s used %s, want GET", req.URL.Path, req.Method)
		}
		if strings.Contains(req.URL.RawQuery, "apikey") {
			t.Fatalf("%s carried the credential in the query: %s", req.URL.Path, req.URL.RawQuery)
		}
	}
}

func TestCommitmentOfTradersQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, cotRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	cases := []struct {
		name   string
		query  CotQuery
		member string
		reason error
	}{
		{"comma in symbol", NewCotQuery().WithSymbol("VX,PA"), "symbol", ErrCommaInTicker},
		{"blank symbol", NewCotQuery().WithSymbol(" "), "symbol", ErrEmptyValue},
		{"zero from", NewCotQuery().WithFrom(Date{}), "from", ErrZeroTemporalValue},
		{"zero to", NewCotQuery().WithSymbol("VX").WithTo(Date{}), "to", ErrZeroTemporalValue},
	}
	for _, tc := range cases {
		_, err := client.CommitmentOfTraders.Report(ctx, tc.query)
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != tc.member+": "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want %s: %v", tc.name, err, tc.member, tc.reason)
		}
		_, err = client.CommitmentOfTraders.Analysis(ctx, tc.query)
		if analysis := assertQuoteError(t, err, CategoryValidation, 0, ""); analysis.Message != typed.Message {
			t.Fatalf("%s: Analysis message %q differs from Report message %q", tc.name, analysis.Message, typed.Message)
		}
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}
	empty := NewCotQuery()
	if empty.Symbol() != nil || empty.From() != nil || empty.To() != nil {
		t.Fatalf("NewCotQuery() set a parameter: %+v", empty)
	}
	if q := NewCotQuery().WithSymbol(" VX "); q.Symbol() == nil || *q.Symbol() != " VX " {
		t.Fatalf("Symbol() normalized the ticker: %v", q.Symbol())
	}
}

func TestCommitmentOfTradersMethodsReportNumberKindAsDecodeError(t *testing.T) {
	t.Parallel()
	fixture := string(readFixture(t, "cot_report.json"))
	corrupted := strings.Replace(fixture, `"pctOfOpenInterestAll": 100,`, `"pctOfOpenInterestAll": "100",`, 1)
	if corrupted == fixture {
		t.Fatal("cot_report.json no longer spells pctOfOpenInterestAll as the integer 100")
	}
	server, _ := newServer(t, jsonHandler(corrupted))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))

	_, err := client.CommitmentOfTraders.Report(context.Background(), NewCotQuery())
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "commitment-of-traders-report")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"pctOfOpenInterestAll"`) ||
		!strings.Contains(cause.Error(), "JSON number") {
		t.Fatalf("cause = %v, want it to name the non-number member pctOfOpenInterestAll", cause)
	}
}
