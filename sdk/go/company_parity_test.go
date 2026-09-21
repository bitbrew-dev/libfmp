package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

// Every shared fixture a Rust test decodes into a company model, through the
// ADR 0030 parity helper (crates/libfmp/tests/company_*_responses.rs).
func TestCompanyFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	for _, name := range []string{"company_profile.json", "company_profile_multiple.json", "company_empty.json"} {
		assertFixtureParity[CompanyProfile](t, name)
	}
	assertFixtureParity[CompanyProfile](t, "company_profile_unknown.json", "futureField")
	assertFixtureParity[CompanyNote](t, "company_note.json")
	assertFixtureParity[StockPeer](t, "stock_peer.json")
	assertFixtureParity[DelistedCompany](t, "company_delisted.json")
	assertFixtureParity[EmployeeCount](t, "company_employee_count.json")
	for _, name := range []string{"company_market_capitalization.json",
		"company_historical_market_capitalization.json"} {
		assertFixtureParity[MarketCapitalizationRecord](t, name)
	}
	assertFixtureParity[CompanyShareFloat](t, "company_shares_float.json")
	assertFixtureParity[AllSharesFloatRecord](t, "company_shares_float_all.json")
	for _, name := range []string{"company_mergers_acquisitions_latest.json",
		"company_mergers_acquisitions_search.json", "company_mergers_acquisitions_multiple.json"} {
		assertFixtureParity[MergerAcquisition](t, name)
	}
	assertFixtureParity[MergerAcquisition](t, "company_mergers_acquisitions_unknown.json", "futureField")
	for _, name := range []string{"company_key_executives.json", "company_key_executives_dynamic.json"} {
		assertFixtureParity[CompanyExecutive](t, name)
	}
	for _, name := range []string{"company_executive_compensation.json",
		"company_executive_compensation_large.json"} {
		assertFixtureParity[ExecutiveCompensation](t, name)
	}
	assertFixtureParity[ExecutiveCompensationBenchmark](t, "company_executive_compensation_benchmark.json")
}

// Exact values copied from crates/libfmp/tests/company_responses.rs.
func TestDocumentedCompanyProfileNoteAndPeerDecodeExactValues(t *testing.T) {
	t.Parallel()
	profiles := assertFixtureParity[CompanyProfile](t, "company_profile.json")
	if len(profiles) != 1 {
		t.Fatalf("rows = %d, want 1", len(profiles))
	}
	profile := profiles[0]
	want := CompanyProfile{
		Symbol: "AAPL", Price: 331.85501, MarketCap: 4_874_072_686_740, Beta: 1.097, LastDividend: 1.05,
		Range: "201.5-344.57", Change: -6.33498, ChangePercentage: -1.8732, Volume: 28_718_014,
		AverageVolume: 55_309_000, CompanyName: "Apple Inc.", Currency: "USD", Cik: "0000320193",
		Isin: "US0378331005", Cusip: "037833100", ExchangeFullName: "NASDAQ Global Select", Exchange: "NASDAQ",
		Industry: "Consumer Electronics", Website: "https://www.apple.com", Description: profile.Description,
		Ceo: "Timothy D. Cook", Sector: "Technology", Country: "US", FullTimeEmployees: "166000",
		Phone: "(408) 996-1010", Address: "One Apple Park Way", City: "Cupertino", State: "CA", Zip: "95014",
		Image: "https://images.financialmodelingprep.com/symbol/AAPL.png", IpoDate: mustDate(t, "1980-12-12"),
		DefaultImage: false, IsEtf: false, IsActivelyTrading: true, IsAdr: false, IsFund: false,
	}
	if profile != want {
		t.Fatalf("profile = %+v, want %+v", profile, want)
	}
	if !strings.HasPrefix(profile.Description, "Apple Inc. is a global technology corporation") {
		t.Fatalf("description = %q", profile.Description)
	}
	encoded, err := json.Marshal(profile)
	if err != nil || !strings.Contains(string(encoded), `"fullTimeEmployees":"166000"`) ||
		!strings.Contains(string(encoded), `"marketCap":4874072686740`) {
		t.Fatalf("re-encoded profile = %s, %v", encoded, err)
	}

	multiple := assertFixtureParity[CompanyProfile](t, "company_profile_multiple.json")
	if len(multiple) != 2 || multiple[0].MarketCap != 9_007_199_254_740_993 || multiple[0].Volume != 18_446_744_073_709_551_615 ||
		multiple[0].AverageVolume != 4_294_967_296 || multiple[0].FullTimeEmployees != "42" ||
		multiple[0].Sector != "Future Sector" || multiple[0].Industry != "Future Industry" {
		t.Fatalf("company_profile_multiple = %+v", multiple)
	}
	if unknown := assertFixtureParity[CompanyProfile](t, "company_profile_unknown.json", "futureField"); len(unknown) != 1 ||
		unknown[0].Symbol != "AAPL" {
		t.Fatalf("company_profile_unknown = %+v", unknown)
	}
	if empty := assertFixtureParity[CompanyProfile](t, "company_empty.json"); empty == nil || len(empty) != 0 {
		t.Fatalf("company_empty = %#v, want a non-nil empty slice", empty)
	}

	notes := assertFixtureParity[CompanyNote](t, "company_note.json")
	if want := (CompanyNote{Cik: "0000320193", Symbol: "AAPL", Title: "0.000% Notes due 2025", Exchange: "NASDAQ"}); len(notes) != 1 ||
		notes[0] != want {
		t.Fatalf("company_note = %+v", notes)
	}
	peers := assertFixtureParity[StockPeer](t, "stock_peer.json")
	if want := (StockPeer{Symbol: "GOOGL", CompanyName: "Alphabet Inc.", Price: 333.84, MarketCap: 4_040_168_831_718}); len(peers) != 1 ||
		peers[0] != want {
		t.Fatalf("stock_peer = %+v", peers)
	}
	if encoded, err := json.Marshal(peers[0]); err != nil || !strings.Contains(string(encoded), `"mktCap":4040168831718`) {
		t.Fatalf("re-encoded peer = %s, %v", encoded, err)
	}
}

// Exact values copied from crates/libfmp/tests/company_workforce_responses.rs
// and company_market_data_responses.rs.
func TestDocumentedWorkforceAndMarketDataFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	delisted := assertFixtureParity[DelistedCompany](t, "company_delisted.json")
	if want := (DelistedCompany{Symbol: "CCIX", CompanyName: "Churchill Capital Corp IX Ordinary Shares", Exchange: "NASDAQ",
		IpoDate: mustDate(t, "2007-03-01"), DelistedDate: mustDate(t, "2026-07-28")}); len(delisted) != 1 || delisted[0] != want {
		t.Fatalf("company_delisted = %+v", delisted)
	}
	employees := assertFixtureParity[EmployeeCount](t, "company_employee_count.json")
	if want := (EmployeeCount{Symbol: "AAPL", Cik: "0000320193", AcceptanceTime: mustDateTime(t, "2025-10-31 06:01:26"),
		PeriodOfReport: mustDate(t, "2025-09-27"), CompanyName: "Apple Inc.", FormType: "10-K",
		FilingDate: mustDate(t, "2025-10-31"), EmployeeCount: 166_000,
		Source: "https://www.sec.gov/Archives/edgar/data/320193/000032019325000079/0000320193-25-000079-index.htm"}); len(employees) != 1 ||
		employees[0] != want {
		t.Fatalf("company_employee_count = %+v", employees)
	}

	caps := assertFixtureParity[MarketCapitalizationRecord](t, "company_market_capitalization.json")
	if want := (MarketCapitalizationRecord{Symbol: "AAPL", Date: mustDate(t, "2026-07-30"), MarketCap: 4_874_072_686_740}); len(caps) != 1 ||
		caps[0] != want {
		t.Fatalf("company_market_capitalization = %+v", caps)
	}
	historical := assertFixtureParity[MarketCapitalizationRecord](t, "company_historical_market_capitalization.json")
	if len(historical) != 1 || historical[0].Symbol != "AAPL" || historical[0].MarketCap != 4_879_177_245_542 {
		t.Fatalf("company_historical_market_capitalization = %+v", historical)
	}
	floats := assertFixtureParity[CompanyShareFloat](t, "company_shares_float.json")
	if want := (CompanyShareFloat{Symbol: "AAPL", Date: mustDateTime(t, "2026-07-30 15:48:00"), FreeFloat: 99.83000000136171,
		FloatShares: 14_662_387_495, OutstandingShares: 14_687_356_000,
		Source: "https://www.sec.gov/Archives/edgar/data/320193/000032019326000013/aapl-20260328.htm"}); len(floats) != 1 ||
		floats[0] != want {
		t.Fatalf("company_shares_float = %+v", floats)
	}
	all := assertFixtureParity[AllSharesFloatRecord](t, "company_shares_float_all.json")
	if want := (AllSharesFloatRecord{Symbol: "000001.SZ", Date: mustDateTime(t, "2026-07-29 14:23:30"), FreeFloat: 41.40900000201062,
		FloatShares: 8_035_796_667, OutstandingShares: 19_405_918_198}); len(all) != 1 || all[0] != want {
		t.Fatalf("company_shares_float_all = %+v", all)
	}
}

// Exact values copied from crates/libfmp/tests/company_mergers_acquisitions_responses.rs
// and company_governance_responses.rs.
func TestDocumentedMergerAndGovernanceFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	latest := assertFixtureParity[MergerAcquisition](t, "company_mergers_acquisitions_latest.json")
	if want := (MergerAcquisition{Symbol: "AGH", CompanyName: "Aureus Greenway Holdings Inc", Cik: "0002009312",
		TargetedCompanyName: "Aureus Greenway Holdings, Inc.", TargetedCik: "0002009312", TargetedSymbol: "PUSA",
		TransactionDate: mustDate(t, "2026-07-29"), AcceptedDate: mustDateTime(t, "2026-07-29 16:00:46"),
		Link: "https://www.sec.gov/Archives/edgar/data/2009312/000149315226035181/forms-4.htm"}); len(latest) != 1 ||
		latest[0] != want {
		t.Fatalf("company_mergers_acquisitions_latest = %+v", latest)
	}
	search := assertFixtureParity[MergerAcquisition](t, "company_mergers_acquisitions_search.json")
	if len(search) != 1 || search[0].Symbol != "PEGY" || search[0].Cik != "0000022701" || search[0].TargetedSymbol != "JCS" ||
		search[0].TransactionDate != mustDate(t, "2021-11-12") {
		t.Fatalf("company_mergers_acquisitions_search = %+v", search)
	}
	multiple := assertFixtureParity[MergerAcquisition](t, "company_mergers_acquisitions_multiple.json")
	if len(multiple) != 2 || multiple[0].TargetedSymbol != "PUSA" || multiple[1].TargetedSymbol != "JCS" {
		t.Fatalf("company_mergers_acquisitions_multiple = %+v", multiple)
	}

	executives := assertFixtureParity[CompanyExecutive](t, "company_key_executives.json")
	if len(executives) != 1 || executives[0].Title != "Vice President of Worldwide Communications" ||
		executives[0].Name != "Kristin Huguet Quayle" || executives[0].CurrencyPay != "USD" ||
		executives[0].Gender != "female" || !executives[0].Active {
		t.Fatalf("company_key_executives = %+v", executives)
	}
	if executives[0].Pay != nil || executives[0].YearBorn != nil || executives[0].TitleSince != nil {
		t.Fatalf("null dynamic members decoded as non-nil: %+v", executives[0])
	}
	dynamic := assertFixtureParity[CompanyExecutive](t, "company_key_executives_dynamic.json")
	if len(dynamic) != 2 || dynamic[1].Active {
		t.Fatalf("company_key_executives_dynamic = %+v", dynamic)
	}
	assertCanonicalJSON(t, dynamic[1].Pay, `{"amount":"00123.450","components":[1,true,null]}`)
	assertCanonicalJSON(t, dynamic[1].YearBorn, `"01980"`)
	assertCanonicalJSON(t, dynamic[1].TitleSince, `1704067200`)

	compensation := assertFixtureParity[ExecutiveCompensation](t, "company_executive_compensation.json")
	if want := (ExecutiveCompensation{Cik: "0000320193", Symbol: "AAPL", CompanyName: "Apple Inc.",
		FilingDate: mustDate(t, "2026-01-08"), AcceptedDate: mustDateTime(t, "2026-01-08 16:31:36"),
		NameAndPosition: "Luca Maestri Former Senior Vice President, Chief Financial Officer", Year: 2025, Salary: 819_231,
		Bonus: 0, StockAward: 13_003_031, OptionAward: 0, IncentivePlanCompensation: 1_638_462,
		AllOtherCompensation: 22_204, Total: 15_482_928,
		Link: "https://www.sec.gov/Archives/edgar/data/320193/000130817926000008/0001308179-26-000008-index.htm"}); len(compensation) != 1 ||
		compensation[0] != want {
		t.Fatalf("company_executive_compensation = %+v", compensation)
	}
	large := assertFixtureParity[ExecutiveCompensation](t, "company_executive_compensation_large.json")
	if len(large) != 2 || large[1].Salary != 5_000_000_000 || large[1].AllOtherCompensation != 10_000_000_000 ||
		large[1].Total != 18_446_744_073_709_551_615 {
		t.Fatalf("company_executive_compensation_large = %+v", large)
	}
	benchmark := assertFixtureParity[ExecutiveCompensationBenchmark](t, "company_executive_compensation_benchmark.json")
	if want := (ExecutiveCompensationBenchmark{IndustryTitle: "ABRASIVE, ASBESTOS & MISC NONMETALLIC MINERAL PRODS", Year: 2024,
		AverageCompensation: 784_407.5555555555}); len(benchmark) != 1 || benchmark[0] != want {
		t.Fatalf("company_executive_compensation_benchmark = %+v", benchmark)
	}
}

// The missing-required-member path for the company domain: the all-company
// share-float payload has no source member, so it must not decode as
// CompanyShareFloat (mirroring the Rust is_err assertion), and the generated
// decoder names the first missing or null member like serde does.
func TestCompanyRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	var floats []CompanyShareFloat
	err := json.Unmarshal(readFixture(t, "company_shares_float_all.json"), &floats)
	assertCompanyDecodeError(t, err, "CompanyShareFloat", "source")

	cases := []struct {
		name   string
		wire   string
		member string
	}{
		{"missing member", `[{"symbol":"AAPL","marketCap":1}]`, "date"},
		{"null member", `[{"symbol":"AAPL","date":"2026-07-30","marketCap":null}]`, "marketCap"},
		{"empty object", `[{}]`, "symbol"},
		{"null element", `[null]`, "symbol"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []MarketCapitalizationRecord
			assertCompanyDecodeError(t, json.Unmarshal([]byte(tc.wire), &rows), "MarketCapitalizationRecord", tc.member)
		})
	}

	var executives []CompanyExecutive
	wire := `[{"title":"CEO","name":"A","pay":null,"currencyPay":"USD","gender":"male","yearBorn":null,"titleSince":null}]`
	assertCompanyDecodeError(t, json.Unmarshal([]byte(wire), &executives), "CompanyExecutive", "active")
}

func assertCompanyDecodeError(t *testing.T, err error, model, member string) {
	t.Helper()
	var typed *Error
	if !errors.As(err, &typed) || typed.Category != CategoryDecode {
		t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
	}
	if !strings.Contains(typed.Message, `"`+member+`"`) || !strings.Contains(typed.Message, model) {
		t.Fatalf("message = %q, want it to name member %q of %s", typed.Message, member, model)
	}
}

// assertCanonicalJSON compares a decoded dynamic member with its expected
// JSON after RFC 8785 canonicalization, so fixture whitespace cannot matter.
func assertCanonicalJSON(t *testing.T, got *jsontext.Value, want string) {
	t.Helper()
	if got == nil {
		t.Fatalf("dynamic member is nil, want %s", want)
	}
	canonical := got.Clone()
	if err := canonical.Canonicalize(); err != nil {
		t.Fatalf("canonicalize %s: %v", *got, err)
	}
	if string(canonical) != want {
		t.Fatalf("dynamic member = %s, want %s", canonical, want)
	}
}

func mustDate(t *testing.T, value string) Date {
	t.Helper()
	date, err := ParseDate(value)
	if err != nil {
		t.Fatalf("ParseDate(%q): %v", value, err)
	}
	return date
}

func mustDateTime(t *testing.T, value string) DateTime {
	t.Helper()
	dt, err := ParseDateTime(value)
	if err != nil {
		t.Fatalf("ParseDateTime(%q): %v", value, err)
	}
	return dt
}
