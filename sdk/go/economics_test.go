package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// economicsRoutes is the exact path and query table of the four economics
// endpoints, keyed on path plus RawQuery so the independent omission of each
// optional date can share the table with the full query. Wire text copied
// from crates/libfmp/tests/economics_rates_indicators_endpoints.rs and
// economics_calendar_risk_endpoints.rs.
var economicsRoutes = map[string]string{
	"/router/stable/treasury-rates?from=2026-01-27&to=2026-04-27":               "treasury_rates.json",
	"/router/stable/treasury-rates?to=2026-04-27":                               "treasury_rates.json",
	"/router/stable/economic-indicators?name=GDP&from=2026-01-27&to=2026-04-27": "economic_indicators.json",
	"/router/stable/economic-indicators?name=futureProviderIndicator":           "economic_indicators.json",
	"/router/stable/economic-calendar?country=US&from=2026-01-27&to=2026-04-27": "economic_calendar.json",
	"/router/stable/economic-calendar?country=US":                               "economic_calendar.json",
	"/router/stable/economic-calendar":                                          "economic_calendar.json",
	"/router/stable/market-risk-premium":                                        "market_risk_premium.json",
}

func economicsRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		key := r.URL.Path
		if r.URL.RawQuery != "" {
			key += "?" + r.URL.RawQuery
		}
		fixture, ok := economicsRoutes[key]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", key)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestEconomicsMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, economicsRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	from, to := mustParseDate(t, "2026-01-27"), mustParseDate(t, "2026-04-27")

	rates, err := client.Economics.TreasuryRates(ctx, NewTreasuryRatesQuery().WithFrom(from).WithTo(to))
	if err != nil || len(rates) != 1 || rates[0].Year30 != 5.2 {
		t.Fatalf("TreasuryRates = %+v, %v", rates, err)
	}
	if rates, err = client.Economics.TreasuryRates(ctx, NewTreasuryRatesQuery().WithTo(to)); err != nil || len(rates) != 1 {
		t.Fatalf("TreasuryRates with only to = %+v, %v", rates, err)
	}
	indicators, err := client.Economics.Indicators(ctx,
		NewEconomicIndicatorsQuery(EconomicIndicatorGdp).WithFrom(from).WithTo(to))
	if err != nil || len(indicators) != 1 || indicators[0].Value != 31_422.526 {
		t.Fatalf("Indicators = %+v, %v", indicators, err)
	}
	if indicators, err = client.Economics.Indicators(ctx,
		NewEconomicIndicatorsQuery("futureProviderIndicator")); err != nil || len(indicators) != 1 {
		t.Fatalf("Indicators with an open name = %+v, %v", indicators, err)
	}
	events, err := client.Economics.Calendar(ctx, NewEconomicCalendarQuery().WithCountry("US").WithFrom(from).WithTo(to))
	if err != nil || len(events) != 1 || events[0].Actual != 13.6 {
		t.Fatalf("Calendar = %+v, %v", events, err)
	}
	if events, err = client.Economics.Calendar(ctx, NewEconomicCalendarQuery().WithCountry("US")); err != nil || len(events) != 1 {
		t.Fatalf("Calendar with only country = %+v, %v", events, err)
	}
	if events, err = client.Economics.Calendar(ctx, NewEconomicCalendarQuery()); err != nil || len(events) != 1 {
		t.Fatalf("Calendar without filters = %+v, %v", events, err)
	}
	premiums, err := client.Economics.MarketRiskPremium(ctx)
	if err != nil || len(premiums) != 1 || premiums[0].Country != "Zimbabwe" {
		t.Fatalf("MarketRiskPremium = %+v, %v", premiums, err)
	}

	requests := rec.all()
	if len(requests) != len(economicsRoutes) {
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

func TestEconomicsQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, economicsRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()

	_, err := client.Economics.Indicators(ctx, NewEconomicIndicatorsQuery(" "))
	typed := assertQuoteError(t, err, CategoryValidation, 0, "")
	if !errors.Is(err, ErrEmptyValue) || typed.Message != "name: "+ErrEmptyValue.Error() {
		t.Fatalf("blank indicator: error = %v", err)
	}
	_, err = client.Economics.Indicators(ctx, NewEconomicIndicatorsQuery("GDP\t"))
	if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrControlCharacterValue) ||
		typed.Message != "name: "+ErrControlCharacterValue.Error() {
		t.Fatalf("control character in indicator: error = %v", err)
	}
	_, err = client.Economics.Calendar(ctx, NewEconomicCalendarQuery().WithCountry(""))
	if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrEmptyValue) ||
		typed.Message != "country: "+ErrEmptyValue.Error() {
		t.Fatalf("empty country: error = %v", err)
	}
	_, err = client.Economics.TreasuryRates(ctx, NewTreasuryRatesQuery().WithFrom(Date{}))
	if typed = assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrZeroTemporalValue) ||
		typed.Message != "from: "+ErrZeroTemporalValue.Error() {
		t.Fatalf("zero date: error = %v", err)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	q := NewEconomicIndicatorsQuery(EconomicIndicatorCpi)
	if q.Name() != EconomicIndicatorCpi || q.From() != nil || q.To() != nil {
		t.Fatalf("getters = %q %v %v", q.Name(), q.From(), q.To())
	}
	to := mustParseDate(t, "2026-04-27")
	if got := q.WithTo(to).To(); got == nil || *got != to || q.To() != nil {
		t.Fatalf("WithTo mutated the receiver or lost the value: %v %v", got, q.To())
	}
	calendar := NewEconomicCalendarQuery().WithCountry("US")
	if country := calendar.Country(); country == nil || *country != "US" || calendar.From() != nil {
		t.Fatalf("calendar getters = %v %v", country, calendar.From())
	}
}

func TestEconomicsMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[{"country":"Zimbabwe","continent":"Africa","countryRiskPremium":11.66}]`))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))

	_, err := client.Economics.MarketRiskPremium(context.Background())
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "market-risk-premium")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"totalEquityRiskPremium"`) {
		t.Fatalf("cause = %v, want it to name the missing member totalEquityRiskPremium", cause)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}
