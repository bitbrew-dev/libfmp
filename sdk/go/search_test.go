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

// searchRoutes is the exact request-URI table of the six search endpoints,
// copied from the URL assertions in crates/libfmp/tests/search_endpoints.rs.
// Keying on the full request URI lets one path carry its required-only and
// with-setter variants (wire order and escaping matter: spaces and slashes
// are escaped as the Rust encoder does, and the caret of an index ticker too).
var searchRoutes = map[string]string{
	"/router/stable/search-symbol?query=Apple+%2F+Class+A&limit=4294967295&exchange=NASDAQ+Global": "search_symbol.json",
	"/router/stable/search-symbol?query=AAPL":                                                      "search_symbol.json",
	"/router/stable/search-name?query=AA&limit=0&exchange=CRYPTO":                                  "search_name.json",
	"/router/stable/search-cik?cik=0000320193&limit=50":                                            "search_cik.json",
	"/router/stable/search-cusip?cusip=037833100":                                                  "search_cusip.json",
	"/router/stable/search-isin?isin=US0378331005":                                                 "search_isin.json",
	"/router/stable/search-exchange-variants?symbol=%5EVIX":                                        "search_exchange_variants.json",
}

func searchRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := searchRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestSearchMethodsUseExactPathsAndWireParameterOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, searchRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	symbols, err := client.Search.Symbol(ctx, NewSymbolSearchQuery("Apple / Class A").
		WithExchange("NASDAQ Global").WithLimit(math.MaxUint32))
	if err != nil || len(symbols) != 1 || symbols[0].Symbol != "AAPL" {
		t.Fatalf("Symbol = %+v, %v", symbols, err)
	}
	bare, err := client.Search.Symbol(ctx, NewSymbolSearchQuery("AAPL"))
	if err != nil || len(bare) != 1 || bare[0].ExchangeFullName != "NASDAQ Global Select" {
		t.Fatalf("Symbol bare = %+v, %v", bare, err)
	}
	names, err := client.Search.Name(ctx, NewNameSearchQuery("AA").WithLimit(0).WithExchange("CRYPTO"))
	if err != nil || len(names) != 1 || names[0].Symbol != "AAGUSD" {
		t.Fatalf("Name = %+v, %v", names, err)
	}
	ciks, err := client.Search.Cik(ctx, NewCikSearchQuery("0000320193").WithLimit(50))
	if err != nil || len(ciks) != 1 || ciks[0].Cik != "0000320193" {
		t.Fatalf("Cik = %+v, %v", ciks, err)
	}
	cusips, err := client.Search.Cusip(ctx, NewCusipSearchQuery("037833100"))
	if err != nil || len(cusips) != 1 || cusips[0].Symbol != "APC.F" {
		t.Fatalf("Cusip = %+v, %v", cusips, err)
	}
	isins, err := client.Search.Isin(ctx, NewIsinSearchQuery("US0378331005"))
	if err != nil || len(isins) != 1 || isins[0].MarketCap != 4_874_072_686_740 {
		t.Fatalf("Isin = %+v, %v", isins, err)
	}
	variants, err := client.Search.ExchangeVariants(ctx, NewExchangeVariantsQuery("^VIX"))
	if err != nil || len(variants) != 1 || variants[0].ExchangeShortName != "NASDAQ" {
		t.Fatalf("ExchangeVariants = %+v, %v", variants, err)
	}

	requests := rec.all()
	if len(requests) != len(searchRoutes) {
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
// any request: the exchange-variants ticker rejects a comma, the open
// search-term, CIK, CUSIP, ISIN, and exchange-code strings accept one, and all
// reject empty or control text.
func TestSearchQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, searchRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	cases := []struct {
		name   string
		call   func() error
		member string
		reason error
	}{
		{"symbol whitespace query", func() error {
			_, err := client.Search.Symbol(ctx, NewSymbolSearchQuery(" \t"))
			return err
		}, "query", ErrEmptyValue},
		{"symbol control exchange", func() error {
			_, err := client.Search.Symbol(ctx, NewSymbolSearchQuery("AAPL").WithExchange("NAS\nDAQ"))
			return err
		}, "exchange", ErrControlCharacterValue},
		{"name empty query", func() error {
			_, err := client.Search.Name(ctx, NewNameSearchQuery(""))
			return err
		}, "query", ErrEmptyValue},
		{"cik empty", func() error {
			_, err := client.Search.Cik(ctx, NewCikSearchQuery(""))
			return err
		}, "cik", ErrEmptyValue},
		{"cusip control", func() error {
			_, err := client.Search.Cusip(ctx, NewCusipSearchQuery("0378\x0033100"))
			return err
		}, "cusip", ErrControlCharacterValue},
		{"isin whitespace", func() error {
			_, err := client.Search.Isin(ctx, NewIsinSearchQuery("   "))
			return err
		}, "isin", ErrEmptyValue},
		{"variants comma ticker", func() error {
			_, err := client.Search.ExchangeVariants(ctx, NewExchangeVariantsQuery("AAPL,MSFT"))
			return err
		}, "symbol", ErrCommaInTicker},
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

	// A comma passes the open-string rule of SearchTerm, so it reaches the
	// wire (and is rejected there by the route table, not by validation).
	_, err := client.Search.Symbol(ctx, NewSymbolSearchQuery("Apple, Inc"))
	if errors.Is(err, ErrCommaInTicker) || rec.count() != 1 {
		t.Fatalf("a comma in a search term was rejected before the request: %v (%d requests)", err, rec.count())
	}

	// Getters return the values as given, never normalized, and the setters
	// copy the query instead of mutating the receiver.
	base := NewSymbolSearchQuery(" AAPL ")
	set := base.WithExchange(" NASDAQ ").WithLimit(7)
	if base.Query() != " AAPL " || base.Exchange() != nil || base.Limit() != nil ||
		set.Exchange() == nil || *set.Exchange() != " NASDAQ " || set.Limit() == nil || *set.Limit() != 7 {
		t.Fatalf("SymbolSearchQuery setters mutated the receiver or normalized a value: %+v %+v", base, set)
	}
	if q := NewCikSearchQuery("320193"); q.Cik() != "320193" || q.Limit() != nil {
		t.Fatalf("CikSearchQuery getters = %q, %v", q.Cik(), q.Limit())
	}
}

func TestSearchMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[{"symbol":"AAPL","name":"Apple Inc.","currency":"USD"}]`))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Search.Symbol(context.Background(), NewSymbolSearchQuery("AAPL"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "search-symbol")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"exchangeFullName"`) {
		t.Fatalf("cause = %v, want it to name the missing member exchangeFullName", cause)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}
