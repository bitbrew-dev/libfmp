package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"slices"
	"strings"
	"testing"
)

func TestFundraisingFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[CrowdfundingOffering](t, "crowdfunding_offerings_latest.json")
	assertFixtureParity[CrowdfundingOffering](t, "crowdfunding_offerings_by_cik.json")
	assertFixtureParity[CrowdfundingOfferingSearchResult](t, "crowdfunding_offerings_search.json")
	assertFixtureParity[RegulationDOffering](t, "fundraising_latest.json")
	assertFixtureParity[RegulationDOffering](t, "fundraising_by_cik.json")
	assertFixtureParity[RegulationDOfferingSearchResult](t, "fundraising_search.json")
}

// Exact values copied from crates/libfmp/tests/fundraising_crowdfunding_responses.rs:
// US-spelled dates, the two "EquiValent" wire keys the provider misspells,
// the Y/N flag kept as text, and the offering price kept in its wire spelling.
func TestDocumentedCrowdfundingOfferingsDecodeExactValuesAndKeepWireHazards(t *testing.T) {
	t.Parallel()
	latest := assertFixtureParity[CrowdfundingOffering](t, "crowdfunding_offerings_latest.json")
	if len(latest) != 1 {
		t.Fatalf("rows = %d, want 1", len(latest))
	}
	row := latest[0]
	if row.CIK != "0001621902" || row.IntermediaryCommissionCIK != "0001669191" ||
		row.Date.String() != "11-22-2011" || row.FilingDate.String() != "2026-07-30 00:00:00" ||
		row.AcceptedDate.String() != "2026-07-30 12:54:38" || row.OfferingDeadlineDate.String() != "10-31-2026" ||
		row.FormType != "C/A" || row.OverSubscriptionAccepted != "Y" || row.NumberOfSecurityOffered != 100_000 ||
		row.CurrentNumberOfEmployees != 5 || string(row.OfferingPrice) != "0.1" ||
		row.NetIncomeMostRecentFiscalYear != -152_577 || row.NetIncomePriorFiscalYear != -105_631 ||
		row.SecurityOfferedOtherDescription != nil {
		t.Fatalf("crowdfunding_offerings_latest = %+v", row)
	}
	if row.Date.Date().String() != "2011-11-22" {
		t.Fatalf("USDate.Date() = %s, want the same civil date", row.Date.Date())
	}
	members := memberSet(t, row)
	if len(members) != 48 {
		t.Fatalf("re-encoded members = %d, want the documented 48", len(members))
	}
	for _, key := range []string{"cashAndCashEquiValentMostRecentFiscalYear", "cashAndCashEquiValentPriorFiscalYear"} {
		if !slices.Contains(members, key) {
			t.Fatalf("missing exact wire key %q in %v", key, members)
		}
	}
	if slices.Contains(members, "cashAndCashEquivalentMostRecentFiscalYear") {
		t.Fatal("corrected but wrong wire key cashAndCashEquivalentMostRecentFiscalYear was emitted")
	}

	byCIK := assertFixtureParity[CrowdfundingOffering](t, "crowdfunding_offerings_by_cik.json")[0]
	if byCIK.CIK != "0001916078" || byCIK.IntermediaryCommissionCIK != "0001665160" ||
		string(byCIK.OfferingPrice) != "2" || byCIK.NetIncomeMostRecentFiscalYear != -964_551 ||
		byCIK.NetIncomePriorFiscalYear != -10_860 || byCIK.SecurityOfferedOtherDescription == nil ||
		*byCIK.SecurityOfferedOtherDescription != "Non-Voting Common Stock" {
		t.Fatalf("crowdfunding_offerings_by_cik = %+v", byCIK)
	}
	encoded, err := json.Marshal(byCIK)
	if err != nil {
		t.Fatal(err)
	}
	if text := string(encoded); !strings.Contains(text, `"offeringPrice":2,`) || !strings.Contains(text, `"date":"12-31-2021"`) {
		t.Fatalf("re-encoded row rewrote the offering price or the US date: %s", text)
	}
}

// Exact values copied from crates/libfmp/tests/fundraising_regulation_d_responses.rs:
// the required-present nullable flag and the empty-string date sentinel.
func TestDocumentedRegulationDOfferingsDecodeExactValues(t *testing.T) {
	t.Parallel()
	latest := assertFixtureParity[RegulationDOffering](t, "fundraising_latest.json")
	if len(latest) != 1 {
		t.Fatalf("rows = %d, want 1", len(latest))
	}
	row := latest[0]
	if row.CIK != "0002127786" || row.Date.String() != "2026-07-30" || row.FilingDate.String() != "2026-07-30 00:00:00" ||
		row.AcceptedDate.String() != "2026-07-30 13:05:23" || row.FormType != "D" ||
		row.IncorporatedWithinFiveYears == nil || !*row.IncorporatedWithinFiveYears ||
		row.YearOfIncorporation != "2026" || row.DateOfFirstSale != nil || row.TotalNumberAlreadyInvested != 0 ||
		row.IsAmendment || !row.SecuritiesOfferedAreOfEquityType {
		t.Fatalf("fundraising_latest = %+v", row)
	}
	if members := memberSet(t, row); len(members) != 43 {
		t.Fatalf("re-encoded members = %d, want the documented 43", len(members))
	}

	byCIK := assertFixtureParity[RegulationDOffering](t, "fundraising_by_cik.json")[0]
	if byCIK.CIK != "0001547416" || byCIK.IncorporatedWithinFiveYears != nil || byCIK.YearOfIncorporation != "" ||
		byCIK.DateOfFirstSale == nil || *byCIK.DateOfFirstSale != mustParseDate(t, "2014-02-14") ||
		byCIK.TotalOfferingAmount != 71_999_990 || byCIK.TotalNumberAlreadyInvested != 24 {
		t.Fatalf("fundraising_by_cik = %+v", byCIK)
	}
}

// Exact values copied from crates/libfmp/tests/fundraising_search_responses.rs.
// The crowdfunding date is a required key whose only documented value is
// null, so a future non-null value is kept raw and re-encoded unchanged.
func TestDocumentedOfferingSearchResultsDecodeExactValues(t *testing.T) {
	t.Parallel()
	crowdfunding := assertFixtureParity[CrowdfundingOfferingSearchResult](t, "crowdfunding_offerings_search.json")
	if want := (CrowdfundingOfferingSearchResult{CIK: "0001912939", Name: "Enotap LLC"}); len(crowdfunding) != 1 ||
		crowdfunding[0].CIK != want.CIK || crowdfunding[0].Name != want.Name || crowdfunding[0].Date != nil {
		t.Fatalf("crowdfunding_offerings_search = %+v", crowdfunding)
	}
	regulationD := assertFixtureParity[RegulationDOfferingSearchResult](t, "fundraising_search.json")
	want := RegulationDOfferingSearchResult{CIK: "0001547416", Name: "NJOY INC", Date: mustParseDateTime(t, "2014-02-28 16:00:25")}
	if len(regulationD) != 1 || regulationD[0] != want {
		t.Fatalf("fundraising_search = %+v", regulationD)
	}

	future := `{"cik":"0001912939","name":"Enotap LLC","date":{"provider":["shape",1]}}`
	var decoded CrowdfundingOfferingSearchResult
	if err := json.Unmarshal([]byte(future), &decoded); err != nil {
		t.Fatalf("future date shape rejected: %v", err)
	}
	if decoded.Date == nil || string(*decoded.Date) != `{"provider":["shape",1]}` {
		t.Fatalf("future date = %v, want the raw object", decoded.Date)
	}
	encoded, err := json.Marshal(decoded)
	if err != nil {
		t.Fatal(err)
	}
	if string(encoded) != future {
		t.Fatalf("re-encoded = %s, want %s", encoded, future)
	}
}

// Required keys, the three required-present nullable members (a string, a
// bool, and a raw value), the empty_date sentinel, and the number-kind check
// are enforced as the Rust decoder enforces them.
func TestFundraisingRequiredMembersAndCodecsAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	const crowdfunding = "crowdfunding_offerings_latest.json"
	const regulationD = "fundraising_latest.json"
	const search = "crowdfunding_offerings_search.json"
	decode := func(t *testing.T, fixture string, wire []byte) error {
		t.Helper()
		switch fixture {
		case crowdfunding:
			var rows []CrowdfundingOffering
			return json.Unmarshal(wire, &rows)
		case regulationD:
			var rows []RegulationDOffering
			return json.Unmarshal(wire, &rows)
		default:
			var rows []CrowdfundingOfferingSearchResult
			return json.Unmarshal(wire, &rows)
		}
	}
	rejected := []struct {
		name    string
		fixture string
		member  string
		value   jsontext.Value
		message string
	}{
		{"missing cik", crowdfunding, "cik", nil,
			`required member "cik" of CrowdfundingOffering is missing or null`},
		{"null cik", crowdfunding, "cik", jsontext.Value(`null`),
			`required member "cik" of CrowdfundingOffering is missing or null`},
		{"missing nullable description", crowdfunding, "securityOfferedOtherDescription", nil,
			`required member "securityOfferedOtherDescription" of CrowdfundingOffering is missing or null`},
		{"string offering price", crowdfunding, "offeringPrice", jsontext.Value(`"0.1"`),
			`member "offeringPrice" of CrowdfundingOffering must be a JSON number`},
		{"null offering price", crowdfunding, "offeringPrice", jsontext.Value(`null`),
			`required member "offeringPrice" of CrowdfundingOffering is missing or null`},
		{"missing nullable flag", regulationD, "incorporatedWithinFiveYears", nil,
			`required member "incorporatedWithinFiveYears" of RegulationDOffering is missing or null`},
		{"null empty_date", regulationD, "dateOfFirstSale", jsontext.Value(`null`),
			`member "dateOfFirstSale" of RegulationDOffering must be a JSON string`},
		{"missing empty_date", regulationD, "dateOfFirstSale", nil,
			`required member "dateOfFirstSale" of RegulationDOffering is missing or null`},
		{"missing search date", search, "date", nil,
			`required member "date" of CrowdfundingOfferingSearchResult is missing or null`},
	}
	for _, tc := range rejected {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			err := decode(t, tc.fixture, mutateFixtureMember(t, tc.fixture, tc.member, tc.value))
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode || typed.Message != tc.message {
				t.Fatalf("error = %v (%T), want CategoryDecode %q", err, err, tc.message)
			}
		})
	}
	accepted := []struct {
		name    string
		fixture string
		member  string
		value   jsontext.Value
	}{
		{"null description", crowdfunding, "securityOfferedOtherDescription", jsontext.Value(`null`)},
		{"integer offering price", crowdfunding, "offeringPrice", jsontext.Value(`2`)},
		{"null flag", regulationD, "incorporatedWithinFiveYears", jsontext.Value(`null`)},
		{"dated first sale", regulationD, "dateOfFirstSale", jsontext.Value(`"2014-02-14"`)},
		{"null search date", search, "date", jsontext.Value(`null`)},
	}
	for _, tc := range accepted {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			if err := decode(t, tc.fixture, mutateFixtureMember(t, tc.fixture, tc.member, tc.value)); err != nil {
				t.Fatalf("rejected: %v", err)
			}
		})
	}
	for _, wire := range []string{`[{"cik":"0001621902","date":"2011-11-22"}]`, `[{"cik":"0001621902","date":"11/22/2011"}]`} {
		if err := decode(t, crowdfunding, []byte(wire)); err == nil {
			t.Fatalf("a non-US date shape decoded: %s", wire)
		}
	}
	var wrongFlag []CrowdfundingOffering
	if err := json.Unmarshal(mutateFixtureMember(t, crowdfunding, "overSubscriptionAccepted", jsontext.Value(`true`)), &wrongFlag); err == nil {
		t.Fatal("a JSON bool decoded into the Y/N string flag")
	}
}
