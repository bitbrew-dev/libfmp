package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// analystRoutes is the exact path and query table of the eight analyst
// endpoints, one per query shape: required ticker and frequency with both
// setters (analyst-estimates, wire text copied from
// crates/libfmp/tests/analyst_estimates_ratings_endpoints.rs), ticker only,
// and ticker plus an optional limit that is set (ratings-historical) or left
// unset (grades-historical).
var analystRoutes = map[string]struct {
	query   string
	fixture string
}{
	"/router/stable/analyst-estimates":      {"symbol=BRK.B+%2F+Class+A&period=quarter&page=0&limit=4294967295", "financial_estimates.json"},
	"/router/stable/ratings-snapshot":       {"symbol=AAPL", "ratings_snapshot.json"},
	"/router/stable/ratings-historical":     {"symbol=AAPL&limit=0", "historical_ratings.json"},
	"/router/stable/price-target-summary":   {"symbol=AAPL", "price_target_summary.json"},
	"/router/stable/price-target-consensus": {"symbol=AAPL", "price_target_consensus.json"},
	"/router/stable/grades":                 {"symbol=AAPL", "stock_grades.json"},
	"/router/stable/grades-historical":      {"symbol=AAPL", "historical_stock_grades.json"},
	"/router/stable/grades-consensus":       {"symbol=AAPL", "stock_grades_summary.json"},
}

func analystRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		route, ok := analystRoutes[r.URL.Path]
		if !ok || r.URL.RawQuery != route.query || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s?%s", r.URL.Path, r.URL.RawQuery)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, route.fixture))
	}
}

func TestAnalystMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, analystRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()

	estimates, err := client.Analyst.FinancialEstimates(ctx,
		NewFinancialEstimatesQuery("BRK.B / Class A", RetrievalFrequencyQuarterly).WithPage(0).WithLimit(4_294_967_295))
	if err != nil || len(estimates) != 1 || estimates[0].NumAnalystsRevenue != 16 {
		t.Fatalf("FinancialEstimates = %+v, %v", estimates, err)
	}
	snapshot, err := client.Analyst.RatingsSnapshot(ctx, NewRatingsSnapshotQuery("AAPL"))
	if err != nil || len(snapshot) != 1 || snapshot[0].Rating != "B" {
		t.Fatalf("RatingsSnapshot = %+v, %v", snapshot, err)
	}
	ratings, err := client.Analyst.HistoricalRatings(ctx, NewHistoricalRatingsQuery("AAPL").WithLimit(0))
	if err != nil || len(ratings) != 1 || ratings[0].PriceToBookScore != 1 {
		t.Fatalf("HistoricalRatings = %+v, %v", ratings, err)
	}
	summary, err := client.Analyst.PriceTargetSummary(ctx, NewPriceTargetSummaryQuery("AAPL"))
	if err != nil || len(summary) != 1 || summary[0].AllTimeCount != 254 {
		t.Fatalf("PriceTargetSummary = %+v, %v", summary, err)
	}
	consensus, err := client.Analyst.PriceTargetConsensus(ctx, NewPriceTargetConsensusQuery("AAPL"))
	if err != nil || len(consensus) != 1 || consensus[0].TargetConsensus != 337.67 {
		t.Fatalf("PriceTargetConsensus = %+v, %v", consensus, err)
	}
	grades, err := client.Analyst.StockGrades(ctx, NewStockGradesQuery("AAPL"))
	if err != nil || len(grades) != 1 || grades[0].GradingCompany != "Morgan Stanley" {
		t.Fatalf("StockGrades = %+v, %v", grades, err)
	}
	historical, err := client.Analyst.HistoricalStockGrades(ctx, NewHistoricalStockGradesQuery("AAPL"))
	if err != nil || len(historical) != 1 || historical[0].AnalystRatingsStrongBuy != 6 {
		t.Fatalf("HistoricalStockGrades = %+v, %v", historical, err)
	}
	buckets, err := client.Analyst.StockGradesSummary(ctx, NewStockGradesSummaryQuery("AAPL"))
	if err != nil || len(buckets) != 1 || buckets[0].Consensus != "Buy" {
		t.Fatalf("StockGradesSummary = %+v, %v", buckets, err)
	}

	requests := rec.all()
	if len(requests) != len(analystRoutes) {
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

func TestAnalystQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, analystRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()

	_, err := client.Analyst.FinancialEstimates(ctx, NewFinancialEstimatesQuery("AAPL", "yearly"))
	typed := assertQuoteError(t, err, CategoryValidation, 0, "")
	if !errors.Is(err, ErrUnknownWireValue) || !strings.HasPrefix(typed.Message, "period: ") {
		t.Fatalf("unknown frequency: error = %v", err)
	}
	_, err = client.Analyst.FinancialEstimates(ctx, NewFinancialEstimatesQuery("AAPL,MSFT", RetrievalFrequencyAnnual))
	if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrCommaInTicker) ||
		typed.Message != "symbol: "+ErrCommaInTicker.Error() {
		t.Fatalf("comma ticker: error = %v", err)
	}
	_, err = client.Analyst.HistoricalStockGrades(ctx, NewHistoricalStockGradesQuery(" ").WithLimit(5))
	if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrEmptyValue) ||
		typed.Message != "symbol: "+ErrEmptyValue.Error() {
		t.Fatalf("blank ticker: error = %v", err)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	q := NewFinancialEstimatesQuery(" AAPL ", RetrievalFrequencyAnnual)
	if q.Symbol() != " AAPL " || q.Period() != RetrievalFrequencyAnnual || q.Page() != nil || q.Limit() != nil {
		t.Fatalf("getters = %q %q %v %v", q.Symbol(), q.Period(), q.Page(), q.Limit())
	}
	if limit := q.WithLimit(7).Limit(); limit == nil || *limit != 7 || q.Limit() != nil {
		t.Fatalf("WithLimit mutated the receiver or lost the value: %v %v", limit, q.Limit())
	}
}

func TestAnalystMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[{"symbol":"AAPL","targetHigh":400,"targetLow":250}]`))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))

	_, err := client.Analyst.PriceTargetConsensus(context.Background(), NewPriceTargetConsensusQuery("AAPL"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "price-target-consensus")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"targetConsensus"`) {
		t.Fatalf("cause = %v, want it to name the missing member targetConsensus", cause)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}
