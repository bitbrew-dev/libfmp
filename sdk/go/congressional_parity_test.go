package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"reflect"
	"strings"
	"testing"
)

// congressionalTradeFixtures are the eight fixtures the Rust round-trip test
// decodes into CongressionalTrade (crates/libfmp/tests/congressional_responses.rs).
var congressionalTradeFixtures = []string{
	"congress_senate_latest.json",
	"congress_house_latest.json",
	"congress_senate_trades.json",
	"congress_senate_trades_by_name.json",
	"congress_senate_trades_by_id.json",
	"congress_house_trades.json",
	"congress_house_trades_by_name.json",
	"congress_house_trades_by_id.json",
}

// Every shared fixture a Rust test decodes into a congressional model,
// through the ADR 0030 parity helper. CongressionalDebtDetails and
// CongressionalNetWorthRange have no fixture of their own: they are proven
// as nested members of the net-worth entry.
func TestCongressionalFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	for _, name := range congressionalTradeFixtures {
		assertFixtureParity[CongressionalTrade](t, name)
	}
	assertFixtureParity[CongressionalMemberProfile](t, "congress_senate_profile.json")
	assertFixtureParity[CongressionalMemberPosition](t, "congress_senate_positions.json")
	assertFixtureParity[CongressionalMemberNetWorth](t, "congress_senate_net_worth.json")
	assertFixtureParity[CongressionalMemberNetWorthAggregate](t, "congress_senate_net_worth_aggregated.json")
}

// Exact values copied from crates/libfmp/tests/congressional_responses.rs:
// the by-name Senate row keeps its empty symbol and its "False" flag, and
// the latest Senate row has no capital-gains member at all, so it decodes to
// nil and is omitted again on re-encoding (skip_serializing_if in Rust).
func TestDocumentedCongressionalTradesDecodeExactValues(t *testing.T) {
	t.Parallel()
	byName := assertFixtureParity[CongressionalTrade](t, "congress_senate_trades_by_name.json")
	if len(byName) != 1 {
		t.Fatalf("rows = %d, want 1", len(byName))
	}
	got := byName[0]
	flag := got.CapitalGainsOver200Usd
	got.CapitalGainsOver200Usd = nil
	want := CongressionalTrade{
		Symbol: "", MemberID: "M000934", DisclosureDate: mustParseDate(t, "2026-07-21"),
		TransactionDate: mustParseDate(t, "2026-06-23"), FirstName: "Jerry", LastName: "Moran", Office: "Jerry Moran",
		District: "KS", Owner: "Self", AssetDescription: "Berkshire Hathaway Inc", AssetType: "Stock",
		TransactionType: "Purchase", Amount: "$15,001 - $50,000", Comment: "",
		Link: "https://efdsearch.senate.gov/search/view/ptr/cff197e2-b90b-4b53-b97d-eecf226a1980/",
	}
	if got != want {
		t.Fatalf("congress_senate_trades_by_name = %+v, want %+v", got, want)
	}
	if flag == nil || *flag != "False" {
		t.Fatalf("capitalGainsOver200USD = %v, want \"False\"", flag)
	}

	latest := assertFixtureParity[CongressionalTrade](t, "congress_senate_latest.json")
	if len(latest) != 1 || latest[0].MemberID != "M001242" || latest[0].Symbol != "CM" ||
		latest[0].AssetType != "Corporate Bond" || latest[0].CapitalGainsOver200Usd != nil {
		t.Fatalf("congress_senate_latest = %+v", latest)
	}
	encoded, err := json.Marshal(latest[0])
	if err != nil || strings.Contains(string(encoded), "capitalGainsOver200USD") ||
		!strings.Contains(string(encoded), `"senateID":"M001242"`) || !strings.Contains(string(encoded), `"type":"Sale"`) {
		t.Fatalf("re-encoded latest row = %s, %v", encoded, err)
	}

	house := assertFixtureParity[CongressionalTrade](t, "congress_house_latest.json")
	if members := memberSet(t, house[0]); len(members) != 16 || house[0].District != "FL23" ||
		house[0].TransactionDate != mustParseDate(t, "2026-06-17") {
		t.Fatalf("congress_house_latest = %+v with members %v", house, members)
	}
	for _, name := range congressionalTradeFixtures[2:] {
		rows := assertFixtureParity[CongressionalTrade](t, name)
		if len(rows) != 1 || rows[0].CapitalGainsOver200Usd == nil || *rows[0].CapitalGainsOver200Usd != "False" {
			t.Fatalf("%s = %+v, want one row with a \"False\" flag", name, rows)
		}
	}
}

// Exact values copied from crates/libfmp/tests/congressional_member_endpoints.rs
// and congressional_net_worth_endpoints.rs.
func TestDocumentedMemberAndNetWorthFixturesDecodeExactValues(t *testing.T) {
	t.Parallel()
	profiles := assertFixtureParity[CongressionalMemberProfile](t, "congress_senate_profile.json")
	if want := (CongressionalMemberProfile{MemberID: "L000397", FirstName: "Zoe", LastName: "Lofgren",
		BirthDate: mustParseDate(t, "1947-12-20"), LatestParty: "Democrat", LatestState: "CA", LatestPosition: "Representative",
		Image: new("https://images.financialmodelingprep.com/senate/L000397.jpg"), Active: true,
		YearsActive: 31.6}); len(profiles) != 1 || !reflect.DeepEqual(profiles[0], want) {
		t.Fatalf("congress_senate_profile = %+v", profiles)
	}

	positions := assertFixtureParity[CongressionalMemberPosition](t, "congress_senate_positions.json")
	if len(positions) != 1 {
		t.Fatalf("rows = %d, want 1", len(positions))
	}
	position := positions[0]
	if position.EndDate != nil {
		t.Fatalf("endDate = %v, want nil for null", position.EndDate)
	}
	if position.MemberID != "Z000018" || position.CongressNumber != 119 || position.StartDate != mustParseDate(t, "2025-01-02") ||
		position.Party != "Republican" || position.Position != "Representative" || position.State != "MT" ||
		position.YearsInTerm != 0.7 {
		t.Fatalf("congress_senate_positions = %+v", position)
	}
	if encoded, err := json.Marshal(position); err != nil || !strings.Contains(string(encoded), `"endDate":null`) {
		t.Fatalf("re-encoded position = %s, %v", encoded, err)
	}

	entries := assertFixtureParity[CongressionalMemberNetWorth](t, "congress_senate_net_worth.json")
	if len(entries) != 1 {
		t.Fatalf("rows = %d, want 1", len(entries))
	}
	entry := entries[0]
	if entry.MemberID != "P000197" || entry.FormType != "House Report" || entry.Year != 2022 ||
		entry.FilingDate != mustParseDate(t, "2023-05-15") || entry.Section != "Liabilities" ||
		!reflect.DeepEqual(entry.Category, new("Mortgage & Real Estate Liability")) ||
		!reflect.DeepEqual(entry.Name, new("Union Bank of California")) ||
		entry.AssetType != "Mortgage on 2640 Broadway, San Francisco, CA" || !reflect.DeepEqual(entry.Owner, new("Joint")) ||
		!reflect.DeepEqual(entry.Value, new(3_000_001.0)) ||
		entry.Link != "https://disclosures-clerk.house.gov/public_disc/financial-pdfs/2022/10053231.pdf" {
		t.Fatalf("congress_senate_net_worth = %+v", entry)
	}
	if entry.IncomeType != nil || entry.Comment != nil || entry.IncomeRange != nil || entry.Income != nil {
		t.Fatalf("null members decoded as non-nil: %+v", entry)
	}
	if !reflect.DeepEqual(entry.DebtDetails, &CongressionalDebtDetails{DateIncurred: new("September 2007")}) {
		t.Fatalf("debtDetails = %+v", entry.DebtDetails)
	}
	if !reflect.DeepEqual(entry.ValueRange, &CongressionalNetWorthRange{Min: 1_000_001, Max: new(int64(5_000_000))}) {
		t.Fatalf("valueRange = %+v", entry.ValueRange)
	}

	totals := assertFixtureParity[CongressionalMemberNetWorthAggregate](t, "congress_senate_net_worth_aggregated.json")
	if want := (CongressionalMemberNetWorthAggregate{MemberID: "P000197", Year: 2024, Total: 225_219_551,
		RealEstateLiabilities: new(27_000_005.0), CashAndCashEquivalents: 291_009, BusinessAndSelfEmployment: new(0.0),
		RealEstate: new(45_032_504.0), OwnershipInterest: new(70_140_014.0), Stock: new(136_748_525.0), Options: new(0.0),
		RevolvingAndCreditLines: new(1_500_002.0), AssetBackedSecurities: new(4_475_006.0), BusinessLiabilities: new(3_000_001.0),
		MutualFundsAndETFs: 32_501}); len(totals) != 1 || !reflect.DeepEqual(totals[0], want) {
		t.Fatalf("congress_senate_net_worth_aggregated = %+v", totals)
	}
	if encoded, err := json.Marshal(totals[0]); err != nil || !strings.Contains(string(encoded), `"mutualFundsAndETFs":32501`) {
		t.Fatalf("re-encoded aggregate = %s, %v", encoded, err)
	}
}

// The missing-required-member path for the congressional domain, mirroring
// the is_err assertions of the Rust tests: a required_option member must be
// present even though it may be null, a nested model reports its own missing
// member, and the one optional trade member may be absent or null.
func TestCongressionalRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	const position = `{"senateID":"Z000018","congressNumber":119,"startDate":"2025-01-02",` +
		`"party":"Republican","position":"Representative","state":"MT","yearsInTerm":0.7`
	const entry = `{"senateID":"P000197","formType":"House Report","year":2022,"filingDate":"2023-05-15",` +
		`"section":"Liabilities","category":"Mortgage","name":"Bank","assetType":"Mortgage","incomeType":null,` +
		`"owner":"Joint","comment":null,"valueRange":null,"value":-1,"incomeRange":{"min":-10,"max":20},` +
		`"income":null,"link":"https://example.test"`
	cases := []struct {
		name   string
		wire   string
		model  string
		member string
	}{
		{"position without endDate", `[` + position + `}]`, "CongressionalMemberPosition", "endDate"},
		{"position with null senateID", `[` + strings.Replace(position, `"Z000018"`, `null`, 1) + `,"endDate":null}]`,
			"CongressionalMemberPosition", "senateID"},
		{"entry without debtDetails", `[` + entry + `}]`, "CongressionalMemberNetWorth", "debtDetails"},
		{"entry with half a range",
			`[` + strings.Replace(entry, `"valueRange":null`, `"valueRange":{"min":1}`, 1) + `,"debtDetails":null}]`,
			"CongressionalNetWorthRange", "max"},
		{"trade without senateID", `[{"symbol":"AAPL"}]`, "CongressionalTrade", "senateID"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var err error
			switch tc.model {
			case "CongressionalTrade":
				var rows []CongressionalTrade
				err = json.Unmarshal([]byte(tc.wire), &rows)
			case "CongressionalMemberPosition":
				var rows []CongressionalMemberPosition
				err = json.Unmarshal([]byte(tc.wire), &rows)
			default:
				var rows []CongressionalMemberNetWorth
				err = json.Unmarshal([]byte(tc.wire), &rows)
			}
			assertCompanyDecodeError(t, err, tc.model, tc.member)
		})
	}

	var positions []CongressionalMemberPosition
	if err := json.Unmarshal([]byte(`[`+position+`,"endDate":"2027-01-03"}]`), &positions); err != nil ||
		len(positions) != 1 || positions[0].EndDate == nil || *positions[0].EndDate != mustParseDate(t, "2027-01-03") {
		t.Fatalf("position with a dated endDate = %+v, %v", positions, err)
	}
	var entries []CongressionalMemberNetWorth
	if err := json.Unmarshal([]byte(`[`+entry+`,"debtDetails":null}]`), &entries); err != nil || len(entries) != 1 ||
		entries[0].DebtDetails != nil || !reflect.DeepEqual(entries[0].Value, new(-1.0)) || entries[0].IncomeRange == nil ||
		entries[0].IncomeRange.Min != -10 {
		t.Fatalf("entry with null debtDetails = %+v, %v", entries, err)
	}
	income := `[` + strings.Replace(entry, `"income":null`, `"income":{"amount":"00123.450","kinds":[1,true,null]}`, 1) +
		`,"debtDetails":null}]`
	if err := json.Unmarshal([]byte(income), &entries); err != nil || len(entries) != 1 {
		t.Fatalf("entry with dynamic income = %+v, %v", entries, err)
	}
	assertCanonicalJSON(t, entries[0].Income, `{"amount":"00123.450","kinds":[1,true,null]}`)

	house := readFixture(t, "congress_house_latest.json")
	for _, replacement := range []string{``, `"capitalGainsOver200USD": null,`} {
		var trades []CongressionalTrade
		wire := strings.Replace(string(house), `"capitalGainsOver200USD": "False",`, replacement, 1)
		if err := json.Unmarshal([]byte(wire), &trades); err != nil || len(trades) != 1 || trades[0].CapitalGainsOver200Usd != nil {
			t.Fatalf("trade with capital gains %q = %+v, %v", replacement, trades, err)
		}
	}
}

// Mirrors documented_fields_have_strict_types_and_only_image_and_end_date_are_nullable in
// crates/libfmp/tests/congressional_member_endpoints.rs: members FMP sends as null
// decode to nil, while a missing member still fails.
func TestCongressionalMemberProfileImageDecodesNullAsNil(t *testing.T) {
	t.Parallel()
	const fixture = "congress_senate_profile.json"
	cases := []struct {
		member string
		isNil  func(CongressionalMemberProfile) bool
	}{
		{"image", func(row CongressionalMemberProfile) bool { return row.Image == nil }},
	}
	for _, tc := range cases {
		t.Run(tc.member, func(t *testing.T) {
			t.Parallel()
			var missing []CongressionalMemberProfile
			if err := json.Unmarshal(mutateFixtureMember(t, fixture, tc.member, nil), &missing); err == nil {
				t.Fatalf("missing %s decoded", tc.member)
			}
			var rows []CongressionalMemberProfile
			if err := json.Unmarshal(mutateFixtureMember(t, fixture, tc.member, jsontext.Value(`null`)), &rows); err != nil ||
				len(rows) != 1 || !tc.isNil(rows[0]) {
				t.Fatalf("null %s = %+v, %v, want nil", tc.member, rows, err)
			}
		})
	}
}

// Mirrors itemized_fractional_value_and_null_members_decode_exactly and
// aggregated_required_fields_are_strict_and_omittable_fields_may_be_absent in
// crates/libfmp/tests/congressional_net_worth_endpoints.rs: fractional amounts
// decode exactly, null itemized members and absent aggregate members decode to
// nil, and an empty debtDetails object has no dateIncurred.
func TestCongressionalNetWorthFractionalNullAndAbsentMembers(t *testing.T) {
	t.Parallel()
	const netWorth = "congress_senate_net_worth.json"
	var entries []CongressionalMemberNetWorth
	fractional := mutateFixtureMember(t, netWorth, "value", jsontext.Value(`32500.5`))
	if err := json.Unmarshal(fractional, &entries); err != nil || len(entries) != 1 ||
		!reflect.DeepEqual(entries[0].Value, new(32500.5)) {
		t.Fatalf("fractional value = %+v, %v", entries, err)
	}
	nullable := []struct {
		member string
		isNil  func(CongressionalMemberNetWorth) bool
	}{
		{"category", func(row CongressionalMemberNetWorth) bool { return row.Category == nil }},
		{"name", func(row CongressionalMemberNetWorth) bool { return row.Name == nil }},
		{"owner", func(row CongressionalMemberNetWorth) bool { return row.Owner == nil }},
		{"value", func(row CongressionalMemberNetWorth) bool { return row.Value == nil }},
		{"valueRange.max", func(row CongressionalMemberNetWorth) bool {
			return row.ValueRange != nil && row.ValueRange.Max == nil
		}},
	}
	for _, tc := range nullable {
		var rows []CongressionalMemberNetWorth
		if err := json.Unmarshal(mutateFixtureMember(t, netWorth, tc.member, jsontext.Value(`null`)), &rows); err != nil ||
			len(rows) != 1 || !tc.isNil(rows[0]) {
			t.Fatalf("null %s = %+v, %v, want nil", tc.member, rows, err)
		}
		if err := json.Unmarshal(mutateFixtureMember(t, netWorth, tc.member, nil), &rows); err == nil {
			t.Fatalf("missing %s decoded", tc.member)
		}
	}
	if err := json.Unmarshal(mutateFixtureMember(t, netWorth, "debtDetails.dateIncurred", nil), &entries); err != nil ||
		len(entries) != 1 || entries[0].DebtDetails == nil || entries[0].DebtDetails.DateIncurred != nil {
		t.Fatalf("absent dateIncurred = %+v, %v", entries, err)
	}
	if encoded, err := json.Marshal(entries[0].DebtDetails); err != nil || string(encoded) != `{}` {
		t.Fatalf("re-encoded debtDetails = %s, %v", encoded, err)
	}

	const sparse = `[{"cashAndCashEquivalents":121004.5,"mutualFundsAndETFs":34526531.5,` +
		`"senateID":"M000355","total":59082540.5,"year":2023}]`
	var totals []CongressionalMemberNetWorthAggregate
	if err := json.Unmarshal([]byte(sparse), &totals); err != nil || len(totals) != 1 {
		t.Fatalf("sparse aggregate = %+v, %v", totals, err)
	}
	if want := (CongressionalMemberNetWorthAggregate{MemberID: "M000355", Year: 2023, Total: 59_082_540.5,
		CashAndCashEquivalents: 121_004.5, MutualFundsAndETFs: 34_526_531.5}); !reflect.DeepEqual(totals[0], want) {
		t.Fatalf("sparse aggregate = %+v, want %+v", totals[0], want)
	}
	encoded, err := json.Marshal(totals)
	if err != nil {
		t.Fatal(err)
	}
	assertCanonicalJSON(t, (*jsontext.Value)(&encoded), sparse)
	fractionalTotals := strings.Replace(sparse, `"year":2023`, `"year":2023,"realEstate":3000000.5,"stock":8000.5`, 1)
	if err := json.Unmarshal([]byte(fractionalTotals), &totals); err != nil || len(totals) != 1 ||
		!reflect.DeepEqual(totals[0].RealEstate, new(3_000_000.5)) || !reflect.DeepEqual(totals[0].Stock, new(8_000.5)) {
		t.Fatalf("fractional aggregate = %+v, %v", totals, err)
	}
	if err := json.Unmarshal(mutateFixtureMember(t, "congress_senate_net_worth_aggregated.json", "realEstate", jsontext.Value(`null`)), &totals); err != nil ||
		len(totals) != 1 || totals[0].RealEstate != nil {
		t.Fatalf("null realEstate = %+v, %v, want nil", totals, err)
	}
	for _, member := range []string{"total", "cashAndCashEquivalents", "mutualFundsAndETFs"} {
		var rows []CongressionalMemberNetWorthAggregate
		if err := json.Unmarshal(mutateFixtureMember(t, "congress_senate_net_worth_aggregated.json", member, nil), &rows); err == nil {
			t.Fatalf("missing %s decoded", member)
		}
	}
}

func TestCongressionalNetWorthIncomeRangeDecodesEmptyAndNullAsNil(t *testing.T) {
	t.Parallel()
	const netWorth = "congress_senate_net_worth.json"
	for _, raw := range []string{`""`, `null`} {
		var rows []CongressionalMemberNetWorth
		if err := json.Unmarshal(mutateFixtureMember(t, netWorth, "incomeRange", jsontext.Value(raw)), &rows); err != nil ||
			len(rows) != 1 || rows[0].IncomeRange != nil {
			t.Fatalf("incomeRange %s = %+v, %v, want nil", raw, rows, err)
		}
		encoded, err := json.Marshal(rows[0])
		if err != nil || !strings.Contains(string(encoded), `"incomeRange":null`) {
			t.Fatalf("re-encoded incomeRange %s = %s, %v", raw, encoded, err)
		}
	}
	var rows []CongressionalMemberNetWorth
	object := mutateFixtureMember(t, netWorth, "incomeRange", jsontext.Value(`{"min":1001,"max":15000}`))
	if err := json.Unmarshal(object, &rows); err != nil || len(rows) != 1 ||
		!reflect.DeepEqual(rows[0].IncomeRange, &CongressionalNetWorthRange{Min: 1001, Max: new(int64(15000))}) {
		t.Fatalf("incomeRange object = %+v, %v", rows, err)
	}
	for _, tc := range []struct {
		raw, path string
		kind      DecodeKind
	}{
		{`"` + decodePathSentinel + `"`, "/0/incomeRange", DecodeKindWrongType},
		{`31337`, "/0/incomeRange", DecodeKindWrongType},
		{`{"max":15000}`, "/0/incomeRange/min", DecodeKindMissingMember},
	} {
		body := mutateFixtureMember(t, netWorth, "incomeRange", jsontext.Value(tc.raw))
		err := json.Unmarshal(body, &rows)
		if err == nil {
			t.Fatalf("incomeRange %s decoded", tc.raw)
		}
		if path, kind := decodeLocation(body, err); path != tc.path || kind != tc.kind {
			t.Fatalf("incomeRange %s: Path = %q, DecodeKind = %v, want %q %v", tc.raw, path, kind, tc.path, tc.kind)
		}
		if strings.Contains(err.Error(), decodePathSentinel) || strings.Contains(err.Error(), "31337") {
			t.Fatalf("decode error leaked the member value: %q", err)
		}
	}
	if err := json.Unmarshal(mutateFixtureMember(t, netWorth, "incomeRange", nil), &rows); err == nil {
		t.Fatal("missing incomeRange decoded")
	}
}
