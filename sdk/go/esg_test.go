package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// esgRoutes is the exact path and query table of the three ESG endpoints
// (crates/libfmp/tests/esg_endpoints.rs), keyed by "path?rawquery" under the
// test client's "/router/stable" prefix. The benchmark path appears three
// times because its only parameter is optional and is sent verbatim.
var esgRoutes = map[string]string{
	"/router/stable/esg-disclosures?symbol=BRK.B+%2F+Class+A": "esg_disclosures.json",
	"/router/stable/esg-ratings?symbol=BRK.B+%2F+Class+A":     "esg_ratings.json",
	"/router/stable/esg-benchmark?":                           "esg_benchmark.json",
	"/router/stable/esg-benchmark?year=FY+2024%2F25":          "esg_benchmark.json",
	"/router/stable/esg-benchmark?year=2023":                  "esg_benchmark.json",
}

func esgRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := esgRoutes[r.URL.Path+"?"+r.URL.RawQuery]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s?%s", r.URL.Path, r.URL.RawQuery)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestEsgMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, esgRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	symbol := NewEsgSymbolQuery("BRK.B / Class A")

	disclosures, err := client.Esg.Disclosures(ctx, symbol)
	if err != nil || len(disclosures) != 1 || disclosures[0].EsgScore != 56.79 {
		t.Fatalf("Disclosures = %+v, %v", disclosures, err)
	}
	ratings, err := client.Esg.Ratings(ctx, symbol)
	if err != nil || len(ratings) != 1 || ratings[0].EsgRiskRating != "B" {
		t.Fatalf("Ratings = %+v, %v", ratings, err)
	}
	benchmarks, err := client.Esg.Benchmark(ctx, NewEsgBenchmarkQuery())
	if err != nil || len(benchmarks) != 1 || benchmarks[0].EsgScore != 65.63 {
		t.Fatalf("Benchmark = %+v, %v", benchmarks, err)
	}
	if _, err := client.Esg.Benchmark(ctx, NewEsgBenchmarkQuery().WithYear("FY 2024/25")); err != nil {
		t.Fatalf("Benchmark with a provider year string: %v", err)
	}
	if _, err := client.Esg.Benchmark(ctx, NewEsgBenchmarkQuery().WithYear("2023")); err != nil {
		t.Fatalf("Benchmark with a plain year: %v", err)
	}

	want := []string{
		"/router/stable/esg-disclosures?symbol=BRK.B+%2F+Class+A",
		"/router/stable/esg-ratings?symbol=BRK.B+%2F+Class+A",
		"/router/stable/esg-benchmark?",
		"/router/stable/esg-benchmark?year=FY+2024%2F25",
		"/router/stable/esg-benchmark?year=2023",
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

func TestEsgQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, esgRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
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
		_, err := client.Esg.Disclosures(ctx, NewEsgSymbolQuery(tc.symbol))
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != "symbol: "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want symbol: %v", tc.name, err, tc.reason)
		}
		_, err = client.Esg.Ratings(ctx, NewEsgSymbolQuery(tc.symbol))
		if ratings := assertQuoteError(t, err, CategoryValidation, 0, ""); ratings.Message != typed.Message {
			t.Fatalf("%s: Ratings message %q differs from Disclosures message %q", tc.name, ratings.Message, typed.Message)
		}
	}
	for _, tc := range []struct {
		name   string
		year   string
		reason error
	}{
		{"blank year", " ", ErrEmptyValue},
		{"control year", "20\x0023", ErrControlCharacterValue},
	} {
		_, err := client.Esg.Benchmark(ctx, NewEsgBenchmarkQuery().WithYear(tc.year))
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != "year: "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want year: %v", tc.name, err, tc.reason)
		}
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}
	if q := NewEsgSymbolQuery(" AAPL "); q.Symbol() != " AAPL " {
		t.Fatalf("Symbol() normalized the ticker: %q", q.Symbol())
	}
	if q := NewEsgBenchmarkQuery(); q.Year() != nil {
		t.Fatalf("NewEsgBenchmarkQuery() set a year: %v", *q.Year())
	}
	if q := NewEsgBenchmarkQuery().WithYear("FY 2024/25"); q.Year() == nil || *q.Year() != "FY 2024/25" {
		t.Fatalf("Year() normalized the provider string: %v", q.Year())
	}
	commaYear, err := NewEsgBenchmarkQuery().WithYear("2023,2024").params()
	if err != nil || len(commaYear) != 1 || commaYear[0].Value != "2023,2024" {
		t.Fatalf("a comma in the benchmark year was not accepted as BenchmarkYear allows: %v, %v", commaYear, err)
	}
}

func TestEsgMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	fixture := string(readFixture(t, "esg_disclosures.json"))
	corrupted := strings.Replace(fixture, `"ESGScore": 56.79,`, ``, 1)
	if corrupted == fixture {
		t.Fatal("esg_disclosures.json no longer spells the ESGScore member as 56.79")
	}
	server, _ := newServer(t, jsonHandler(corrupted))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Esg.Disclosures(context.Background(), NewEsgSymbolQuery("AAPL"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "esg-disclosures")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"ESGScore"`) {
		t.Fatalf("cause = %v, want it to name the missing member ESGScore", cause)
	}
}
