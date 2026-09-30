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

// screenerRoutes is the exact request-URI table of the company screener,
// copied from the URL assertions in crates/libfmp/tests/screener_endpoint.rs.
// Keying on the full request URI lets the one path carry its no-query,
// all-twenty-filters, zero-and-false, and direct-transport variants (wire
// order and escaping matter: spaces, ampersands, and slashes are escaped as
// the Rust encoder does, and 500.0 or 2.0 are sent as 500 and 2).
var screenerRoutes = map[string]string{
	"/router/stable/company-screener": "company_screener.json",
	"/router/stable/company-screener?marketCapMoreThan=9007199254740993&marketCapLowerThan=18446744073709551615" +
		"&sector=Technology+%26+AI&industry=Consumer+Electronics+%2F+Devices&betaMoreThan=0.5&betaLowerThan=1.5" +
		"&priceMoreThan=10.25&priceLowerThan=500&dividendMoreThan=0.5&dividendLowerThan=2&volumeMoreThan=1000" +
		"&volumeLowerThan=18446744073709551615&exchange=NASDAQ+Global&country=US+%2F+CA&isEtf=false&isFund=false" +
		"&isActivelyTrading=true&page=0&limit=4294967295&includeAllShareClasses=false": "company_screener.json",
	"/router/stable/company-screener?marketCapMoreThan=0&priceMoreThan=0&volumeMoreThan=0&isEtf=false&isFund=false" +
		"&isActivelyTrading=false&page=0&limit=0&includeAllShareClasses=false": "company_screener_empty.json",
	"/router/stable/company-screener?sector=Technology&limit=1000": "company_screener_multiple.json",
}

func screenerRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := screenerRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

// screenerAllFilters mirrors all_filters() in screener_endpoint.rs: every one
// of the twenty optional setters, set in an order that differs from the wire
// order to prove the encoder follows the registry, not the call sequence.
func screenerAllFilters() CompanyScreenerQuery {
	return NewCompanyScreenerQuery().
		WithIncludeAllShareClasses(false).WithLimit(math.MaxUint32).WithPage(0).
		WithIsActivelyTrading(true).WithIsFund(false).WithIsETF(false).
		WithCountry("US / CA").WithExchange("NASDAQ Global").
		WithVolumeLowerThan(math.MaxUint64).WithVolumeMoreThan(1_000).
		WithDividendLowerThan(2.0).WithDividendMoreThan(0.5).
		WithPriceLowerThan(500.0).WithPriceMoreThan(10.25).
		WithBetaLowerThan(1.5).WithBetaMoreThan(0.5).
		WithIndustry("Consumer Electronics / Devices").WithSector("Technology & AI").
		WithMarketCapLowerThan(math.MaxUint64).WithMarketCapMoreThan(9_007_199_254_740_993)
}

func TestScreenerCompaniesUsesExactPathAndWireParameterOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, screenerRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()

	bare, err := client.Screener.Companies(ctx, NewCompanyScreenerQuery())
	if err != nil || len(bare) != 1 || bare[0].Symbol != "AAPL" || bare[0].MarketCap != 4_885_602_246_714 {
		t.Fatalf("Companies = %+v, %v", bare, err)
	}
	full, err := client.Screener.Companies(ctx, screenerAllFilters())
	if err != nil || len(full) != 1 {
		t.Fatalf("Companies all filters = %+v, %v", full, err)
	}
	// Set zero and false values are emitted while absent filters are omitted
	// (zero_and_false_are_emitted_while_absent_filters_are_omitted).
	zero, err := client.Screener.Companies(ctx, NewCompanyScreenerQuery().
		WithMarketCapMoreThan(0).WithPriceMoreThan(0).WithVolumeMoreThan(0).
		WithIsETF(false).WithIsFund(false).WithIsActivelyTrading(false).
		WithPage(0).WithLimit(0).WithIncludeAllShareClasses(false))
	if err != nil || len(zero) != 0 {
		t.Fatalf("Companies zero filters = %+v, %v", zero, err)
	}
	// The direct-transport contract of screener_endpoint.rs.
	direct, err := client.Screener.Companies(ctx, NewCompanyScreenerQuery().WithSector("Technology").WithLimit(1_000))
	if err != nil || len(direct) != 2 || direct[0].Volume != math.MaxUint64 || direct[1].IsFund == nil || !*direct[1].IsFund {
		t.Fatalf("Companies direct = %+v, %v", direct, err)
	}

	requests := rec.all()
	if len(requests) != len(screenerRoutes) {
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

// The open string filters are validated the way the Rust newtypes are and
// the decimal filters the way FiniteDecimal::new is, before any request. The
// integer and boolean filters have no rejection path, as in Rust.
func TestScreenerQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, screenerRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	cases := []struct {
		name   string
		query  CompanyScreenerQuery
		member string
		reason error
	}{
		{"empty sector", NewCompanyScreenerQuery().WithSector(""), "sector", ErrEmptyValue},
		{"control industry", NewCompanyScreenerQuery().WithIndustry("Consumer\nElectronics"), "industry", ErrControlCharacterValue},
		{"whitespace exchange", NewCompanyScreenerQuery().WithExchange(" \t"), "exchange", ErrEmptyValue},
		{"control country", NewCompanyScreenerQuery().WithCountry("US\x00"), "country", ErrControlCharacterValue},
		{"nan beta", NewCompanyScreenerQuery().WithBetaMoreThan(math.NaN()), "betaMoreThan", ErrNonFiniteDecimal},
		{"infinite price", NewCompanyScreenerQuery().WithPriceLowerThan(math.Inf(1)), "priceLowerThan", ErrNonFiniteDecimal},
		{"negative infinite dividend", NewCompanyScreenerQuery().WithDividendMoreThan(math.Inf(-1)), "dividendMoreThan", ErrNonFiniteDecimal},
		// The first invalid filter in wire order is reported, as the Rust
		// constructors fail at the first invalid value.
		{"first in wire order", NewCompanyScreenerQuery().WithCountry("").WithSector(" "), "sector", ErrEmptyValue},
	}
	for _, tc := range cases {
		_, err := client.Screener.Companies(ctx, tc.query)
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, tc.reason) || typed.Message != tc.member+": "+tc.reason.Error() {
			t.Fatalf("%s: error = %v, want %s: %v", tc.name, err, tc.member, tc.reason)
		}
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	// Commas are allowed in the open string filters, as the Rust newtypes
	// accept them; only tickers reject them.
	if _, err := NewCompanyScreenerQuery().WithSector("Technology, AI").WithCountry("US,CA").params(); err != nil {
		t.Fatalf("comma in an open string filter was rejected: %v", err)
	}

	// Getters return the values as given, never normalized, unset filters
	// read as nil, and the setters copy the query instead of mutating the
	// receiver (fluent_query_exposes_all_twenty_private_filters).
	base := NewCompanyScreenerQuery()
	set := screenerAllFilters()
	if base.Sector() != nil || base.MarketCapMoreThan() != nil || base.IsETF() != nil || base.Limit() != nil ||
		base.BetaMoreThan() != nil {
		t.Fatalf("NewCompanyScreenerQuery has a set filter: %+v", base)
	}
	if *set.MarketCapMoreThan() != 9_007_199_254_740_993 || *set.MarketCapLowerThan() != math.MaxUint64 ||
		*set.Sector() != "Technology & AI" || *set.Industry() != "Consumer Electronics / Devices" ||
		*set.BetaMoreThan() != 0.5 || *set.BetaLowerThan() != 1.5 || *set.PriceMoreThan() != 10.25 ||
		*set.PriceLowerThan() != 500 || *set.DividendMoreThan() != 0.5 || *set.DividendLowerThan() != 2 ||
		*set.VolumeMoreThan() != 1_000 || *set.VolumeLowerThan() != math.MaxUint64 ||
		*set.Exchange() != "NASDAQ Global" || *set.Country() != "US / CA" ||
		*set.IsETF() || *set.IsFund() || !*set.IsActivelyTrading() ||
		*set.Page() != 0 || *set.Limit() != math.MaxUint32 || *set.IncludeAllShareClasses() {
		t.Fatalf("CompanyScreenerQuery getters do not return the twenty filters as given: %+v", set)
	}
	spaced := base.WithSector(" Technology ")
	if base.Sector() != nil || spaced.Sector() == nil || *spaced.Sector() != " Technology " {
		t.Fatalf("WithSector mutated the receiver or normalized the value: %+v %+v", base, spaced)
	}
}

func TestScreenerCompaniesReportsMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[{"symbol":"AAPL","companyName":"Apple Inc.","marketCap":1}]`))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Screener.Companies(context.Background(), NewCompanyScreenerQuery())
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "company-screener")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"sector"`) {
		t.Fatalf("cause = %v, want it to name the missing member sector", cause)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}
