package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// marketRoutes is the exact path and query table of the eleven market
// endpoints (crates/libfmp/src/endpoints/market.rs and its query tests),
// keyed by "path?rawquery" under the test client's "/router/stable" prefix.
// The snapshot queries send date, exchange, sector or industry; the sector
// histories send from, exchange, sector, to; the industry histories send
// industry, exchange, from, to; the mover lists send nothing.
var marketRoutes = map[string]string{
	"/router/stable/sector-performance-snapshot?date=2024-02-01&exchange=NEW+%2F+EXCHANGE&sector=Future+%26+Energy":       "sector_performance_snapshot.json",
	"/router/stable/industry-performance-snapshot?date=2024-02-01&industry=Advertising+Agencies":                          "industry_performance_snapshot.json",
	"/router/stable/sector-pe-snapshot?date=2024-02-01&exchange=NASDAQ":                                                   "sector_pe_snapshot.json",
	"/router/stable/industry-pe-snapshot?date=2024-02-01":                                                                 "industry_pe_snapshot.json",
	"/router/stable/historical-sector-performance?from=2024-02-01&exchange=NASDAQ&sector=Energy&to=2024-03-01":            "historical_sector_performance.json",
	"/router/stable/historical-industry-performance?industry=Biotechnology&exchange=NASDAQ&from=2024-02-01&to=2024-03-01": "historical_industry_performance.json",
	"/router/stable/historical-sector-pe?sector=Energy&to=2024-03-01":                                                     "historical_sector_pe.json",
	"/router/stable/historical-industry-pe?industry=Biotechnology&from=2024-02-01":                                        "historical_industry_pe.json",
	"/router/stable/biggest-gainers?": "biggest_gainers.json",
	"/router/stable/biggest-losers?":  "biggest_losers.json",
	"/router/stable/most-actives?":    "most_actives.json",
}

func marketRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := marketRoutes[r.URL.Path+"?"+r.URL.RawQuery]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s?%s", r.URL.Path, r.URL.RawQuery)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestMarketMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, marketRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	feb := mustParseDate(t, "2024-02-01")
	mar := mustParseDate(t, "2024-03-01")

	sectors, err := client.Market.SectorPerformanceSnapshot(ctx,
		NewSectorPerformanceSnapshotQuery(feb).WithExchange("NEW / EXCHANGE").WithSector("Future & Energy"))
	if err != nil || len(sectors) != 1 || sectors[0].AverageChange != -0.31481377464310634 {
		t.Fatalf("SectorPerformanceSnapshot = %+v, %v", sectors, err)
	}
	industries, err := client.Market.IndustryPerformanceSnapshot(ctx,
		NewIndustryPerformanceSnapshotQuery(feb).WithIndustry("Advertising Agencies"))
	if err != nil || len(industries) != 1 || industries[0].Industry != "Advertising Agencies" {
		t.Fatalf("IndustryPerformanceSnapshot = %+v, %v", industries, err)
	}
	sectorPes, err := client.Market.SectorPeSnapshot(ctx, NewSectorPeSnapshotQuery(feb).WithExchange("NASDAQ"))
	if err != nil || len(sectorPes) != 1 || sectorPes[0].Pe != 15.687711758428254 {
		t.Fatalf("SectorPeSnapshot = %+v, %v", sectorPes, err)
	}
	industryPes, err := client.Market.IndustryPeSnapshot(ctx, NewIndustryPeSnapshotQuery(feb))
	if err != nil || len(industryPes) != 1 || industryPes[0].Pe != 71.09601665201151 {
		t.Fatalf("IndustryPeSnapshot = %+v, %v", industryPes, err)
	}
	historicalSectors, err := client.Market.HistoricalSectorPerformance(ctx,
		NewHistoricalSectorPerformanceQuery("Energy").WithFrom(feb).WithExchange("NASDAQ").WithTo(mar))
	if err != nil || len(historicalSectors) != 1 || historicalSectors[0].AverageChange != 1.3989969286740689 {
		t.Fatalf("HistoricalSectorPerformance = %+v, %v", historicalSectors, err)
	}
	historicalIndustries, err := client.Market.HistoricalIndustryPerformance(ctx,
		NewHistoricalIndustryPerformanceQuery("Biotechnology").WithExchange("NASDAQ").WithFrom(feb).WithTo(mar))
	if err != nil || len(historicalIndustries) != 1 || historicalIndustries[0].Industry != "Biotechnology" {
		t.Fatalf("HistoricalIndustryPerformance = %+v, %v", historicalIndustries, err)
	}
	historicalSectorPes, err := client.Market.HistoricalSectorPe(ctx, NewHistoricalSectorPeQuery("Energy").WithTo(mar))
	if err != nil || len(historicalSectorPes) != 1 || historicalSectorPes[0].Pe != 5.4165892628211205 {
		t.Fatalf("HistoricalSectorPe = %+v, %v", historicalSectorPes, err)
	}
	historicalIndustryPes, err := client.Market.HistoricalIndustryPe(ctx,
		NewHistoricalIndustryPeQuery("Biotechnology").WithFrom(feb))
	if err != nil || len(historicalIndustryPes) != 1 || historicalIndustryPes[0].Pe != 8.129037884885042 {
		t.Fatalf("HistoricalIndustryPe = %+v, %v", historicalIndustryPes, err)
	}
	gainers, err := client.Market.BiggestGainers(ctx)
	if err != nil || len(gainers) != 1 || gainers[0].Symbol != "MOTS" || gainers[0].ChangesPercentage != 100 {
		t.Fatalf("BiggestGainers = %+v, %v", gainers, err)
	}
	losers, err := client.Market.BiggestLosers(ctx)
	if err != nil || len(losers) != 1 || losers[0].Symbol != "SPEC" || losers[0].Change != -0.002 {
		t.Fatalf("BiggestLosers = %+v, %v", losers, err)
	}
	actives, err := client.Market.MostActives(ctx)
	if err != nil || len(actives) != 1 || actives[0].Symbol != "LUCY" || actives[0].Price != 1.85 {
		t.Fatalf("MostActives = %+v, %v", actives, err)
	}

	want := []string{
		"/router/stable/sector-performance-snapshot?date=2024-02-01&exchange=NEW+%2F+EXCHANGE&sector=Future+%26+Energy",
		"/router/stable/industry-performance-snapshot?date=2024-02-01&industry=Advertising+Agencies",
		"/router/stable/sector-pe-snapshot?date=2024-02-01&exchange=NASDAQ",
		"/router/stable/industry-pe-snapshot?date=2024-02-01",
		"/router/stable/historical-sector-performance?from=2024-02-01&exchange=NASDAQ&sector=Energy&to=2024-03-01",
		"/router/stable/historical-industry-performance?industry=Biotechnology&exchange=NASDAQ&from=2024-02-01&to=2024-03-01",
		"/router/stable/historical-sector-pe?sector=Energy&to=2024-03-01",
		"/router/stable/historical-industry-pe?industry=Biotechnology&from=2024-02-01",
		"/router/stable/biggest-gainers?",
		"/router/stable/biggest-losers?",
		"/router/stable/most-actives?",
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

func TestMarketQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, marketRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	feb := mustParseDate(t, "2024-02-01")
	cases := []struct {
		name   string
		call   func() error
		member string
		reason error
	}{
		{"zero snapshot date", func() error {
			_, err := client.Market.SectorPerformanceSnapshot(ctx, NewSectorPerformanceSnapshotQuery(Date{}))
			return err
		}, "date", ErrZeroTemporalValue},
		{"blank snapshot exchange", func() error {
			_, err := client.Market.IndustryPeSnapshot(ctx, NewIndustryPeSnapshotQuery(feb).WithExchange(" "))
			return err
		}, "exchange", ErrEmptyValue},
		{"control snapshot industry", func() error {
			_, err := client.Market.IndustryPerformanceSnapshot(ctx,
				NewIndustryPerformanceSnapshotQuery(feb).WithIndustry("Bio\ntech"))
			return err
		}, "industry", ErrControlCharacterValue},
		{"empty history sector", func() error {
			_, err := client.Market.HistoricalSectorPerformance(ctx, NewHistoricalSectorPerformanceQuery(""))
			return err
		}, "sector", ErrEmptyValue},
		{"zero history from", func() error {
			_, err := client.Market.HistoricalSectorPe(ctx, NewHistoricalSectorPeQuery("Energy").WithFrom(Date{}))
			return err
		}, "from", ErrZeroTemporalValue},
		{"zero history to", func() error {
			_, err := client.Market.HistoricalIndustryPe(ctx, NewHistoricalIndustryPeQuery("Biotechnology").WithTo(Date{}))
			return err
		}, "to", ErrZeroTemporalValue},
	}
	for _, tc := range cases {
		err := tc.call()
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != tc.member+": "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want %s: %v", tc.name, err, tc.member, tc.reason)
		}
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	snapshot := NewSectorPeSnapshotQuery(feb)
	if snapshot.Date() != feb || snapshot.Exchange() != nil || snapshot.Sector() != nil {
		t.Fatalf("NewSectorPeSnapshotQuery set an optional filter: %+v", snapshot)
	}
	if q := snapshot.WithSector(" Basic Materials "); q.Sector() == nil || *q.Sector() != " Basic Materials " {
		t.Fatalf("Sector() normalized the provider text: %v", q.Sector())
	}
	history := NewHistoricalIndustryPerformanceQuery("Consumer Electronics, Retail")
	if history.Industry() != "Consumer Electronics, Retail" || history.Exchange() != nil ||
		history.From() != nil || history.To() != nil {
		t.Fatalf("NewHistoricalIndustryPerformanceQuery set an optional filter: %+v", history)
	}
	params, err := history.params()
	if err != nil || len(params) != 1 || params[0].Value != "Consumer Electronics, Retail" {
		t.Fatalf("a comma in the industry was not accepted as Industry allows: %v, %v", params, err)
	}
}

func TestMarketMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	fixture := string(readFixture(t, "biggest_gainers.json"))
	corrupted := strings.Replace(fixture, `"changesPercentage": 100,`, ``, 1)
	if corrupted == fixture {
		t.Fatal("biggest_gainers.json no longer spells the changesPercentage member as 100")
	}
	server, _ := newServer(t, jsonHandler(corrupted))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))

	_, err := client.Market.BiggestGainers(context.Background())
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "biggest-gainers")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"changesPercentage"`) {
		t.Fatalf("cause = %v, want it to name the missing member changesPercentage", cause)
	}
}
