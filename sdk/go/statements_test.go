package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

const (
	statementsTicker     = "BRK.B / Class A"
	statementsXlsxType   = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
	statementsXlsxBytes  = "opaque workbook bytes: not ZIP validated"
	statementsDispositon = "attachment; filename=provider-report.xlsx"
)

// statementsRoutes is the exact path and query table of the 27 statements
// endpoints, one query shape per method, with the wire text copied from the
// custom_proxy tests in crates/libfmp/tests/statements_*_endpoints.rs. The
// xlsx route answers with bytes and the official spreadsheet media type.
var statementsRoutes = map[string]struct {
	query   string
	fixture string
}{
	"/router/stable/income-statement":                     {"symbol=BRK.B+%2F+Class+A&limit=5&period=Q1", "income_statement.json"},
	"/router/stable/income-statement-ttm":                 {"symbol=BRK.B+%2F+Class+A&limit=7", "income_statement_ttm.json"},
	"/router/stable/balance-sheet-statement":              {"symbol=BRK.B+%2F+Class+A&limit=5&period=Q1", "balance_sheet_statement.json"},
	"/router/stable/balance-sheet-statement-ttm":          {"symbol=BRK.B+%2F+Class+A&limit=7", "balance_sheet_statement_ttm.json"},
	"/router/stable/cash-flow-statement":                  {"symbol=BRK.B+%2F+Class+A&limit=5&period=Q1", "cash_flow_statement.json"},
	"/router/stable/cash-flow-statement-ttm":              {"symbol=BRK.B+%2F+Class+A&limit=7", "cash_flow_statement_ttm.json"},
	"/router/stable/key-metrics":                          {"symbol=BRK.B+%2F+Class+A&limit=5&period=annual", "key_metrics.json"},
	"/router/stable/key-metrics-ttm":                      {"symbol=BRK.B+%2F+Class+A", "key_metrics_ttm.json"},
	"/router/stable/ratios":                               {"symbol=BRK.B+%2F+Class+A&limit=7&period=Q4", "financial_ratios.json"},
	"/router/stable/ratios-ttm":                           {"symbol=BRK.B+%2F+Class+A", "financial_ratios_ttm.json"},
	"/router/stable/income-statement-growth":              {"symbol=BRK.B+%2F+Class+A&limit=5&period=Q1", "income_statement_growth.json"},
	"/router/stable/balance-sheet-statement-growth":       {"symbol=BRK.B+%2F+Class+A&limit=7&period=quarter", "balance_sheet_statement_growth.json"},
	"/router/stable/cash-flow-statement-growth":           {"symbol=BRK.B+%2F+Class+A&limit=5&period=Q1", "cash_flow_statement_growth.json"},
	"/router/stable/financial-growth":                     {"symbol=BRK.B+%2F+Class+A&limit=7&period=quarter", "financial_statement_growth.json"},
	"/router/stable/income-statement-as-reported":         {"symbol=BRK.B+%2F+Class+A&limit=5&period=annual", "income_statement_as_reported.json"},
	"/router/stable/balance-sheet-statement-as-reported":  {"symbol=BRK.B+%2F+Class+A&limit=7&period=quarter", "balance_sheet_statement_as_reported.json"},
	"/router/stable/cash-flow-statement-as-reported":      {"symbol=BRK.B+%2F+Class+A&limit=5&period=annual", "cash_flow_statement_as_reported.json"},
	"/router/stable/financial-statement-full-as-reported": {"symbol=BRK.B+%2F+Class+A&limit=7&period=quarter", "financial_statement_full_as_reported.json"},
	"/router/stable/financial-reports-dates":              {"symbol=BRK.B+%2F+Class+A", "financial_reports_dates.json"},
	"/router/stable/financial-reports-json":               {"symbol=BRK.B+%2F+Class+A&year=2022&period=Q3", "financial_reports_json.json"},
	"/router/stable/financial-reports-xlsx":               {"symbol=BRK.B+%2F+Class+A&year=2022&period=FY", ""},
	"/router/stable/revenue-product-segmentation":         {"symbol=BRK.B+%2F+Class+A&period=annual&structure=flat", "revenue_product_segmentation.json"},
	"/router/stable/revenue-geographic-segmentation":      {"symbol=BRK.B+%2F+Class+A", "revenue_geographic_segmentation.json"},
	"/router/stable/latest-financial-statements":          {"page=0&limit=250", "latest_financial_statements.json"},
	"/router/stable/financial-scores":                     {"symbol=BRK.B+%2F+Class+A", "financial_scores.json"},
	"/router/stable/owner-earnings":                       {"symbol=BRK.B+%2F+Class+A&limit=5", "owner_earnings.json"},
	"/router/stable/enterprise-values":                    {"symbol=BRK.B+%2F+Class+A&limit=7&period=quarter", "enterprise_values.json"},
}

func statementsRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		route, ok := statementsRoutes[r.URL.Path]
		if !ok || r.URL.RawQuery != route.query || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s?%s", r.URL.Path, r.URL.RawQuery)
			return
		}
		if route.fixture == "" {
			w.Header().Set("Content-Type", statementsXlsxType)
			w.Header().Set("Content-Disposition", statementsDispositon)
			_, _ = w.Write([]byte(statementsXlsxBytes))
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, route.fixture))
	}
}

// statementsCheck fails unless a call returned exactly one row that
// satisfies the predicate, keeping the 27-call route test readable.
func statementsCheck[T any](t *testing.T, name string, rows []T, err error, ok func(T) bool) {
	t.Helper()
	if err != nil || len(rows) != 1 || !ok(rows[0]) {
		t.Fatalf("%s = %+v, %v", name, rows, err)
	}
}

func TestStatementsMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, statementsRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	s := client.Statements

	income, err := s.Income.Statement(ctx, NewIncomeStatementQuery(statementsTicker).WithLimit(5).WithPeriod(StatementPeriodQ1))
	statementsCheck(t, "Income.Statement", income, err, func(r IncomeStatement) bool { return r.Revenue == 416_161_000_000 })
	incomeTtm, err := s.Income.StatementTTM(ctx, NewIncomeStatementTTMQuery(statementsTicker).WithLimit(7))
	statementsCheck(t, "Income.StatementTTM", incomeTtm, err, func(r IncomeStatement) bool { return r.Period == "Q2" })
	balance, err := s.Balance.Statement(ctx, NewBalanceSheetStatementQuery(statementsTicker).WithLimit(5).WithPeriod(StatementPeriodQ1))
	statementsCheck(t, "Balance.Statement", balance, err, func(r BalanceSheetStatement) bool { return r.TotalAssets == 359_241_000_000 })
	balanceTtm, err := s.Balance.StatementTTM(ctx, NewBalanceSheetStatementTTMQuery(statementsTicker).WithLimit(7))
	statementsCheck(t, "Balance.StatementTTM", balanceTtm, err, func(r BalanceSheetStatementTTM) bool { return r.TotalAssets == 371_082_000_000 })
	cash, err := s.CashFlow.Statement(ctx, NewCashFlowStatementQuery(statementsTicker).WithLimit(5).WithPeriod(StatementPeriodQ1))
	statementsCheck(t, "CashFlow.Statement", cash, err, func(r CashFlowStatement) bool { return r.FreeCashFlow == 98_767_000_000 })
	cashTtm, err := s.CashFlow.StatementTTM(ctx, NewCashFlowStatementTTMQuery(statementsTicker).WithLimit(7))
	statementsCheck(t, "CashFlow.StatementTTM", cashTtm, err, func(r CashFlowStatement) bool { return r.FreeCashFlow == 129_174_000_000 })
	metrics, err := s.Metrics.KeyMetrics(ctx, NewKeyMetricsQuery(statementsTicker).WithLimit(5).WithPeriod(StatementPeriodAnnual))
	statementsCheck(t, "Metrics.KeyMetrics", metrics, err, func(r KeyMetrics) bool { return r.InterestBurden == 1 })
	metricsTtm, err := s.Metrics.KeyMetricsTTM(ctx, NewKeyMetricsTTMQuery(statementsTicker))
	statementsCheck(t, "Metrics.KeyMetricsTTM", metricsTtm, err, func(r KeyMetricsTTM) bool { return r.CurrentRatioTTM == 1.07035746912159 })
	ratios, err := s.Ratios.FinancialRatios(ctx, NewFinancialRatiosQuery(statementsTicker).WithLimit(7).WithPeriod(StatementPeriodQ4))
	statementsCheck(t, "Ratios.FinancialRatios", ratios, err, func(r FinancialRatios) bool { return r.FiscalYear == "2025" })
	ratiosTtm, err := s.Ratios.FinancialRatiosTTM(ctx, NewFinancialRatiosTTMQuery(statementsTicker))
	statementsCheck(t, "Ratios.FinancialRatiosTTM", ratiosTtm, err, func(r FinancialRatiosTTM) bool { return r.EnterpriseValueTTM == 4_922_455_686_740 })
	incomeGrowth, err := s.Growth.Income(ctx, NewIncomeStatementGrowthQuery(statementsTicker).WithLimit(5).WithPeriod(StatementPeriodQ1))
	statementsCheck(t, "Growth.Income", incomeGrowth, err, func(r IncomeStatementGrowth) bool { return r.GrowthSellingAndMarketingExpenses == -1 })
	balanceGrowth, err := s.Growth.BalanceSheet(ctx, NewBalanceSheetStatementGrowthQuery(statementsTicker).WithLimit(7).WithPeriod(StatementPeriodQuarterly))
	statementsCheck(t, "Growth.BalanceSheet", balanceGrowth, err, func(r BalanceSheetStatementGrowth) bool { return r.GrowthTaxPayables == -1 })
	cashGrowth, err := s.Growth.CashFlow(ctx, NewCashFlowStatementGrowthQuery(statementsTicker).WithLimit(5).WithPeriod(StatementPeriodQ1))
	statementsCheck(t, "Growth.CashFlow", cashGrowth, err, func(r CashFlowStatementGrowth) bool { return r.GrowthInventory == 2.338432122370937 })
	combinedGrowth, err := s.Growth.Financial(ctx, NewFinancialStatementGrowthQuery(statementsTicker).WithLimit(7).WithPeriod(StatementPeriodQuarterly))
	statementsCheck(t, "Growth.Financial", combinedGrowth, err, func(r FinancialStatementGrowth) bool { return r.EPSGrowth == 0.22585924713584285 })
	asIncome, err := s.AsReported.Income(ctx, NewIncomeStatementAsReportedQuery(statementsTicker).WithLimit(5).WithPeriod(RetrievalFrequencyAnnual))
	statementsCheck(t, "AsReported.Income", asIncome, err, func(r AsReportedFinancialStatement) bool { return r.FiscalYear == 2025 })
	asBalance, err := s.AsReported.BalanceSheet(ctx, NewBalanceSheetStatementAsReportedQuery(statementsTicker).WithLimit(7).WithPeriod(RetrievalFrequencyQuarterly))
	statementsCheck(t, "AsReported.BalanceSheet", asBalance, err, func(r AsReportedFinancialStatement) bool { return r.Period == "FY" })
	asCash, err := s.AsReported.CashFlow(ctx, NewCashFlowStatementAsReportedQuery(statementsTicker).WithLimit(5).WithPeriod(RetrievalFrequencyAnnual))
	statementsCheck(t, "AsReported.CashFlow", asCash, err, func(r AsReportedFinancialStatement) bool { return r.ReportedCurrency == "USD" })
	asFull, err := s.AsReported.Full(ctx, NewFinancialStatementFullAsReportedQuery(statementsTicker).WithLimit(7).WithPeriod(RetrievalFrequencyQuarterly))
	statementsCheck(t, "AsReported.Full", asFull, err, func(r AsReportedFinancialStatement) bool { return r.Data.Kind() == '{' })
	dates, err := s.Reports.Dates(ctx, NewFinancialReportsDatesQuery(statementsTicker))
	statementsCheck(t, "Reports.Dates", dates, err, func(r FinancialReportDate) bool { return r.FiscalYear == 2026 })
	reports, err := s.Reports.JSON(ctx, NewFinancialReportsJSONQuery(statementsTicker, 2022, FiscalPeriodQ3))
	statementsCheck(t, "Reports.JSON", reports, err, func(r FinancialReportJSON) bool { return r.Year == "2022" })
	xlsx, err := s.Reports.Xlsx(ctx, NewFinancialReportsXlsxQuery(statementsTicker, 2022, FiscalPeriodFullYear))
	if err != nil || string(xlsx.Data) != statementsXlsxBytes || xlsx.ContentType != statementsXlsxType ||
		xlsx.ContentDisposition != statementsDispositon || xlsx.MediaType() != statementsXlsxType {
		t.Fatalf("Reports.Xlsx = %v, %v", xlsx, err)
	}
	product, err := s.Segmentation.RevenueProduct(ctx, NewRevenueProductSegmentationQuery(statementsTicker).WithPeriod(RetrievalFrequencyAnnual).WithStructure(SegmentationStructureFlat))
	statementsCheck(t, "Segmentation.RevenueProduct", product, err, func(r RevenueSegmentation) bool { return r.FiscalYear == 2025 })
	geographic, err := s.Segmentation.RevenueGeographic(ctx, NewRevenueGeographicSegmentationQuery(statementsTicker))
	statementsCheck(t, "Segmentation.RevenueGeographic", geographic, err, func(r RevenueSegmentation) bool { return r.Period == "FY" })
	latest, err := s.Summaries.LatestFinancialStatements(ctx, NewLatestFinancialStatementsQuery().WithPage(0).WithLimit(250))
	statementsCheck(t, "Summaries.LatestFinancialStatements", latest, err, func(r LatestFinancialStatement) bool { return r.Symbol == "UFPI" })
	scores, err := s.Summaries.FinancialScores(ctx, NewFinancialScoresQuery(statementsTicker))
	statementsCheck(t, "Summaries.FinancialScores", scores, err, func(r FinancialScore) bool { return r.PiotroskiScore == 9 })
	owner, err := s.Summaries.OwnerEarnings(ctx, NewOwnerEarningsQuery(statementsTicker).WithLimit(5))
	statementsCheck(t, "Summaries.OwnerEarnings", owner, err, func(r OwnerEarnings) bool { return r.AveragePpe == 0.13466 })
	enterprise, err := s.Summaries.EnterpriseValues(ctx, NewEnterpriseValuesQuery(statementsTicker).WithLimit(7).WithPeriod(StatementPeriodQuarterly))
	statementsCheck(t, "Summaries.EnterpriseValues", enterprise, err, func(r EnterpriseValue) bool { return r.StockPrice == 255.46 })

	requests := rec.all()
	if len(requests) != len(statementsRoutes) {
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

func TestStatementsQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, statementsRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	_, err := client.Statements.Income.Statement(ctx, NewIncomeStatementQuery("AAPL").WithPeriod("yearly"))
	typed := assertQuoteError(t, err, CategoryValidation, 0, "")
	if !errors.Is(err, ErrUnknownWireValue) || typed.Message != "period: "+ErrUnknownWireValue.Error()+", expected one of Q1, Q2, Q3, Q4, FY, annual, quarter" {
		t.Fatalf("unknown statement period: error = %v", err)
	}
	_, err = client.Statements.Reports.JSON(ctx, NewFinancialReportsJSONQuery("AAPL", 2022, "annual"))
	typed = assertQuoteError(t, err, CategoryValidation, 0, "")
	if !errors.Is(err, ErrUnknownWireValue) || typed.Message != "period: "+ErrUnknownWireValue.Error()+", expected one of Q1, Q2, Q3, Q4, FY" {
		t.Fatalf("frequency as fiscal period: error = %v", err)
	}
	_, err = client.Statements.Segmentation.RevenueProduct(ctx, NewRevenueProductSegmentationQuery("AAPL").WithStructure("nested"))
	typed = assertQuoteError(t, err, CategoryValidation, 0, "")
	if !errors.Is(err, ErrUnknownWireValue) || typed.Message != "structure: "+ErrUnknownWireValue.Error()+", expected one of flat" {
		t.Fatalf("unknown structure: error = %v", err)
	}
	_, err = client.Statements.Summaries.FinancialScores(ctx, NewFinancialScoresQuery("AAPL,MSFT"))
	if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrCommaInTicker) ||
		typed.Message != "symbol: "+ErrCommaInTicker.Error() {
		t.Fatalf("comma ticker: error = %v", err)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	q := NewIncomeStatementQuery(" AAPL ")
	if q.Symbol() != " AAPL " || q.Limit() != nil || q.Period() != nil {
		t.Fatalf("getters = %q %v %v", q.Symbol(), q.Limit(), q.Period())
	}
	if period := q.WithPeriod(StatementPeriodFullYear).Period(); period == nil || *period != StatementPeriodFullYear || q.Period() != nil {
		t.Fatalf("WithPeriod mutated the receiver or lost the value: %v %v", period, q.Period())
	}
	report := NewFinancialReportsXlsxQuery("AAPL", 2022, FiscalPeriodQ4)
	if report.Symbol() != "AAPL" || report.Year() != 2022 || report.Period() != FiscalPeriodQ4 {
		t.Fatalf("xlsx getters = %q %d %q", report.Symbol(), report.Year(), report.Period())
	}
}

func TestStatementsXlsxRejectsAnUnexpectedMediaTypeAfterOneRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[]`))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Statements.Reports.Xlsx(context.Background(), NewFinancialReportsXlsxQuery("AAPL", 2022, FiscalPeriodFullYear))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "financial-reports-xlsx")
	if typed.Message != "successful response used an unexpected content type" || rec.count() != 1 {
		t.Fatalf("xlsx with a JSON body: error = %v after %d requests", err, rec.count())
	}
}

func TestStatementsMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[{"symbol":"AAPL","date":"2025-09-27","stockPrice":255.46,"numberOfShares":1,`+
		`"marketCapitalization":1,"minusCashAndCashEquivalents":1,"addTotalDebt":1}]`))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Statements.Summaries.EnterpriseValues(context.Background(), NewEnterpriseValuesQuery("AAPL"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "enterprise-values")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"enterpriseValue"`) {
		t.Fatalf("cause = %v, want it to name the missing member enterpriseValue", cause)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}
