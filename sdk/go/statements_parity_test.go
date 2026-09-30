package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"reflect"
	"testing"
)

// Every shared fixture a Rust test decodes into a statements model (grep
// fixtures/ over crates/libfmp/tests/statements_*.rs), through the ADR 0030
// parity rule. The dynamic members of the as-reported, segmentation, and
// report fixtures are re-emitted byte for byte, so no fixture carries an
// intentionally unknown member.
func TestStatementsFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[IncomeStatement](t, "income_statement.json")
	assertFixtureParity[IncomeStatement](t, "income_statement_ttm.json")
	assertFixtureParity[BalanceSheetStatement](t, "balance_sheet_statement.json")
	assertFixtureParity[BalanceSheetStatementTTM](t, "balance_sheet_statement_ttm.json")
	assertFixtureParity[CashFlowStatement](t, "cash_flow_statement.json")
	assertFixtureParity[CashFlowStatement](t, "cash_flow_statement_ttm.json")
	assertFixtureParity[KeyMetrics](t, "key_metrics.json")
	assertFixtureParity[KeyMetricsTTM](t, "key_metrics_ttm.json")
	assertFixtureParity[FinancialRatios](t, "financial_ratios.json")
	assertFixtureParity[FinancialRatiosTTM](t, "financial_ratios_ttm.json")
	assertFixtureParity[IncomeStatementGrowth](t, "income_statement_growth.json")
	assertFixtureParity[BalanceSheetStatementGrowth](t, "balance_sheet_statement_growth.json")
	assertFixtureParity[CashFlowStatementGrowth](t, "cash_flow_statement_growth.json")
	assertFixtureParity[FinancialStatementGrowth](t, "financial_statement_growth.json")
	assertFixtureParity[AsReportedFinancialStatement](t, "income_statement_as_reported.json")
	assertFixtureParity[AsReportedFinancialStatement](t, "balance_sheet_statement_as_reported.json")
	assertFixtureParity[AsReportedFinancialStatement](t, "cash_flow_statement_as_reported.json")
	assertFixtureParity[AsReportedFinancialStatement](t, "financial_statement_full_as_reported.json")
	assertFixtureParity[FinancialReportDate](t, "financial_reports_dates.json")
	assertObjectFixtureParity[FinancialReportJSON](t, "financial_reports_json.json")
	assertFixtureParity[RevenueSegmentation](t, "revenue_product_segmentation.json")
	assertFixtureParity[RevenueSegmentation](t, "revenue_geographic_segmentation.json")
	assertFixtureParity[LatestFinancialStatement](t, "latest_financial_statements.json")
	assertFixtureParity[FinancialScore](t, "financial_scores.json")
	assertFixtureParity[OwnerEarnings](t, "owner_earnings.json")
	assertFixtureParity[EnterpriseValue](t, "enterprise_values.json")
}

// Exact values copied from historical_expected in
// crates/libfmp/tests/statements_income_responses.rs.
func TestDocumentedIncomeStatementDecodesAll39FieldsExactly(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[IncomeStatement](t, "income_statement.json")
	want := IncomeStatement{
		Date: mustParseDate(t, "2025-09-27"), Symbol: "AAPL", ReportedCurrency: "USD", CIK: "0000320193",
		FilingDate: new(mustParseDate(t, "2025-10-31")), AcceptedDate: new(mustParseDateTime(t, "2025-10-31 06:01:26")),
		FiscalYear: "2025", Period: "FY",
		Revenue: 416_161_000_000, CostOfRevenue: 220_960_000_000, GrossProfit: 195_201_000_000,
		ResearchAndDevelopmentExpenses: 34_550_000_000, GeneralAndAdministrativeExpenses: 27_601_000_000,
		SellingAndMarketingExpenses: 0, SellingGeneralAndAdministrativeExpenses: 27_601_000_000,
		OtherExpenses: 0, OperatingExpenses: 62_151_000_000, CostAndExpenses: 283_111_000_000,
		NetInterestIncome: 0, InterestIncome: new(0.0), InterestExpense: 0,
		DepreciationAndAmortization: 11_698_000_000, Ebitda: 144_427_000_000, Ebit: 132_729_000_000,
		NonOperatingIncomeExcludingInterest: 321_000_000, OperatingIncome: 133_050_000_000,
		TotalOtherIncomeExpensesNet: -321_000_000, IncomeBeforeTax: 132_729_000_000,
		IncomeTaxExpense: 20_719_000_000, NetIncomeFromContinuingOperations: 112_010_000_000,
		NetIncomeFromDiscontinuedOperations: 0, OtherAdjustmentsToNetIncome: 0, NetIncome: 112_010_000_000,
		NetIncomeDeductions: 0, BottomLineNetIncome: 112_010_000_000, EPS: 7.49, EPSDiluted: 7.46,
		WeightedAverageShsOut: 14_948_500_000, WeightedAverageShsOutDil: 15_004_697_000,
	}
	if len(rows) != 1 || !reflect.DeepEqual(rows[0], want) {
		t.Fatalf("income_statement = %+v, want %+v", rows, want)
	}
	if got := memberSet(t, rows[0]); len(got) != 39 {
		t.Fatalf("re-encoded members = %d, want 39", len(got))
	}
	ttm := assertFixtureParity[IncomeStatement](t, "income_statement_ttm.json")
	if len(ttm) != 1 || ttm[0].Date != mustParseDate(t, "2026-03-28") || ttm[0].Period != "Q2" ||
		ttm[0].Revenue != 451_442_000_000 || ttm[0].AcceptedDate == nil || *ttm[0].AcceptedDate != mustParseDateTime(t, "2026-05-01 10:01:00") {
		t.Fatalf("income_statement_ttm = %+v", ttm)
	}
}

func TestDocumentedBalanceSheetAndCashFlowFixturesDecodeExactAmounts(t *testing.T) {
	t.Parallel()
	balance := assertFixtureParity[BalanceSheetStatement](t, "balance_sheet_statement.json")
	if len(balance) != 1 || balance[0].CIK != "0000320193" || balance[0].FiscalYear != "2025" ||
		balance[0].Period != "FY" || !amountIs(balance[0].TotalAssets, 359_241_000_000) ||
		!amountIs(balance[0].RetainedEarnings, -14_264_000_000) || len(memberSet(t, balance[0])) != 61 {
		t.Fatalf("balance_sheet_statement = %+v", balance)
	}
	ttm := assertFixtureParity[BalanceSheetStatementTTM](t, "balance_sheet_statement_ttm.json")
	if len(ttm) != 1 || ttm[0].Date != mustParseDate(t, "2026-03-28") || ttm[0].Period != "Q2" ||
		ttm[0].TotalAssets != 371_082_000_000 || ttm[0].RetainedEarnings != 12_359_000_000 ||
		len(memberSet(t, ttm[0])) != 60 {
		t.Fatalf("balance_sheet_statement_ttm = %+v", ttm)
	}
	cash := assertFixtureParity[CashFlowStatement](t, "cash_flow_statement.json")
	if len(cash) != 1 || cash[0].NetIncome != 112_010_000_000 || cash[0].FreeCashFlow != 98_767_000_000 ||
		!amountIs(cash[0].NetCashProvidedByFinancingActivities, -120_686_000_000) || len(memberSet(t, cash[0])) != 47 {
		t.Fatalf("cash_flow_statement = %+v", cash)
	}
	cashTTM := assertFixtureParity[CashFlowStatement](t, "cash_flow_statement_ttm.json")
	if len(cashTTM) != 1 || cashTTM[0].Period != "Q2" || cashTTM[0].FreeCashFlow != 129_174_000_000 ||
		!amountIs(cashTTM[0].NetCashProvidedByFinancingActivities, -114_244_000_000) {
		t.Fatalf("cash_flow_statement_ttm = %+v", cashTTM)
	}
}

// statementsWithMember returns the first row of a fixture with one member
// replaced by raw JSON text, the way the Rust boundary tests mutate a
// serde_json::Value before decoding.
func statementsWithMember(t *testing.T, fixture, member, raw string) []byte {
	t.Helper()
	var rows []map[string]jsontext.Value
	if err := json.Unmarshal(readFixture(t, fixture), &rows); err != nil {
		t.Fatalf("%s: %v", fixture, err)
	}
	if _, ok := rows[0][member]; !ok {
		t.Fatalf("%s: member %q is absent from the first row", fixture, member)
	}
	rows[0][member] = jsontext.Value(raw)
	encoded, err := json.Marshal(rows[:1])
	if err != nil {
		t.Fatal(err)
	}
	return encoded
}

// statementsRoundTripExtreme decodes a mutated fixture row into T and proves
// the member re-encodes to the exact digits, as the Rust tests assert with
// serde_json::to_value.
func statementsRoundTripExtreme[T any](t *testing.T, fixture, member, digits string) T {
	t.Helper()
	var rows []T
	if err := json.Unmarshal(statementsWithMember(t, fixture, member, digits), &rows); err != nil {
		t.Fatalf("%s with %s=%s: %v", fixture, member, digits, err)
	}
	encoded, err := json.Marshal(rows[0])
	if err != nil {
		t.Fatal(err)
	}
	var members map[string]jsontext.Value
	if err := json.Unmarshal(encoded, &members); err != nil {
		t.Fatal(err)
	}
	if got := string(members[member]); got != digits {
		t.Fatalf("%s: %s re-encoded as %s, want %s", fixture, member, got, digits)
	}
	return rows[0]
}

// The large-value cases of statements_income_responses.rs,
// statements_balance_responses.rs, statements_cash_flow_responses.rs, and
// statements_summary_responses.rs: StatementAmount is float64, so large signed
// integral amounts round-trip as JSON integers; share quantities keep a
// fractional value.
func TestStatementAmountsPreserveLargeIntegralValuesAndShareQuantitiesKeepFractions(t *testing.T) {
	t.Parallel()
	large := "9000000000000000000"
	negative := "-9000000000000000000"

	income := statementsRoundTripExtreme[IncomeStatement](t, "income_statement.json", "revenue", large)
	if income.Revenue != 9e18 {
		t.Fatalf("revenue = %v", income.Revenue)
	}
	income = statementsRoundTripExtreme[IncomeStatement](t, "income_statement.json", "totalOtherIncomeExpensesNet", negative)
	if income.TotalOtherIncomeExpensesNet != -9e18 {
		t.Fatalf("totalOtherIncomeExpensesNet = %v", income.TotalOtherIncomeExpensesNet)
	}
	income = statementsRoundTripExtreme[IncomeStatement](t, "income_statement.json", "weightedAverageShsOutDil", "15004697000.5")
	if income.WeightedAverageShsOutDil != 15_004_697_000.5 {
		t.Fatalf("weightedAverageShsOutDil = %v", income.WeightedAverageShsOutDil)
	}

	balance := statementsRoundTripExtreme[BalanceSheetStatement](t, "balance_sheet_statement.json", "totalAssets", large)
	balanceTTM := statementsRoundTripExtreme[BalanceSheetStatementTTM](t, "balance_sheet_statement_ttm.json", "retainedEarnings", negative)
	if !amountIs(balance.TotalAssets, 9e18) || balanceTTM.RetainedEarnings != -9e18 {
		t.Fatalf("balance extremes = %v %v", balance.TotalAssets, balanceTTM.RetainedEarnings)
	}
	for _, fixture := range []string{"cash_flow_statement.json", "cash_flow_statement_ttm.json"} {
		cash := statementsRoundTripExtreme[CashFlowStatement](t, fixture, "freeCashFlow", large)
		financing := statementsRoundTripExtreme[CashFlowStatement](t, fixture, "netCashProvidedByFinancingActivities", negative)
		if cash.FreeCashFlow != 9e18 || !amountIs(financing.NetCashProvidedByFinancingActivities, -9e18) {
			t.Fatalf("%s extremes = %v %v", fixture, cash.FreeCashFlow, financing.NetCashProvidedByFinancingActivities)
		}
	}
	owner := statementsRoundTripExtreme[OwnerEarnings](t, "owner_earnings.json", "growthCapex", "-9000000000000000000")
	if owner.GrowthCapex != -9_000_000_000_000_000_000 {
		t.Fatalf("growthCapex = %v", owner.GrowthCapex)
	}

	// A signed value in an unsigned Count and a number in a FiscalYearString
	// are decode errors, as serde reports them.
	var scores []FinancialScore
	if err := json.Unmarshal(statementsWithMember(t, "financial_scores.json", "piotroskiScore", "-1"), &scores); err == nil {
		t.Fatal("negative piotroskiScore decoded into a uint64")
	}
	var incomes []IncomeStatement
	if err := json.Unmarshal(statementsWithMember(t, "income_statement.json", "fiscalYear", "2025"), &incomes); err == nil {
		t.Fatal("numeric fiscalYear decoded into a string")
	}
	var balances []BalanceSheetStatementTTM
	if err := json.Unmarshal(statementsWithMember(t, "balance_sheet_statement_ttm.json", "fiscalYear", "2026"), &balances); err == nil {
		t.Fatal("numeric fiscalYear decoded into a string on the TTM contract")
	}
}

// amountIs reports whether an optional statement amount is present and equal
// to want.
func amountIs(got *float64, want float64) bool {
	return got != nil && *got == want
}

// The pre-IPO stub-quarter members of statements_balance_responses.rs and
// statements_cash_flow_responses.rs: FMP sends null for them, which decodes to
// nil and re-encodes as null.
var (
	balanceStubQuarterNullMembers = []string{
		"accountPayables", "accountsReceivables", "accruedExpenses", "accumulatedOtherComprehensiveIncomeLoss",
		"additionalPaidInCapital", "capitalLeaseObligationsCurrent", "capitalLeaseObligationsNonCurrent",
		"cashAndCashEquivalents", "cashAndShortTermInvestments", "commonStock", "deferredRevenue",
		"deferredRevenueNonCurrent", "deferredTaxLiabilitiesNonCurrent", "goodwill", "goodwillAndIntangibleAssets",
		"intangibleAssets", "inventory", "longTermDebt", "longTermInvestments", "netReceivables", "otherPayables",
		"prepaids", "propertyPlantEquipmentNet", "retainedEarnings", "shortTermDebt", "shortTermInvestments",
		"taxAssets", "taxPayables", "totalAssets", "totalCurrentAssets", "totalCurrentLiabilities", "totalEquity",
		"totalLiabilities", "totalLiabilitiesAndTotalEquity", "totalNonCurrentAssets", "totalNonCurrentLiabilities",
		"totalStockholdersEquity", "treasuryStock",
	}
	cashFlowStubQuarterNullMembers = []string{
		"accountsPayables", "accountsReceivables", "cashAtBeginningOfPeriod", "commonDividendsPaid",
		"commonStockRepurchased", "deferredIncomeTax", "incomeTaxesPaid", "interestPaid", "inventory",
		"longTermNetDebtIssuance", "netCashProvidedByFinancingActivities", "netCommonStockIssuance",
		"netDividendsPaid", "netPreferredStockIssuance", "netStockIssuance", "preferredDividendsPaid",
		"purchasesOfInvestments", "salesMaturitiesOfInvestments", "stockBasedCompensation",
	}
)

func statementsWithNullMembers(t *testing.T, fixture string, members []string) []byte {
	t.Helper()
	var rows []map[string]jsontext.Value
	if err := json.Unmarshal(readFixture(t, fixture), &rows); err != nil {
		t.Fatalf("%s: %v", fixture, err)
	}
	for _, member := range members {
		if _, ok := rows[0][member]; !ok {
			t.Fatalf("%s: member %q is absent from the first row", fixture, member)
		}
		rows[0][member] = jsontext.Value("null")
	}
	encoded, err := json.Marshal(rows[:1])
	if err != nil {
		t.Fatal(err)
	}
	return encoded
}

func assertStubMembersReencodeNull(t *testing.T, row any, members []string) {
	t.Helper()
	encoded, err := json.Marshal(row)
	if err != nil {
		t.Fatal(err)
	}
	var got map[string]jsontext.Value
	if err := json.Unmarshal(encoded, &got); err != nil {
		t.Fatal(err)
	}
	for _, member := range members {
		if string(got[member]) != "null" {
			t.Fatalf("%s re-encoded as %s, want null", member, got[member])
		}
	}
}

func TestPreIPOStubRowsDecodeNullAmountsAsNil(t *testing.T) {
	t.Parallel()
	var balance []BalanceSheetStatement
	if err := json.Unmarshal(statementsWithNullMembers(t, "balance_sheet_statement.json", balanceStubQuarterNullMembers), &balance); err != nil {
		t.Fatalf("balance stub row: %v", err)
	}
	if balance[0].TotalAssets != nil || balance[0].OtherReceivables == 0 {
		t.Fatalf("balance stub row = %+v", balance[0])
	}
	assertStubMembersReencodeNull(t, balance[0], balanceStubQuarterNullMembers)

	var cash []CashFlowStatement
	if err := json.Unmarshal(statementsWithNullMembers(t, "cash_flow_statement.json", cashFlowStubQuarterNullMembers), &cash); err != nil {
		t.Fatalf("cash flow stub row: %v", err)
	}
	if cash[0].StockBasedCompensation != nil || cash[0].NetIncome == 0 {
		t.Fatalf("cash flow stub row = %+v", cash[0])
	}
	assertStubMembersReencodeNull(t, cash[0], cashFlowStubQuarterNullMembers)
}

// Members FMP sends as null outside stub quarters (issue #368 audit): each
// decodes to nil and re-encodes as null, mirroring the nullable-member tests
// in the income, metrics, summary and as-reported Rust response suites.
func TestNullableStatementMembersDecodeNullAsNil(t *testing.T) {
	t.Parallel()
	incomeMembers := []string{"interestIncome", "filingDate", "acceptedDate"}
	var income []IncomeStatement
	if err := json.Unmarshal(statementsWithNullMembers(t, "income_statement.json", incomeMembers), &income); err != nil {
		t.Fatalf("income: %v", err)
	}
	if income[0].InterestIncome != nil || income[0].FilingDate != nil || income[0].AcceptedDate != nil {
		t.Fatalf("income = %+v", income[0])
	}
	assertStubMembersReencodeNull(t, income[0], incomeMembers)

	var metrics []KeyMetrics
	if err := json.Unmarshal(statementsWithNullMembers(t, "key_metrics.json", []string{"grahamNumber"}), &metrics); err != nil {
		t.Fatalf("key metrics: %v", err)
	}
	if metrics[0].GrahamNumber != nil {
		t.Fatalf("key metrics = %+v", metrics[0])
	}
	assertStubMembersReencodeNull(t, metrics[0], []string{"grahamNumber"})

	var metricsTTM []KeyMetricsTTM
	if err := json.Unmarshal(statementsWithNullMembers(t, "key_metrics_ttm.json", []string{"grahamNumberTTM"}), &metricsTTM); err != nil {
		t.Fatalf("key metrics ttm: %v", err)
	}
	if metricsTTM[0].GrahamNumberTTM != nil {
		t.Fatalf("key metrics ttm = %+v", metricsTTM[0])
	}
	assertStubMembersReencodeNull(t, metricsTTM[0], []string{"grahamNumberTTM"})

	var scores []FinancialScore
	if err := json.Unmarshal(statementsWithNullMembers(t, "financial_scores.json", []string{"altmanZScore"}), &scores); err != nil {
		t.Fatalf("financial score: %v", err)
	}
	if scores[0].AltmanZScore != nil {
		t.Fatalf("financial score = %+v", scores[0])
	}
	assertStubMembersReencodeNull(t, scores[0], []string{"altmanZScore"})

	var asReported []AsReportedFinancialStatement
	if err := json.Unmarshal(statementsWithNullMembers(t, "income_statement_as_reported.json", []string{"reportedCurrency"}), &asReported); err != nil {
		t.Fatalf("as reported: %v", err)
	}
	if asReported[0].ReportedCurrency != nil {
		t.Fatalf("as reported = %+v", asReported[0])
	}
	assertStubMembersReencodeNull(t, asReported[0], []string{"reportedCurrency"})
}
