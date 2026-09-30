package fmp

import (
	"encoding/json/v2"
	"errors"
	"math"
	"reflect"
	"strings"
	"testing"
)

// The 4 screener fixtures over the single response model, decoded through
// the shared parity helper (ADR 0030: identical bytes for the Rust and Go
// decoders). Every member is required and non-null; the unknown fixture
// carries one extra provider member on purpose.
func TestScreenerFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[CompanyScreenerResult](t, "company_screener.json")
	assertFixtureParity[CompanyScreenerResult](t, "company_screener_empty.json")
	assertFixtureParity[CompanyScreenerResult](t, "company_screener_multiple.json")
	assertFixtureParity[CompanyScreenerResult](t, "company_screener_unknown.json", "futureProviderField")
}

// Exact values copied from crates/libfmp/tests/screener_responses.rs
// (documented_company_screener_entry_decodes_every_exact_field).
func TestDocumentedCompanyScreenerResultDecodesExactValues(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[CompanyScreenerResult](t, "company_screener.json")
	want := CompanyScreenerResult{
		Symbol:             "AAPL",
		CompanyName:        "Apple Inc.",
		MarketCap:          4_885_602_246_714,
		Sector:             "Technology",
		Industry:           "Consumer Electronics",
		Beta:               new(1.097),
		Price:              new(332.64001),
		LastAnnualDividend: new(1.05),
		Volume:             29_909_012,
		Exchange:           "NASDAQ Global Select",
		ExchangeShortName:  "NASDAQ",
		Country:            new("US"),
		IsETF:              false,
		IsFund:             new(false),
		IsActivelyTrading:  true,
	}
	if len(rows) != 1 || !reflect.DeepEqual(rows[0], want) {
		t.Fatalf("company_screener = %+v, want %+v", rows, want)
	}

	// The re-encoded object uses the camelCase wire names, never the Rust
	// field names.
	encoded, err := json.Marshal(rows[0])
	if err != nil {
		t.Fatal(err)
	}
	for _, member := range []string{`"companyName":"Apple Inc."`, `"marketCap":4885602246714`, `"lastAnnualDividend":1.05`,
		`"exchangeShortName":"NASDAQ"`, `"isEtf":false`, `"isFund":false`, `"isActivelyTrading":true`} {
		if !strings.Contains(string(encoded), member) {
			t.Fatalf("CompanyScreenerResult re-encoded = %s, want it to contain %s", encoded, member)
		}
	}
	if strings.Contains(string(encoded), "company_name") {
		t.Fatalf("CompanyScreenerResult re-encoded with a snake_case member: %s", encoded)
	}
}

// Exact values copied from crates/libfmp/tests/screener_responses.rs
// (screener_arrays_preserve_empty_multiple_unknown_and_large_integer_values):
// marketCap and volume are float64, so 2^53+1 rounds to 2^53 and u64::MAX
// rounds to 2^64.
func TestScreenerArraysPreserveEmptyMultipleUnknownAndLargeIntegers(t *testing.T) {
	t.Parallel()
	empty := assertFixtureParity[CompanyScreenerResult](t, "company_screener_empty.json")
	if len(empty) != 0 {
		t.Fatalf("company_screener_empty = %+v, want no rows", empty)
	}

	multiple := assertFixtureParity[CompanyScreenerResult](t, "company_screener_multiple.json")
	if len(multiple) != 2 {
		t.Fatalf("company_screener_multiple = %+v, want two rows", multiple)
	}
	first := CompanyScreenerResult{
		Symbol: "BIG", CompanyName: "Beyond Float Precision Corp.", MarketCap: 9_007_199_254_740_992,
		Sector: "Future Sector", Industry: "Future Industry", Beta: new(0.0), Price: new(0.0),
		LastAnnualDividend: new(0.0), Volume: math.MaxUint64, Exchange: "Future Exchange", ExchangeShortName: "NEXT",
		Country: new("ZZ"), IsETF: false, IsFund: new(false), IsActivelyTrading: false,
	}
	if !reflect.DeepEqual(multiple[0], first) || multiple[0].MarketCap < 1<<53 {
		t.Fatalf("company_screener_multiple[0] = %+v, want %+v", multiple[0], first)
	}
	second := CompanyScreenerResult{
		Symbol: "FUND", CompanyName: "Example Fund", MarketCap: 4_294_967_296,
		Sector: "Financial Services", Industry: "Asset Management", Beta: new(1.25), Price: new(42.5),
		LastAnnualDividend: new(2.5), Volume: 4_294_967_296, Exchange: "New York Stock Exchange", ExchangeShortName: "NYSE",
		Country: new("US"), IsETF: false, IsFund: new(true), IsActivelyTrading: true,
	}
	if !reflect.DeepEqual(multiple[1], second) {
		t.Fatalf("company_screener_multiple[1] = %+v, want %+v", multiple[1], second)
	}

	unknown := assertFixtureParity[CompanyScreenerResult](t, "company_screener_unknown.json", "futureProviderField")
	if len(unknown) != 1 || unknown[0].Symbol != "AAPL" {
		t.Fatalf("company_screener_unknown = %+v, want the AAPL row", unknown)
	}

	// The contract is a bare array: an object envelope is rejected.
	var enveloped []CompanyScreenerResult
	if err := json.Unmarshal([]byte(`{"companies":[]}`), &enveloped); err == nil {
		t.Fatal("an object envelope decoded into a bare-array contract")
	}
}

// Issue #368: FMP sends null for beta, price, lastAnnualDividend, country, and
// isFund on some rows; a page carrying one null beta row decodes whole, and
// every nullable member decodes null to nil and re-encodes it as null.
func TestScreenerNullableMembersDecodeNullToNil(t *testing.T) {
	t.Parallel()
	page := assertFixtureParity[CompanyScreenerResult](t, "company_screener_null_beta_synthetic.json")
	if len(page) != 40 || page[37].Beta != nil || page[36].Beta == nil || *page[36].Beta != 1.097 {
		t.Fatalf("null beta page: %d rows, row 37 beta = %v, row 36 beta = %v", len(page), page[37].Beta, page[36].Beta)
	}

	row := string(readFixture(t, "company_screener.json"))
	for _, member := range []string{`"beta": 1.097`, `"price": 332.64001`, `"lastAnnualDividend": 1.05`, `"country": "US"`, `"isFund": false`} {
		name, _, _ := strings.Cut(member, ":")
		row = strings.Replace(row, member, name+": null", 1)
	}
	var rows []CompanyScreenerResult
	if err := json.Unmarshal([]byte(row), &rows); err != nil {
		t.Fatal(err)
	}
	got := rows[0]
	if got.Beta != nil || got.Price != nil || got.LastAnnualDividend != nil || got.Country != nil || got.IsFund != nil {
		t.Fatalf("null members = %+v, want nil pointers", got)
	}
	encoded, err := json.Marshal(got)
	if err != nil || !strings.Contains(string(encoded), `"beta":null`) || !strings.Contains(string(encoded), `"isFund":null`) {
		t.Fatalf("re-encoded = %s, %v", encoded, err)
	}
}

// The missing-required-member path of the generated decoder, once for this
// domain: serde rejects a missing and a null member alike, and the error
// names the model and the first offending member.
func TestScreenerRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	row := string(readFixture(t, "company_screener.json"))
	cases := []struct {
		name   string
		wire   string
		member string
	}{
		{"missing member", `[{"symbol":"AAPL","companyName":"Apple Inc."}]`, "marketCap"},
		{"null member", strings.Replace(row, `"marketCap": 4885602246714`, `"marketCap": null`, 1), "marketCap"},
		{"empty object", `[{}]`, "symbol"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []CompanyScreenerResult
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) || !strings.Contains(typed.Message, "CompanyScreenerResult") {
				t.Fatalf("message = %q, want it to name member %q of CompanyScreenerResult", typed.Message, tc.member)
			}
		})
	}
}
