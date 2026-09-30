package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"reflect"
	"strings"
	"testing"
)

// The 12 sec_filings fixtures over the 4 response models plus the dynamic
// classification search, decoded through the shared parity helper (ADR 0030:
// identical bytes for the Rust and Go decoders). Three filing-search fixtures
// omit the optional hasFinancials member, which serde and the Go model both
// write back as null.
func TestSecFilingsFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[SECFiling](t, "latest_8k_sec_filings.json")
	assertFixtureParity[SECFiling](t, "latest_sec_filings.json")
	omitted := []string{"hasFinancials"}
	assertFixtureParityNullWhenOmitted[SECFiling](t, "sec_filings_by_form_type.json", omitted)
	assertFixtureParityNullWhenOmitted[SECFiling](t, "sec_filings_by_symbol.json", omitted)
	assertFixtureParityNullWhenOmitted[SECFiling](t, "sec_filings_by_cik.json", omitted)
	assertFixtureParity[SECCompanySearchResult](t, "sec_companies_by_name.json")
	assertFixtureParity[SECCompanySearchResult](t, "sec_companies_by_symbol.json")
	assertFixtureParity[SECCompanySearchResult](t, "sec_companies_by_cik.json")
	assertFixtureParity[SECCompanySearchResult](t, "all_industry_classifications.json")
	assertFixtureParity[SECCompanyProfile](t, "sec_company_profile.json")
	assertFixtureParity[SicClassification](t, "industry_classifications.json")
	assertFixtureParity[jsontext.Value](t, "industry_classification_search.json")
}

// Exact values copied from crates/libfmp/tests/sec_filings_responses.rs
// (all_five_filing_routes_share_the_exact_eight_field_model): the 8-K row
// carries a null hasFinancials, the financials row carries true, and the
// three search rows omit it.
func TestDocumentedSecFilingRowsDecodeExactValues(t *testing.T) {
	t.Parallel()
	latest8k := assertFixtureParity[SECFiling](t, "latest_8k_sec_filings.json")
	if len(latest8k) != 1 {
		t.Fatalf("latest_8k_sec_filings = %+v, want one row", latest8k)
	}
	row := latest8k[0]
	if row.Symbol != "SUNE" || row.CIK != "0000022701" || row.FormType != "8-K" ||
		row.FilingDate != mustParseDateTime(t, "2024-03-04 00:00:00") ||
		row.AcceptedDate != mustParseDateTime(t, "2024-03-01 22:47:48") ||
		row.HasFinancials != nil ||
		!strings.HasSuffix(row.Link, "-index.htm") || !strings.HasSuffix(row.FinalLink, "_8k.htm") {
		t.Fatalf("latest_8k_sec_filings[0] = %+v", row)
	}

	latest := assertFixtureParity[SECFiling](t, "latest_sec_filings.json")
	if len(latest) != 1 || latest[0].Symbol != "DNN" || latest[0].HasFinancials == nil || !*latest[0].HasFinancials {
		t.Fatalf("latest_sec_filings = %+v, want DNN with hasFinancials true", latest)
	}

	omitted := []string{"hasFinancials"}
	for _, fixture := range []string{"sec_filings_by_form_type.json", "sec_filings_by_symbol.json", "sec_filings_by_cik.json"} {
		rows := assertFixtureParityNullWhenOmitted[SECFiling](t, fixture, omitted)
		if len(rows) != 1 || rows[0].HasFinancials != nil {
			t.Fatalf("%s = %+v, want one row without hasFinancials", fixture, rows)
		}
		encoded, err := json.Marshal(rows[0])
		if err != nil {
			t.Fatal(err)
		}
		if !strings.Contains(string(encoded), `"hasFinancials":null`) {
			t.Fatalf("%s re-encoded = %s, want hasFinancials null as serde writes None", fixture, encoded)
		}
	}
	bySymbol := assertFixtureParityNullWhenOmitted[SECFiling](t, "sec_filings_by_symbol.json", omitted)
	byCIK := assertFixtureParityNullWhenOmitted[SECFiling](t, "sec_filings_by_cik.json", omitted)
	if bySymbol[0].Symbol != "AAPL" || byCIK[0].CIK != "0000320193" || bySymbol[0].FormType != "4" ||
		byCIK[0].AcceptedDate.String() != "2024-03-01 18:36:45" {
		t.Fatalf("by_symbol = %+v, by_cik = %+v", bySymbol[0], byCIK[0])
	}
}

// Exact values copied from crates/libfmp/tests/sec_filings_responses.rs
// (company_search_fixtures_preserve_none_empty_strings_addresses_and_leading_zeroes):
// the literal "None" symbol, empty strings, and leading zeroes survive.
func TestDocumentedSecCompanySearchRowsDecodeExactValues(t *testing.T) {
	t.Parallel()
	names := assertFixtureParity[SECCompanySearchResult](t, "sec_companies_by_name.json")
	wantName := SECCompanySearchResult{
		Symbol:          "None",
		Name:            "BERKSHIRE MULTIFAMILY VALUE FUND II LP",
		CIK:             "0001418405",
		SicCode:         "",
		IndustryTitle:   "",
		BusinessAddress: "c/o Berkshire Property Advisors LLC, Boston MA 02108",
		PhoneNumber:     new("(617) 646-2300"),
	}
	if len(names) != 1 || !reflect.DeepEqual(names[0], wantName) {
		t.Fatalf("sec_companies_by_name = %+v, want %+v", names, wantName)
	}

	bySymbol := assertFixtureParity[SECCompanySearchResult](t, "sec_companies_by_symbol.json")
	byCIK := assertFixtureParity[SECCompanySearchResult](t, "sec_companies_by_cik.json")
	if len(bySymbol) != 1 || len(byCIK) != 1 || !reflect.DeepEqual(bySymbol[0], byCIK[0]) ||
		bySymbol[0].Symbol != "AAPL" || bySymbol[0].Name != "APPLE INC." || byCIK[0].CIK != "0000320193" {
		t.Fatalf("by_symbol = %+v, by_cik = %+v, want the same AAPL row", bySymbol, byCIK)
	}

	all := assertFixtureParity[SECCompanySearchResult](t, "all_industry_classifications.json")
	if len(all) != 1 || all[0].Symbol != "0Q16.L" || all[0].CIK != "0000070858" ||
		all[0].BusinessAddress != "['BANK OF AMERICA CORPORATE CENTER', 'CHARLOTTE NC 28255']" {
		t.Fatalf("all_industry_classifications = %+v", all)
	}
}

// Exact values copied from crates/libfmp/tests/sec_filings_responses.rs
// (full_profile_decodes_exactly_thirty_five_fields_and_documented_types and
// null_only_profile_security_type_preserves_future_json_without_inventing_a_type):
// the null securityType is a present member, and any future JSON there is
// kept verbatim.
func TestDocumentedSecCompanyProfileDecodesExactValues(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[SECCompanyProfile](t, "sec_company_profile.json")
	if len(rows) != 1 {
		t.Fatalf("sec_company_profile = %+v, want one row", rows)
	}
	row := rows[0]
	if row.Symbol != "AAPL" || row.CIK != "0000320193" || row.ISIN != "US0378331005" || row.Country != "US" ||
		row.Exchange != "NASDAQ" || row.IPODate != mustParseDate(t, "1980-12-12") || !reflect.DeepEqual(row.Employees, new("166000")) ||
		row.PriceCurrency != "USD" || row.MarketSector != "Technology" || row.SecurityType != nil ||
		!row.IsActive || row.IsETF || row.IsAdr || row.IsFund {
		t.Fatalf("sec_company_profile[0] = %+v", row)
	}
	if members := memberSet(t, row); len(members) != 35 {
		t.Fatalf("re-encoded profile has %d members, want 35: %v", len(members), members)
	}

	future := `{"providerKind":["stock",1,true,null]}`
	source := strings.Replace(string(readFixture(t, "sec_company_profile.json")),
		`"securityType": null`, `"securityType": `+future, 1)
	var typed []SECCompanyProfile
	if err := json.Unmarshal([]byte(source), &typed); err != nil {
		t.Fatalf("profile with a future securityType: %v", err)
	}
	if typed[0].SecurityType == nil || string(*typed[0].SecurityType) != future {
		t.Fatalf("securityType = %v, want %s verbatim", typed[0].SecurityType, future)
	}
	if encoded := memberValues(t, typed[0]); string(encoded["securityType"]) != future {
		t.Fatalf("re-encoded securityType = %s, want %s", encoded["securityType"], future)
	}

	missing := strings.Replace(string(readFixture(t, "sec_company_profile.json")),
		`"securityType": null,`, ``, 1)
	var rejected []SECCompanyProfile
	err := json.Unmarshal([]byte(missing), &rejected)
	var typedErr *Error
	if !errors.As(err, &typedErr) || typedErr.Category != CategoryDecode || !strings.Contains(typedErr.Message, `"securityType"`) {
		t.Fatalf("profile without securityType: error = %v, want a decode error naming securityType", err)
	}
}

// Exact values copied from crates/libfmp/tests/sec_filings_responses.rs
// (sic_list_is_typed_but_documented_empty_search_stays_raw): the SIC list is
// typed and the documented empty search row stays a raw object.
func TestDocumentedSicClassificationAndRawSearchRowsDecodeExactValues(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[SicClassification](t, "industry_classifications.json")
	want := SicClassification{Office: "Office of Life Sciences", SicCode: "100", IndustryTitle: "AGRICULTURAL PRODUCTION-CROPS"}
	if len(rows) != 1 || rows[0] != want {
		t.Fatalf("industry_classifications = %+v, want %+v", rows, want)
	}

	raw := assertFixtureParity[jsontext.Value](t, "industry_classification_search.json")
	if len(raw) != 1 || raw[0].Kind() != '{' || strings.TrimSpace(string(raw[0])) != "{}" {
		t.Fatalf("industry_classification_search = %s, want one empty object", raw)
	}
	arbitrary := `[{"future":[1,true,null],"nested":{"sicCode":"07371"}}]`
	var dynamic []jsontext.Value
	if err := json.Unmarshal([]byte(arbitrary), &dynamic); err != nil {
		t.Fatal(err)
	}
	encoded, err := json.Marshal(dynamic)
	if err != nil || string(encoded) != arbitrary {
		t.Fatalf("dynamic rows re-encoded = %s, %v, want %s byte-identical", encoded, err, arbitrary)
	}
}

// Every contract is a bare array that preserves empty and multiple rows and
// rejects an object envelope
// (every_contract_is_a_bare_array_accepting_empty_and_multiple_rows).
func TestSecFilingsContractsAreBareArrays(t *testing.T) {
	t.Parallel()
	row := strings.TrimSpace(string(readFixture(t, "latest_sec_filings.json")))
	row = strings.TrimSuffix(strings.TrimPrefix(row, "["), "]")
	var two []SECFiling
	if err := json.Unmarshal([]byte("["+row+","+row+"]"), &two); err != nil || len(two) != 2 ||
		two[0].Symbol != two[1].Symbol || *two[0].HasFinancials != *two[1].HasFinancials {
		t.Fatalf("two-row array = %+v, %v", two, err)
	}
	var empty []SECFiling
	if err := json.Unmarshal([]byte(`[]`), &empty); err != nil || len(empty) != 0 {
		t.Fatalf("empty array = %+v, %v", empty, err)
	}
	var enveloped []SECFiling
	if err := json.Unmarshal([]byte(`{"data":[]}`), &enveloped); err == nil {
		t.Fatal("an object envelope decoded into a bare-array contract")
	}
}

// The missing-required-member path of the generated decoders, once for this
// domain: serde rejects a missing and a null member alike, and a malformed
// date-time never decodes into a DateTime member
// (typed_contracts_require_documented_non_null_fields_and_accept_unknown_fields).
func TestSecFilingsRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name   string
		wire   string
		member string
	}{
		{"missing member", `[{"office":"Office of Life Sciences","industryTitle":"CROPS"}]`, "sicCode"},
		{"null member", `[{"office":null,"sicCode":"100","industryTitle":"CROPS"}]`, "office"},
		{"empty object", `[{}]`, "office"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []SicClassification
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) || !strings.Contains(typed.Message, "SicClassification") {
				t.Fatalf("message = %q, want it to name member %q of SicClassification", typed.Message, tc.member)
			}
		})
	}
	var rows []SECFiling
	malformed := strings.Replace(string(readFixture(t, "latest_sec_filings.json")),
		`"acceptedDate": "2024-03-01 16:52:35"`, `"acceptedDate": "2024-03-01T16:52:35Z"`, 1)
	if err := json.Unmarshal([]byte(malformed), &rows); err == nil {
		t.Fatal("a malformed date-time decoded into a DateTime member")
	}
	var filings []SECFiling
	err := json.Unmarshal([]byte(`[{"symbol":"DNN","cik":"0001063259","filingDate":"2024-03-01 00:00:00",`+
		`"acceptedDate":"2024-03-01 16:52:35","formType":"6-K","hasFinancials":true,"link":null,"finalLink":"x"}]`), &filings)
	var typed *Error
	if !errors.As(err, &typed) || !strings.Contains(typed.Message, `"link"`) {
		t.Fatalf("SECFiling error = %v, want it to name the null member link", err)
	}
}

// Mirrors nullable_profile_and_search_members_are_required_and_null_decodes_to_none in
// crates/libfmp/tests/sec_filings_responses.rs: members FMP sends as null
// decode to nil, while a missing member still fails.
func TestSecCompanyProfileNullableMembersDecodeNullAsNil(t *testing.T) {
	t.Parallel()
	const fixture = "sec_company_profile.json"
	cases := []struct {
		member string
		isNil  func(SECCompanyProfile) bool
	}{
		{"employees", func(row SECCompanyProfile) bool { return row.Employees == nil }},
		{"fiscalYearEnd", func(row SECCompanyProfile) bool { return row.FiscalYearEnd == nil }},
	}
	for _, tc := range cases {
		t.Run(tc.member, func(t *testing.T) {
			t.Parallel()
			var missing []SECCompanyProfile
			if err := json.Unmarshal(mutateFixtureMember(t, fixture, tc.member, nil), &missing); err == nil {
				t.Fatalf("missing %s decoded", tc.member)
			}
			var rows []SECCompanyProfile
			if err := json.Unmarshal(mutateFixtureMember(t, fixture, tc.member, jsontext.Value(`null`)), &rows); err != nil ||
				len(rows) != 1 || !tc.isNil(rows[0]) {
				t.Fatalf("null %s = %+v, %v, want nil", tc.member, rows, err)
			}
		})
	}
}

// Mirrors nullable_profile_and_search_members_are_required_and_null_decodes_to_none in
// crates/libfmp/tests/sec_filings_responses.rs: members FMP sends as null
// decode to nil, while a missing member still fails.
func TestSecCompanySearchPhoneNumberDecodesNullAsNil(t *testing.T) {
	t.Parallel()
	const fixture = "sec_companies_by_name.json"
	cases := []struct {
		member string
		isNil  func(SECCompanySearchResult) bool
	}{
		{"phoneNumber", func(row SECCompanySearchResult) bool { return row.PhoneNumber == nil }},
	}
	for _, tc := range cases {
		t.Run(tc.member, func(t *testing.T) {
			t.Parallel()
			var missing []SECCompanySearchResult
			if err := json.Unmarshal(mutateFixtureMember(t, fixture, tc.member, nil), &missing); err == nil {
				t.Fatalf("missing %s decoded", tc.member)
			}
			var rows []SECCompanySearchResult
			if err := json.Unmarshal(mutateFixtureMember(t, fixture, tc.member, jsontext.Value(`null`)), &rows); err != nil ||
				len(rows) != 1 || !tc.isNil(rows[0]) {
				t.Fatalf("null %s = %+v, %v, want nil", tc.member, rows, err)
			}
		})
	}
}
