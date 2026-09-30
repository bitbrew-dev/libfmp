package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"reflect"
	"strings"
	"testing"
)

// Exact values copied from crates/libfmp/tests/statements_metrics_responses.rs,
// statements_ratios_responses.rs, statements_growth_responses.rs, and
// statements_cash_combined_growth_responses.rs.
func TestDocumentedMetricsRatiosAndGrowthFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	metrics := assertFixtureParity[KeyMetrics](t, "key_metrics.json")
	if len(metrics) != 1 || metrics[0].Symbol != "AAPL" || metrics[0].Date != mustParseDate(t, "2025-09-27") ||
		metrics[0].FiscalYear != "2025" || metrics[0].Period != "FY" || metrics[0].MarketCap != 3_818_743_810_000 ||
		metrics[0].EnterpriseValue != 3_895_186_810_000 || metrics[0].CurrentRatio != 0.8932929222186667 ||
		metrics[0].InterestBurden != 1.0 || metrics[0].IntangiblesToTotalAssets != 0.0 ||
		len(memberSet(t, metrics[0])) != 47 {
		t.Fatalf("key_metrics = %+v", metrics)
	}
	metricsTTM := assertFixtureParity[KeyMetricsTTM](t, "key_metrics_ttm.json")
	if len(metricsTTM) != 1 || metricsTTM[0].InterestBurdenTTM != 1.0 || metricsTTM[0].CurrentRatioTTM != 1.07035746912159 ||
		metricsTTM[0].EnterpriseValueTTM != 4_922_455_686_740 || len(memberSet(t, metricsTTM[0])) != 43 {
		t.Fatalf("key_metrics_ttm = %+v", metricsTTM)
	}
	ratios := assertFixtureParity[FinancialRatios](t, "financial_ratios.json")
	if len(ratios) != 1 || ratios[0].Symbol != "AAPL" || ratios[0].FiscalYear != "2025" ||
		ratios[0].WorkingCapitalTurnoverRatio != -20.261496141580857 || ratios[0].InterestCoverageRatio != 0.0 ||
		ratios[0].DividendYieldPercentage != 0.4038238951672435 || len(memberSet(t, ratios[0])) != 66 {
		t.Fatalf("financial_ratios = %+v", ratios)
	}
	ratiosTTM := assertFixtureParity[FinancialRatiosTTM](t, "financial_ratios_ttm.json")
	if len(ratiosTTM) != 1 || ratiosTTM[0].EnterpriseValueTTM != 4_922_455_686_740 ||
		ratiosTTM[0].InterestCoverageRatioTTM != 0.0 || ratiosTTM[0].NetIncomePerEbtTTM != 0.8300602695198754 ||
		len(memberSet(t, ratiosTTM[0])) != 62 {
		t.Fatalf("financial_ratios_ttm = %+v", ratiosTTM)
	}

	income := assertFixtureParity[IncomeStatementGrowth](t, "income_statement_growth.json")
	if len(income) != 1 || income[0].GrowthSellingAndMarketingExpenses != -1.0 ||
		income[0].GrowthGeneralAndAdministrativeExpenses != 2.700858138911236 ||
		income[0].GrowthTotalOtherIncomeExpensesNet != -2.193308550185874 || len(memberSet(t, income[0])) != 34 {
		t.Fatalf("income_statement_growth = %+v", income)
	}
	balance := assertFixtureParity[BalanceSheetStatementGrowth](t, "balance_sheet_statement_growth.json")
	if len(balance) != 1 || balance[0].GrowthTaxPayables != -1.0 || balance[0].GrowthOtherPayables != -0.5106950866508778 ||
		len(memberSet(t, balance[0])) != 56 {
		t.Fatalf("balance_sheet_statement_growth = %+v", balance)
	}
	cash := assertFixtureParity[CashFlowStatementGrowth](t, "cash_flow_statement_growth.json")
	if len(cash) != 1 || cash[0].GrowthChangeInWorkingCapital != -7.847439057792386 ||
		cash[0].GrowthInventory != 2.338432122370937 || len(memberSet(t, cash[0])) != 42 {
		t.Fatalf("cash_flow_statement_growth = %+v", cash)
	}
	combined := assertFixtureParity[FinancialStatementGrowth](t, "financial_statement_growth.json")
	if len(combined) != 1 || combined[0].EbitGrowth != 0.0748592946511722 || combined[0].EPSGrowth != 0.22585924713584285 ||
		combined[0].BookValuePerShareGrowth != 0.3289327621427069 ||
		combined[0].TenYRevenueGrowthPerShare != 1.7413426413189617 || len(memberSet(t, combined[0])) != 44 {
		t.Fatalf("financial_statement_growth = %+v", combined)
	}
}

// statementsObjectMembers decodes a JSON object held in a jsontext.Value.
func statementsObjectMembers(t *testing.T, raw jsontext.Value) map[string]jsontext.Value {
	t.Helper()
	var members map[string]jsontext.Value
	if err := json.Unmarshal(raw, &members); err != nil {
		t.Fatalf("not a JSON object: %v", err)
	}
	return members
}

// Exact values copied from statements_as_reported_responses.rs,
// statements_as_reported_cash_full_responses.rs, and
// statements_segmentation_responses.rs: the envelope is strict and the data
// object keeps every provider-native key verbatim.
func TestDocumentedAsReportedAndSegmentationFixturesKeepDynamicData(t *testing.T) {
	t.Parallel()
	for fixture, keys := range map[string]int{
		"income_statement_as_reported.json":         24,
		"balance_sheet_statement_as_reported.json":  31,
		"cash_flow_statement_as_reported.json":      28,
		"financial_statement_full_as_reported.json": 300,
	} {
		rows := assertFixtureParity[AsReportedFinancialStatement](t, fixture)
		if len(rows) != 1 || rows[0].Symbol != "AAPL" || rows[0].FiscalYear != 2025 || rows[0].Period != "FY" ||
			rows[0].ReportedCurrency == nil || *rows[0].ReportedCurrency != "USD" || rows[0].Date != mustParseDate(t, "2025-09-26") {
			t.Fatalf("%s = %+v", fixture, rows)
		}
		if data := statementsObjectMembers(t, rows[0].Data); len(data) != keys {
			t.Fatalf("%s: data has %d keys, want %d", fixture, len(data), keys)
		}
	}
	income := assertFixtureParity[AsReportedFinancialStatement](t, "income_statement_as_reported.json")
	if eps := statementsObjectMembers(t, income[0].Data)["earningspersharebasic"]; string(eps) != "7.49" {
		t.Fatalf("earningspersharebasic = %s", eps)
	}

	product := assertFixtureParity[RevenueSegmentation](t, "revenue_product_segmentation.json")
	if len(product) != 1 || product[0].Symbol != "AAPL" || product[0].FiscalYear != 2025 || product[0].Period != "FY" ||
		product[0].ReportedCurrency != "USD" || product[0].Date != mustParseDate(t, "2025-09-27") {
		t.Fatalf("revenue_product_segmentation = %+v", product)
	}
	segments := statementsObjectMembers(t, product[0].Data)
	if len(segments) != 5 || string(segments["Mac"]) != "33708000000" ||
		string(segments["Wearables, Home and Accessories"]) != "35686000000" || string(segments["iPhone"]) != "209586000000" {
		t.Fatalf("product segments = %v", segments)
	}
	geographic := assertFixtureParity[RevenueSegmentation](t, "revenue_geographic_segmentation.json")
	regions := statementsObjectMembers(t, geographic[0].Data)
	if len(regions) != 5 || string(regions["Greater China Segment"]) != "64377000000" ||
		string(regions["Rest of Asia Pacific Segment"]) != "33696000000" {
		t.Fatalf("geographic segments = %v", regions)
	}
}

// Exact values copied from statements_report_responses.rs. The links are
// plain strings (ADR 0030 type table): the SDK never formats a decoded row.
func TestDocumentedFinancialReportFixturesKeepHeadersAndDynamicSections(t *testing.T) {
	t.Parallel()
	dates := assertFixtureParity[FinancialReportDate](t, "financial_reports_dates.json")
	want := FinancialReportDate{
		Symbol: "AAPL", FiscalYear: 2026, Period: "Q2",
		LinkJSON: "https://financialmodelingprep.com/stable/financial-reports-json?symbol=AAPL&year=2026&period=Q2&apikey=[REDACTED]",
		LinkXlsx: "https://financialmodelingprep.com/stable/financial-reports-xlsx?symbol=AAPL&year=2026&period=Q2&apikey=[REDACTED]",
	}
	if len(dates) != 1 || dates[0] != want {
		t.Fatalf("financial_reports_dates = %+v, want %+v", dates, want)
	}

	// Trimmed from a live AAPL 2023 Q1 response captured on 2026-09-30: one
	// bare object, not an array.
	report := assertObjectFixtureParity[FinancialReportJSON](t, "financial_reports_json.json")
	if report.Symbol != "AAPL" || report.Period != "Q1" || report.Year != "2023" {
		t.Fatalf("financial_reports_json headers = %+v", report)
	}
	sections := statementsObjectMembers(t, report.Sections)
	if len(sections) != 3 || len(memberSet(t, report)) != 6 {
		t.Fatalf("sections = %d, re-encoded members = %d", len(sections), len(memberSet(t, report)))
	}
	for _, name := range []string{"CONDENSED CONSOLIDATED BALANC_2", "Shareholders' Equity - Addition", "Revenue - Additional Informatio"} {
		if _, ok := sections[name]; !ok {
			t.Fatalf("section %q is missing", name)
		}
	}
	for _, reserved := range []string{"symbol", "period", "year"} {
		if _, ok := sections[reserved]; ok {
			t.Fatalf("header %q leaked into the sections", reserved)
		}
	}
	// NBSP, fractional values, and scientific notation survive as raw text;
	// null survives in the inline empty-report probe below.
	var equity []map[string][]jsontext.Value
	var nbsp string
	if err := json.Unmarshal(sections["Shareholders' Equity - Addition"], &equity); err != nil ||
		json.Unmarshal(equity[2]["Share Repurchase Program [Line Items]"][0], &nbsp) != nil || nbsp != "\u00a0" {
		t.Fatalf("shareholders' equity section = %v, %v", equity, err)
	}
	var revenue []map[string][]jsontext.Value
	if err := json.Unmarshal(sections["Revenue - Additional Informatio"], &revenue); err != nil || len(revenue[2]["Total deferred revenue"]) != 2 ||
		string(revenue[2]["Total deferred revenue"][0]) != "12.6" {
		t.Fatalf("revenue section = %v, %v", revenue, err)
	}
	var balance []map[string][]jsontext.Value
	if err := json.Unmarshal(sections["CONDENSED CONSOLIDATED BALANC_2"], &balance); err != nil || string(balance[2]["Common stock, par value (in dollars per share)"][0]) != "1e-05" {
		t.Fatalf("balance sheet section = %v, %v", balance, err)
	}

	// A section named like a header cannot be re-encoded, as the Rust
	// serializer rejects reserved keys; a missing or numeric header is a
	// decode error; a report with no sections holds an empty object.
	for _, reserved := range []string{"symbol", "period", "year"} {
		clash := report
		clash.Sections = jsontext.Value(`{"` + reserved + `":"attacker-controlled replacement"}`)
		if _, err := json.Marshal(clash); err == nil || !strings.Contains(err.Error(), reserved) {
			t.Fatalf("reserved key %q: marshal error = %v", reserved, err)
		}
	}
	var decoded FinancialReportJSON
	if err := json.Unmarshal([]byte(`{"symbol":"AAPL","period":"FY","Cover Page":[]}`), &decoded); err == nil ||
		!strings.Contains(err.Error(), `"year"`) {
		t.Fatalf("missing year: error = %v", err)
	}
	if err := json.Unmarshal([]byte(`{"symbol":"AAPL","period":"Q1","year":2023}`), &decoded); err == nil {
		t.Fatal("numeric year decoded into a string")
	}
	if err := json.Unmarshal([]byte(`[{"symbol":"AAPL","period":"Q1","year":"2023"}]`), &decoded); err == nil {
		t.Fatal("a bare array decoded into one report")
	}
	if err := json.Unmarshal([]byte(`{"symbol":"TEST","period":"Q1","year":"0007"}`), &decoded); err != nil ||
		string(decoded.Sections) != "{}" || len(memberSet(t, decoded)) != 3 {
		t.Fatalf("empty report = %+v, %v", decoded, err)
	}
	if err := json.Unmarshal([]byte(`{"symbol":"TEST","period":"Q1","year":"0007","S":[null]}`), &decoded); err != nil ||
		string(decoded.Sections) != `{"S":[null]}` {
		t.Fatalf("null in a section = %+v, %v", decoded, err)
	}
}

// Exact values copied from statements_summary_responses.rs.
func TestDocumentedSummaryFixturesDecodeEveryFieldExactly(t *testing.T) {
	t.Parallel()
	latest := assertFixtureParity[LatestFinancialStatement](t, "latest_financial_statements.json")
	wantLatest := LatestFinancialStatement{
		Symbol: "UFPI", CalendarYear: 2026, Period: "Q2",
		Date: mustParseDate(t, "2026-06-27"), DateAdded: mustParseDateTime(t, "2026-07-30 13:17:26"),
	}
	if len(latest) != 1 || latest[0] != wantLatest {
		t.Fatalf("latest_financial_statements = %+v, want %+v", latest, wantLatest)
	}
	scores := assertFixtureParity[FinancialScore](t, "financial_scores.json")
	wantScore := FinancialScore{
		Symbol: "AAPL", ReportedCurrency: "USD", AltmanZScore: new(14.041374927993303), PiotroskiScore: 9,
		WorkingCapital: 9_473_000_000, TotalAssets: 371_082_000_000, RetainedEarnings: 12_359_000_000,
		Ebit: 147_722_000_000, MarketCap: 5_042_169_135_511, TotalLiabilities: 264_591_000_000, Revenue: 451_442_000_000,
	}
	if len(scores) != 1 || !reflect.DeepEqual(scores[0], wantScore) {
		t.Fatalf("financial_scores = %+v, want %+v", scores, wantScore)
	}
	owner := assertFixtureParity[OwnerEarnings](t, "owner_earnings.json")
	wantOwner := OwnerEarnings{
		Symbol: "AAPL", ReportedCurrency: "USD", FiscalYear: "2026", Period: "Q2", Date: mustParseDate(t, "2026-03-28"),
		AveragePpe: 0.13466, MaintenanceCapex: 159_994_500, OwnersEarnings: 28_861_994_500, GrowthCapex: -2_130_994_500,
		OwnersEarningsPerShare: 1.95,
	}
	if len(owner) != 1 || owner[0] != wantOwner {
		t.Fatalf("owner_earnings = %+v, want %+v", owner, wantOwner)
	}
	enterprise := assertFixtureParity[EnterpriseValue](t, "enterprise_values.json")
	wantEnterprise := EnterpriseValue{
		Symbol: "AAPL", Date: mustParseDate(t, "2025-09-27"), StockPrice: 255.46, NumberOfShares: 14_948_500_000,
		MarketCapitalization: 3_818_743_810_000, MinusCashAndCashEquivalents: 35_934_000_000,
		AddTotalDebt: 112_377_000_000, EnterpriseValue: 3_895_186_810_000,
	}
	if len(enterprise) != 1 || enterprise[0] != wantEnterprise {
		t.Fatalf("enterprise_values = %+v, want %+v", enterprise, wantEnterprise)
	}
	// The averagePPE acronym is exact: a camelCase respelling is a missing member.
	var respelled []OwnerEarnings
	err := json.Unmarshal([]byte(`[{"symbol":"AAPL","reportedCurrency":"USD","fiscalYear":"2026","period":"Q2","date":"2026-03-28",`+
		`"averagePpe":0.1,"maintenanceCapex":1,"ownersEarnings":1,"growthCapex":1,"ownersEarningsPerShare":1}]`), &respelled)
	if err == nil || !strings.Contains(err.Error(), `"averagePPE"`) {
		t.Fatalf("respelled averagePPE: error = %v", err)
	}
}
