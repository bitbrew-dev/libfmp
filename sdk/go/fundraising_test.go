package fmp

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"testing"
)

// fundraisingRoutes is the exact request-URI table of the six fundraising
// endpoints, copied from the URL assertions in
// crates/libfmp/tests/fundraising_*_endpoints.rs. The latest feeds appear
// with several queries because every parameter is independently optional.
var fundraisingRoutes = map[string]string{
	"/router/stable/crowdfunding-offerings-latest":                       "crowdfunding_offerings_latest.json",
	"/router/stable/crowdfunding-offerings-latest?page=0&limit=100":      "crowdfunding_offerings_latest.json",
	"/router/stable/crowdfunding-offerings-latest?limit=0":               "crowdfunding_offerings_latest.json",
	"/router/stable/crowdfunding-offerings?cik=0001916078":               "crowdfunding_offerings_by_cik.json",
	"/router/stable/crowdfunding-offerings-search?name=NJOY+%2F+Class+A": "crowdfunding_offerings_search.json",
	"/router/stable/fundraising-search?name=NJOY+%2F+Class+A":            "fundraising_search.json",
	"/router/stable/fundraising-latest?page=0&limit=10&cik=0002013736":   "fundraising_latest.json",
	"/router/stable/fundraising-latest?cik=0002013736":                   "fundraising_latest.json",
	"/router/stable/fundraising-latest?page=4294967295&limit=0":          "fundraising_latest.json",
	"/router/stable/fundraising?cik=0001547416":                          "fundraising_by_cik.json",
}

func fundraisingRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := fundraisingRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestFundraisingMethodsUseExactPathsAndWireOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, fundraisingRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	fund := client.Fundraising

	latest, err := fund.LatestCrowdfundingOfferings(ctx, NewLatestCrowdfundingOfferingsQuery())
	if err != nil || len(latest) != 1 || latest[0].CIK != "0001621902" {
		t.Fatalf("LatestCrowdfundingOfferings = %+v, %v", latest, err)
	}
	if _, err := fund.LatestCrowdfundingOfferings(ctx, NewLatestCrowdfundingOfferingsQuery().WithPage(0).WithLimit(100)); err != nil {
		t.Fatalf("LatestCrowdfundingOfferings paged: %v", err)
	}
	if _, err := fund.LatestCrowdfundingOfferings(ctx, NewLatestCrowdfundingOfferingsQuery().WithLimit(0)); err != nil {
		t.Fatalf("LatestCrowdfundingOfferings limit only: %v", err)
	}
	byCIK, err := fund.CrowdfundingOfferingsByCIK(ctx, NewOfferingByCIKQuery("0001916078"))
	if err != nil || len(byCIK) != 1 || string(byCIK[0].OfferingPrice) != "2" {
		t.Fatalf("CrowdfundingOfferingsByCIK = %+v, %v", byCIK, err)
	}
	search, err := fund.SearchCrowdfundingOfferings(ctx, NewOfferingSearchQuery("NJOY / Class A"))
	if err != nil || len(search) != 1 || search[0].CIK != "0001912939" {
		t.Fatalf("SearchCrowdfundingOfferings = %+v, %v", search, err)
	}
	regulationD, err := fund.SearchRegulationDOfferings(ctx, NewOfferingSearchQuery("NJOY / Class A"))
	if err != nil || len(regulationD) != 1 || regulationD[0].CIK != "0001547416" {
		t.Fatalf("SearchRegulationDOfferings = %+v, %v", regulationD, err)
	}
	full := NewLatestRegulationDOfferingsQuery().WithPage(0).WithLimit(10).WithCIK("0002013736")
	offerings, err := fund.LatestRegulationDOfferings(ctx, full)
	if err != nil || len(offerings) != 1 || offerings[0].CIK != "0002127786" {
		t.Fatalf("LatestRegulationDOfferings = %+v, %v", offerings, err)
	}
	if _, err := fund.LatestRegulationDOfferings(ctx, NewLatestRegulationDOfferingsQuery().WithCIK("0002013736")); err != nil {
		t.Fatalf("LatestRegulationDOfferings cik only: %v", err)
	}
	if _, err := fund.LatestRegulationDOfferings(ctx, NewLatestRegulationDOfferingsQuery().WithLimit(0).WithPage(4_294_967_295)); err != nil {
		t.Fatalf("LatestRegulationDOfferings page and limit: %v", err)
	}
	dOfferings, err := fund.RegulationDOfferingsByCIK(ctx, NewOfferingByCIKQuery("0001547416"))
	if err != nil || len(dOfferings) != 1 || dOfferings[0].TotalNumberAlreadyInvested != 24 {
		t.Fatalf("RegulationDOfferingsByCIK = %+v, %v", dOfferings, err)
	}

	want := []string{
		"/router/stable/crowdfunding-offerings-latest",
		"/router/stable/crowdfunding-offerings-latest?page=0&limit=100",
		"/router/stable/crowdfunding-offerings-latest?limit=0",
		"/router/stable/crowdfunding-offerings?cik=0001916078",
		"/router/stable/crowdfunding-offerings-search?name=NJOY+%2F+Class+A",
		"/router/stable/fundraising-search?name=NJOY+%2F+Class+A",
		"/router/stable/fundraising-latest?page=0&limit=10&cik=0002013736",
		"/router/stable/fundraising-latest?cik=0002013736",
		"/router/stable/fundraising-latest?page=4294967295&limit=0",
		"/router/stable/fundraising?cik=0001547416",
	}
	requests := rec.all()
	if len(requests) != len(want) {
		t.Fatalf("requests = %d, want exactly one per call", len(requests))
	}
	for index, req := range requests {
		if got := req.URL.RequestURI(); got != want[index] {
			t.Fatalf("request %d = %s, want %s", index, got, want[index])
		}
		if req.Method != http.MethodGet {
			t.Fatalf("%s used %s, want GET", req.URL.Path, req.Method)
		}
		if strings.Contains(req.URL.RawQuery, "apikey") {
			t.Fatalf("%s carried the credential in the query: %s", req.URL.Path, req.URL.RawQuery)
		}
	}
}

func TestFundraisingQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, fundraisingRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	fund := client.Fundraising
	cases := []struct {
		name   string
		call   func() error
		member string
		reason error
	}{
		{"empty search name", func() error {
			_, err := fund.SearchCrowdfundingOfferings(ctx, NewOfferingSearchQuery(""))
			return err
		}, "name", ErrEmptyValue},
		{"whitespace search name", func() error {
			_, err := fund.SearchRegulationDOfferings(ctx, NewOfferingSearchQuery(" \t "))
			return err
		}, "name", ErrEmptyValue},
		{"control character in search name", func() error {
			_, err := fund.SearchRegulationDOfferings(ctx, NewOfferingSearchQuery("NJOY\nINC"))
			return err
		}, "name", ErrControlCharacterValue},
		{"empty cik", func() error {
			_, err := fund.CrowdfundingOfferingsByCIK(ctx, NewOfferingByCIKQuery(""))
			return err
		}, "cik", ErrEmptyValue},
		{"newline cik", func() error {
			_, err := fund.RegulationDOfferingsByCIK(ctx, NewOfferingByCIKQuery("\n"))
			return err
		}, "cik", ErrEmptyValue},
		{"blank optional cik", func() error {
			_, err := fund.LatestRegulationDOfferings(ctx, NewLatestRegulationDOfferingsQuery().WithCIK(" "))
			return err
		}, "cik", ErrEmptyValue},
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
	empty := NewLatestRegulationDOfferingsQuery()
	if empty.Page() != nil || empty.Limit() != nil || empty.CIK() != nil {
		t.Fatalf("NewLatestRegulationDOfferingsQuery() set a parameter: %+v", empty)
	}
	if q := NewOfferingSearchQuery("NJOY / Class A"); q.Name() != "NJOY / Class A" {
		t.Fatalf("Name() normalized the search text: %q", q.Name())
	}
	if q := NewOfferingByCIKQuery("0001916078"); q.CIK() != "0001916078" {
		t.Fatalf("CIK() dropped the leading zeros: %q", q.CIK())
	}
}

func TestFundraisingMethodsReportNumberKindAsDecodeError(t *testing.T) {
	t.Parallel()
	fixture := string(readFixture(t, "crowdfunding_offerings_latest.json"))
	corrupted := strings.Replace(fixture, `"offeringPrice": 0.1,`, `"offeringPrice": "0.1",`, 1)
	if corrupted == fixture {
		t.Fatal("crowdfunding_offerings_latest.json no longer spells offeringPrice as the decimal 0.1")
	}
	server, _ := newServer(t, jsonHandler(corrupted))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))

	_, err := client.Fundraising.LatestCrowdfundingOfferings(context.Background(), NewLatestCrowdfundingOfferingsQuery())
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "crowdfunding-offerings-latest")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"offeringPrice"`) ||
		!strings.Contains(cause.Error(), "JSON number") {
		t.Fatalf("cause = %v, want it to name the non-number member offeringPrice", cause)
	}
}
