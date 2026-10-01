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
	assertCSVFixtureParity[BulkIncomeStatement](t, "bulk_income_statements.csv", 39)
	assertCSVFixtureParity[BulkIncomeStatementGrowth](t, "bulk_income_statement_growth.csv", 34)
	assertCSVFixtureParity[BulkBalanceSheetStatement](t, "bulk_balance_sheet_statements.csv", 61)
	assertCSVFixtureParity[BulkBalanceSheetStatementGrowth](t, "bulk_balance_sheet_statement_growth.csv", 56)
	assertCSVFixtureParity[BulkCashFlowStatement](t, "bulk_cash_flow_statements.csv", 47)
	assertCSVFixtureParity[BulkCashFlowStatementGrowth](t, "bulk_cash_flow_statement_growth.csv", 42)
	assertCSVFixtureParity[BulkEodBar](t, "bulk_eod.csv", 8)
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
	if ratings[0].Rating != "B-" || cellText(ratings[0].PriceToBookScore) != "4" {
		t.Fatalf("bulk_stock_ratings = %+v", ratings[0])
	}
	dcf := assertCSVFixtureParity[BulkDCFValuation](t, "bulk_dcf_valuations.csv", 4)
	if encoded, err := json.Marshal(dcf[0]); err != nil || !strings.Contains(string(encoded), `"Stock Price":"7.62"`) {
		t.Fatalf("re-encoded dcf = %s, %v", encoded, err)
	}
	if dcf[2].Symbol != "000023.SZ" || dcf[2].DCF != nil || cellText(dcf[2].StockPrice) != "1.72" {
		t.Fatalf("empty dcf cell = %+v", dcf[2])
	}
	scores := assertCSVFixtureParity[BulkFinancialScore](t, "bulk_financial_scores.csv", 11)
	if scores[2].Symbol != "AAAU" || scores[2].ReportedCurrency != nil || scores[2].AltmanZScore != nil ||
		cellText(scores[2].PiotroskiScore) != "2" {
		t.Fatalf("empty score cells = %+v", scores[2])
	}
	targets := assertCSVFixtureParity[BulkPriceTargetSummary](t, "bulk_price_target_summaries.csv", 10)
	if targets[0].Publishers != `["StreetInsider","Benzinga","Pulse 2.0"]` ||
		cellText(targets[0].AllTimeAvgPriceTarget) != "159.49" {
		t.Fatalf("bulk_price_target_summaries = %+v", targets[0])
	}
	holdings := assertCSVFixtureParity[BulkETFHolding](t, "bulk_etf_holdings.csv", 9)
	if holdings[0].Symbol != " -- " || cellText(holdings[0].Asset) != "TDW" ||
		holdings[0].LastUpdated != mustParseDate(t, "2026-09-27") {
		t.Fatalf("bulk_etf_holdings = %+v", holdings[0])
	}
	if holdings[2].Name != "Other/Cash" || holdings[2].Asset != nil || holdings[2].ISIN != nil || holdings[2].CUSIP != "" {
		t.Fatalf("empty holding cells = %+v", holdings[2])
	}
}

// Exact values copied from crates/libfmp/tests/bulk_metrics_responses.rs,
// including the beyond-u64, high-precision, and exponent cells that must
// survive untouched (numeric strings are never parsed).
func TestBulkMetricsPreserveNumericTextIncludingBeyondU64(t *testing.T) {
	t.Parallel()
	metrics := assertCSVFixtureParity[BulkKeyMetricsTTM](t, "bulk_key_metrics_ttm.csv", 43)
	if metrics[0].Symbol != "000001.SZ" || cellText(metrics[0].MarketCap) != "224526473551" ||
		cellText(metrics[0].EvToEbitdaTTM) != "29.23198788110799" {
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
	if cellText(ratios[0].GrossProfitMarginTTM) != "0.5535250166330814" || cellText(ratios[0].NetIncomePerEbtTTM) != "0.83539656299258" {
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
	if kind != DecodeKindNone || cellText(decoded[0].MarketCap) != beyondU64 ||
		cellText(decoded[0].FreeCashFlowToFirmTTM) != highPrecision || cellText(decoded[0].EvToSalesTTM) != "6.9148336e-9" {
		t.Fatalf("beyond-u64 decode = %+v, %v", decoded, err)
	}

	peers := assertCSVFixtureParity[BulkStockPeer](t, "bulk_stock_peers.csv", 2)
	if want := (BulkStockPeer{Symbol: "000001.SZ",
		Peers: "3698.HK,600000.SS,600015.SS,600016.SS,600036.SS,601166.SS,601658.SS"}); peers[0] != want {
		t.Fatalf("bulk_stock_peers = %+v", peers[0])
	}
	surprises := assertCSVFixtureParity[BulkEarningsSurprise](t, "bulk_earnings_surprises.csv", 5)
	if surprises[0].Symbol != "AUTO.OL" || cellText(surprises[0].EPSActual) != "0.1332" ||
		surprises[0].LastUpdated != mustParseDate(t, "2025-10-07") {
		t.Fatalf("bulk_earnings_surprises = %+v", surprises[0])
	}
	if surprises[2].Symbol != "GUD.TO" || surprises[2].EPSEstimated != nil || cellText(surprises[2].EPSActual) != "0.00084" {
		t.Fatalf("empty earnings cell = %+v", surprises[2])
	}
	if metrics[2].Symbol != "ADAMO" || metrics[2].EnterpriseValueTTM != nil || peers[2].Peers != "" {
		t.Fatalf("empty metric cells = %+v, %+v", metrics[2], peers[2])
	}
}

// Exact values copied from crates/libfmp/tests/bulk_{income,balance,cash_eod}_responses.rs:
// identity fields keep their provider representations (ten-zero CIK, naive
// accepted date, period) and the provider typos keep their wire spelling.
func TestBulkStatementsDecodeExactValuesAndKeepProviderTypos(t *testing.T) {
	t.Parallel()
	income := assertCSVFixtureParity[BulkIncomeStatement](t, "bulk_income_statements.csv", 39)
	if income[0].Symbol != "000001.SZ" || income[0].ReportedCurrency != "CNY" || income[0].CIK != "0000000000" ||
		income[0].Date != mustParseDate(t, "2024-12-31") ||
		income[0].AcceptedDate != mustParseDateTime(t, "2024-12-31 00:00:00") || income[0].FiscalYear != "2024" ||
		income[0].Period != "FY" || cellText(income[0].Revenue) != "251641000000" {
		t.Fatalf("bulk_income_statements = %+v", income[0])
	}
	if income[2].Symbol != "OASMY" || income[2].EPS != nil || cellText(income[2].NetIncome) != "-39754000" {
		t.Fatalf("empty income cells = %+v", income[2])
	}
	incomeGrowth := assertCSVFixtureParity[BulkIncomeStatementGrowth](t, "bulk_income_statement_growth.csv", 34)
	if encoded, err := json.Marshal(incomeGrowth[0]); err != nil || !strings.Contains(string(encoded), `"growthEBITDA":`) ||
		!strings.Contains(string(encoded), `"growthEPSDiluted":`) || !strings.Contains(string(encoded), `"growthEBIT":`) {
		t.Fatalf("re-encoded income growth = %s, %v", encoded, err)
	}

	balance := assertCSVFixtureParity[BulkBalanceSheetStatement](t, "bulk_balance_sheet_statements.csv", 61)
	if balance[1].Symbol != "0002.KL" || balance[1].ReportedCurrency != "MYR" ||
		balance[1].AcceptedDate != mustParseDateTime(t, "2025-06-30 00:00:00") || cellText(balance[0].TotalAssets) != "5769270000000" {
		t.Fatalf("bulk_balance_sheet_statements = %+v", balance)
	}
	balanceGrowth := assertCSVFixtureParity[BulkBalanceSheetStatementGrowth](t, "bulk_balance_sheet_statement_growth.csv", 56)
	if encoded, err := json.Marshal(balanceGrowth[0]); err != nil ||
		!strings.Contains(string(encoded), `"growthOthertotalStockholdersEquity":`) ||
		!strings.Contains(string(encoded), `"growthTotalLiabilitiesAndStockholdersEquity":`) {
		t.Fatalf("re-encoded balance growth = %s, %v", encoded, err)
	}

	cashFlow := assertCSVFixtureParity[BulkCashFlowStatement](t, "bulk_cash_flow_statements.csv", 47)
	if cashFlow[0].CIK != "0000000000" || cashFlow[0].AcceptedDate != mustParseDateTime(t, "2024-03-30 20:00:00") ||
		cashFlow[0].Period != "Q1" || cashFlow[0].NetIncome != "14932000000" {
		t.Fatalf("bulk_cash_flow_statements = %+v", cashFlow[0])
	}
	cashGrowth := assertCSVFixtureParity[BulkCashFlowStatementGrowth](t, "bulk_cash_flow_statement_growth.csv", 42)
	if encoded, err := json.Marshal(cashGrowth[0]); err != nil ||
		!strings.Contains(string(encoded), `"growthNetCashProvidedByOperatingActivites":`) ||
		!strings.Contains(string(encoded), `"growthOtherInvestingActivites":`) ||
		!strings.Contains(string(encoded), `"growthNetCashUsedForInvestingActivites":`) ||
		!strings.Contains(string(encoded), `"growthOtherFinancingActivites":`) {
		t.Fatalf("re-encoded cash-flow growth = %s, %v", encoded, err)
	}

	eod := assertCSVFixtureParity[BulkEodBar](t, "bulk_eod.csv", 8)
	if want := (BulkEodBar{Symbol: "HKDCNH", Date: mustParseDate(t, "2025-06-02"), Open: "0.91858", Low: "0.91692",
		High: "0.9186", Close: "0.91741", AdjClose: "0.91741", Volume: "0"}); eod[1] != want {
		t.Fatalf("bulk_eod = %+v", eod[1])
	}
}
