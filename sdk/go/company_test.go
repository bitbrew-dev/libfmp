package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// companyRoutes is the exact request-URI table of one method per query shape
// of the company domain, copied from the URL assertions in
// crates/libfmp/tests/company_*_endpoints.rs. Keying on the full request URI
// lets one path carry its no-query and with-setter variants.
var companyRoutes = map[string]string{
	"/router/stable/profile?symbol=AAPL":                                                                   "company_profile.json",
	"/router/stable/profile-cik?cik=0000320193":                                                            "company_profile.json",
	"/router/stable/market-capitalization-batch?symbols=AAPL%2CMSFT":                                       "company_market_capitalization.json",
	"/router/stable/mergers-acquisitions-search?name=Pineapple+Energy":                                     "company_mergers_acquisitions_search.json",
	"/router/stable/delisted-companies":                                                                    "company_delisted.json",
	"/router/stable/delisted-companies?page=0&limit=101":                                                   "company_delisted.json",
	"/router/stable/employee-count?symbol=AAPL&limit=10001":                                                "company_employee_count.json",
	"/router/stable/historical-market-capitalization?symbol=AAPL&from=2026-04-16":                          "company_historical_market_capitalization.json",
	"/router/stable/historical-market-capitalization?symbol=AAPL&limit=5001&from=2026-07-16&to=2026-04-16": "company_historical_market_capitalization.json",
	"/router/stable/executive-compensation-benchmark":                                                      "company_executive_compensation_benchmark.json",
	"/router/stable/executive-compensation-benchmark?year=FY+2024%2F25":                                    "company_executive_compensation_benchmark.json",
	"/router/stable/key-executives?symbol=BRK.B":                                                           "company_key_executives_dynamic.json",
}

func companyRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := companyRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestCompanyMethodsUseExactPathsAndWireParameterOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, companyRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()

	profile, err := client.Company.Profile(ctx, NewProfileQuery("AAPL"))
	if err != nil || len(profile) != 1 || profile[0].Cik != "0000320193" {
		t.Fatalf("Profile = %+v, %v", profile, err)
	}
	byCik, err := client.Company.ProfileByCik(ctx, NewProfileByCikQuery("0000320193"))
	if err != nil || len(byCik) != 1 || byCik[0].Symbol != "AAPL" {
		t.Fatalf("ProfileByCik = %+v, %v", byCik, err)
	}
	batch, err := client.Company.MarketCapitalizationBatch(ctx, NewMarketCapitalizationBatchQuery([]string{"AAPL", "MSFT"}))
	if err != nil || len(batch) != 1 || batch[0].MarketCap != 4_874_072_686_740 {
		t.Fatalf("MarketCapitalizationBatch = %+v, %v", batch, err)
	}
	search, err := client.Company.MergersAcquisitionsSearch(ctx, NewMergersAcquisitionsSearchQuery("Pineapple Energy"))
	if err != nil || len(search) != 1 || search[0].TargetedSymbol != "JCS" {
		t.Fatalf("MergersAcquisitionsSearch = %+v, %v", search, err)
	}

	delisted, err := client.Company.DelistedCompanies(ctx, NewDelistedCompaniesQuery())
	if err != nil || len(delisted) != 1 || delisted[0].Symbol != "CCIX" {
		t.Fatalf("DelistedCompanies = %+v, %v", delisted, err)
	}
	paged, err := client.Company.DelistedCompanies(ctx, NewDelistedCompaniesQuery().WithPage(0).WithLimit(101))
	if err != nil || len(paged) != 1 {
		t.Fatalf("DelistedCompanies paged = %+v, %v", paged, err)
	}
	employees, err := client.Company.EmployeeCount(ctx, NewEmployeeCountQuery("AAPL").WithLimit(10_001))
	if err != nil || len(employees) != 1 || employees[0].EmployeeCount != 166_000 {
		t.Fatalf("EmployeeCount = %+v, %v", employees, err)
	}

	from := mustParseDate(t, "2026-04-16")
	to := mustParseDate(t, "2026-07-16")
	history, err := client.Company.HistoricalMarketCapitalization(ctx,
		NewHistoricalMarketCapitalizationQuery("AAPL").WithFrom(from))
	if err != nil || len(history) != 1 || history[0].MarketCap != 4_879_177_245_542 {
		t.Fatalf("HistoricalMarketCapitalization from = %+v, %v", history, err)
	}
	reversed, err := client.Company.HistoricalMarketCapitalization(ctx,
		NewHistoricalMarketCapitalizationQuery("AAPL").WithTo(from).WithFrom(to).WithLimit(5_001))
	if err != nil || len(reversed) != 1 {
		t.Fatalf("HistoricalMarketCapitalization all = %+v, %v", reversed, err)
	}

	benchmark, err := client.Company.ExecutiveCompensationBenchmark(ctx, NewExecutiveCompensationBenchmarkQuery())
	if err != nil || len(benchmark) != 1 || benchmark[0].Year != 2024 {
		t.Fatalf("ExecutiveCompensationBenchmark = %+v, %v", benchmark, err)
	}
	yearly, err := client.Company.ExecutiveCompensationBenchmark(ctx,
		NewExecutiveCompensationBenchmarkQuery().WithYear("FY 2024/25"))
	if err != nil || len(yearly) != 1 {
		t.Fatalf("ExecutiveCompensationBenchmark year = %+v, %v", yearly, err)
	}
	executives, err := client.Company.KeyExecutives(ctx, NewKeyExecutivesQuery("BRK.B"))
	if err != nil || len(executives) != 2 || executives[1].YearBorn == nil {
		t.Fatalf("KeyExecutives = %+v, %v", executives, err)
	}

	requests := rec.all()
	if len(requests) != len(companyRoutes) {
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

func TestCompanyQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, companyRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	cases := []struct {
		name     string
		call     func() error
		argument string
		reason   error
	}{
		{"empty cik", func() error {
			_, err := client.Company.ProfileByCik(ctx, NewProfileByCikQuery(""))
			return err
		}, "cik", ErrEmptyValue},
		{"whitespace search term", func() error {
			_, err := client.Company.MergersAcquisitionsSearch(ctx, NewMergersAcquisitionsSearchQuery(" \t"))
			return err
		}, "name", ErrEmptyValue},
		{"comma in batch ticker", func() error {
			_, err := client.Company.MarketCapitalizationBatch(ctx, NewMarketCapitalizationBatchQuery([]string{"AAPL,MSFT"}))
			return err
		}, "symbols", ErrCommaInTicker},
		{"control character ticker", func() error {
			_, err := client.Company.KeyExecutives(ctx, NewKeyExecutivesQuery("AA\nPL"))
			return err
		}, "symbol", ErrControlCharacterValue},
		{"zero from date", func() error {
			_, err := client.Company.HistoricalMarketCapitalization(ctx,
				NewHistoricalMarketCapitalizationQuery("AAPL").WithFrom(Date{}))
			return err
		}, "from", ErrZeroTemporalValue},
		{"empty benchmark year", func() error {
			_, err := client.Company.ExecutiveCompensationBenchmark(ctx, NewExecutiveCompensationBenchmarkQuery().WithYear(""))
			return err
		}, "year", ErrEmptyValue},
	}
	for _, tc := range cases {
		err := tc.call()
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != tc.argument+": "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want %s: %v", tc.name, err, tc.argument, tc.reason)
		}
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	base := NewDelistedCompaniesQuery()
	paged := base.WithPage(0)
	if base.Page() != nil || paged.Page() == nil || *paged.Page() != 0 || paged.Limit() != nil {
		t.Fatalf("WithPage mutated the receiver or dropped the zero page: base=%+v paged=%+v", base, paged)
	}
	if q := NewProfileByCikQuery(" 0000320193 "); q.Cik() != " 0000320193 " {
		t.Fatalf("Cik() normalized the value: %q", q.Cik())
	}
}

func TestCompanyMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, _ := newServer(t, jsonHandler(string(readFixture(t, "company_shares_float_all.json"))))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))

	_, err := client.Company.SharesFloat(context.Background(), NewSharesFloatQuery("AAPL"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "shares-float")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"source"`) {
		t.Fatalf("cause = %v, want it to name the missing member source", cause)
	}
}
