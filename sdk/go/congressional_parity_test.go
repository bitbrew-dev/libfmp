package fmp

import (
	"encoding/json/v2"
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
		Image: "https://images.financialmodelingprep.com/senate/L000397.jpg", Active: true,
		YearsActive: 31.6}); len(profiles) != 1 || profiles[0] != want {
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
		entry.Category != "Mortgage & Real Estate Liability" || entry.Name != "Union Bank of California" ||
		entry.AssetType != "Mortgage on 2640 Broadway, San Francisco, CA" || entry.Owner != "Joint" ||
		entry.Value != 3_000_001 ||
		entry.Link != "https://disclosures-clerk.house.gov/public_disc/financial-pdfs/2022/10053231.pdf" {
		t.Fatalf("congress_senate_net_worth = %+v", entry)
	}
	if entry.IncomeType != nil || entry.Comment != nil || entry.IncomeRange != nil || entry.Income != nil {
		t.Fatalf("null members decoded as non-nil: %+v", entry)
	}
	if entry.DebtDetails == nil || *entry.DebtDetails != (CongressionalDebtDetails{DateIncurred: "September 2007"}) {
		t.Fatalf("debtDetails = %+v", entry.DebtDetails)
	}
	if entry.ValueRange == nil || *entry.ValueRange != (CongressionalNetWorthRange{Min: 1_000_001, Max: 5_000_000}) {
		t.Fatalf("valueRange = %+v", entry.ValueRange)
	}

	totals := assertFixtureParity[CongressionalMemberNetWorthAggregate](t, "congress_senate_net_worth_aggregated.json")
	if want := (CongressionalMemberNetWorthAggregate{MemberID: "P000197", Year: 2024, Total: 225_219_551,
		RealEstateLiabilities: 27_000_005, CashAndCashEquivalents: 291_009, BusinessAndSelfEmployment: 0,
		RealEstate: 45_032_504, OwnershipInterest: 70_140_014, Stock: 136_748_525, Options: 0,
		RevolvingAndCreditLines: 1_500_002, AssetBackedSecurities: 4_475_006, BusinessLiabilities: 3_000_001,
		MutualFundsAndEtfs: 32_501}); len(totals) != 1 || totals[0] != want {
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
		{"entry with empty debtDetails", `[` + entry + `,"debtDetails":{}}]`, "CongressionalDebtDetails", "dateIncurred"},
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
		entries[0].DebtDetails != nil || entries[0].Value != -1 || entries[0].IncomeRange == nil ||
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
