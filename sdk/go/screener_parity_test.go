package fmp

import (
	"encoding/json/v2"
	"errors"
	"math"
	"strings"
	"testing"
)

// The 4 screener fixtures over the single response model, decoded through
// the shared parity helper (ADR 0030: identical bytes for the Rust and Go
// decoders). Every member is required and non-null; the unknown fixture
// carries one extra provider member on purpose.
func TestScreenerFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[CompanyScreenerEntry](t, "company_screener.json")
	assertFixtureParity[CompanyScreenerEntry](t, "company_screener_empty.json")
	assertFixtureParity[CompanyScreenerEntry](t, "company_screener_multiple.json")
	assertFixtureParity[CompanyScreenerEntry](t, "company_screener_unknown.json", "futureProviderField")
}

// Exact values copied from crates/libfmp/tests/screener_responses.rs
// (documented_company_screener_entry_decodes_every_exact_field).
func TestDocumentedCompanyScreenerEntryDecodesExactValues(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[CompanyScreenerEntry](t, "company_screener.json")
	want := CompanyScreenerEntry{
		Symbol:             "AAPL",
		CompanyName:        "Apple Inc.",
		MarketCap:          4_885_602_246_714,
		Sector:             "Technology",
		Industry:           "Consumer Electronics",
		Beta:               1.097,
		Price:              332.64001,
		LastAnnualDividend: 1.05,
		Volume:             29_909_012,
		Exchange:           "NASDAQ Global Select",
		ExchangeShortName:  "NASDAQ",
		Country:            "US",
		IsEtf:              false,
		IsFund:             false,
		IsActivelyTrading:  true,
	}
	if len(rows) != 1 || rows[0] != want {
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
			t.Fatalf("CompanyScreenerEntry re-encoded = %s, want it to contain %s", encoded, member)
		}
	}
	if strings.Contains(string(encoded), "company_name") {
		t.Fatalf("CompanyScreenerEntry re-encoded with a snake_case member: %s", encoded)
	}
}

// Exact values copied from crates/libfmp/tests/screener_responses.rs
// (screener_arrays_preserve_empty_multiple_unknown_and_large_integer_values):
// marketCap and volume are float64, so 2^53+1 rounds to 2^53 and u64::MAX
// rounds to 2^64.
func TestScreenerArraysPreserveEmptyMultipleUnknownAndLargeIntegers(t *testing.T) {
	t.Parallel()
	empty := assertFixtureParity[CompanyScreenerEntry](t, "company_screener_empty.json")
	if len(empty) != 0 {
		t.Fatalf("company_screener_empty = %+v, want no rows", empty)
	}

	multiple := assertFixtureParity[CompanyScreenerEntry](t, "company_screener_multiple.json")
	if len(multiple) != 2 {
		t.Fatalf("company_screener_multiple = %+v, want two rows", multiple)
	}
	first := CompanyScreenerEntry{
		Symbol: "BIG", CompanyName: "Beyond Float Precision Corp.", MarketCap: 9_007_199_254_740_992,
		Sector: "Future Sector", Industry: "Future Industry", Beta: 0, Price: 0, LastAnnualDividend: 0,
		Volume: math.MaxUint64, Exchange: "Future Exchange", ExchangeShortName: "NEXT", Country: "ZZ",
		IsEtf: false, IsFund: false, IsActivelyTrading: false,
	}
	if multiple[0] != first || multiple[0].MarketCap < 1<<53 {
		t.Fatalf("company_screener_multiple[0] = %+v, want %+v", multiple[0], first)
	}
	second := CompanyScreenerEntry{
		Symbol: "FUND", CompanyName: "Example Fund", MarketCap: 4_294_967_296,
		Sector: "Financial Services", Industry: "Asset Management", Beta: 1.25, Price: 42.5, LastAnnualDividend: 2.5,
		Volume: 4_294_967_296, Exchange: "New York Stock Exchange", ExchangeShortName: "NYSE", Country: "US",
		IsEtf: false, IsFund: true, IsActivelyTrading: true,
	}
	if multiple[1] != second {
		t.Fatalf("company_screener_multiple[1] = %+v, want %+v", multiple[1], second)
	}

	unknown := assertFixtureParity[CompanyScreenerEntry](t, "company_screener_unknown.json", "futureProviderField")
	if len(unknown) != 1 || unknown[0].Symbol != "AAPL" {
		t.Fatalf("company_screener_unknown = %+v, want the AAPL row", unknown)
	}

	// The contract is a bare array: an object envelope is rejected.
	var enveloped []CompanyScreenerEntry
	if err := json.Unmarshal([]byte(`{"companies":[]}`), &enveloped); err == nil {
		t.Fatal("an object envelope decoded into a bare-array contract")
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
			var rows []CompanyScreenerEntry
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) || !strings.Contains(typed.Message, "CompanyScreenerEntry") {
				t.Fatalf("message = %q, want it to name member %q of CompanyScreenerEntry", typed.Message, tc.member)
			}
		})
	}
}
