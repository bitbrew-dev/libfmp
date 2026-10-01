package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// bulkRoutes is the exact request-URI table of every bulk method, one per
// registry entry, with the wire order copied from the URL assertions in
// crates/libfmp/tests/bulk_*_endpoints.rs (year before period, one GET, no
// fan-out over parts).
var bulkRoutes = map[string]string{
	"/router/stable/profile-bulk?part=0":                                     "bulk_company_profiles.csv",
	"/router/stable/rating-bulk":                                             "bulk_stock_ratings.csv",
	"/router/stable/dcf-bulk":                                                "bulk_dcf_valuations.csv",
	"/router/stable/scores-bulk":                                             "bulk_financial_scores.csv",
	"/router/stable/price-target-summary-bulk":                               "bulk_price_target_summaries.csv",
	"/router/stable/etf-holder-bulk?part=segment+A%2F7":                      "bulk_etf_holdings.csv",
	"/router/stable/upgrades-downgrades-consensus-bulk":                      "bulk_upgrades_downgrades_consensus.csv",
	"/router/stable/key-metrics-ttm-bulk":                                    "bulk_key_metrics_ttm.csv",
	"/router/stable/ratios-ttm-bulk":                                         "bulk_financial_ratios_ttm.csv",
	"/router/stable/peers-bulk":                                              "bulk_stock_peers.csv",
	"/router/stable/earnings-surprises-bulk?year=4294967295":                 "bulk_earnings_surprises.csv",
	"/router/stable/income-statement-bulk?year=2026&period=Q1":               "bulk_income_statements.csv",
	"/router/stable/income-statement-growth-bulk?year=2026&period=Q2":        "bulk_income_statement_growth.csv",
	"/router/stable/balance-sheet-statement-bulk?year=2026&period=Q3":        "bulk_balance_sheet_statements.csv",
	"/router/stable/balance-sheet-statement-growth-bulk?year=2026&period=Q4": "bulk_balance_sheet_statement_growth.csv",
	"/router/stable/cash-flow-statement-bulk?year=4294967295&period=FY":      "bulk_cash_flow_statements.csv",
	"/router/stable/cash-flow-statement-growth-bulk?year=2025&period=FY":     "bulk_cash_flow_statement_growth.csv",
	"/router/stable/eod-bulk?date=2024-10-22":                                "bulk_eod.csv",
}

func bulkRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := bulkRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		if strings.HasSuffix(fixture, ".csv") {
			w.Header().Set("Content-Type", "text/csv")
		}
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestBulkMethodsUseExactPathsAndWireParameterOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, bulkRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	profiles, err := client.Bulk.CompanyProfiles(ctx, NewBulkPartQuery("0"))
	if err != nil || len(profiles) != 4 || profiles[0].Symbol != "WMB" || profiles[0].MarketCap != 82_906_291_252 {
		t.Fatalf("CompanyProfiles = %+v, %v", profiles, err)
	}
	ratings, err := client.Bulk.StockRatings(ctx)
	if err != nil || len(ratings) != 2 || ratings[1].Rating != "C+" {
		t.Fatalf("StockRatings = %+v, %v", ratings, err)
	}
	dcf, err := client.Bulk.DCFValuations(ctx)
	if err != nil || len(dcf) != 3 || cellText(dcf[1].StockPrice) != "2.39" {
		t.Fatalf("DCFValuations = %+v, %v", dcf, err)
	}
	scores, err := client.Bulk.FinancialScores(ctx)
	if err != nil || len(scores) != 3 || cellText(scores[0].PiotroskiScore) != "4" {
		t.Fatalf("FinancialScores = %+v, %v", scores, err)
	}
	targets, err := client.Bulk.PriceTargetSummaries(ctx)
	if err != nil || len(targets) != 2 || targets[1].Symbol != "AA" {
		t.Fatalf("PriceTargetSummaries = %+v, %v", targets, err)
	}
	holdings, err := client.Bulk.ETFHoldings(ctx, NewBulkPartQuery("segment A/7"))
	if err != nil || len(holdings) != 4 || cellText(holdings[1].Asset) != "3665.TW" {
		t.Fatalf("ETFHoldings = %+v, %v", holdings, err)
	}
	consensus, err := client.Bulk.UpgradesDowngradesConsensus(ctx)
	if err != nil || len(consensus) != 2 || consensus[1].Consensus != "Hold" {
		t.Fatalf("UpgradesDowngradesConsensus = %+v, %v", consensus, err)
	}
	metrics, err := client.Bulk.KeyMetricsTTM(ctx)
	if err != nil || len(metrics) != 3 || cellText(metrics[0].MarketCap) != "224526473551" {
		t.Fatalf("KeyMetricsTTM = %+v, %v", metrics, err)
	}
	ratios, err := client.Bulk.FinancialRatiosTTM(ctx)
	if err != nil || len(ratios) != 3 || cellText(ratios[0].GrossProfitMarginTTM) != "0.5535250166330814" {
		t.Fatalf("FinancialRatiosTTM = %+v, %v", ratios, err)
	}
	peers, err := client.Bulk.StockPeers(ctx)
	if err != nil || len(peers) != 3 || peers[0].Peers != "3698.HK,600000.SS,600015.SS,600016.SS,600036.SS,601166.SS,601658.SS" {
		t.Fatalf("StockPeers = %+v, %v", peers, err)
	}
	surprises, err := client.Bulk.EarningsSurprises(ctx, NewBulkYearQuery(4_294_967_295))
	if err != nil || len(surprises) != 3 || surprises[0].Symbol != "AUTO.OL" {
		t.Fatalf("EarningsSurprises = %+v, %v", surprises, err)
	}
	income, err := client.Bulk.IncomeStatements(ctx, NewBulkStatementQuery(2026, FiscalPeriodQ1))
	if err != nil || len(income) != 3 || cellText(income[0].Revenue) != "251641000000" {
		t.Fatalf("IncomeStatements = %+v, %v", income, err)
	}
	incomeGrowth, err := client.Bulk.IncomeStatementGrowth(ctx, NewBulkStatementQuery(2026, FiscalPeriodQ2))
	if err != nil || len(incomeGrowth) != 2 || cellText(incomeGrowth[0].GrowthRevenue) != "-0.08220846812871789" {
		t.Fatalf("IncomeStatementGrowth = %+v, %v", incomeGrowth, err)
	}
	balance, err := client.Bulk.BalanceSheetStatements(ctx, NewBulkStatementQuery(2026, FiscalPeriodQ3))
	if err != nil || len(balance) != 2 || balance[1].Symbol != "0002.KL" {
		t.Fatalf("BalanceSheetStatements = %+v, %v", balance, err)
	}
	balanceGrowth, err := client.Bulk.BalanceSheetStatementGrowth(ctx, NewBulkStatementQuery(2026, FiscalPeriodQ4))
	if err != nil || len(balanceGrowth) != 2 || cellText(balanceGrowth[0].GrowthShortTermInvestments) != "0.3981363245708905" {
		t.Fatalf("BalanceSheetStatementGrowth = %+v, %v", balanceGrowth, err)
	}
	cashFlow, err := client.Bulk.CashFlowStatements(ctx, NewBulkStatementQuery(4_294_967_295, FiscalPeriodFullYear))
	if err != nil || len(cashFlow) != 2 || cellText(cashFlow[0].NetIncome) != "14932000000" {
		t.Fatalf("CashFlowStatements = %+v, %v", cashFlow, err)
	}
	cashGrowth, err := client.Bulk.CashFlowStatementGrowth(ctx, NewBulkStatementQuery(2025, FiscalPeriodFullYear))
	if err != nil || len(cashGrowth) != 2 || cellText(cashGrowth[0].GrowthNetIncome) != "-0.04191152728446884" {
		t.Fatalf("CashFlowStatementGrowth = %+v, %v", cashGrowth, err)
	}
	eod, err := client.Bulk.Eod(ctx, NewBulkEodQuery(mustParseDate(t, "2024-10-22")))
	if err != nil || len(eod) != 2 || cellText(eod[1].Close) != "0.91741" {
		t.Fatalf("Eod = %+v, %v", eod, err)
	}

	requests := rec.all()
	if len(requests) != len(bulkRoutes) {
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

// BulkPart mirrors the open Rust BulkPart (non-empty, control-free, no
// numeric inference); the year is the unvalidated u32 Year; the period is the
// closed FiscalPeriod wire enum; the date is the strict Date.
func TestBulkQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, bulkRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	cases := []struct {
		name     string
		call     func() error
		argument string
		reason   error
		message  string
	}{
		{"empty part", func() error {
			_, err := client.Bulk.CompanyProfiles(ctx, NewBulkPartQuery(""))
			return err
		}, "part", ErrEmptyValue, ""},
		{"whitespace part", func() error {
			_, err := client.Bulk.ETFHoldings(ctx, NewBulkPartQuery("  "))
			return err
		}, "part", ErrEmptyValue, ""},
		{"control character part", func() error {
			_, err := client.Bulk.ETFHoldings(ctx, NewBulkPartQuery("part\n1"))
			return err
		}, "part", ErrControlCharacterValue, ""},
		{"retrieval frequency is not a fiscal period", func() error {
			_, err := client.Bulk.IncomeStatements(ctx, NewBulkStatementQuery(2026, FiscalPeriod("annual")))
			return err
		}, "period", ErrUnknownWireValue, "period: " + ErrUnknownWireValue.Error() + ", expected one of Q1, Q2, Q3, Q4, FY"},
		{"zero date", func() error {
			_, err := client.Bulk.Eod(ctx, NewBulkEodQuery(Date{}))
			return err
		}, "date", ErrZeroTemporalValue, ""},
	}
	for _, tc := range cases {
		err := tc.call()
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		want := tc.message
		if want == "" {
			want = tc.argument + ": " + tc.reason.Error()
		}
		if !errors.Is(err, tc.reason) || typed.Message != want {
			t.Fatalf("%s: error = %v, want %q", tc.name, err, want)
		}
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	if q := NewBulkPartQuery(" 0001 "); q.Part() != " 0001 " {
		t.Fatalf("Part() normalized the value: %q", q.Part())
	}
	if q := NewBulkStatementQuery(2026, FiscalPeriodQ1); q.Year() != 2026 || q.Period() != FiscalPeriodQ1 {
		t.Fatalf("BulkStatementQuery accessors = %d, %q", q.Year(), q.Period())
	}
	if q := NewBulkYearQuery(0); q.Year() != 0 {
		t.Fatalf("Year() = %d, want the unvalidated 0", q.Year())
	}
}

// A bulk body missing a model member's column surfaces as a Decode error on
// the endpoint at the first row and that member.
func TestBulkMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	lines := strings.Split(strings.TrimSuffix(string(readFixture(t, "bulk_etf_holdings.csv")), "\n"), "\n")
	if !strings.HasSuffix(lines[0], `,"lastUpdated"`) {
		t.Fatal("fixture no longer ends its header with the lastUpdated column")
	}
	for index, line := range lines {
		lines[index] = line[:strings.LastIndex(line, ",")]
	}
	body := strings.Join(lines, "\n") + "\n"
	server, _ := newServer(t, func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "text/csv")
		_, _ = w.Write([]byte(body))
	})
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Bulk.ETFHoldings(context.Background(), NewBulkPartQuery("0"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "etf-holder-bulk")
	if typed.Path != "/0/lastUpdated" || typed.DecodeKind != DecodeKindMissingMember {
		t.Fatalf("error = %+v, want a missing member at /0/lastUpdated", typed)
	}
}
