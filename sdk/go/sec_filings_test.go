package fmp

import (
	"context"
	"encoding/json/v2"
	"errors"
	"fmt"
	"math"
	"net/http"
	"strings"
	"testing"
)

// secFilingsRoutes is the exact request-URI table of the twelve sec_filings
// endpoints, copied from the URL assertions in
// crates/libfmp/tests/sec_*_endpoints.rs. Keying on the full request URI lets
// one path carry its no-setter and with-setter variants (wire order and
// escaping matter: spaces, slashes, and commas are escaped as the Rust
// encoder does, and the profile CIK key is spelled cik-A).
var secFilingsRoutes = map[string]string{
	"/router/stable/sec-filings-8k?from=2024-01-01&to=2024-03-01&page=0&limit=4294967295":                                 "latest_8k_sec_filings.json",
	"/router/stable/sec-filings-8k?from=2024-01-01&to=2024-03-01":                                                         "latest_8k_sec_filings.json",
	"/router/stable/sec-filings-financials?from=2024-01-01&to=2024-03-01":                                                 "latest_sec_filings.json",
	"/router/stable/sec-filings-search/form-type?formType=8-K&from=2024-01-01&to=2024-03-01&page=0&limit=0":               "sec_filings_by_form_type.json",
	"/router/stable/sec-filings-search/symbol?symbol=BRK.B+%2F+Class+A&from=2024-01-01&to=2024-03-01":                     "sec_filings_by_symbol.json",
	"/router/stable/sec-filings-search/cik?cik=0000320193&from=2024-01-01&to=2024-03-01&page=4294967295&limit=4294967295": "sec_filings_by_cik.json",
	"/router/stable/sec-filings-company-search/name?company=Berkshire%2C+Hathaway+%2F+Fund":                               "sec_companies_by_name.json",
	"/router/stable/sec-filings-company-search/symbol?symbol=BRK.B+%2F+Class+A":                                           "sec_companies_by_symbol.json",
	"/router/stable/sec-filings-company-search/cik?cik=0000320193":                                                        "sec_companies_by_cik.json",
	"/router/stable/sec-profile?symbol=AAPL&cik-A=0000320193":                                                             "sec_company_profile.json",
	"/router/stable/sec-profile?symbol=AAPL":                                                                              "sec_company_profile.json",
	"/router/stable/standard-industrial-classification-list?industryTitle=SERVICES%2C+NEC+%2F+OTHER&sicCode=07371":        "industry_classifications.json",
	"/router/stable/standard-industrial-classification-list":                                                              "industry_classifications.json",
	"/router/stable/industry-classification-search?symbol=BRK.B+%2F+Class+A&cik=0000320193&sicCode=07371":                 "industry_classification_search.json",
	"/router/stable/industry-classification-search":                                                                       "industry_classification_search.json",
	"/router/stable/all-industry-classification?page=0&limit=4294967295":                                                  "all_industry_classifications.json",
	"/router/stable/all-industry-classification":                                                                          "all_industry_classifications.json",
}

func secFilingsRouter(t *testing.T) http.HandlerFunc {
	t.Helper()
	return func(w http.ResponseWriter, r *http.Request) {
		fixture, ok := secFilingsRoutes[r.URL.RequestURI()]
		if !ok || r.Header.Get("apikey") != "route-secret" {
			w.WriteHeader(http.StatusNotFound)
			_, _ = fmt.Fprintf(w, "unexpected request %s", r.URL.RequestURI())
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write(readFixture(t, fixture))
	}
}

func TestSecFilingsMethodsUseExactPathsAndWireParameterOrder(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, secFilingsRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	from := mustParseDate(t, "2024-01-01")
	to := mustParseDate(t, "2024-03-01")

	latest8k, err := client.SECFilings.Latest8K(ctx, NewLatest8KSECFilingsQuery(from, to).WithLimit(math.MaxUint32).WithPage(0))
	if err != nil || len(latest8k) != 1 || latest8k[0].Symbol != "SUNE" || latest8k[0].HasFinancials != nil {
		t.Fatalf("Latest8K = %+v, %v", latest8k, err)
	}
	if bare, err := client.SECFilings.Latest8K(ctx, NewLatest8KSECFilingsQuery(from, to)); err != nil || len(bare) != 1 {
		t.Fatalf("Latest8K without setters = %+v, %v", bare, err)
	}
	latest, err := client.SECFilings.Latest(ctx, NewLatestSECFilingsQuery(from, to))
	if err != nil || len(latest) != 1 || latest[0].Symbol != "DNN" || latest[0].HasFinancials == nil || !*latest[0].HasFinancials {
		t.Fatalf("Latest = %+v, %v", latest, err)
	}
	byForm, err := client.SECFilings.ByFormType(ctx, NewSECFilingsByFormTypeQuery("8-K", from, to).WithLimit(0).WithPage(0))
	if err != nil || len(byForm) != 1 || byForm[0].FormType != "8-K" || byForm[0].HasFinancials != nil {
		t.Fatalf("ByFormType = %+v, %v", byForm, err)
	}
	bySymbol, err := client.SECFilings.BySymbol(ctx, NewSECFilingsBySymbolQuery("BRK.B / Class A", from, to))
	if err != nil || len(bySymbol) != 1 || bySymbol[0].Symbol != "AAPL" {
		t.Fatalf("BySymbol = %+v, %v", bySymbol, err)
	}
	byCik, err := client.SECFilings.ByCIK(ctx, NewSECFilingsByCIKQuery("0000320193", from, to).
		WithLimit(math.MaxUint32).WithPage(math.MaxUint32))
	if err != nil || len(byCik) != 1 || byCik[0].CIK != "0000320193" || byCik[0].AcceptedDate.String() != "2024-03-01 18:36:45" {
		t.Fatalf("ByCIK = %+v, %v", byCik, err)
	}

	byName, err := client.SECFilings.SearchCompaniesByName(ctx, NewSECCompaniesByNameQuery("Berkshire, Hathaway / Fund"))
	if err != nil || len(byName) != 1 || byName[0].Symbol != "None" || byName[0].CIK != "0001418405" {
		t.Fatalf("SearchCompaniesByName = %+v, %v", byName, err)
	}
	companyBySymbol, err := client.SECFilings.SearchCompaniesBySymbol(ctx, NewSECCompaniesBySymbolQuery("BRK.B / Class A"))
	if err != nil || len(companyBySymbol) != 1 || companyBySymbol[0].Name != "APPLE INC." {
		t.Fatalf("SearchCompaniesBySymbol = %+v, %v", companyBySymbol, err)
	}
	companyByCik, err := client.SECFilings.SearchCompaniesByCIK(ctx, NewSECCompaniesByCIKQuery("0000320193"))
	if err != nil || len(companyByCik) != 1 || companyByCik[0] != companyBySymbol[0] {
		t.Fatalf("SearchCompaniesByCIK = %+v, %v", companyByCik, err)
	}
	profile, err := client.SECFilings.CompanyProfile(ctx, NewSECCompanyProfileQuery("AAPL").WithCIKA("0000320193"))
	if err != nil || len(profile) != 1 || profile[0].Symbol != "AAPL" || profile[0].SecurityType != nil {
		t.Fatalf("CompanyProfile = %+v, %v", profile, err)
	}
	if bare, err := client.SECFilings.CompanyProfile(ctx, NewSECCompanyProfileQuery("AAPL")); err != nil || len(bare) != 1 {
		t.Fatalf("CompanyProfile without cik-A = %+v, %v", bare, err)
	}

	classifications, err := client.SECFilings.IndustryClassifications(ctx, NewIndustryClassificationsQuery().
		WithSicCode("07371").WithIndustryTitle("SERVICES, NEC / OTHER"))
	if err != nil || len(classifications) != 1 || classifications[0].SicCode != "100" {
		t.Fatalf("IndustryClassifications = %+v, %v", classifications, err)
	}
	if bare, err := client.SECFilings.IndustryClassifications(ctx, NewIndustryClassificationsQuery()); err != nil || len(bare) != 1 {
		t.Fatalf("IndustryClassifications without filters = %+v, %v", bare, err)
	}
	search, err := client.SECFilings.SearchIndustryClassifications(ctx, NewIndustryClassificationSearchQuery().
		WithSicCode("07371").WithCIK("0000320193").WithSymbol("BRK.B / Class A"))
	if err != nil || len(search) != 1 || strings.TrimSpace(string(search[0])) != "{}" {
		t.Fatalf("SearchIndustryClassifications = %s, %v", search, err)
	}
	if bare, err := client.SECFilings.SearchIndustryClassifications(ctx, NewIndustryClassificationSearchQuery()); err != nil || len(bare) != 1 {
		t.Fatalf("SearchIndustryClassifications without filters = %s, %v", bare, err)
	}
	all, err := client.SECFilings.AllIndustryClassifications(ctx, NewAllIndustryClassificationsQuery().
		WithLimit(math.MaxUint32).WithPage(0))
	if err != nil || len(all) != 1 || all[0].Symbol != "0Q16.L" {
		t.Fatalf("AllIndustryClassifications = %+v, %v", all, err)
	}
	if bare, err := client.SECFilings.AllIndustryClassifications(ctx, NewAllIndustryClassificationsQuery()); err != nil || len(bare) != 1 {
		t.Fatalf("AllIndustryClassifications without pagination = %+v, %v", bare, err)
	}

	requests := rec.all()
	if len(requests) != len(secFilingsRoutes) {
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

// The dynamic contract of the classification search (the first
// response = "dynamic" endpoint): rows are raw JSON objects returned
// byte-identical, and a non-object row is a decode error, as
// Vec<DynamicObject> rejects it in Rust.
func TestSecFilingsDynamicSearchRowsSurviveByteIdenticalAndMustBeObjects(t *testing.T) {
	t.Parallel()
	body := `[{"future":[1,true,null],"nested":{"sicCode":"07371"}},{}]`
	server, _ := newServer(t, jsonHandler(body))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	rows, err := client.SECFilings.SearchIndustryClassifications(context.Background(), NewIndustryClassificationSearchQuery())
	if err != nil || len(rows) != 2 {
		t.Fatalf("SearchIndustryClassifications = %s, %v", rows, err)
	}
	if string(rows[0]) != `{"future":[1,true,null],"nested":{"sicCode":"07371"}}` || string(rows[1]) != `{}` {
		t.Fatalf("rows = %s, want the wire objects verbatim", rows)
	}
	encoded, err := json.Marshal(rows)
	if err != nil || string(encoded) != body {
		t.Fatalf("re-encoded = %s, %v, want %s byte-identical", encoded, err, body)
	}

	scalar, rec := newServer(t, jsonHandler(`[{"sicCode":"07371"},1]`))
	client = newClient(t, scalar, WithAuthentication(FMPHeader("route-secret")))
	_, err = client.SECFilings.SearchIndustryClassifications(context.Background(), NewIndustryClassificationSearchQuery())
	typed := assertQuoteError(t, err, CategoryDecode, 0, "industry-classification-search")
	if !strings.Contains(typed.Message, "row 1") {
		t.Fatalf("message = %q, want it to name row 1", typed.Message)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}

// Every argument is validated the way the Rust newtypes are, before any
// request: tickers reject a comma, the open CIK, form-type, and search-term
// strings accept one, all reject empty or control text, and a zero Date has
// no wire form.
func TestSecFilingsQueriesAreValidatedBeforeAnyRequest(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, secFilingsRouter(t))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	ctx := context.Background()
	from := mustParseDate(t, "2024-01-01")
	to := mustParseDate(t, "2024-03-01")
	cases := []struct {
		name   string
		call   func() error
		member string
		reason error
	}{
		{"filings comma ticker", func() error {
			_, err := client.SECFilings.BySymbol(ctx, NewSECFilingsBySymbolQuery("AAPL,MSFT", from, to))
			return err
		}, "symbol", ErrCommaInTicker},
		{"filings empty cik", func() error {
			_, err := client.SECFilings.ByCIK(ctx, NewSECFilingsByCIKQuery(" ", from, to))
			return err
		}, "cik", ErrEmptyValue},
		{"filings control form type", func() error {
			_, err := client.SECFilings.ByFormType(ctx, NewSECFilingsByFormTypeQuery("8-\tK", from, to))
			return err
		}, "formType", ErrControlCharacterValue},
		{"latest zero from", func() error {
			_, err := client.SECFilings.Latest8K(ctx, NewLatest8KSECFilingsQuery(Date{}, to))
			return err
		}, "from", ErrZeroTemporalValue},
		{"latest zero to", func() error {
			_, err := client.SECFilings.Latest(ctx, NewLatestSECFilingsQuery(from, Date{}))
			return err
		}, "to", ErrZeroTemporalValue},
		{"company name empty", func() error {
			_, err := client.SECFilings.SearchCompaniesByName(ctx, NewSECCompaniesByNameQuery(""))
			return err
		}, "company", ErrEmptyValue},
		{"profile empty cik-A", func() error {
			_, err := client.SECFilings.CompanyProfile(ctx, NewSECCompanyProfileQuery("AAPL").WithCIKA(""))
			return err
		}, "cik-A", ErrEmptyValue},
		{"classification search comma ticker", func() error {
			_, err := client.SECFilings.SearchIndustryClassifications(ctx, NewIndustryClassificationSearchQuery().WithSymbol("AAPL,MSFT"))
			return err
		}, "symbol", ErrCommaInTicker},
		{"classification list control title", func() error {
			_, err := client.SECFilings.IndustryClassifications(ctx, NewIndustryClassificationsQuery().WithIndustryTitle("SERVICES\n"))
			return err
		}, "industryTitle", ErrControlCharacterValue},
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
	base := NewSECCompanyProfileQuery(" AAPL ")
	set := base.WithCIKA("0000320193")
	if base.CIKA() != nil || set.CIKA() == nil || *set.CIKA() != "0000320193" || set.Symbol() != " AAPL " {
		t.Fatalf("SECCompanyProfileQuery setters mutated the receiver or normalized a value: %+v %+v", base, set)
	}
	filings := NewSECFilingsByFormTypeQuery("8-K", from, to)
	paged := filings.WithPage(3)
	if filings.Page() != nil || paged.Page() == nil || *paged.Page() != 3 || paged.From() != from || paged.To() != to ||
		paged.FormType() != "8-K" || paged.Limit() != nil {
		t.Fatalf("SECFilingsByFormTypeQuery setters mutated the receiver: %+v %+v", filings, paged)
	}
}

func TestSecFilingsMethodsReportMissingMembersAsDecodeErrors(t *testing.T) {
	t.Parallel()
	server, rec := newServer(t, jsonHandler(`[{"symbol":"SUNE","cik":"0000022701","filingDate":"2024-03-04 00:00:00",`+
		`"acceptedDate":"2024-03-01 22:47:48","formType":"8-K"}]`))
	client := newClient(t, server, WithAuthentication(FMPHeader("route-secret")))
	from := mustParseDate(t, "2024-01-01")
	to := mustParseDate(t, "2024-03-01")

	_, err := client.SECFilings.Latest8K(context.Background(), NewLatest8KSECFilingsQuery(from, to))
	typed := assertQuoteError(t, err, CategoryDecode, http.StatusOK, "sec-filings-8k")
	if cause := typed.Unwrap(); cause == nil || !strings.Contains(cause.Error(), `"link"`) {
		t.Fatalf("cause = %v, want it to name the missing member link", cause)
	}
	if rec.count() != 1 {
		t.Fatalf("decode failure was retried: %d requests", rec.count())
	}
}
