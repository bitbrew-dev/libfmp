package fmp

import (
	"encoding/csv"
	"encoding/json/v2"
	"slices"
	"strings"
	"testing"
)

// Every shared fixture a Rust test decodes into a bulk model (or, for
// profile-bulk, into the company CompanyProfile), through the ADR 0030 parity
// helper (crates/libfmp/tests/bulk_*_responses.rs). Bulk bodies are read
// relatively from the shared fixture directory, never copied.
func TestBulkFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertCSVFixtureParity[CompanyProfile](t, "bulk_company_profiles.csv", 36)
	assertCSVFixtureParity[BulkStockRating](t, "bulk_stock_ratings.csv", 9)
	assertCSVFixtureParity[BulkDCFValuation](t, "bulk_dcf_valuations.csv", 4)
	assertCSVFixtureParity[BulkFinancialScore](t, "bulk_financial_scores.csv", 11)
	assertCSVFixtureParity[BulkPriceTargetSummary](t, "bulk_price_target_summaries.csv", 10)
	assertCSVFixtureParity[BulkETFHolding](t, "bulk_etf_holdings.csv", 9)
	assertCSVFixtureParity[BulkUpgradesDowngradesConsensus](t, "bulk_upgrades_downgrades_consensus.csv", 7)
	assertCSVFixtureParity[BulkKeyMetricsTTM](t, "bulk_key_metrics_ttm.csv", 43)
	assertCSVFixtureParity[BulkFinancialRatiosTTM](t, "bulk_financial_ratios_ttm.csv", 60)
	assertCSVFixtureParity[BulkStockPeer](t, "bulk_stock_peers.csv", 2)
	assertCSVFixtureParity[BulkEarningsSurprise](t, "bulk_earnings_surprises.csv", 5)
	assertFixtureParity[BulkIncomeStatement](t, "bulk_income_statements.json")
	assertFixtureParity[BulkIncomeStatementGrowth](t, "bulk_income_statement_growth.json")
	assertFixtureParity[BulkBalanceSheetStatement](t, "bulk_balance_sheet_statements.json")
	assertFixtureParity[BulkBalanceSheetStatementGrowth](t, "bulk_balance_sheet_statement_growth.json")
	assertFixtureParity[BulkCashFlowStatement](t, "bulk_cash_flow_statements.json")
	assertFixtureParity[BulkCashFlowStatementGrowth](t, "bulk_cash_flow_statement_growth.json")
	assertFixtureParity[BulkEodBar](t, "bulk_eod.json")
}

// Exact values copied from crates/libfmp/tests/bulk_snapshot_responses.rs:
// every numeric cell stays the provider's text, and the provider spellings
// (the "Stock Price" column, the quoted publishers list, the " -- " ETF
// symbol) are preserved verbatim.
func TestBulkSnapshotsDecodeExactValues(t *testing.T) {
	t.Parallel()
	profiles := assertCSVFixtureParity[CompanyProfile](t, "bulk_company_profiles.csv", 36)
	if profiles[0].Symbol != "WMB" || profiles[0].CompanyName != "The Williams Companies, Inc." ||
		profiles[0].CIK == nil || *profiles[0].CIK != "0000107263" || profiles[0].IsETF {
		t.Fatalf("bulk_company_profiles = %+v", profiles[0])
	}
	ratings := assertCSVFixtureParity[BulkStockRating](t, "bulk_stock_ratings.csv", 9)
	if want := (BulkStockRating{Symbol: "000001.SZ", Date: mustParseDate(t, "2026-09-30"), Rating: "B-",
		DiscountedCashFlowScore: "1", ReturnOnEquityScore: "3", ReturnOnAssetsScore: "2", DebtToEquityScore: "1",
		PriceToEarningsScore: "4", PriceToBookScore: "4"}); ratings[0] != want {
		t.Fatalf("bulk_stock_ratings = %+v", ratings[0])
	}
	dcf := assertCSVFixtureParity[BulkDCFValuation](t, "bulk_dcf_valuations.csv", 4)
	if want := (BulkDCFValuation{Symbol: "000006.SZ", Date: mustParseDate(t, "2026-09-29"),
		DCF: "2.525226853334803", StockPrice: "7.62"}); dcf[0] != want {
		t.Fatalf("bulk_dcf_valuations = %+v", dcf[0])
	}
	if encoded, err := json.Marshal(dcf[0]); err != nil || !strings.Contains(string(encoded), `"Stock Price":"7.62"`) {
		t.Fatalf("re-encoded dcf = %s, %v", encoded, err)
	}
	targets := assertCSVFixtureParity[BulkPriceTargetSummary](t, "bulk_price_target_summaries.csv", 10)
	if targets[0].Symbol != "A" || targets[0].Publishers != `["StreetInsider","Benzinga","Pulse 2.0"]` ||
		targets[0].AllTimeAvgPriceTarget != "159.49" {
		t.Fatalf("bulk_price_target_summaries = %+v", targets[0])
	}
	holdings := assertCSVFixtureParity[BulkETFHolding](t, "bulk_etf_holdings.csv", 9)
	if want := (BulkETFHolding{Symbol: " -- ", Name: "Tidewater Inc", SharesNumber: "91962", Asset: "TDW",
		WeightPercentage: "2.63", CUSIP: "88642R109", ISIN: "US88642R1095", MarketValue: "6457163.796",
		LastUpdated: mustParseDate(t, "2026-09-27")}); holdings[0] != want {
		t.Fatalf("bulk_etf_holdings = %+v", holdings[0])
	}
}

// Exact values copied from crates/libfmp/tests/bulk_metrics_responses.rs,
// including the beyond-u64, high-precision, and exponent cells that must
// survive untouched (numeric strings are never parsed).
func TestBulkMetricsPreserveNumericTextIncludingBeyondU64(t *testing.T) {
	t.Parallel()
	metrics := assertCSVFixtureParity[BulkKeyMetricsTTM](t, "bulk_key_metrics_ttm.csv", 43)
	if metrics[0].Symbol != "000001.SZ" || metrics[0].MarketCap != "224526473551" ||
		metrics[0].EvToEbitdaTTM != "29.23198788110799" {
		t.Fatalf("bulk_key_metrics_ttm = %+v", metrics[0])
	}
	encoded, err := json.Marshal(metrics[0])
	if err != nil || !strings.Contains(string(encoded), `"evToEBITDATTM":`) ||
		!strings.Contains(string(encoded), `"netDebtToEBITDATTM":`) ||
		!strings.Contains(string(encoded), `"researchAndDevelopementToRevenueTTM":`) ||
		strings.Contains(string(encoded), "Ttm") {
		t.Fatalf("re-encoded key metrics = %s, %v", encoded, err)
	}
	ratios := assertCSVFixtureParity[BulkFinancialRatiosTTM](t, "bulk_financial_ratios_ttm.csv", 60)
	if ratios[0].GrossProfitMarginTTM != "0.5535250166330814" || ratios[0].NetIncomePerEbtTTM != "0.83539656299258" {
		t.Fatalf("bulk_financial_ratios_ttm = %+v", ratios[0])
	}

	const beyondU64 = "18446744073709551616"
	const highPrecision = "-12345678901234567890123456789.123456789012345678901234567890"
	records, err := csv.NewReader(strings.NewReader(string(readFixture(t, "bulk_key_metrics_ttm.csv")))).ReadAll()
	if err != nil {
		t.Fatal(err)
	}
	row := slices.Clone(records[1])
	for name, value := range map[string]string{"marketCap": beyondU64, "freeCashFlowToFirmTTM": highPrecision,
		"evToSalesTTM": "6.9148336e-9"} {
		row[slices.Index(records[0], name)] = value
	}
	var body strings.Builder
	writer := csv.NewWriter(&body)
	_ = writer.WriteAll([][]string{records[0], row})
	decoded, _, kind, err := decodeCSVRows[BulkKeyMetricsTTM]([]byte(body.String()))
	if kind != DecodeKindNone || decoded[0].MarketCap != beyondU64 ||
		decoded[0].FreeCashFlowToFirmTTM != highPrecision || decoded[0].EvToSalesTTM != "6.9148336e-9" {
		t.Fatalf("beyond-u64 decode = %+v, %v", decoded, err)
	}

	peers := assertCSVFixtureParity[BulkStockPeer](t, "bulk_stock_peers.csv", 2)
	if want := (BulkStockPeer{Symbol: "000001.SZ",
		Peers: "3698.HK,600000.SS,600015.SS,600016.SS,600036.SS,601166.SS,601658.SS"}); peers[0] != want {
		t.Fatalf("bulk_stock_peers = %+v", peers[0])
	}
	surprises := assertCSVFixtureParity[BulkEarningsSurprise](t, "bulk_earnings_surprises.csv", 5)
	if want := (BulkEarningsSurprise{Symbol: "AUTO.OL", Date: mustParseDate(t, "2024-12-31"), EPSActual: "0.1332",
		EPSEstimated: "0.1581", LastUpdated: mustParseDate(t, "2025-10-07")}); surprises[0] != want {
		t.Fatalf("bulk_earnings_surprises = %+v", surprises[0])
	}
}

// Exact values copied from crates/libfmp/tests/bulk_{income,balance,cash_eod}_responses.rs:
// identity fields keep their documented representations (ten-zero CIK, naive
// accepted date, Q1 period) and the provider typos keep their wire spelling.
func TestBulkStatementsDecodeExactValuesAndKeepProviderTypos(t *testing.T) {
	t.Parallel()
	income := assertFixtureParity[BulkIncomeStatement](t, "bulk_income_statements.json")
	if len(income) != 1 || income[0].Symbol != "000001.SZ" || income[0].ReportedCurrency != "CNY" ||
		income[0].CIK != "0000000000" || income[0].Date != mustParseDate(t, "2025-03-31") ||
		income[0].FilingDate != mustParseDate(t, "2025-03-31") ||
		income[0].AcceptedDate != mustParseDateTime(t, "2025-03-31 00:00:00") || income[0].FiscalYear != "2025" ||
		income[0].Period != "Q1" || income[0].Revenue != "33644000000" || income[0].CostOfRevenue != "0" ||
		income[0].TotalOtherIncomeExpensesNet != "-7392000000" || income[0].EPS != "0.62" ||
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
		balance[0].CIK != "0000000000" || balance[0].FilingDate != mustParseDate(t, "2025-05-31") ||
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
	if len(cashFlow) != 1 || cashFlow[0].Symbol != "000001.SZ" || cashFlow[0].CIK != "0000000000" ||
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
