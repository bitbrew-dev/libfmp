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
	"/router/stable/profile-bulk?part=0":                                     "bulk_company_profiles.json",
	"/router/stable/rating-bulk":                                             "bulk_stock_ratings.json",
	"/router/stable/dcf-bulk":                                                "bulk_dcf_valuations.json",
	"/router/stable/scores-bulk":                                             "bulk_financial_scores.json",
	"/router/stable/price-target-summary-bulk":                               "bulk_price_target_summaries.json",
	"/router/stable/etf-holder-bulk?part=segment+A%2F7":                      "bulk_etf_holdings.json",
	"/router/stable/upgrades-downgrades-consensus-bulk":                      "bulk_upgrades_downgrades_consensus.json",
	"/router/stable/key-metrics-ttm-bulk":                                    "bulk_key_metrics_ttm.json",
	"/router/stable/ratios-ttm-bulk":                                         "bulk_financial_ratios_ttm.json",
	"/router/stable/peers-bulk":                                              "bulk_stock_peers.json",
	"/router/stable/earnings-surprises-bulk?year=4294967295":                 "bulk_earnings_surprises.json",
	"/router/stable/income-statement-bulk?year=2026&period=Q1":               "bulk_income_statements.json",
	"/router/stable/income-statement-growth-bulk?year=2026&period=Q2":        "bulk_income_statement_growth.json",
	"/router/stable/balance-sheet-statement-bulk?year=2026&period=Q3":        "bulk_balance_sheet_statements.json",
	"/router/stable/balance-sheet-statement-growth-bulk?year=2026&period=Q4": "bulk_balance_sheet_statement_growth.json",
	"/router/stable/cash-flow-statement-bulk?year=4294967295&period=FY":      "bulk_cash_flow_statements.json",
	"/router/stable/cash-flow-statement-growth-bulk?year=2025&period=FY":     "bulk_cash_flow_statement_growth.json",
	"/router/stable/eod-bulk?date=2024-10-22":                                "bulk_eod.json",
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
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestBulkMethodsUseExactPathsAndWireParameterOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, bulkRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	profiles, err := client.Bulk.CompanyProfiles(ctx, NewBulkPartQuery("0"))
	if err != nil || len(profiles) != 1 || profiles[0].Symbol != "AAPL" || profiles[0].MarketCap != 4_009_711_150_080 {
		t.Fatalf("CompanyProfiles = %+v, %v", profiles, err)
	}
	ratings, err := client.Bulk.StockRatings(ctx)
	if err != nil || len(ratings) != 1 || ratings[0].Rating != "B+" {
		t.Fatalf("StockRatings = %+v, %v", ratings, err)
	}
	dcf, err := client.Bulk.DcfValuations(ctx)
	if err != nil || len(dcf) != 1 || dcf[0].StockPrice != "6.54" {
		t.Fatalf("DcfValuations = %+v, %v", dcf, err)
	}
	scores, err := client.Bulk.FinancialScores(ctx)
	if err != nil || len(scores) != 1 || scores[0].PiotroskiScore != "5" {
		t.Fatalf("FinancialScores = %+v, %v", scores, err)
	}
	targets, err := client.Bulk.PriceTargetSummaries(ctx)
	if err != nil || len(targets) != 1 || targets[0].Symbol != "A" {
		t.Fatalf("PriceTargetSummaries = %+v, %v", targets, err)
	}
	holdings, err := client.Bulk.EtfHoldings(ctx, NewBulkPartQuery("segment A/7"))
	if err != nil || len(holdings) != 1 || holdings[0].LastUpdatedRaw != `2024-09-06"` {
		t.Fatalf("EtfHoldings = %+v, %v", holdings, err)
	}
	consensus, err := client.Bulk.UpgradesDowngradesConsensus(ctx)
	if err != nil || len(consensus) != 1 || consensus[0].Consensus != "Buy" {
		t.Fatalf("UpgradesDowngradesConsensus = %+v, %v", consensus, err)
	}
	metrics, err := client.Bulk.KeyMetricsTtm(ctx)
	if err != nil || len(metrics) != 1 || metrics[0].MarketCap != "249171756000" {
		t.Fatalf("KeyMetricsTtm = %+v, %v", metrics, err)
	}
	ratios, err := client.Bulk.FinancialRatiosTtm(ctx)
	if err != nil || len(ratios) != 1 || ratios[0].GrossProfitMarginTtm != "1.1622776732779352" {
		t.Fatalf("FinancialRatiosTtm = %+v, %v", ratios, err)
	}
	peers, err := client.Bulk.StockPeers(ctx)
	if err != nil || len(peers) != 1 || peers[0].Peers != "600036.SS" {
		t.Fatalf("StockPeers = %+v, %v", peers, err)
	}
	surprises, err := client.Bulk.EarningsSurprises(ctx, NewBulkYearQuery(4_294_967_295))
	if err != nil || len(surprises) != 1 || surprises[0].Symbol != "AMKYF" {
		t.Fatalf("EarningsSurprises = %+v, %v", surprises, err)
	}
	income, err := client.Bulk.IncomeStatements(ctx, NewBulkStatementQuery(2026, FiscalPeriodQ1))
	if err != nil || len(income) != 1 || income[0].Revenue != "33644000000" {
		t.Fatalf("IncomeStatements = %+v, %v", income, err)
	}
	incomeGrowth, err := client.Bulk.IncomeStatementGrowth(ctx, NewBulkStatementQuery(2026, FiscalPeriodQ2))
	if err != nil || len(incomeGrowth) != 1 || incomeGrowth[0].GrowthEbit != "1" {
		t.Fatalf("IncomeStatementGrowth = %+v, %v", incomeGrowth, err)
	}
	balance, err := client.Bulk.BalanceSheetStatements(ctx, NewBulkStatementQuery(2026, FiscalPeriodQ3))
	if err != nil || len(balance) != 1 || balance[0].Symbol != "MTLRP.ME" {
		t.Fatalf("BalanceSheetStatements = %+v, %v", balance, err)
	}
	balanceGrowth, err := client.Bulk.BalanceSheetStatementGrowth(ctx, NewBulkStatementQuery(2026, FiscalPeriodQ4))
	if err != nil || len(balanceGrowth) != 1 || balanceGrowth[0].GrowthShortTermInvestments != "0" {
		t.Fatalf("BalanceSheetStatementGrowth = %+v, %v", balanceGrowth, err)
	}
	cashFlow, err := client.Bulk.CashFlowStatements(ctx, NewBulkStatementQuery(4_294_967_295, FiscalPeriodFullYear))
	if err != nil || len(cashFlow) != 1 || cashFlow[0].NetIncome != "0" {
		t.Fatalf("CashFlowStatements = %+v, %v", cashFlow, err)
	}
	cashGrowth, err := client.Bulk.CashFlowStatementGrowth(ctx, NewBulkStatementQuery(2025, FiscalPeriodFullYear))
	if err != nil || len(cashGrowth) != 1 || cashGrowth[0].GrowthNetDebtIssuance != "1" {
		t.Fatalf("CashFlowStatementGrowth = %+v, %v", cashGrowth, err)
	}
	eod, err := client.Bulk.Eod(ctx, NewBulkEodQuery(mustParseDate(t, "2024-10-22")))
	if err != nil || len(eod) != 1 || eod[0].Volume != "920904" {
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
			_, err := client.Bulk.EtfHoldings(ctx, NewBulkPartQuery("  "))
			return err
		}, "part", ErrEmptyValue, ""},
		{"control character part", func() error {
			_, err := client.Bulk.EtfHoldings(ctx, NewBulkPartQuery("part\n1"))
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

// A bulk row missing its raw-keyed member surfaces as a Decode error on the
// endpoint, exactly like a missing tagged member does.
func TestBulkMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	body := strings.Replace(string(readFixture(t, "bulk_etf_holdings.json")), `"lastUpdated\"": "2024-09-06\"",`, "", 1)
	if body == string(readFixture(t, "bulk_etf_holdings.json")) {
		t.Fatal("fixture no longer carries the quoted lastUpdated member on one line")
	}
	server, _ := newServer(t, jsonHandler(body))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Bulk.EtfHoldings(context.Background(), NewBulkPartQuery("0"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "etf-holder-bulk")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"lastUpdated\""`) {
		t.Fatalf("cause = %v, want it to name the missing member lastUpdated\"", cause)
	}
}
