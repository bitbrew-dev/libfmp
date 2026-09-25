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

// insiderTradingRoutes is the exact request-URI table of the six
// insider_trading endpoints, copied from the URL assertions in
// crates/libfmp/tests/insider_trading_*_endpoint{,s}.rs. Keying on the full
// request URI lets one path carry its no-query and with-setter variants (wire
// order and escaping matter: spaces, slashes, and commas are escaped as the
// Rust encoder does).
var insiderTradingRoutes = map[string]string{
	"/router/stable/insider-trading/latest":                                         "latest_insider_trades.json",
	"/router/stable/insider-trading/latest?date=2026-01-27&page=0&limit=4294967295": "latest_insider_trades.json",
	"/router/stable/insider-trading/search":                                         "searched_insider_trades.json",
	"/router/stable/insider-trading/search?symbol=BRK.B+%2F+Class+A&page=4294967295&limit=0" +
		"&reportingCik=0001496686&companyCik=0000320193&transactionType=S-Sale+%2F+future": "searched_insider_trades.json",
	"/router/stable/insider-trading/reporting-name?name=Zuckerberg%2C+Mark+%2F+Meta":               "insider_reporting_names.json",
	"/router/stable/insider-trading-transaction-type":                                              "insider_transaction_types.json",
	"/router/stable/insider-trading/statistics?symbol=BRK.B+%2F+Class+A":                           "insider_trade_statistics.json",
	"/router/stable/acquisition-of-beneficial-ownership?symbol=AAPL":                               "beneficial_ownership_acquisitions.json",
	"/router/stable/acquisition-of-beneficial-ownership?symbol=BRK.B+%2F+Class+A&limit=4294967295": "beneficial_ownership_acquisitions.json",
}

func insiderTradingRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := insiderTradingRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestInsiderTradingMethodsUseExactPathsAndWireParameterOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, insiderTradingRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	latest, err := client.InsiderTrading.LatestTrades(ctx, NewLatestInsiderTradesQuery())
	if err != nil || len(latest) != 1 || latest[0].Symbol != "TRMK" {
		t.Fatalf("LatestTrades = %+v, %v", latest, err)
	}
	filtered, err := client.InsiderTrading.LatestTrades(ctx, NewLatestInsiderTradesQuery().
		WithLimit(math.MaxUint32).WithPage(0).WithDate(mustParseDate(t, "2026-01-27")))
	if err != nil || len(filtered) != 1 {
		t.Fatalf("LatestTrades filtered = %+v, %v", filtered, err)
	}
	searched, err := client.InsiderTrading.SearchTrades(ctx, NewInsiderTradesSearchQuery())
	if err != nil || len(searched) != 1 || searched[0].ReportingCIK != "0001661867" {
		t.Fatalf("SearchTrades = %+v, %v", searched, err)
	}
	full, err := client.InsiderTrading.SearchTrades(ctx, NewInsiderTradesSearchQuery().
		WithTransactionType("S-Sale / future").WithCompanyCIK("0000320193").WithReportingCIK("0001496686").
		WithLimit(0).WithPage(math.MaxUint32).WithSymbol("BRK.B / Class A"))
	if err != nil || len(full) != 1 {
		t.Fatalf("SearchTrades full = %+v, %v", full, err)
	}
	names, err := client.InsiderTrading.SearchReportingNames(ctx, NewInsiderReportingNameSearchQuery("Zuckerberg, Mark / Meta"))
	if err != nil || len(names) != 1 || names[0].ReportingName != "Zuckerberg Mark" {
		t.Fatalf("SearchReportingNames = %+v, %v", names, err)
	}
	types, err := client.InsiderTrading.TransactionTypes(ctx)
	if err != nil || len(types) != 1 || types[0].TransactionType != "A-Award" {
		t.Fatalf("TransactionTypes = %+v, %v", types, err)
	}
	statistics, err := client.InsiderTrading.TradeStatistics(ctx, NewInsiderTradeStatisticsQuery("BRK.B / Class A"))
	if err != nil || len(statistics) != 1 || statistics[0].Quarter != 2 {
		t.Fatalf("TradeStatistics = %+v, %v", statistics, err)
	}
	ownership, err := client.InsiderTrading.BeneficialOwnershipAcquisitions(ctx, NewBeneficialOwnershipAcquisitionsQuery("AAPL"))
	if err != nil || len(ownership) != 1 || ownership[0].CUSIP != "037833100" {
		t.Fatalf("BeneficialOwnershipAcquisitions = %+v, %v", ownership, err)
	}
	limited, err := client.InsiderTrading.BeneficialOwnershipAcquisitions(ctx,
		NewBeneficialOwnershipAcquisitionsQuery("BRK.B / Class A").WithLimit(math.MaxUint32))
	if err != nil || len(limited) != 1 {
		t.Fatalf("BeneficialOwnershipAcquisitions limited = %+v, %v", limited, err)
	}

	requests := rec.all()
	if len(requests) != len(insiderTradingRoutes) {
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

// Every string argument is validated the way the Rust newtypes are, before
// any request: tickers reject a comma, the open CIK, search-term, and
// transaction-type strings accept one, and all reject empty or control text.
func TestInsiderTradingQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, insiderTradingRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	cases := []struct {
		name   string
		call   func() error
		member string
		reason error
	}{
		{"statistics whitespace ticker", func() error {
			_, err := client.InsiderTrading.TradeStatistics(ctx, NewInsiderTradeStatisticsQuery(" \t"))
			return err
		}, "symbol", ErrEmptyValue},
		{"ownership comma ticker", func() error {
			_, err := client.InsiderTrading.BeneficialOwnershipAcquisitions(ctx, NewBeneficialOwnershipAcquisitionsQuery("AAPL,MSFT"))
			return err
		}, "symbol", ErrCommaInTicker},
		{"search comma ticker", func() error {
			_, err := client.InsiderTrading.SearchTrades(ctx, NewInsiderTradesSearchQuery().WithSymbol("AAPL,MSFT"))
			return err
		}, "symbol", ErrCommaInTicker},
		{"search empty reporting cik", func() error {
			_, err := client.InsiderTrading.SearchTrades(ctx, NewInsiderTradesSearchQuery().WithReportingCIK(""))
			return err
		}, "reportingCik", ErrEmptyValue},
		{"search control transaction type", func() error {
			_, err := client.InsiderTrading.SearchTrades(ctx, NewInsiderTradesSearchQuery().WithTransactionType("S-\nSale"))
			return err
		}, "transactionType", ErrControlCharacterValue},
		{"reporting name empty", func() error {
			_, err := client.InsiderTrading.SearchReportingNames(ctx, NewInsiderReportingNameSearchQuery(""))
			return err
		}, "name", ErrEmptyValue},
		{"latest zero date", func() error {
			_, err := client.InsiderTrading.LatestTrades(ctx, NewLatestInsiderTradesQuery().WithDate(Date{}))
			return err
		}, "date", ErrZeroTemporalValue},
	}
	for _, tc := range cases {
		err := tc.call()
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != tc.member+": "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want %s: %v", tc.name, err, tc.member, tc.reason)
		}
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	// Getters return the values as given, never normalized, and the setters
	// copy the query instead of mutating the receiver.
	base := NewInsiderTradesSearchQuery()
	set := base.WithSymbol(" AAPL ").WithTransactionType("S-Sale / future")
	if base.Symbol() != nil || base.TransactionType() != nil || set.Symbol() == nil || *set.Symbol() != " AAPL " ||
		set.TransactionType() == nil || *set.TransactionType() != "S-Sale / future" {
		t.Fatalf("InsiderTradesSearchQuery setters mutated the receiver or normalized a value: %+v %+v", base, set)
	}
	if q := NewInsiderReportingNameSearchQuery("  Zuckerberg, Mark  "); q.Name() != "  Zuckerberg, Mark  " {
		t.Fatalf("Name() normalized the search term: %q", q.Name())
	}
}

func TestInsiderTradingMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[{"symbol":"AAPL","cik":"0000320193","year":2026,"quarter":2}]`))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.InsiderTrading.TradeStatistics(context.Background(), NewInsiderTradeStatisticsQuery("AAPL"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "insider-trading/statistics")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"acquiredTransactions"`) {
		t.Fatalf("cause = %v, want it to name the missing member acquiredTransactions", cause)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}
