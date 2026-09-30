package fmp

import (
	"encoding/json/v2"
	"errors"
	"math"
	"reflect"
	"strings"
	"testing"
)

// The 9 search fixtures over the 6 response models, decoded through the
// shared parity helper (ADR 0030: identical bytes for the Rust and Go
// decoders). search_symbol_unknown carries one intentionally unknown member
// with a nested object value.
func TestSearchFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	for _, name := range []string{"search_symbol.json", "search_empty.json", "search_symbol_multiple.json"} {
		assertFixtureParity[SymbolSearchResult](t, name)
	}
	assertFixtureParity[SymbolSearchResult](t, "search_symbol_unknown.json", "futureProviderField")
	assertFixtureParity[NameSearchResult](t, "search_name.json")
	assertFixtureParity[CIKSearchResult](t, "search_cik.json")
	assertFixtureParity[CUSIPSearchResult](t, "search_cusip.json")
	assertFixtureParity[ISINSearchResult](t, "search_isin.json")
	assertFixtureParity[ExchangeVariant](t, "search_exchange_variants.json")
}

// Exact values copied from crates/libfmp/tests/search_responses.rs
// (documented_symbol_and_name_results_decode_their_exact_fields and
// search_arrays_preserve_empty_multiple_and_unknown_field_shapes).
func TestDocumentedSymbolAndNameResultsDecodeExactValues(t *testing.T) {
	t.Parallel()
	symbols := assertFixtureParity[SymbolSearchResult](t, "search_symbol.json")
	wantSymbol := SymbolSearchResult{
		Symbol: "AAPL", Name: "Apple Inc.", Currency: "USD",
		ExchangeFullName: "NASDAQ Global Select", Exchange: "NASDAQ",
	}
	if len(symbols) != 1 || symbols[0] != wantSymbol {
		t.Fatalf("search_symbol = %+v, want %+v", symbols, wantSymbol)
	}
	names := assertFixtureParity[NameSearchResult](t, "search_name.json")
	wantName := NameSearchResult{
		Symbol: "AAGUSD", Name: "AAG USD", Currency: "USD", ExchangeFullName: "CCC", Exchange: "CRYPTO",
	}
	if len(names) != 1 || names[0] != wantName {
		t.Fatalf("search_name = %+v, want %+v", names, wantName)
	}
	encoded, err := json.Marshal(symbols[0])
	if err != nil || !strings.Contains(string(encoded), `"exchangeFullName":"NASDAQ Global Select"`) ||
		strings.Contains(string(encoded), `"companyName"`) {
		t.Fatalf("re-encoded symbol row = %s, %v", encoded, err)
	}

	empty := assertFixtureParity[SymbolSearchResult](t, "search_empty.json")
	multiple := assertFixtureParity[SymbolSearchResult](t, "search_symbol_multiple.json")
	unknown := assertFixtureParity[SymbolSearchResult](t, "search_symbol_unknown.json", "futureProviderField")
	if len(empty) != 0 || len(multiple) != 2 || multiple[0].Symbol != "000001.SZ" || multiple[1].Symbol != "^VIX" ||
		len(unknown) != 1 || unknown[0].Symbol != "AAPL" {
		t.Fatalf("empty = %+v, multiple = %+v, unknown = %+v", empty, multiple, unknown)
	}
}

// Exact values copied from crates/libfmp/tests/search_responses.rs
// (documented_identifier_results_preserve_wire_names_and_large_caps): the
// wire names stay as documented and market caps exceed the u32 range.
func TestDocumentedIdentifierResultsDecodeExactValues(t *testing.T) {
	t.Parallel()
	cik := assertFixtureParity[CIKSearchResult](t, "search_cik.json")
	wantCIK := CIKSearchResult{
		Symbol: "AAPL", CompanyName: "Apple Inc.", CIK: "0000320193",
		ExchangeFullName: "NASDAQ Global Select", Exchange: "NASDAQ", Currency: "USD",
	}
	if len(cik) != 1 || cik[0] != wantCIK {
		t.Fatalf("search_cik = %+v, want %+v", cik, wantCIK)
	}
	cusip := assertFixtureParity[CUSIPSearchResult](t, "search_cusip.json")
	wantCUSIP := CUSIPSearchResult{Symbol: "APC.F", CompanyName: "Apple Inc.", CUSIP: "037833100", MarketCap: 4_227_021_056_800}
	if len(cusip) != 1 || cusip[0] != wantCUSIP || cusip[0].MarketCap <= math.MaxUint32 {
		t.Fatalf("search_cusip = %+v, want %+v", cusip, wantCUSIP)
	}
	isin := assertFixtureParity[ISINSearchResult](t, "search_isin.json")
	wantISIN := ISINSearchResult{Symbol: "AAPL", Name: "Apple Inc.", ISIN: "US0378331005", MarketCap: 4_874_072_686_740}
	if len(isin) != 1 || isin[0] != wantISIN || isin[0].MarketCap <= math.MaxUint32 {
		t.Fatalf("search_isin = %+v, want %+v", isin, wantISIN)
	}

	cikWire, err := json.Marshal(cik[0])
	if err != nil || !strings.Contains(string(cikWire), `"companyName":"Apple Inc."`) ||
		strings.Contains(string(cikWire), `"name"`) {
		t.Fatalf("re-encoded cik row = %s, %v", cikWire, err)
	}
	cusipWire, err := json.Marshal(cusip[0])
	if err != nil || !strings.Contains(string(cusipWire), `"marketCap":4227021056800`) {
		t.Fatalf("re-encoded cusip row = %s, %v", cusipWire, err)
	}
	isinWire, err := json.Marshal(isin[0])
	if err != nil || !strings.Contains(string(isinWire), `"marketCap":4874072686740`) {
		t.Fatalf("re-encoded isin row = %s, %v", isinWire, err)
	}
}

// Exact values copied from crates/libfmp/tests/search_responses.rs
// (documented_exchange_variant_decodes_every_field_and_inverted_exchange_names):
// the provider's exchange member is the full name and exchangeShortName is
// the code, and the market cap is spelled mktCap on the wire.
func TestDocumentedExchangeVariantDecodesExactValues(t *testing.T) {
	t.Parallel()
	rows := assertFixtureParity[ExchangeVariant](t, "search_exchange_variants.json")
	if len(rows) != 1 {
		t.Fatalf("rows = %d, want 1", len(rows))
	}
	row := rows[0]
	want := ExchangeVariant{
		Symbol: "AAPL", Price: new(331.85501), Beta: 1.097, VolAvg: 55_309_000, MarketCap: 4_874_072_686_740,
		LastDiv: 1.05, Range: new("201.5-344.57"), Changes: new(-6.33498), CompanyName: "Apple Inc.", Currency: "USD",
		CIK: new("0000320193"), ISIN: new("US0378331005"), CUSIP: new("037833100"), Exchange: "NASDAQ Global Select",
		ExchangeShortName: "NASDAQ", Industry: "Consumer Electronics", Website: new("https://www.apple.com"),
		Description: row.Description, Ceo: new("Timothy D. Cook"), Sector: "Technology", Country: "US",
		FullTimeEmployees: new("166000"), Phone: new("(408) 996-1010"), Address: new("One Apple Park Way"),
		City: new("Cupertino"), State: new("CA"), Zip: new("95014"), DCFDiff: new(191.60731), DCF: 140.70269296445176,
		Image: "https://images.financialmodelingprep.com/symbol/AAPL.png", IPODate: mustParseDate(t, "1980-12-12"),
		DefaultImage: false, IsETF: false, IsActivelyTrading: true, IsAdr: false, IsFund: false,
	}
	if !reflect.DeepEqual(row, want) {
		t.Fatalf("search_exchange_variants[0] = %+v, want %+v", row, want)
	}
	if !strings.HasPrefix(row.Description, "Apple Inc. is a global") || row.MarketCap <= math.MaxUint32 {
		t.Fatalf("description = %q, marketCap = %v", row.Description, row.MarketCap)
	}
	encoded, err := json.Marshal(row)
	if err != nil {
		t.Fatal(err)
	}
	for _, member := range []string{`"volAvg":55309000`, `"mktCap":4874072686740`, `"changes":-6.33498`,
		`"exchange":"NASDAQ Global Select"`, `"exchangeShortName":"NASDAQ"`, `"fullTimeEmployees":"166000"`,
		`"range":"201.5-344.57"`, `"zip":"95014"`} {
		if !strings.Contains(string(encoded), member) {
			t.Fatalf("re-encoded ExchangeVariant = %s, want it to contain %s", encoded, member)
		}
	}
	for _, absent := range []string{`"marketCap"`, `"change"`} {
		if strings.Contains(string(encoded), absent) {
			t.Fatalf("re-encoded ExchangeVariant = %s, must not contain %s", encoded, absent)
		}
	}
}

// Issue #368: FMP sends null for 15 ExchangeVariant members on some listings
// and "" for cusip; both decode to nil, and nil re-encodes as null.
func TestExchangeVariantNullableMembersDecodeNullAndEmptyCUSIPToNil(t *testing.T) {
	t.Parallel()
	var wire []map[string]any
	if err := json.Unmarshal(readFixture(t, "search_exchange_variants.json"), &wire); err != nil {
		t.Fatal(err)
	}
	for _, member := range []string{"price", "range", "changes", "cik", "isin", "website", "ceo", "fullTimeEmployees",
		"phone", "address", "city", "state", "zip", "dcfDiff"} {
		wire[0][member] = nil
	}
	for _, cusip := range []any{nil, ""} {
		wire[0]["cusip"] = cusip
		body, err := json.Marshal(wire)
		if err != nil {
			t.Fatal(err)
		}
		var rows []ExchangeVariant
		if err := json.Unmarshal(body, &rows); err != nil {
			t.Fatalf("cusip %q: %v", cusip, err)
		}
		row := rows[0]
		for name, member := range map[string]any{"price": row.Price, "range": row.Range, "changes": row.Changes,
			"cik": row.CIK, "isin": row.ISIN, "cusip": row.CUSIP, "website": row.Website, "ceo": row.Ceo,
			"fullTimeEmployees": row.FullTimeEmployees, "phone": row.Phone, "address": row.Address, "city": row.City,
			"state": row.State, "zip": row.Zip, "dcfDiff": row.DCFDiff} {
			if !reflect.ValueOf(member).IsNil() {
				t.Fatalf("cusip %q: %s = %v, want nil", cusip, name, member)
			}
		}
		encoded, err := json.Marshal(row)
		if err != nil || !strings.Contains(string(encoded), `"cusip":null`) || !strings.Contains(string(encoded), `"dcfDiff":null`) {
			t.Fatalf("re-encoded = %s, %v", encoded, err)
		}
	}
}

// The missing-required-member path of the generated decoders, once for this
// domain: serde rejects a missing and a null member alike, and a malformed
// date never decodes into a Date member.
func TestSearchRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name   string
		wire   string
		member string
	}{
		{"missing member", `[{"symbol":"AAPL","name":"Apple Inc.","currency":"USD","exchangeFullName":"NASDAQ Global Select"}]`, "exchange"},
		{"null member", `[{"symbol":"AAPL","name":null,"currency":"USD","exchangeFullName":"NASDAQ Global Select","exchange":"NASDAQ"}]`, "name"},
		{"empty object", `[{}]`, "symbol"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []SymbolSearchResult
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) || !strings.Contains(typed.Message, "SymbolSearchResult") {
				t.Fatalf("message = %q, want it to name member %q of SymbolSearchResult", typed.Message, tc.member)
			}
		})
	}
	var variants []ExchangeVariant
	malformed := strings.Replace(string(readFixture(t, "search_exchange_variants.json")),
		`"ipoDate": "1980-12-12"`, `"ipoDate": "12/12/1980"`, 1)
	if err := json.Unmarshal([]byte(malformed), &variants); err == nil {
		t.Fatal("a malformed date decoded into a Date member")
	}
	var enveloped []SymbolSearchResult
	if err := json.Unmarshal([]byte(`{"results":[]}`), &enveloped); err == nil {
		t.Fatal("an object envelope decoded into a bare-array contract")
	}
}
