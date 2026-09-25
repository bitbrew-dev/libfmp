package fmp

import (
	"context"
	"errors"
	"fmt"
	"math"
	"net/http"
	"strings"
	"testing"
)

// dcfDocumentedAssumptions is the exact wire text of every assumption from
// crates/libfmp/tests/dcf_custom_endpoints.rs, in the order the Rust query
// encodes them after the symbol.
const dcfDocumentedAssumptions = "revenueGrowthPct=0.1094119804597946&ebitdaPct=0.31273548388" +
	"&depreciationAndAmortizationPct=0.0345531631720999&cashAndShortTermInvestmentsPct=0.2344222126801843" +
	"&receivablesPct=0.1533770531229388&inventoriesPct=0.0155245674227653&payablePct=0.1614868903169657" +
	"&ebitPct=0.2781823207138459&capitalExpenditurePct=0.0306025847141713&operatingCashFlowPct=0.2886333485760204" +
	"&sellingGeneralAndAdministrativeExpensesPct=0.0662854095187211&taxRate=0.14919579658453103" +
	"&longTermGrowthRate=4&costOfDebt=3.64&costOfEquity=9.51168&marketRiskPremium=4.72&beta=1.244&riskFreeRate=3.64"

// dcfRoutes is keyed by path plus raw query, since the custom paths are
// requested with more than one query shape: every assumption omitted, a few
// set (zero and negative encoded exactly), and all eighteen in wire order.
var dcfRoutes = map[string]string{
	"/router/stable/discounted-cash-flow?symbol=BRK.B+%2F+Class+A":         "discounted_cash_flow.json",
	"/router/stable/levered-discounted-cash-flow?symbol=BRK.B+%2F+Class+A": "levered_discounted_cash_flow.json",
	"/router/stable/custom-discounted-cash-flow?symbol=AAPL":               "custom_discounted_cash_flow.json",
	"/router/stable/custom-discounted-cash-flow?symbol=BRK.B+%2F+Class+A&" +
		dcfDocumentedAssumptions: "custom_discounted_cash_flow.json",
	"/router/stable/custom-levered-discounted-cash-flow?symbol=AAPL&revenueGrowthPct=0&taxRate=-1.25" +
		"&longTermGrowthRate=4": "custom_levered_discounted_cash_flow.json",
	"/router/stable/custom-levered-discounted-cash-flow?symbol=BRK.B+%2F+Class+A&" +
		dcfDocumentedAssumptions: "custom_levered_discounted_cash_flow.json",
}

func dcfRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := dcfRoutes[r.URL.Path+"?"+r.URL.RawQuery]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s?%s", r.URL.Path, r.URL.RawQuery)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func dcfDocumentedQuery(symbol string) CustomDcfQuery {
	return NewCustomDcfQuery(symbol).
		WithRevenueGrowthPct(0.1094119804597946).
		WithEbitdaPct(0.31273548388).
		WithDepreciationAndAmortizationPct(0.0345531631720999).
		WithCashAndShortTermInvestmentsPct(0.2344222126801843).
		WithReceivablesPct(0.1533770531229388).
		WithInventoriesPct(0.0155245674227653).
		WithPayablePct(0.1614868903169657).
		WithEbitPct(0.2781823207138459).
		WithCapitalExpenditurePct(0.0306025847141713).
		WithOperatingCashFlowPct(0.2886333485760204).
		WithSellingGeneralAndAdministrativeExpensesPct(0.0662854095187211).
		WithTaxRate(0.14919579658453103).
		WithLongTermGrowthRate(4).
		WithCostOfDebt(3.64).
		WithCostOfEquity(9.51168).
		WithMarketRiskPremium(4.72).
		WithBeta(1.244).
		WithRiskFreeRate(3.64)
}

func TestDcfMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, dcfRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	standard, err := client.Dcf.Standard(ctx, NewDcfQuery("BRK.B / Class A"))
	if err != nil || len(standard) != 1 || standard[0].Dcf != 147.10881272667325 {
		t.Fatalf("Standard = %+v, %v", standard, err)
	}
	levered, err := client.Dcf.Levered(ctx, NewDcfQuery("BRK.B / Class A"))
	if err != nil || len(levered) != 1 || levered[0].Dcf != 140.6429495133426 {
		t.Fatalf("Levered = %+v, %v", levered, err)
	}
	omitted, err := client.Dcf.Custom(ctx, NewCustomDcfQuery("AAPL"))
	if err != nil || len(omitted) != 1 || omitted[0].EquityValuePerShare != 147.18 {
		t.Fatalf("Custom (no assumptions) = %+v, %v", omitted, err)
	}
	custom, err := client.Dcf.Custom(ctx, dcfDocumentedQuery("BRK.B / Class A"))
	if err != nil || len(custom) != 1 || custom[0].EquityValuePerShare != 147.18 {
		t.Fatalf("Custom (documented) = %+v, %v", custom, err)
	}
	partial, err := client.Dcf.CustomLevered(ctx, NewCustomDcfQuery("AAPL").
		WithRevenueGrowthPct(0).WithTaxRate(-1.25).WithLongTermGrowthRate(4))
	if err != nil || len(partial) != 1 || partial[0].EquityValuePerShare != 140.71 {
		t.Fatalf("CustomLevered (partial) = %+v, %v", partial, err)
	}
	full, err := client.Dcf.CustomLevered(ctx, dcfDocumentedQuery("BRK.B / Class A"))
	if err != nil || len(full) != 1 || full[0].EquityValuePerShare != 140.71 {
		t.Fatalf("CustomLevered (documented) = %+v, %v", full, err)
	}

	requests := rec.all()
	if len(requests) != len(dcfRoutes) {
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

func TestDcfQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, dcfRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	_, err := client.Dcf.Standard(ctx, NewDcfQuery("AAPL,MSFT"))
	typed := assertQuoteError(t, err, CategoryValidation, 0, "")
	if !errors.Is(err, ErrCommaInTicker) || typed.Message != "symbol: "+ErrCommaInTicker.Error() {
		t.Fatalf("comma ticker: error = %v", err)
	}
	_, err = client.Dcf.CustomLevered(ctx, NewCustomDcfQuery(" ").WithBeta(1.2))
	if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrEmptyValue) ||
		typed.Message != "symbol: "+ErrEmptyValue.Error() {
		t.Fatalf("blank ticker: error = %v", err)
	}
	for name, value := range map[string]float64{"nan": math.NaN(), "inf": math.Inf(1), "neg-inf": math.Inf(-1)} {
		_, err = client.Dcf.Custom(ctx, NewCustomDcfQuery("AAPL").WithBeta(value))
		if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrNonFiniteDecimal) ||
			typed.Message != "beta: "+ErrNonFiniteDecimal.Error() {
			t.Fatalf("%s beta: error = %v", name, err)
		}
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	q := NewCustomDcfQuery(" AAPL ")
	if q.Symbol() != " AAPL " || q.Beta() != nil || q.RiskFreeRate() != nil {
		t.Fatalf("getters = %q %v %v", q.Symbol(), q.Beta(), q.RiskFreeRate())
	}
	if beta := q.WithBeta(1.244).Beta(); beta == nil || *beta != 1.244 || q.Beta() != nil {
		t.Fatalf("WithBeta mutated the receiver or lost the value: %v %v", beta, q.Beta())
	}
}

func TestDcfMethodsReportMalformedRootsAsDecodeErrorsPerEndpoint(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`{}`))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	calls := []struct {
		endpoint string
		call     func() error
	}{
		{"discounted-cash-flow", func() error {
			_, err := client.Dcf.Standard(ctx, NewDcfQuery("AAPL"))
			return err
		}},
		{"levered-discounted-cash-flow", func() error {
			_, err := client.Dcf.Levered(ctx, NewDcfQuery("AAPL"))
			return err
		}},
		{"custom-discounted-cash-flow", func() error {
			_, err := client.Dcf.Custom(ctx, NewCustomDcfQuery("AAPL"))
			return err
		}},
		{"custom-levered-discounted-cash-flow", func() error {
			_, err := client.Dcf.CustomLevered(ctx, NewCustomDcfQuery("AAPL"))
			return err
		}},
	}
	for _, tc := range calls {
		_ = assertQuoteError(t, tc.call(), CategoryDecode, http.StatusOK, tc.endpoint)
	}
	if rec.count() != len(calls) {
		t.Fatalf("decode failures were retried: %d requests", rec.count())
	}
}
