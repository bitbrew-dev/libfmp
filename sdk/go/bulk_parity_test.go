package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"strconv"
	"strings"
	"testing"
)

// Every shared fixture a Rust test decodes into a bulk model (or, for
// profile-bulk, into the company CompanyProfile), through the ADR 0030 parity
// helper (crates/libfmp/tests/bulk_*_responses.rs). Bulk bodies are read
// relatively from the shared fixture directory, never copied.
func TestBulkFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[CompanyProfile](t, "bulk_company_profiles.json")
	assertFixtureParity[BulkStockRating](t, "bulk_stock_ratings.json")
	assertFixtureParity[BulkDcfValuation](t, "bulk_dcf_valuations.json")
	assertFixtureParity[BulkFinancialScore](t, "bulk_financial_scores.json")
	assertFixtureParity[BulkPriceTargetSummary](t, "bulk_price_target_summaries.json")
	assertFixtureParity[BulkEtfHolding](t, "bulk_etf_holdings.json")
	assertFixtureParity[BulkUpgradesDowngradesConsensus](t, "bulk_upgrades_downgrades_consensus.json")
	assertFixtureParity[BulkKeyMetricsTtm](t, "bulk_key_metrics_ttm.json")
	assertFixtureParity[BulkFinancialRatiosTtm](t, "bulk_financial_ratios_ttm.json")
	assertFixtureParity[BulkStockPeer](t, "bulk_stock_peers.json")
	assertFixtureParity[BulkEarningsSurprise](t, "bulk_earnings_surprises.json")
	assertFixtureParity[BulkIncomeStatement](t, "bulk_income_statements.json")
	assertFixtureParity[BulkIncomeStatementGrowth](t, "bulk_income_statement_growth.json")
	assertFixtureParity[BulkBalanceSheetStatement](t, "bulk_balance_sheet_statements.json")
	assertFixtureParity[BulkBalanceSheetStatementGrowth](t, "bulk_balance_sheet_statement_growth.json")
	assertFixtureParity[BulkCashFlowStatement](t, "bulk_cash_flow_statements.json")
	assertFixtureParity[BulkCashFlowStatementGrowth](t, "bulk_cash_flow_statement_growth.json")
	assertFixtureParity[BulkEodBar](t, "bulk_eod.json")
}

// Exact values copied from crates/libfmp/tests/bulk_snapshot_responses.rs:
// every numeric cell stays the provider's string, and the documented source
// defects (the "Stock Price" key, the malformed publishers text, the empty
// CUSIP and consensus symbol) are preserved verbatim.
func TestDocumentedBulkSnapshotsDecodeExactValues(t *testing.T) {
	t.Parallel()
	profiles := assertFixtureParity[CompanyProfile](t, "bulk_company_profiles.json")
	if len(profiles) != 1 || profiles[0].Symbol != "AAPL" || profiles[0].Cik != "0000320193" {
		t.Fatalf("bulk_company_profiles = %+v", profiles)
	}
	ratings := assertFixtureParity[BulkStockRating](t, "bulk_stock_ratings.json")
	if want := (BulkStockRating{Symbol: "000001.SZ", Date: mustParseDate(t, "2025-07-09"), Rating: "B+",
		DiscountedCashFlowScore: "5", ReturnOnEquityScore: "3", ReturnOnAssetsScore: "2", DebtToEquityScore: "1",
		PriceToEarningsScore: "4", PriceToBookScore: "4"}); len(ratings) != 1 || ratings[0] != want {
		t.Fatalf("bulk_stock_ratings = %+v", ratings)
	}
	dcf := assertFixtureParity[BulkDcfValuation](t, "bulk_dcf_valuations.json")
	if want := (BulkDcfValuation{Symbol: "000002.SZ", Date: mustParseDate(t, "2025-07-09"),
		Dcf: "179.6654688379575", StockPrice: "6.54"}); len(dcf) != 1 || dcf[0] != want {
		t.Fatalf("bulk_dcf_valuations = %+v", dcf)
	}
	if encoded, err := json.Marshal(dcf[0]); err != nil || !strings.Contains(string(encoded), `"Stock Price":"6.54"`) {
		t.Fatalf("re-encoded dcf = %s, %v", encoded, err)
	}
	scores := assertFixtureParity[BulkFinancialScore](t, "bulk_financial_scores.json")
	if len(scores) != 1 || scores[0].Symbol != "000001.SZ" || scores[0].ReportedCurrency != "CNY" ||
		scores[0].AltmanZScore != "0.29153682196643543" || scores[0].PiotroskiScore != "5" ||
		scores[0].TotalAssets != "5777858000000" {
		t.Fatalf("bulk_financial_scores = %+v", scores)
	}
	targets := assertFixtureParity[BulkPriceTargetSummary](t, "bulk_price_target_summaries.json")
	if len(targets) != 1 || targets[0].Symbol != "A" || targets[0].Publishers != `[""TheFly"` ||
		targets[0].LastMonthCount != "0" || targets[0].AllTimeAvgPriceTarget != "146.61" {
		t.Fatalf("bulk_price_target_summaries = %+v", targets)
	}
	consensus := assertFixtureParity[BulkUpgradesDowngradesConsensus](t, "bulk_upgrades_downgrades_consensus.json")
	if want := (BulkUpgradesDowngradesConsensus{Symbol: "", StrongBuy: "0", Buy: "1", Hold: "1", Sell: "0",
		StrongSell: "0", Consensus: "Buy"}); len(consensus) != 1 || consensus[0] != want {
		t.Fatalf("bulk_upgrades_downgrades_consensus = %+v", consensus)
	}
}

// The ETF holding row carries the documented wire key `lastUpdated"`, which
// encoding/json/v2 cannot spell in a struct tag: the member travels through
// the embedded fallback and MarshalJSONTo (see rawMember). Its value keeps
// the trailing quote, re-encodes under the exact key, and is required.
func TestBulkEtfHoldingKeepsTheQuotedLastUpdatedKeyVerbatim(t *testing.T) {
	t.Parallel()
	holdings := assertFixtureParity[BulkEtfHolding](t, "bulk_etf_holdings.json")
	if want := (BulkEtfHolding{Symbol: "EXCH.AS", Name: "SAMSUNG ELECTRO MECHANICS LTD", SharesNumber: "15514",
		Asset: "009150.KS", WeightPercentage: "0.09611", Cusip: "", Isin: "KR7009150004", MarketValue: "1553142.49",
		LastUpdatedRaw: `2024-09-06"`}); len(holdings) != 1 || holdings[0] != want {
		t.Fatalf("bulk_etf_holdings = %+v", holdings)
	}
	encoded, err := json.Marshal(holdings)
	if err != nil || !strings.Contains(string(encoded), `"lastUpdated\"":"2024-09-06\""`) ||
		strings.Contains(string(encoded), `"lastUpdated":`) || strings.Contains(string(encoded), "RawMembers") {
		t.Fatalf("re-encoded holdings = %s, %v", encoded, err)
	}
	var decoded []BulkEtfHolding
	if err := json.Unmarshal(encoded, &decoded); err != nil || len(decoded) != 1 || decoded[0] != holdings[0] {
		t.Fatalf("round trip = %+v, %v", decoded, err)
	}

	var wire []map[string]jsontext.Value
	if err := json.Unmarshal(readFixture(t, "bulk_etf_holdings.json"), &wire); err != nil {
		t.Fatal(err)
	}
	for _, tc := range []struct{ name, member, value string }{
		{"missing raw key", `lastUpdated"`, ""},
		{"null raw key", `lastUpdated"`, "null"},
		{"missing tagged member", "symbol", ""},
		{"null tagged member", "symbol", "null"},
	} {
		row := map[string]jsontext.Value{}
		for member, value := range wire[0] {
			row[member] = value
		}
		delete(row, tc.member)
		if tc.value != "" {
			row[tc.member] = jsontext.Value(tc.value)
		}
		body, err := json.Marshal([]map[string]jsontext.Value{row})
		if err != nil {
			t.Fatal(err)
		}
		var rows []BulkEtfHolding
		err = json.Unmarshal(body, &rows)
		var typed *Error
		if !errors.As(err, &typed) || typed.Category != CategoryDecode ||
			!strings.Contains(typed.Message, "required member "+strconv.Quote(tc.member)+" of BulkEtfHolding") {
			t.Fatalf("%s: error = %v, want a Decode *Error naming %q", tc.name, err, tc.member)
		}
	}
}

// Exact values copied from crates/libfmp/tests/bulk_metrics_responses.rs,
// including the beyond-u64 and high-precision strings that must survive
// untouched (ADR 0028: numeric strings are never parsed).
func TestBulkMetricsPreserveNumericTextIncludingBeyondU64(t *testing.T) {
	t.Parallel()
	metrics := assertFixtureParity[BulkKeyMetricsTtm](t, "bulk_key_metrics_ttm.json")
	if len(metrics) != 1 || metrics[0].Symbol != "000001.SZ" || metrics[0].MarketCap != "249171756000" ||
		metrics[0].EnterpriseValueTtm != "-496959244000" || metrics[0].CurrentRatioTtm != "0" ||
		metrics[0].FreeCashFlowToFirmTtm != "-35237570137.11014" {
		t.Fatalf("bulk_key_metrics_ttm = %+v", metrics)
	}
	encoded, err := json.Marshal(metrics[0])
	if err != nil || !strings.Contains(string(encoded), `"evToEBITDATTM":`) ||
		!strings.Contains(string(encoded), `"netDebtToEBITDATTM":`) ||
		!strings.Contains(string(encoded), `"researchAndDevelopementToRevenueTTM":`) ||
		strings.Contains(string(encoded), "Ttm") {
		t.Fatalf("re-encoded key metrics = %s, %v", encoded, err)
	}
	ratios := assertFixtureParity[BulkFinancialRatiosTtm](t, "bulk_financial_ratios_ttm.json")
	if len(ratios) != 1 || ratios[0].EnterpriseValueTtm != "-496959244000" || ratios[0].ReceivablesTurnoverTtm != "0" ||
		ratios[0].GrossProfitMarginTtm != "1.1622776732779352" {
		t.Fatalf("bulk_financial_ratios_ttm = %+v", ratios)
	}
	if encoded, err := json.Marshal(ratios[0]); err != nil || !strings.Contains(string(encoded), `"netIncomePerEBTTTM":`) {
		t.Fatalf("re-encoded ratios = %s, %v", encoded, err)
	}

	const beyondU64 = "18446744073709551616"
	const highPrecision = "-12345678901234567890123456789.123456789012345678901234567890"
	var wire []map[string]jsontext.Value
	if err := json.Unmarshal(readFixture(t, "bulk_key_metrics_ttm.json"), &wire); err != nil {
		t.Fatal(err)
	}
	wire[0]["marketCap"] = jsontext.Value(`"` + beyondU64 + `"`)
	wire[0]["freeCashFlowToFirmTTM"] = jsontext.Value(`"` + highPrecision + `"`)
	huge, err := json.Marshal(wire)
	if err != nil {
		t.Fatal(err)
	}
	var decoded []BulkKeyMetricsTtm
	if err := json.Unmarshal(huge, &decoded); err != nil || len(decoded) != 1 ||
		decoded[0].MarketCap != beyondU64 || decoded[0].FreeCashFlowToFirmTtm != highPrecision {
		t.Fatalf("beyond-u64 decode = %+v, %v", decoded, err)
	}
	if encoded, err := json.Marshal(decoded[0]); err != nil ||
		!strings.Contains(string(encoded), `"marketCap":"`+beyondU64+`"`) ||
		!strings.Contains(string(encoded), `"freeCashFlowToFirmTTM":"`+highPrecision+`"`) {
		t.Fatalf("beyond-u64 re-encode = %s, %v", encoded, err)
	}
	wire[0]["marketCap"] = jsontext.Value(`249171756000`)
	numeric, err := json.Marshal(wire)
	if err != nil {
		t.Fatal(err)
	}
	if err := json.Unmarshal(numeric, &decoded); err == nil {
		t.Fatalf("a JSON number decoded into the numeric-string member marketCap: %+v", decoded)
	}

	peers := assertFixtureParity[BulkStockPeer](t, "bulk_stock_peers.json")
	if want := (BulkStockPeer{Symbol: "000001.SZ", Peers: "600036.SS"}); len(peers) != 1 || peers[0] != want {
		t.Fatalf("bulk_stock_peers = %+v", peers)
	}
	surprises := assertFixtureParity[BulkEarningsSurprise](t, "bulk_earnings_surprises.json")
	if want := (BulkEarningsSurprise{Symbol: "AMKYF", Date: mustParseDate(t, "2025-07-09"), EpsActual: "0.3631",
		EpsEstimated: "0.3615", LastUpdated: mustParseDate(t, "2025-07-09")}); len(surprises) != 1 || surprises[0] != want {
		t.Fatalf("bulk_earnings_surprises = %+v", surprises)
	}
}

// Exact values copied from crates/libfmp/tests/bulk_{income,balance,cash_eod}_responses.rs:
// identity fields keep their documented representations (ten-zero CIK, naive
// accepted date, Q1 period) and the provider typos keep their wire spelling.
func TestBulkStatementsDecodeExactValuesAndKeepProviderTypos(t *testing.T) {
	t.Parallel()
	income := assertFixtureParity[BulkIncomeStatement](t, "bulk_income_statements.json")
	if len(income) != 1 || income[0].Symbol != "000001.SZ" || income[0].ReportedCurrency != "CNY" ||
		income[0].Cik != "0000000000" || income[0].Date != mustParseDate(t, "2025-03-31") ||
		income[0].FilingDate != mustParseDate(t, "2025-03-31") ||
		income[0].AcceptedDate != mustParseDateTime(t, "2025-03-31 00:00:00") || income[0].FiscalYear != "2025" ||
		income[0].Period != "Q1" || income[0].Revenue != "33644000000" || income[0].CostOfRevenue != "0" ||
		income[0].TotalOtherIncomeExpensesNet != "-7392000000" || income[0].Eps != "0.62" ||
		income[0].WeightedAverageShsOut != "22735483871" {
		t.Fatalf("bulk_income_statements = %+v", income)
	}
	incomeGrowth := assertFixtureParity[BulkIncomeStatementGrowth](t, "bulk_income_statement_growth.json")
	if len(incomeGrowth) != 1 || incomeGrowth[0].GrowthEbit != "1" || incomeGrowth[0].GrowthCostOfRevenue != "0" ||
		incomeGrowth[0].GrowthOtherExpenses != "-0.9860376183912135" ||
		incomeGrowth[0].GrowthOperatingIncome != "-0.018874787810201278" {
		t.Fatalf("bulk_income_statement_growth = %+v", incomeGrowth)
	}
	if encoded, err := json.Marshal(incomeGrowth[0]); err != nil || !strings.Contains(string(encoded), `"growthEBITDA":`) ||
		!strings.Contains(string(encoded), `"growthEPSDiluted":`) || !strings.Contains(string(encoded), `"growthEBIT":`) {
		t.Fatalf("re-encoded income growth = %s, %v", encoded, err)
	}

	balance := assertFixtureParity[BulkBalanceSheetStatement](t, "bulk_balance_sheet_statements.json")
	if len(balance) != 1 || balance[0].Symbol != "MTLRP.ME" || balance[0].ReportedCurrency != "RUB" ||
		balance[0].Cik != "0000000000" || balance[0].FilingDate != mustParseDate(t, "2025-05-31") ||
		balance[0].AcceptedDate != mustParseDateTime(t, "2025-03-31 07:00:00") || balance[0].Period != "Q1" ||
		balance[0].TotalAssets != "247871857000" || balance[0].RetainedEarnings != "-5066509000" ||
		balance[0].OtherAssets != "0" || balance[0].NetDebt != "183764862000" {
		t.Fatalf("bulk_balance_sheet_statements = %+v", balance)
	}
	balanceGrowth := assertFixtureParity[BulkBalanceSheetStatementGrowth](t, "bulk_balance_sheet_statement_growth.json")
	if len(balanceGrowth) != 1 || balanceGrowth[0].GrowthCashAndCashEquivalents != "0.09574482145872953" ||
		balanceGrowth[0].GrowthShortTermInvestments != "0" || balanceGrowth[0].GrowthTotalPayables != "-0.12022416350749959" {
		t.Fatalf("bulk_balance_sheet_statement_growth = %+v", balanceGrowth)
	}
	if encoded, err := json.Marshal(balanceGrowth[0]); err != nil ||
		!strings.Contains(string(encoded), `"growthOthertotalStockholdersEquity":`) ||
		!strings.Contains(string(encoded), `"growthTotalLiabilitiesAndStockholdersEquity":`) {
		t.Fatalf("re-encoded balance growth = %s, %v", encoded, err)
	}

	cashFlow := assertFixtureParity[BulkCashFlowStatement](t, "bulk_cash_flow_statements.json")
	if len(cashFlow) != 1 || cashFlow[0].Symbol != "000001.SZ" || cashFlow[0].Cik != "0000000000" ||
		cashFlow[0].AcceptedDate != mustParseDateTime(t, "2025-03-31 00:00:00") || cashFlow[0].Period != "Q1" ||
		cashFlow[0].OtherNonCashItems != "162946000000" || cashFlow[0].PurchasesOfInvestments != "-227916000000" ||
		cashFlow[0].NetIncome != "0" {
		t.Fatalf("bulk_cash_flow_statements = %+v", cashFlow)
	}
	cashGrowth := assertFixtureParity[BulkCashFlowStatementGrowth](t, "bulk_cash_flow_statement_growth.json")
	if len(cashGrowth) != 1 || cashGrowth[0].GrowthNetCashUsedProvidedByFinancingActivities != "-3.2122934677858628" ||
		cashGrowth[0].GrowthNetDebtIssuance != "1" {
		t.Fatalf("bulk_cash_flow_statement_growth = %+v", cashGrowth)
	}
	if encoded, err := json.Marshal(cashGrowth[0]); err != nil ||
		!strings.Contains(string(encoded), `"growthNetCashProvidedByOperatingActivites":`) ||
		!strings.Contains(string(encoded), `"growthOtherInvestingActivites":`) ||
		!strings.Contains(string(encoded), `"growthNetCashUsedForInvestingActivites":`) ||
		!strings.Contains(string(encoded), `"growthOtherFinancingActivites":`) {
		t.Fatalf("re-encoded cash-flow growth = %s, %v", encoded, err)
	}

	eod := assertFixtureParity[BulkEodBar](t, "bulk_eod.json")
	if want := (BulkEodBar{Symbol: "EGS745W1C011.CA", Date: mustParseDate(t, "2024-10-22"), Open: "2.67", Low: "2.7",
		High: "2.9", Close: "2.93", AdjClose: "2.93", Volume: "920904"}); len(eod) != 1 || eod[0] != want {
		t.Fatalf("bulk_eod = %+v", eod)
	}
}
