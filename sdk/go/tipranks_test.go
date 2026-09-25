package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// tipranksRoutes is the exact request-URI table of the seven TipRanks
// endpoints, copied from the URL assertions in
// crates/libfmp/tests/tipranks_*_endpoints.rs. Keying on the full request URI
// lets one path carry its bare, required-only, and every-setter variants
// (wire order and escaping matter: spaces and slashes are escaped as the Rust
// encoder does, and an explicit false flag is sent, not dropped).
var tipranksRoutes = map[string]string{
	"/router/stable/tipranks-search": "tipranks_ratings_search.json",
	"/router/stable/tipranks-search?expertUID=expert+%2F+one&symbol=RR.L&from=2025-06-10&to=2026-06-10&limit=5000&page=0&nonadjusted=false": "tipranks_ratings_search.json",
	"/router/stable/tipranks-pit-symbol?symbol=AAPL":                                                                                         "tipranks_point_in_time_symbol.json",
	"/router/stable/tipranks-pit-symbol?symbol=BRK.B&date=2026-06-10&limit=5000&page=0&nonadjusted=false":                                    "tipranks_point_in_time_symbol.json",
	"/router/stable/tipranks-pit-analyst":                                                                                                    "tipranks_point_in_time_analyst.json",
	"/router/stable/tipranks-pit-analyst?expertUID=expert+%2F+one&analystName=Keegan+Cox&date=2026-06-10&limit=100&page=0&nonadjusted=false": "tipranks_point_in_time_analyst.json",
	"/router/stable/tipranks-symbol-summary?symbol=AAPL":                                                                                     "tipranks_symbol_summary.json",
	"/router/stable/tipranks-symbol-summary?symbol=BRK.B&from=2025-06-10&to=2026-06-10":                                                      "tipranks_symbol_summary.json",
	"/router/stable/tipranks-analyst-summary?expertUID=expert+%2F+one&to=2026-06-10":                                                         "tipranks_analyst_summary.json",
	"/router/stable/tipranks-firm-summary?firmName=Morgan+Stanley+%2F+Asia&from=2025-06-10":                                                  "tipranks_firm_summary.json",
	"/router/stable/tipranks-analysts":                                                                                                       "tipranks_analysts.json",
	"/router/stable/tipranks-analysts?page=0&limit=1000&firmName=Morgan+Stanley+%2F+Asia&analystName=Andrew+Marok+%2F+Exact":                 "tipranks_analysts.json",
}

func tipranksRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := tipranksRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestTipranksMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, tipranksRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	tip := client.Tipranks
	from := mustParseDate(t, "2025-06-10")
	to := mustParseDate(t, "2026-06-10")

	search, err := tip.SearchRatings(ctx, NewTipRanksSearchQuery())
	if err != nil || len(search) != 1 || search[0].Symbol != "RR.L" {
		t.Fatalf("SearchRatings = %+v, %v", search, err)
	}
	full := NewTipRanksSearchQuery().WithExpertUid("expert / one").WithSymbol("RR.L").WithFrom(from).WithTo(to).
		WithLimit(5_000).WithPage(0).WithNonadjusted(false)
	if _, err := tip.SearchRatings(ctx, full); err != nil {
		t.Fatalf("SearchRatings with every setter: %v", err)
	}
	pitSymbol, err := tip.PointInTimeRatingsBySymbol(ctx, NewPointInTimeRatingsBySymbolQuery("AAPL"))
	if err != nil || len(pitSymbol) != 1 || pitSymbol[0].PriceTarget == nil || string(*pitSymbol[0].PriceTarget) != "380" {
		t.Fatalf("PointInTimeRatingsBySymbol = %+v, %v", pitSymbol, err)
	}
	if _, err := tip.PointInTimeRatingsBySymbol(ctx, NewPointInTimeRatingsBySymbolQuery("BRK.B").WithDate(to).
		WithLimit(5_000).WithPage(0).WithNonadjusted(false)); err != nil {
		t.Fatalf("PointInTimeRatingsBySymbol with every setter: %v", err)
	}
	pitAnalyst, err := tip.PointInTimeRatingsByAnalyst(ctx, NewPointInTimeRatingsByAnalystQuery())
	if err != nil || len(pitAnalyst) != 1 || pitAnalyst[0].Symbol != "0J3K.L" || pitAnalyst[0].PriceTarget != nil {
		t.Fatalf("PointInTimeRatingsByAnalyst = %+v, %v", pitAnalyst, err)
	}
	if _, err := tip.PointInTimeRatingsByAnalyst(ctx, NewPointInTimeRatingsByAnalystQuery().WithExpertUid("expert / one").
		WithAnalystName("Keegan Cox").WithDate(to).WithLimit(100).WithPage(0).WithNonadjusted(false)); err != nil {
		t.Fatalf("PointInTimeRatingsByAnalyst with every setter: %v", err)
	}
	symbolSummary, err := tip.SymbolSummary(ctx, NewTipRanksSymbolSummaryQuery("AAPL"))
	if err != nil || len(symbolSummary) != 1 || symbolSummary[0].TotalRecommendations != 425 {
		t.Fatalf("SymbolSummary = %+v, %v", symbolSummary, err)
	}
	if _, err := tip.SymbolSummary(ctx, NewTipRanksSymbolSummaryQuery("BRK.B").WithFrom(from).WithTo(to)); err != nil {
		t.Fatalf("SymbolSummary with both dates: %v", err)
	}
	analystSummary, err := tip.AnalystSummary(ctx, NewTipRanksAnalystSummaryQuery("expert / one").WithTo(to))
	if err != nil || len(analystSummary) != 1 || analystSummary[0].Misses != 11 {
		t.Fatalf("AnalystSummary = %+v, %v", analystSummary, err)
	}
	firmSummary, err := tip.FirmSummary(ctx, NewTipRanksFirmSummaryQuery("Morgan Stanley / Asia").WithFrom(from))
	if err != nil || len(firmSummary) != 1 || firmSummary[0].FirmName != "Morgan Stanley" {
		t.Fatalf("FirmSummary = %+v, %v", firmSummary, err)
	}
	analysts, err := tip.Analysts(ctx, NewTipRanksAnalystsQuery())
	if err != nil || len(analysts) != 1 || analysts[0].NumOfStars != 5 {
		t.Fatalf("Analysts = %+v, %v", analysts, err)
	}
	if _, err := tip.Analysts(ctx, NewTipRanksAnalystsQuery().WithPage(0).WithLimit(1_000).
		WithFirmName("Morgan Stanley / Asia").WithAnalystName("Andrew Marok / Exact")); err != nil {
		t.Fatalf("Analysts with every setter: %v", err)
	}

	want := []string{
		"/router/stable/tipranks-search",
		"/router/stable/tipranks-search?expertUID=expert+%2F+one&symbol=RR.L&from=2025-06-10&to=2026-06-10&limit=5000&page=0&nonadjusted=false",
		"/router/stable/tipranks-pit-symbol?symbol=AAPL",
		"/router/stable/tipranks-pit-symbol?symbol=BRK.B&date=2026-06-10&limit=5000&page=0&nonadjusted=false",
		"/router/stable/tipranks-pit-analyst",
		"/router/stable/tipranks-pit-analyst?expertUID=expert+%2F+one&analystName=Keegan+Cox&date=2026-06-10&limit=100&page=0&nonadjusted=false",
		"/router/stable/tipranks-symbol-summary?symbol=AAPL",
		"/router/stable/tipranks-symbol-summary?symbol=BRK.B&from=2025-06-10&to=2026-06-10",
		"/router/stable/tipranks-analyst-summary?expertUID=expert+%2F+one&to=2026-06-10",
		"/router/stable/tipranks-firm-summary?firmName=Morgan+Stanley+%2F+Asia&from=2025-06-10",
		"/router/stable/tipranks-analysts",
		"/router/stable/tipranks-analysts?page=0&limit=1000&firmName=Morgan+Stanley+%2F+Asia&analystName=Andrew+Marok+%2F+Exact",
	}
	requests := rec.all()
	if len(requests) != len(want) {
		t.Fatalf("requests = %d, want exactly one per call", len(requests))
	}
	for index, req := range requests {
		if got := req.URL.RequestURI(); got != want[index] {
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

func TestTipranksQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, tipranksRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	tip := client.Tipranks
	cases := []struct {
		name   string
		call   func() error
		member string
		reason error
	}{
		{"empty required expert uid", func() error {
			_, err := tip.AnalystSummary(ctx, NewTipRanksAnalystSummaryQuery(""))
			return err
		}, "expertUID", ErrEmptyValue},
		{"control character in optional expert uid", func() error {
			_, err := tip.SearchRatings(ctx, NewTipRanksSearchQuery().WithExpertUid("expert\nuid"))
			return err
		}, "expertUID", ErrControlCharacterValue},
		{"comma in optional ticker", func() error {
			_, err := tip.SearchRatings(ctx, NewTipRanksSearchQuery().WithSymbol("RR.L,AAPL"))
			return err
		}, "symbol", ErrCommaInTicker},
		{"blank required ticker", func() error {
			_, err := tip.PointInTimeRatingsBySymbol(ctx, NewPointInTimeRatingsBySymbolQuery(" \t "))
			return err
		}, "symbol", ErrEmptyValue},
		{"blank firm name", func() error {
			_, err := tip.FirmSummary(ctx, NewTipRanksFirmSummaryQuery(" "))
			return err
		}, "firmName", ErrEmptyValue},
		{"empty optional analyst name", func() error {
			_, err := tip.Analysts(ctx, NewTipRanksAnalystsQuery().WithAnalystName(""))
			return err
		}, "analystName", ErrEmptyValue},
		{"zero optional date", func() error {
			_, err := tip.SymbolSummary(ctx, NewTipRanksSymbolSummaryQuery("AAPL").WithFrom(Date{}))
			return err
		}, "from", ErrZeroTemporalValue},
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

	empty := NewTipRanksSearchQuery()
	if empty.ExpertUid() != nil || empty.Symbol() != nil || empty.From() != nil || empty.To() != nil ||
		empty.Limit() != nil || empty.Page() != nil || empty.Nonadjusted() != nil {
		t.Fatalf("NewTipRanksSearchQuery() set a parameter: %+v", empty)
	}
	if flag := empty.WithNonadjusted(false).Nonadjusted(); flag == nil || *flag || empty.Nonadjusted() != nil {
		t.Fatalf("WithNonadjusted(false) lost the explicit false or mutated the receiver: %v %v", flag, empty.Nonadjusted())
	}
	if q := NewTipRanksAnalystSummaryQuery("expert / one"); q.ExpertUid() != "expert / one" || q.From() != nil {
		t.Fatalf("ExpertUid() normalized the identifier: %q", q.ExpertUid())
	}
}

func TestTipranksMethodsReportNumberKindAndMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	fixture := string(readFixture(t, "tipranks_ratings_search.json"))
	corrupted := strings.Replace(fixture, `"priceTarget": 1500,`, `"priceTarget": "1500",`, 1)
	if corrupted == fixture {
		t.Fatal("tipranks_ratings_search.json no longer spells priceTarget as the integer 1500")
	}
	server, rec := newServer(t, jsonHandler(corrupted))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Tipranks.SearchRatings(context.Background(), NewTipRanksSearchQuery())
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "tipranks-search")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"priceTarget"`) ||
		!strings.Contains(cause.Error(), "JSON number") {
		t.Fatalf("cause = %v, want it to name the non-number member priceTarget", cause)
	}

	missing, _ := newServer(t, jsonHandler(`[{"expertUID":"0458","analystName":"Sujeeva De Silva","firmName":"Roth MKM"}]`))
	client = newClient(t, missing, WithAuthentication(FMPHeader("route-secret")))
	_, err = client.Tipranks.Analysts(context.Background(), NewTipRanksAnalystsQuery())
	typed = assertQuoteError(t, err, CategoryDecode, http.StatusOK, "tipranks-analysts")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"successRate"`) {
		t.Fatalf("cause = %v, want it to name the first missing member successRate", cause)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}
