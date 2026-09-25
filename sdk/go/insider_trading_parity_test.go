package fmp

import (
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

// The 6 insider_trading fixtures over the 5 response models, decoded through
// the shared parity helper (ADR 0030: identical bytes for the Rust and Go
// decoders). Every member is required and non-null, so no fixture carries an
// unknown or omitted member.
func TestInsiderTradingFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[InsiderTrade](t, "latest_insider_trades.json")
	assertFixtureParity[InsiderTrade](t, "searched_insider_trades.json")
	assertFixtureParity[InsiderReportingName](t, "insider_reporting_names.json")
	assertFixtureParity[InsiderTransactionType](t, "insider_transaction_types.json")
	assertFixtureParity[InsiderTradeStatistics](t, "insider_trade_statistics.json")
	assertFixtureParity[BeneficialOwnershipAcquisition](t, "beneficial_ownership_acquisitions.json")
}

// Exact values copied from crates/libfmp/tests/insider_trading_responses.rs:
// the latest and search feeds share the 16-member trade row.
func TestDocumentedInsiderTradeRowsDecodeExactValues(t *testing.T) {
	t.Parallel()
	latest := assertFixtureParity[InsiderTrade](t, "latest_insider_trades.json")
	searched := assertFixtureParity[InsiderTrade](t, "searched_insider_trades.json")
	if len(latest) != 1 || len(searched) != 1 || latest[0] != searched[0] {
		t.Fatalf("latest = %+v, searched = %+v, want one identical row each", latest, searched)
	}
	want := InsiderTrade{
		Symbol:                   "TRMK",
		FilingDate:               mustParseDate(t, "2026-07-30"),
		TransactionDate:          mustParseDate(t, "2026-07-28"),
		ReportingCik:             "0001661867",
		CompanyCik:               "0000036146",
		TransactionType:          "A-Award",
		SecuritiesOwned:          62_959,
		ReportingName:            "Tate Granville Jr",
		TypeOfOwner:              "officer: Secretary",
		AcquisitionOrDisposition: "A",
		DirectOrIndirect:         "D",
		FormType:                 "4",
		SecuritiesTransacted:     1_608,
		Price:                    0,
		SecurityName:             "Common Stock",
		URL:                      "https://www.sec.gov/Archives/edgar/data/36146/000003614626000087/0000036146-26-000087-index.htm",
	}
	if latest[0] != want {
		t.Fatalf("latest_insider_trades[0] = %+v, want %+v", latest[0], want)
	}

	// Share quantities decode as float64 and an integer price decodes as a price
	// (counts_preserve_u64_quantities_decode_as_f64_and_integer_prices_decode_as_prices).
	var rows []InsiderTrade
	if err := json.Unmarshal([]byte(`[{"symbol":"TRMK","filingDate":"2026-07-30","transactionDate":"2026-07-28",`+
		`"reportingCik":"0001661867","companyCik":"0000036146","transactionType":"A-Award",`+
		`"securitiesOwned":1500.5,"reportingName":"Tate Granville Jr","typeOfOwner":"officer: Secretary",`+
		`"acquisitionOrDisposition":"A","directOrIndirect":"D","formType":"4","securitiesTransacted":-3,`+
		`"price":225,"securityName":"Common Stock","url":"https://example.invalid"}]`), &rows); err != nil {
		t.Fatal(err)
	}
	if rows[0].SecuritiesOwned != 1500.5 || rows[0].SecuritiesTransacted != -3 || rows[0].Price != 225.0 {
		t.Fatalf("share quantities or integer price were not preserved: %+v", rows[0])
	}
}

// Exact values copied from crates/libfmp/tests/insider_trading_responses.rs
// for the reporting-name, taxonomy, statistics, and ownership models.
func TestDocumentedInsiderReferenceRowsDecodeExactValues(t *testing.T) {
	t.Parallel()
	names := assertFixtureParity[InsiderReportingName](t, "insider_reporting_names.json")
	if want := (InsiderReportingName{ReportingCik: "0001548760", ReportingName: "Zuckerberg Mark"}); len(names) != 1 ||
		names[0] != want {
		t.Fatalf("insider_reporting_names = %+v, want %+v", names, want)
	}
	types := assertFixtureParity[InsiderTransactionType](t, "insider_transaction_types.json")
	if len(types) != 1 || types[0].TransactionType != "A-Award" {
		t.Fatalf("insider_transaction_types = %+v", types)
	}

	statistics := assertFixtureParity[InsiderTradeStatistics](t, "insider_trade_statistics.json")
	wantStatistics := InsiderTradeStatistics{
		Symbol: "AAPL", Cik: "0000320193", Year: 2026, Quarter: 2,
		AcquiredTransactions: 7, DisposedTransactions: 40, AcquiredDisposedRatio: 0.175,
		TotalAcquired: 303_199, TotalDisposed: 927_380,
		AverageAcquired: 43_314.1429, AverageDisposed: 23_184.5,
		TotalPurchases: 0, TotalSales: 14,
	}
	if len(statistics) != 1 || statistics[0] != wantStatistics {
		t.Fatalf("insider_trade_statistics = %+v, want %+v", statistics, wantStatistics)
	}
	var full []InsiderTradeStatistics
	if err := json.Unmarshal([]byte(`[{"symbol":"AAPL","cik":"0000320193","year":2026,"quarter":2,`+
		`"acquiredTransactions":18446744073709551615,"disposedTransactions":1,"acquiredDisposedRatio":0.1,`+
		`"totalAcquired":1,"totalDisposed":1,"averageAcquired":1,"averageDisposed":1,"totalPurchases":1,"totalSales":1}]`),
		&full); err != nil || full[0].AcquiredTransactions != 1<<64-1 {
		t.Fatalf("u64 statistics count was not preserved: %+v, %v", full, err)
	}

	ownership := assertFixtureParity[BeneficialOwnershipAcquisition](t, "beneficial_ownership_acquisitions.json")
	wantOwnership := BeneficialOwnershipAcquisition{
		Cik: "0000320193", Symbol: "AAPL",
		FilingDate: mustParseDate(t, "2026-04-29"), AcceptedDate: mustParseDate(t, "2026-04-29"),
		Cusip: "037833100", NameOfReportingPerson: "Vanguard Capital Management",
		CitizenshipOrPlaceOfOrganization: "PENNSYLVANIA",
		SoleVotingPower:                  "0", SharedVotingPower: "0", SoleDispositivePower: "0", SharedDispositivePower: "0",
		AmountBeneficiallyOwned: "1099168953", PercentOfClass: "7.48", TypeOfReportingPerson: "IA",
		URL: "https://www.sec.gov/Archives/edgar/data/320193/000210011926000139/xslSCHEDULE_13G_X02/primary_doc.xml",
	}
	if len(ownership) != 1 || ownership[0] != wantOwnership {
		t.Fatalf("beneficial_ownership_acquisitions = %+v, want %+v", ownership, wantOwnership)
	}
	// The quoted numeric members stay the exact wire text, never scaled or
	// re-spelled (quoted_numeric_ownership_fields_preserve_exact_decimal_text).
	encoded, err := json.Marshal(ownership[0])
	if err != nil {
		t.Fatal(err)
	}
	for _, member := range []string{`"amountBeneficiallyOwned":"1099168953"`, `"percentOfClass":"7.48"`, `"soleVotingPower":"0"`} {
		if !strings.Contains(string(encoded), member) {
			t.Fatalf("BeneficialOwnershipAcquisition re-encoded = %s, want it to contain %s", encoded, member)
		}
	}
}

// Every contract is a bare array that preserves empty and multiple rows and
// rejects an object envelope
// (all_six_contracts_are_bare_arrays_preserving_empty_and_multiple_rows).
func TestInsiderTradingContractsAreBareArrays(t *testing.T) {
	t.Parallel()
	row := strings.TrimSpace(string(readFixture(t, "latest_insider_trades.json")))
	row = strings.TrimSuffix(strings.TrimPrefix(row, "["), "]")
	var two []InsiderTrade
	if err := json.Unmarshal([]byte("["+row+","+row+"]"), &two); err != nil || len(two) != 2 || two[0] != two[1] {
		t.Fatalf("two-row array = %+v, %v", two, err)
	}
	var empty []InsiderTrade
	if err := json.Unmarshal([]byte(`[]`), &empty); err != nil || len(empty) != 0 {
		t.Fatalf("empty array = %+v, %v", empty, err)
	}
	var enveloped []InsiderTrade
	if err := json.Unmarshal([]byte(`{"trades":[]}`), &enveloped); err == nil {
		t.Fatal("an object envelope decoded into a bare-array contract")
	}
}

// The missing-required-member path of the generated decoders, once for this
// domain: serde rejects a missing and a null member alike, and a malformed
// date never decodes into a Date member.
func TestInsiderTradingRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name   string
		wire   string
		member string
	}{
		{"missing member", `[{"reportingCik":"0001548760"}]`, "reportingName"},
		{"null member", `[{"reportingCik":null,"reportingName":"Zuckerberg Mark"}]`, "reportingCik"},
		{"empty object", `[{}]`, "reportingCik"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []InsiderReportingName
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) || !strings.Contains(typed.Message, "InsiderReportingName") {
				t.Fatalf("message = %q, want it to name member %q of InsiderReportingName", typed.Message, tc.member)
			}
		})
	}
	var rows []BeneficialOwnershipAcquisition
	malformed := strings.Replace(string(readFixture(t, "beneficial_ownership_acquisitions.json")),
		`"acceptedDate": "2026-04-29"`, `"acceptedDate": "2026/04/29"`, 1)
	if err := json.Unmarshal([]byte(malformed), &rows); err == nil {
		t.Fatal("a malformed date decoded into a Date member")
	}
	var statistics []InsiderTradeStatistics
	err := json.Unmarshal([]byte(`[{"symbol":"AAPL","cik":"0000320193","year":2026,"quarter":null}]`), &statistics)
	var typed *Error
	if !errors.As(err, &typed) || !strings.Contains(typed.Message, `"quarter"`) {
		t.Fatalf("InsiderTradeStatistics error = %v, want it to name the null member quarter", err)
	}
}
