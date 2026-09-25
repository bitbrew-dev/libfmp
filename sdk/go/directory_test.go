package fmp

import (
	"context"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// directoryRoutes is the exact request-URI table of the eleven directory
// endpoints, copied from the URL assertions in
// crates/libfmp/tests/directory_endpoints.rs and
// directory_taxonomy_endpoints.rs. Keying on the full request URI lets one
// path carry its no-query and with-setter variants (wire order matters).
var directoryRoutes = map[string]string{
	"/router/stable/stock-list":                            "directory_company_symbols.json",
	"/router/stable/financial-statement-symbol-list":       "directory_financial_statement_symbols.json",
	"/router/stable/cik-list":                              "directory_cik_list.json",
	"/router/stable/cik-list?page=0&limit=10001":           "directory_cik_list.json",
	"/router/stable/symbol-change":                         "directory_symbol_changes.json",
	"/router/stable/symbol-change?invalid=false&limit=100": "directory_symbol_changes.json",
	"/router/stable/etf-list":                              "directory_etf_symbols.json",
	"/router/stable/actively-trading-list":                 "directory_actively_trading.json",
	"/router/stable/earnings-transcript-list":              "directory_earnings_transcript_list.json",
	"/router/stable/available-exchanges":                   "directory_available_exchanges.json",
	"/router/stable/available-exchanges?extended=false":    "directory_available_exchanges.json",
	"/router/stable/available-exchanges?extended=true":     "directory_available_exchanges.json",
	"/router/stable/available-sectors":                     "directory_available_sectors.json",
	"/router/stable/available-industries":                  "directory_available_industries.json",
	"/router/stable/available-countries":                   "directory_available_countries.json",
}

func directoryRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := directoryRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestDirectoryMethodsUseExactPathsAndWireParameterOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, directoryRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	companies, err := client.Directory.CompanySymbols(ctx)
	if err != nil || len(companies) != 1 || companies[0].Symbol != "URBANCO.BO" {
		t.Fatalf("CompanySymbols = %+v, %v", companies, err)
	}
	financials, err := client.Directory.FinancialStatementSymbols(ctx)
	if err != nil || len(financials) != 1 || financials[0].TradingCurrency != "CAD" {
		t.Fatalf("FinancialStatementSymbols = %+v, %v", financials, err)
	}
	ciks, err := client.Directory.CikList(ctx, NewCikListQuery())
	if err != nil || len(ciks) != 1 || ciks[0].Cik != "0002137358" {
		t.Fatalf("CikList = %+v, %v", ciks, err)
	}
	paged, err := client.Directory.CikList(ctx, NewCikListQuery().WithLimit(10_001).WithPage(0))
	if err != nil || len(paged) != 1 {
		t.Fatalf("CikList paged = %+v, %v", paged, err)
	}
	changes, err := client.Directory.SymbolChanges(ctx, NewSymbolChangesQuery())
	if err != nil || len(changes) != 1 || changes[0].NewSymbol != "YARW" {
		t.Fatalf("SymbolChanges = %+v, %v", changes, err)
	}
	flagged, err := client.Directory.SymbolChanges(ctx, NewSymbolChangesQuery().WithLimit(100).WithInvalid(false))
	if err != nil || len(flagged) != 1 {
		t.Fatalf("SymbolChanges flagged = %+v, %v", flagged, err)
	}
	etfs, err := client.Directory.EtfSymbols(ctx)
	if err != nil || len(etfs) != 1 || etfs[0].Symbol != "P60.SI" {
		t.Fatalf("EtfSymbols = %+v, %v", etfs, err)
	}
	active, err := client.Directory.ActivelyTrading(ctx)
	if err != nil || len(active) != 1 || active[0].Name != "Urban Company Limited" {
		t.Fatalf("ActivelyTrading = %+v, %v", active, err)
	}
	transcripts, err := client.Directory.EarningsTranscriptList(ctx)
	if err != nil || len(transcripts) != 1 || transcripts[0].NoOfTranscripts != "6" {
		t.Fatalf("EarningsTranscriptList = %+v, %v", transcripts, err)
	}
	exchanges, err := client.Directory.AvailableExchanges(ctx, NewAvailableExchangesQuery())
	if err != nil || len(exchanges) != 1 || exchanges[0].Exchange != "AMEX" {
		t.Fatalf("AvailableExchanges = %+v, %v", exchanges, err)
	}
	for _, extended := range []bool{false, true} {
		rows, err := client.Directory.AvailableExchanges(ctx, NewAvailableExchangesQuery().WithExtended(extended))
		if err != nil || len(rows) != 1 {
			t.Fatalf("AvailableExchanges extended=%v = %+v, %v", extended, rows, err)
		}
	}
	sectors, err := client.Directory.AvailableSectors(ctx)
	if err != nil || len(sectors) != 1 || sectors[0].Sector != "Basic Materials" {
		t.Fatalf("AvailableSectors = %+v, %v", sectors, err)
	}
	industries, err := client.Directory.AvailableIndustries(ctx)
	if err != nil || len(industries) != 1 || industries[0].Industry != "Steel" {
		t.Fatalf("AvailableIndustries = %+v, %v", industries, err)
	}
	countries, err := client.Directory.AvailableCountries(ctx)
	if err != nil || len(countries) != 1 || countries[0].Country != "FK" {
		t.Fatalf("AvailableCountries = %+v, %v", countries, err)
	}

	requests := rec.all()
	if len(requests) != len(directoryRoutes) {
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

// Every directory query is optional-only and built from helpers that cannot
// fail (uint32Param, boolParam), so there is no validation case to prove;
// the value semantics of the setters are what remains to check.
func TestDirectoryQuerySettersReturnCopiesAndKeepGettersNil(t *testing.T) {
	t.Parallel()
	cik := NewCikListQuery()
	if cik.Page() != nil || cik.Limit() != nil {
		t.Fatalf("new CikListQuery getters = %v %v, want nil", cik.Page(), cik.Limit())
	}
	if paged := cik.WithPage(3).WithLimit(7); paged.Page() == nil || *paged.Page() != 3 ||
		paged.Limit() == nil || *paged.Limit() != 7 || cik.Page() != nil || cik.Limit() != nil {
		t.Fatalf("CikListQuery setters mutated the receiver or lost a value: %+v %+v", paged, cik)
	}
	changes := NewSymbolChangesQuery()
	if changes.Invalid() != nil || changes.Limit() != nil {
		t.Fatalf("new SymbolChangesQuery getters = %v %v, want nil", changes.Invalid(), changes.Limit())
	}
	if flagged := changes.WithInvalid(true); flagged.Invalid() == nil || !*flagged.Invalid() || changes.Invalid() != nil {
		t.Fatalf("SymbolChangesQuery.WithInvalid mutated the receiver or lost the value: %+v %+v", flagged, changes)
	}
	exchanges := NewAvailableExchangesQuery()
	if exchanges.Extended() != nil {
		t.Fatalf("new AvailableExchangesQuery.Extended() = %v, want nil", exchanges.Extended())
	}
	if off := exchanges.WithExtended(false); off.Extended() == nil || *off.Extended() || exchanges.Extended() != nil {
		t.Fatalf("AvailableExchangesQuery.WithExtended mutated the receiver or lost the value: %+v %+v", off, exchanges)
	}
}

func TestDirectoryMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[{"exchange":"AMEX","name":"New York Stock Exchange Arca","countryName":"United States of America"}]`))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Directory.AvailableExchanges(context.Background(), NewAvailableExchangesQuery())
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "available-exchanges")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"countryCode"`) {
		t.Fatalf("cause = %v, want it to name the missing member countryCode", cause)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}
