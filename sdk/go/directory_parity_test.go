package fmp

import (
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

// The 11 directory fixtures, one per response model, decoded through the
// shared parity helper (ADR 0030: identical bytes for the Rust and Go
// decoders).
func TestDirectoryFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[CompanySymbol](t, "directory_company_symbols.json")
	assertFixtureParity[FinancialStatementSymbol](t, "directory_financial_statement_symbols.json")
	assertFixtureParity[CIKListing](t, "directory_cik_list.json")
	assertFixtureParity[SymbolChange](t, "directory_symbol_changes.json")
	assertFixtureParity[ETFSymbol](t, "directory_etf_symbols.json")
	assertFixtureParity[ActivelyTradingSymbol](t, "directory_actively_trading.json")
	assertFixtureParity[EarningsTranscriptAvailability](t, "directory_earnings_transcript_list.json")
	assertFixtureParity[AvailableExchange](t, "directory_available_exchanges.json")
	assertFixtureParity[AvailableSector](t, "directory_available_sectors.json")
	assertFixtureParity[AvailableIndustry](t, "directory_available_industries.json")
	assertFixtureParity[AvailableCountry](t, "directory_available_countries.json")
}

// directoryAssertEmptyFixture decodes the shared empty-array fixture into []T
// and fails when any row appears, mirroring the Rust
// every_*_response_contract_preserves_empty_arrays tests.
func directoryAssertEmptyFixture[T any](t *testing.T) {
	t.Helper()
	if rows := assertFixtureParity[T](t, "directory_empty.json"); len(rows) != 0 {
		t.Fatalf("directory_empty.json decoded into %d %T rows, want none", len(rows), *new(T))
	}
}

func TestDirectoryEmptyFixtureDecodesIntoEveryModelWithoutRows(t *testing.T) {
	t.Parallel()
	directoryAssertEmptyFixture[CompanySymbol](t)
	directoryAssertEmptyFixture[FinancialStatementSymbol](t)
	directoryAssertEmptyFixture[CIKListing](t)
	directoryAssertEmptyFixture[SymbolChange](t)
	directoryAssertEmptyFixture[ETFSymbol](t)
	directoryAssertEmptyFixture[ActivelyTradingSymbol](t)
	directoryAssertEmptyFixture[EarningsTranscriptAvailability](t)
	directoryAssertEmptyFixture[AvailableExchange](t)
	directoryAssertEmptyFixture[AvailableSector](t)
	directoryAssertEmptyFixture[AvailableIndustry](t)
	directoryAssertEmptyFixture[AvailableCountry](t)
}

// Exact values copied from crates/libfmp/tests/directory_responses.rs.
func TestDocumentedDirectoryRowsDecodeExactFieldNamesAndWireTypes(t *testing.T) {
	t.Parallel()
	companies := assertFixtureParity[CompanySymbol](t, "directory_company_symbols.json")
	if want := (CompanySymbol{Symbol: "URBANCO.BO", CompanyName: "Urban Company Limited"}); len(companies) != 1 ||
		companies[0] != want {
		t.Fatalf("directory_company_symbols = %+v, want %+v", companies, want)
	}
	financials := assertFixtureParity[FinancialStatementSymbol](t, "directory_financial_statement_symbols.json")
	if want := (FinancialStatementSymbol{Symbol: "RMES.CN", CompanyName: "Red Metal Resources Ltd.",
		TradingCurrency: "CAD", ReportingCurrency: "USD"}); len(financials) != 1 || financials[0] != want {
		t.Fatalf("directory_financial_statement_symbols = %+v, want %+v", financials, want)
	}
	ciks := assertFixtureParity[CIKListing](t, "directory_cik_list.json")
	if want := (CIKListing{CIK: "0002137358", CompanyName: "Osotspa Public Co Limited/ADR"}); len(ciks) != 1 ||
		ciks[0] != want {
		t.Fatalf("directory_cik_list = %+v, want %+v", ciks, want)
	}
	changes := assertFixtureParity[SymbolChange](t, "directory_symbol_changes.json")
	if want := (SymbolChange{Date: mustParseDate(t, "2026-07-28"), CompanyName: "Yarrow Bioscience, Inc. Common Stock",
		OldSymbol: "VYNE", NewSymbol: "YARW"}); len(changes) != 1 || changes[0] != want {
		t.Fatalf("directory_symbol_changes = %+v, want %+v", changes, want)
	}
	etfs := assertFixtureParity[ETFSymbol](t, "directory_etf_symbols.json")
	if want := (ETFSymbol{Symbol: "P60.SI",
		Name: "MULTI-UNITS LUXEMBOURG - Lyxor MSCI AC Asia Pacific Ex Japan UCITS ETF"}); len(etfs) != 1 ||
		etfs[0] != want {
		t.Fatalf("directory_etf_symbols = %+v, want %+v", etfs, want)
	}
	active := assertFixtureParity[ActivelyTradingSymbol](t, "directory_actively_trading.json")
	if want := (ActivelyTradingSymbol{Symbol: "URBANCO.BO", Name: "Urban Company Limited"}); len(active) != 1 ||
		active[0] != want {
		t.Fatalf("directory_actively_trading = %+v, want %+v", active, want)
	}
	transcripts := assertFixtureParity[EarningsTranscriptAvailability](t, "directory_earnings_transcript_list.json")
	if want := (EarningsTranscriptAvailability{Symbol: "INBS", CompanyName: "Intelligent Bio Solutions Inc.",
		NoOfTranscripts: "6"}); len(transcripts) != 1 || transcripts[0] != want {
		t.Fatalf("directory_earnings_transcript_list = %+v, want %+v", transcripts, want)
	}

	// The company-symbol and ETF directories spell their name member
	// differently, and the transcript count stays the exact wire string.
	if members := memberSet(t, companies[0]); strings.Join(members, ",") != "companyName,symbol" {
		t.Fatalf("CompanySymbol members = %v, want companyName and symbol", members)
	}
	if members := memberSet(t, etfs[0]); strings.Join(members, ",") != "name,symbol" {
		t.Fatalf("ETFSymbol members = %v, want name and symbol", members)
	}
	encoded, err := json.Marshal(transcripts[0])
	if err != nil || !strings.Contains(string(encoded), `"noOfTranscripts":"6"`) {
		t.Fatalf("EarningsTranscriptAvailability re-encoded = %s, %v", encoded, err)
	}
}

// Exact values copied from crates/libfmp/tests/directory_taxonomy_responses.rs.
func TestDocumentedTaxonomyRowsDecodeExactFieldNamesAndOpenValues(t *testing.T) {
	t.Parallel()
	exchanges := assertFixtureParity[AvailableExchange](t, "directory_available_exchanges.json")
	want := AvailableExchange{Exchange: "AMEX", Name: "New York Stock Exchange Arca",
		CountryName: "United States of America", CountryCode: "US", SymbolSuffix: "N/A", Delay: "Real-time"}
	if len(exchanges) != 1 || exchanges[0] != want {
		t.Fatalf("directory_available_exchanges = %+v, want %+v", exchanges, want)
	}
	sectors := assertFixtureParity[AvailableSector](t, "directory_available_sectors.json")
	if len(sectors) != 1 || sectors[0].Sector != "Basic Materials" {
		t.Fatalf("directory_available_sectors = %+v", sectors)
	}
	industries := assertFixtureParity[AvailableIndustry](t, "directory_available_industries.json")
	if len(industries) != 1 || industries[0].Industry != "Steel" {
		t.Fatalf("directory_available_industries = %+v", industries)
	}
	countries := assertFixtureParity[AvailableCountry](t, "directory_available_countries.json")
	if len(countries) != 1 || countries[0].Country != "FK" {
		t.Fatalf("directory_available_countries = %+v", countries)
	}
	encoded, err := json.Marshal(exchanges[0])
	if err != nil {
		t.Fatal(err)
	}
	for _, member := range []string{`"countryName":"United States of America"`, `"countryCode":"US"`,
		`"symbolSuffix":"N/A"`, `"delay":"Real-time"`} {
		if !strings.Contains(string(encoded), member) {
			t.Fatalf("AvailableExchange re-encoded = %s, want it to contain %s", encoded, member)
		}
	}
}

// The missing-required-member path of the generated decoders, once for this
// domain: serde rejects a missing and a null member alike, and a malformed
// date never decodes into a Date member.
func TestDirectoryRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name   string
		wire   string
		member string
	}{
		{"missing member", `[{"date":"2026-07-28","companyName":"Yarrow","oldSymbol":"VYNE"}]`, "newSymbol"},
		{"null member", `[{"date":"2026-07-28","companyName":null,"oldSymbol":"VYNE","newSymbol":"YARW"}]`, "companyName"},
		{"empty object", `[{}]`, "date"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []SymbolChange
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) || !strings.Contains(typed.Message, "SymbolChange") {
				t.Fatalf("message = %q, want it to name member %q of SymbolChange", typed.Message, tc.member)
			}
		})
	}
	var rows []SymbolChange
	if err := json.Unmarshal([]byte(`[{"date":"2026/07/28","companyName":"Yarrow","oldSymbol":"VYNE","newSymbol":"YARW"}]`),
		&rows); err == nil {
		t.Fatal("a malformed date decoded into a Date member")
	}
	var countries []AvailableCountry
	err := json.Unmarshal([]byte(`[{"country":null}]`), &countries)
	var typed *Error
	if !errors.As(err, &typed) || !strings.Contains(typed.Message, `"country"`) {
		t.Fatalf("AvailableCountry error = %v, want it to name the null member country", err)
	}
}
