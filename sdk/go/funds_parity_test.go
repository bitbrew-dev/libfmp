package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"math"
	"slices"
	"strings"
	"testing"
)

// The nine funds fixtures, one per registry method. Every member is required
// and non-null in the Rust models, so no fixture carries an intentionally
// unknown member. fund_disclosure_dates.json decodes into the reused
// institutional_ownership Form13fFilingDate model.
func TestFundsFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[ETFFundHolding](t, "etf_fund_holdings.json")
	assertFixtureParity[ETFFundInfo](t, "etf_fund_info.json")
	assertFixtureParity[ETFCountryWeighting](t, "etf_country_weightings.json")
	assertFixtureParity[ETFAssetExposure](t, "etf_asset_exposure.json")
	assertFixtureParity[ETFSectorWeighting](t, "etf_sector_weightings.json")
	assertFixtureParity[FundDisclosureHolder](t, "latest_fund_disclosure_holders.json")
	assertFixtureParity[FundDisclosure](t, "fund_disclosures.json")
	assertFixtureParity[FundDisclosureSearchResult](t, "fund_disclosure_holder_search.json")
	assertFixtureParity[Form13fFilingDate](t, "fund_disclosure_dates.json")
}

// Exact values copied from crates/libfmp/tests/funds_responses.rs; the member
// counts mirror the Rust assert_field_count calls (9, 19, 2, 5, 3).
func TestDocumentedEtfFixturesDecodeExactly(t *testing.T) {
	t.Parallel()
	holdings := assertFixtureParity[ETFFundHolding](t, "etf_fund_holdings.json")
	wantHolding := ETFFundHolding{
		Symbol: "SPY", Asset: "AAPL", Name: "APPLE INC", Isin: "US0378331005", SecurityCusip: "037833100",
		SharesNumber: 181_418_073, WeightPercentage: 7.79997012, MarketValue: 61_679_458_958.0,
		UpdatedAt: mustParseDateTime(t, "2026-07-30 08:07:21"),
	}
	if len(holdings) != 1 || holdings[0] != wantHolding {
		t.Fatalf("etf_fund_holdings = %+v, want %+v", holdings, wantHolding)
	}
	if got := memberSet(t, holdings[0]); len(got) != 9 {
		t.Fatalf("re-encoded holding members = %d, want 9", len(got))
	}

	info := assertFixtureParity[ETFFundInfo](t, "etf_fund_info.json")
	wantSectors := []ETFSectorExposure{
		{Industry: "Basic Materials", Exposure: 1.6916311902850854},
		{Industry: "Cash & Others", Exposure: 0.30489782336177595},
		{Industry: "Communication Services", Exposure: 9.23211353485037},
	}
	if len(info) != 1 || info[0].Symbol != "SPY" || info[0].Name != "State Street SPDR S&P 500 ETF" ||
		!strings.Contains(info[0].Description, "It also can`t reinvest") || info[0].Isin != "US78462F1030" ||
		info[0].AssetClass != "Equity" || info[0].SecurityCusip != "78462F103" || info[0].Domicile != "US" ||
		info[0].ETFCompany != "SPDR" || info[0].ExpenseRatio != 0.09 ||
		info[0].AssetsUnderManagement != 777_349_860_000 || info[0].AvgVolume != 52_093_933 ||
		info[0].InceptionDate != mustParseDate(t, "1993-01-22") || info[0].Nav != 729.27 ||
		info[0].NavCurrency != "USD" || info[0].HoldingsCount != 504 || !info[0].IsActivelyTrading ||
		info[0].UpdatedAt != "2026-07-30T16:00:20.049Z" || !slices.Equal(info[0].SectorsList, wantSectors) {
		t.Fatalf("etf_fund_info = %+v", info)
	}
	if got := memberSet(t, info[0]); len(got) != 19 {
		t.Fatalf("re-encoded info members = %d, want 19", len(got))
	}

	countries := assertFixtureParity[ETFCountryWeighting](t, "etf_country_weightings.json")
	wantCountry := ETFCountryWeighting{Country: "United States", WeightPercentage: "97.26%"}
	if len(countries) != 1 || countries[0] != wantCountry {
		t.Fatalf("etf_country_weightings = %+v, want %+v", countries, wantCountry)
	}
	// The fixture spells weightPercentage as 10.100000000000001, which Go
	// parses correctly rounded to the f64 above 10.1; serde_json's default
	// float parser lands on 10.1, which is what the Rust test asserts.
	assets := assertFixtureParity[ETFAssetExposure](t, "etf_asset_exposure.json")
	wantAsset := ETFAssetExposure{
		Symbol: "ZWT-T.TO", Asset: "AAPL", SharesNumber: 42_372, WeightPercentage: 10.100000000000001,
		MarketValue: 20_141_231.66,
	}
	if len(assets) != 1 || assets[0] != wantAsset {
		t.Fatalf("etf_asset_exposure = %+v, want %+v", assets, wantAsset)
	}
	sectors := assertFixtureParity[ETFSectorWeighting](t, "etf_sector_weightings.json")
	wantSector := ETFSectorWeighting{Symbol: "SPY", Sector: "Basic Materials", WeightPercentage: 1.6916311902850854}
	if len(sectors) != 1 || sectors[0] != wantSector {
		t.Fatalf("etf_sector_weightings = %+v, want %+v", sectors, wantSector)
	}
	if got := memberSet(t, countries[0]); len(got) != 2 {
		t.Fatalf("re-encoded country members = %d, want 2", len(got))
	}
	if got := memberSet(t, assets[0]); len(got) != 5 {
		t.Fatalf("re-encoded asset members = %d, want 5", len(got))
	}
	if got := memberSet(t, sectors[0]); len(got) != 3 {
		t.Fatalf("re-encoded sector members = %d, want 3", len(got))
	}
}

// Exact values copied from crates/libfmp/tests/funds_responses.rs; the member
// counts mirror the Rust assert_field_count calls (7, 23, 13, 3). The 23
// disclosure members prove the cur_cd rename survives the round trip.
func TestDocumentedFundDisclosureFixturesDecodeExactly(t *testing.T) {
	t.Parallel()
	holders := assertFixtureParity[FundDisclosureHolder](t, "latest_fund_disclosure_holders.json")
	wantHolder := FundDisclosureHolder{
		CIK: "0000866256", Holder: "PARNASSUS INCOME FUNDS", SecurityCusip: "037833100", Shares: 3_638_451,
		DateReported: mustParseDate(t, "2026-06-30"), Change: -316_881, WeightPercent: 4.06607721,
	}
	if len(holders) != 1 || holders[0] != wantHolder {
		t.Fatalf("latest_fund_disclosure_holders = %+v, want %+v", holders, wantHolder)
	}
	if got := memberSet(t, holders[0]); len(got) != 7 {
		t.Fatalf("re-encoded holder members = %d, want 7", len(got))
	}

	disclosures := assertFixtureParity[FundDisclosure](t, "fund_disclosures.json")
	wantDisclosure := FundDisclosure{
		CIK: "0000857489", Date: mustParseDate(t, "2023-10-31"), AcceptedDate: mustParseDateTime(t, "2023-12-28 09:26:13"),
		Symbol: "000089.SZ", Name: "Shenzhen Airport Co Ltd", Lei: "3003009W045RIKRBZI44", Title: "SHENZ AIRPORT-A",
		Cusip: "N/A", Isin: "CNE000000VK1", Balance: 2_438_784, Units: "NS", CurrencyCode: "CNY",
		ValUsd: 2_255_873.6, PctVal: 0.0023838966190458206, PayoffProfile: "Long", AssetCat: "EC", IssuerCat: "CORP",
		InvCountry: "CN", IsRestrictedSec: "N", FairValLevel: "2", IsCashCollateral: "N", IsNonCashCollateral: "N",
		IsLoanByFund: "N",
	}
	if len(disclosures) != 1 || disclosures[0] != wantDisclosure {
		t.Fatalf("fund_disclosures = %+v, want %+v", disclosures, wantDisclosure)
	}
	members := memberValues(t, disclosures[0])
	if len(members) != 23 || string(members["cur_cd"]) != `"CNY"` {
		t.Fatalf("re-encoded disclosure members = %d with cur_cd %s, want 23 and \"CNY\"", len(members), members["cur_cd"])
	}

	results := assertFixtureParity[FundDisclosureSearchResult](t, "fund_disclosure_holder_search.json")
	wantResult := FundDisclosureSearchResult{
		Symbol: "FGOAX", CIK: "0000355691", ClassID: "C000024574", SeriesID: "S000009042",
		EntityName: "Federated Hermes Government Income Securities, Inc.", EntityOrgType: "30",
		SeriesName: "Federated Hermes Government Income Securities, Inc.", ClassName: "Class A Shares",
		ReportingFileNumber: "811-03266", Address: "4000 ERICSSON DRIVE", City: "WARRENDALE", ZipCode: "15086-7561",
		State: "PA",
	}
	if len(results) != 1 || results[0] != wantResult {
		t.Fatalf("fund_disclosure_holder_search = %+v, want %+v", results, wantResult)
	}
	if got := memberSet(t, results[0]); len(got) != 13 {
		t.Fatalf("re-encoded search members = %d, want 13", len(got))
	}

	dates := assertFixtureParity[Form13fFilingDate](t, "fund_disclosure_dates.json")
	wantDate := Form13fFilingDate{Date: mustParseDate(t, "2026-04-30"), Year: 2026, Quarter: 2}
	if len(dates) != 1 || dates[0] != wantDate {
		t.Fatalf("fund_disclosure_dates = %+v, want %+v", dates, wantDate)
	}
}

// fundsRewrite replaces members of the first fixture row with raw JSON text.
func fundsRewrite(t *testing.T, fixture string, members map[string]string) []byte {
	t.Helper()
	var rows []map[string]jsontext.Value
	if err := json.Unmarshal(readFixture(t, fixture), &rows); err != nil {
		t.Fatal(err)
	}
	for member, value := range members {
		rows[0][member] = jsontext.Value(value)
	}
	encoded, err := json.Marshal(rows)
	if err != nil {
		t.Fatal(err)
	}
	return encoded
}

// Mirrors integer_widths_signed_change_and_decimal_market_values_are_preserved.
func TestFundsExtremeIntegersDecodeIntoQuantityAmountAndCountMembers(t *testing.T) {
	t.Parallel()
	const maxUint64 = "18446744073709551615"
	var holdings []ETFFundHolding
	if err := json.Unmarshal(fundsRewrite(t, "etf_fund_holdings.json",
		map[string]string{"sharesNumber": maxUint64}), &holdings); err != nil || holdings[0].SharesNumber != math.MaxUint64 {
		t.Fatalf("sharesNumber = %+v, %v", holdings, err)
	}
	var info []ETFFundInfo
	if err := json.Unmarshal(fundsRewrite(t, "etf_fund_info.json", map[string]string{
		"assetsUnderManagement": maxUint64, "avgVolume": maxUint64, "holdingsCount": maxUint64,
	}), &info); err != nil || info[0].AssetsUnderManagement != math.MaxUint64 ||
		info[0].AvgVolume != math.MaxUint64 || info[0].HoldingsCount != math.MaxUint64 {
		t.Fatalf("info integers = %+v, %v", info, err)
	}
	var holders []FundDisclosureHolder
	if err := json.Unmarshal(fundsRewrite(t, "latest_fund_disclosure_holders.json",
		map[string]string{"shares": maxUint64, "change": "-9223372036854775808"}), &holders); err != nil ||
		holders[0].Shares != math.MaxUint64 || holders[0].Change != math.MinInt64 {
		t.Fatalf("holder integers = %+v, %v", holders, err)
	}
	var disclosures []FundDisclosure
	if err := json.Unmarshal(fundsRewrite(t, "fund_disclosures.json",
		map[string]string{"balance": maxUint64, "valUsd": "0.125"}), &disclosures); err != nil ||
		disclosures[0].Balance != math.MaxUint64 || disclosures[0].ValUsd != 0.125 {
		t.Fatalf("disclosure values = %+v, %v", disclosures, err)
	}
}

// Mirrors exact_allocation_fixtures_preserve_string_and_numeric_percent_kinds
// and identifiers_flags_and_temporal_wire_kinds_are_not_coerced: a member of
// the wrong JSON kind is rejected, never coerced. The IsoTimestamp member
// updatedAt of ETFFundInfo is a plain string under the ADR 0030 type table,
// so the Rust rejection of a space-separated timestamp there has no Go
// mirror.
func TestFundsMembersRejectTheWrongWireKind(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name    string
		fixture string
		member  string
		value   string
		decode  func([]byte) error
	}{
		{"country percent as number", "etf_country_weightings.json", "weightPercentage", "97.26",
			func(b []byte) error { var r []ETFCountryWeighting; return json.Unmarshal(b, &r) }},
		{"asset percent as string", "etf_asset_exposure.json", "weightPercentage", `"10.1%"`,
			func(b []byte) error { var r []ETFAssetExposure; return json.Unmarshal(b, &r) }},
		{"sector percent as string", "etf_sector_weightings.json", "weightPercentage", `"1.69%"`,
			func(b []byte) error { var r []ETFSectorWeighting; return json.Unmarshal(b, &r) }},
		{"info trading flag as text", "etf_fund_info.json", "isActivelyTrading", `"Y"`,
			func(b []byte) error { var r []ETFFundInfo; return json.Unmarshal(b, &r) }},
		{"holding updatedAt as ISO timestamp", "etf_fund_holdings.json", "updatedAt", `"2026-07-30T08:07:21.000Z"`,
			func(b []byte) error { var r []ETFFundHolding; return json.Unmarshal(b, &r) }},
		{"disclosure date with time", "fund_disclosures.json", "date", `"2023-10-31 00:00:00"`,
			func(b []byte) error { var r []FundDisclosure; return json.Unmarshal(b, &r) }},
		{"disclosure acceptedDate without time", "fund_disclosures.json", "acceptedDate", `"2023-12-28"`,
			func(b []byte) error { var r []FundDisclosure; return json.Unmarshal(b, &r) }},
		{"holder shares as text", "latest_fund_disclosure_holders.json", "shares", `"3638451"`,
			func(b []byte) error { var r []FundDisclosureHolder; return json.Unmarshal(b, &r) }},
	}
	for _, flag := range []string{"isRestrictedSec", "isCashCollateral", "isNonCashCollateral", "isLoanByFund"} {
		cases = append(cases, struct {
			name    string
			fixture string
			member  string
			value   string
			decode  func([]byte) error
		}{flag + " as bool", "fund_disclosures.json", flag, "false",
			func(b []byte) error { var r []FundDisclosure; return json.Unmarshal(b, &r) }})
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			if err := tc.decode(fundsRewrite(t, tc.fixture, map[string]string{tc.member: tc.value})); err == nil {
				t.Fatalf("%s %s decoded", tc.member, tc.value)
			}
		})
	}
}

// The missing-required-member path of the generated decoders, once for this
// domain, including the nested ETFSectorExposure rows of ETFFundInfo: serde
// rejects a missing and a null member alike, and tolerates an unknown one.
func TestFundsRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name   string
		wire   string
		member string
		model  string
	}{
		{"missing member", `[{"symbol":"SPY","weightPercentage":1.69}]`, "sector", "ETFSectorWeighting"},
		{"null member", `[{"symbol":"SPY","sector":null,"weightPercentage":1.69}]`, "sector", "ETFSectorWeighting"},
		{"empty object", `[{}]`, "symbol", "ETFSectorWeighting"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []ETFSectorWeighting
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) || !strings.Contains(typed.Message, tc.model) {
				t.Fatalf("message = %q, want it to name member %q of %s", typed.Message, tc.member, tc.model)
			}
		})
	}

	source := string(readFixture(t, "etf_fund_info.json"))
	nested := []struct {
		name string
		old  string
		new  string
	}{
		{"missing nested industry", `"industry": "Basic Materials"`, `"industryRenamed": "Basic Materials"`},
		{"null nested industry", `"industry": "Basic Materials"`, `"industry": null`},
		{"missing nested exposure", `"exposure": 1.6916311902850854,`, ``},
		{"null nested exposure", `"exposure": 1.6916311902850854,`, `"exposure": null,`},
	}
	for _, tc := range nested {
		if !strings.Contains(source, tc.old) {
			t.Fatalf("%s: fixture text %q not found", tc.name, tc.old)
		}
		var rows []ETFFundInfo
		err := json.Unmarshal([]byte(strings.Replace(source, tc.old, tc.new, 1)), &rows)
		var typed *Error
		if !errors.As(err, &typed) || typed.Category != CategoryDecode || !strings.Contains(typed.Message, "ETFSectorExposure") {
			t.Fatalf("%s: error = %v, want a CategoryDecode *Error naming ETFSectorExposure", tc.name, err)
		}
	}
	forward := strings.Replace(source, `"industry": "Basic Materials"`,
		`"industry": "Basic Materials", "futureProviderField": [1, true]`, 1)
	var rows []ETFFundInfo
	if err := json.Unmarshal([]byte(forward), &rows); err != nil || len(rows) != 1 || len(rows[0].SectorsList) != 3 {
		t.Fatalf("nested unknown member was not ignored: %+v, %v", rows, err)
	}
}
