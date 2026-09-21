package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// fundsRoutes is the exact path and query table of the nine funds endpoints,
// one per query shape: a required symbol (six methods, one with a space and a
// slash), symbol plus year and quarter with the optional cik set, a name
// search term carrying a comma, and symbol plus the optional cik set. The
// wire text is copied from crates/libfmp/tests/funds_*_endpoints.rs and
// fund_disclosure_endpoints.rs.
var fundsRoutes = map[string]struct {
	query   string
	fixture string
}{
	"/router/stable/etf/holdings":                    {"symbol=BRK.B+%2F+Class+A", "etf_fund_holdings.json"},
	"/router/stable/etf/info":                        {"symbol=SPY", "etf_fund_info.json"},
	"/router/stable/etf/country-weightings":          {"symbol=000089.SZ", "etf_country_weightings.json"},
	"/router/stable/etf/asset-exposure":              {"symbol=AAPL", "etf_asset_exposure.json"},
	"/router/stable/etf/sector-weightings":           {"symbol=ZWT-T.TO", "etf_sector_weightings.json"},
	"/router/stable/funds/disclosure-holders-latest": {"symbol=AAPL", "latest_fund_disclosure_holders.json"},
	"/router/stable/funds/disclosure":                {"symbol=VWO&year=2023&quarter=4&cik=0000857489", "fund_disclosures.json"},
	"/router/stable/funds/disclosure-holders-search": {"name=Federated+Hermes+Government+Income+Securities%2C+Inc.", "fund_disclosure_holder_search.json"},
	"/router/stable/funds/disclosure-dates":          {"symbol=VWO&cik=0000036405", "fund_disclosure_dates.json"},
}

func fundsRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		route, ok := fundsRoutes[r.URL.Path]
		if !ok || r.URL.RawQuery != route.query || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s?%s", r.URL.Path, r.URL.RawQuery)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, route.fixture))
	}
}

func TestFundsMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, fundsRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	ns := client.Funds

	holdings, err := ns.EtfHoldings(ctx, NewEtfHoldingsQuery("BRK.B / Class A"))
	if err != nil || len(holdings) != 1 || holdings[0].MarketValue != 61_679_458_958.0 {
		t.Fatalf("EtfHoldings = %+v, %v", holdings, err)
	}
	info, err := ns.EtfInfo(ctx, NewEtfInfoQuery("SPY"))
	if err != nil || len(info) != 1 || info[0].AssetsUnderManagement != 777_349_860_000 || len(info[0].SectorsList) != 3 {
		t.Fatalf("EtfInfo = %+v, %v", info, err)
	}
	countries, err := ns.EtfCountryWeightings(ctx, NewEtfCountryWeightingsQuery("000089.SZ"))
	if err != nil || len(countries) != 1 || countries[0].WeightPercentage != "97.26%" {
		t.Fatalf("EtfCountryWeightings = %+v, %v", countries, err)
	}
	assets, err := ns.EtfAssetExposure(ctx, NewEtfAssetExposureQuery("AAPL"))
	if err != nil || len(assets) != 1 || assets[0].SharesNumber != 42_372 {
		t.Fatalf("EtfAssetExposure = %+v, %v", assets, err)
	}
	sectors, err := ns.EtfSectorWeightings(ctx, NewEtfSectorWeightingsQuery("ZWT-T.TO"))
	if err != nil || len(sectors) != 1 || sectors[0].Sector != "Basic Materials" {
		t.Fatalf("EtfSectorWeightings = %+v, %v", sectors, err)
	}
	holders, err := ns.LatestFundDisclosureHolders(ctx, NewLatestFundDisclosureHoldersQuery("AAPL"))
	if err != nil || len(holders) != 1 || holders[0].Change != -316_881 {
		t.Fatalf("LatestFundDisclosureHolders = %+v, %v", holders, err)
	}
	disclosures, err := ns.FundDisclosures(ctx, NewFundDisclosureQuery("VWO", 2023, QuarterQ4).WithCik("0000857489"))
	if err != nil || len(disclosures) != 1 || disclosures[0].Cusip != "N/A" || disclosures[0].CurrencyCode != "CNY" {
		t.Fatalf("FundDisclosures = %+v, %v", disclosures, err)
	}
	results, err := ns.SearchFundDisclosureHolders(ctx,
		NewFundDisclosureHolderSearchQuery("Federated Hermes Government Income Securities, Inc."))
	if err != nil || len(results) != 1 || results[0].EntityOrgType != "30" {
		t.Fatalf("SearchFundDisclosureHolders = %+v, %v", results, err)
	}
	dates, err := ns.FundDisclosureDates(ctx, NewFundDisclosureDatesQuery("VWO").WithCik("0000036405"))
	if err != nil || len(dates) != 1 || dates[0].Year != 2026 || dates[0].Quarter != 2 {
		t.Fatalf("FundDisclosureDates = %+v, %v", dates, err)
	}

	requests := rec.all()
	if len(requests) != len(fundsRoutes) {
		t.Fatalf("requests = %d, want exactly one per call", len(requests))
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

// Mirrors optional_ciks_are_omitted_and_direct_auth_preserves_all_exact_urls:
// an unset cik is omitted from the wire entirely.
func TestFundsOptionalCikIsOmittedWhenUnset(t *testing.T) {
	t.Parallel()
	routes := map[string]string{
		"/router/stable/funds/disclosure":       "symbol=VWO&year=2023&quarter=4",
		"/router/stable/funds/disclosure-dates": "symbol=VWO",
	}
	server, rec := newServer(t, func(w http.ResponseWriter, r *http.Request) {
		if routes[r.URL.Path] != r.URL.RawQuery {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s?%s", r.URL.Path, r.URL.RawQuery)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte("[]"))
	})
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()

	disclosures, err := client.Funds.FundDisclosures(ctx, NewFundDisclosureQuery("VWO", 2023, QuarterQ4))
	if err != nil || len(disclosures) != 0 {
		t.Fatalf("FundDisclosures = %+v, %v", disclosures, err)
	}
	dates, err := client.Funds.FundDisclosureDates(ctx, NewFundDisclosureDatesQuery("VWO"))
	if err != nil || len(dates) != 0 {
		t.Fatalf("FundDisclosureDates = %+v, %v", dates, err)
	}
	if rec.count() != 2 {
		t.Fatalf("requests = %d, want 2", rec.count())
	}
}

func TestFundsQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, fundsRouter(t))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))
	ctx := context.Background()
	ns := client.Funds

	_, err := ns.EtfHoldings(ctx, NewEtfHoldingsQuery(" "))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrEmptyValue) ||
		typed.Message != "symbol: "+ErrEmptyValue.Error() {
		t.Fatalf("blank symbol: error = %v", err)
	}
	_, err = ns.EtfAssetExposure(ctx, NewEtfAssetExposureQuery("AAPL,MSFT"))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrCommaInTicker) ||
		typed.Message != "symbol: "+ErrCommaInTicker.Error() {
		t.Fatalf("comma ticker: error = %v", err)
	}
	for _, quarter := range []Quarter{"5", "Q4", "", "4 "} {
		_, err := ns.FundDisclosures(ctx, NewFundDisclosureQuery("VWO", 2023, quarter))
		typed := assertQuoteError(t, err, CategoryValidation, 0, "")
		if !errors.Is(err, ErrUnknownWireValue) ||
			typed.Message != "quarter: "+ErrUnknownWireValue.Error()+", expected one of 1, 2, 3, 4" {
			t.Fatalf("quarter %q: error = %v", quarter, err)
		}
	}
	_, err = ns.FundDisclosureDates(ctx, NewFundDisclosureDatesQuery("VWO").WithCik(" "))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrEmptyValue) ||
		typed.Message != "cik: "+ErrEmptyValue.Error() {
		t.Fatalf("blank optional cik: error = %v", err)
	}
	_, err = ns.SearchFundDisclosureHolders(ctx, NewFundDisclosureHolderSearchQuery("Vanguard\x00"))
	if typed := assertQuoteError(t, err, CategoryValidation, 0, ""); !errors.Is(err, ErrControlCharacterValue) ||
		typed.Message != "name: "+ErrControlCharacterValue.Error() {
		t.Fatalf("control character name: error = %v", err)
	}
	if rec.count() != 0 {
		t.Fatalf("validation failures sent %d requests", rec.count())
	}

	q := NewFundDisclosureQuery(" VWO ", 2023, QuarterQ4)
	if q.Symbol() != " VWO " || q.Year() != 2023 || q.Quarter() != QuarterQ4 || q.Cik() != nil {
		t.Fatalf("getters = %q %d %q %v", q.Symbol(), q.Year(), q.Quarter(), q.Cik())
	}
	if cik := q.WithCik("0000857489").Cik(); cik == nil || *cik != "0000857489" || q.Cik() != nil {
		t.Fatalf("WithCik mutated the receiver or lost the value: %v %v", cik, q.Cik())
	}
	if name := NewFundDisclosureHolderSearchQuery("Fund, Inc.").Name(); name != "Fund, Inc." {
		t.Fatalf("Name() = %q, want the comma preserved", name)
	}
}

func TestFundsMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[{"country":"United States"}]`))
	client := newClient(t, server, WithAuthentication(FmpHeader("route-secret")))

	_, err := client.Funds.EtfCountryWeightings(context.Background(), NewEtfCountryWeightingsQuery("SPY"))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "etf/country-weightings")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"weightPercentage"`) {
		t.Fatalf("cause = %v, want it to name the missing member weightPercentage", cause)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}
