package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"testing"
)

// The eleven market fixtures of crates/libfmp/tests/market_responses.rs. The
// three market-hours fixtures the market_hours_*.rs tests decode belong to the
// market_hours domain and are covered there.
func TestMarketFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[SectorPerformance](t, "sector_performance_snapshot.json")
	assertFixtureParity[SectorPerformance](t, "historical_sector_performance.json")
	assertFixtureParity[IndustryPerformance](t, "industry_performance_snapshot.json")
	assertFixtureParity[IndustryPerformance](t, "historical_industry_performance.json")
	assertFixtureParity[SectorPe](t, "sector_pe_snapshot.json")
	assertFixtureParity[SectorPe](t, "historical_sector_pe.json")
	assertFixtureParity[IndustryPe](t, "industry_pe_snapshot.json")
	assertFixtureParity[IndustryPe](t, "historical_industry_pe.json")
	assertFixtureParity[MarketMover](t, "biggest_gainers.json")
	assertFixtureParity[MarketMover](t, "biggest_losers.json")
	assertFixtureParity[MarketMover](t, "most_actives.json")
}

// Exact values copied from crates/libfmp/tests/market_responses.rs: the
// signed fractional average change of the sector snapshot, the positive one of
// the historical row, and the four-member shape shared by both endpoints.
func TestDocumentedSectorAndIndustryPerformanceDecodeExactValues(t *testing.T) {
	t.Parallel()
	snapshot := assertFixtureParity[SectorPerformance](t, "sector_performance_snapshot.json")
	want := SectorPerformance{Date: mustParseDate(t, "2024-02-01"), Sector: "Basic Materials", Exchange: "NASDAQ",
		AverageChange: -0.31481377464310634}
	if len(snapshot) != 1 || snapshot[0] != want {
		t.Fatalf("sector_performance_snapshot = %+v, want %+v", snapshot, want)
	}
	historical := assertFixtureParity[SectorPerformance](t, "historical_sector_performance.json")
	want = SectorPerformance{Date: mustParseDate(t, "2024-03-01"), Sector: "Energy", Exchange: "NASDAQ",
		AverageChange: 1.3989969286740689}
	if len(historical) != 1 || historical[0] != want {
		t.Fatalf("historical_sector_performance = %+v, want %+v", historical, want)
	}
	if members := memberSet(t, want); len(members) != 4 {
		t.Fatalf("SectorPerformance members = %v, want the documented 4", members)
	}

	industries := assertFixtureParity[IndustryPerformance](t, "industry_performance_snapshot.json")
	wantIndustry := IndustryPerformance{Date: mustParseDate(t, "2024-02-01"), Industry: "Advertising Agencies",
		Exchange: "NASDAQ", AverageChange: 3.8660194344955996}
	if len(industries) != 1 || industries[0] != wantIndustry {
		t.Fatalf("industry_performance_snapshot = %+v, want %+v", industries, wantIndustry)
	}
	historicalIndustries := assertFixtureParity[IndustryPerformance](t, "historical_industry_performance.json")
	if len(historicalIndustries) != 1 || historicalIndustries[0].Industry != "Biotechnology" ||
		historicalIndustries[0].AverageChange != 2.6143442556463383 {
		t.Fatalf("historical_industry_performance = %+v", historicalIndustries)
	}
}

// Exact P/E values copied from crates/libfmp/tests/market_responses.rs, plus
// the integer JSON token the Rust test decodes into the f64 pe member.
func TestDocumentedSectorAndIndustryPeDecodeExactValues(t *testing.T) {
	t.Parallel()
	sectors := assertFixtureParity[SectorPe](t, "sector_pe_snapshot.json")
	want := SectorPe{Date: mustParseDate(t, "2024-02-01"), Sector: "Basic Materials", Exchange: "NASDAQ",
		Pe: 15.687711758428254}
	if len(sectors) != 1 || sectors[0] != want {
		t.Fatalf("sector_pe_snapshot = %+v, want %+v", sectors, want)
	}
	historicalSectors := assertFixtureParity[SectorPe](t, "historical_sector_pe.json")
	if len(historicalSectors) != 1 || historicalSectors[0].Sector != "Energy" ||
		historicalSectors[0].Pe != 5.4165892628211205 {
		t.Fatalf("historical_sector_pe = %+v", historicalSectors)
	}
	industries := assertFixtureParity[IndustryPe](t, "industry_pe_snapshot.json")
	wantIndustry := IndustryPe{Date: mustParseDate(t, "2024-02-01"), Industry: "Advertising Agencies",
		Exchange: "NASDAQ", Pe: 71.09601665201151}
	if len(industries) != 1 || industries[0] != wantIndustry {
		t.Fatalf("industry_pe_snapshot = %+v, want %+v", industries, wantIndustry)
	}
	historicalIndustries := assertFixtureParity[IndustryPe](t, "historical_industry_pe.json")
	if len(historicalIndustries) != 1 || historicalIndustries[0].Date.String() != "2024-03-01" ||
		historicalIndustries[0].Pe != 8.129037884885042 {
		t.Fatalf("historical_industry_pe = %+v", historicalIndustries)
	}

	var integerPe []SectorPe
	err := json.Unmarshal([]byte(`[{"date":"2024-02-29","sector":"Future / Sector","exchange":"NEW-EXCHANGE","pe":-3}]`),
		&integerPe)
	if err != nil || len(integerPe) != 1 || integerPe[0].Pe != -3 || integerPe[0].Sector != "Future / Sector" {
		t.Fatalf("integer pe = %+v, %v", integerPe, err)
	}
}

// Exact mover values copied from crates/libfmp/tests/market_mover_endpoints.rs
// and market_responses.rs, including the integer and zero numeric spellings
// the provider may use for the price and change members.
func TestDocumentedMarketMoversDecodeExactValues(t *testing.T) {
	t.Parallel()
	gainers := assertFixtureParity[MarketMover](t, "biggest_gainers.json")
	want := MarketMover{Symbol: "MOTS", Price: 0.0002, Name: "Motus GI Holdings, Inc.", Change: 0.0001,
		ChangesPercentage: 100, Exchange: "OTC"}
	if len(gainers) != 1 || gainers[0] != want {
		t.Fatalf("biggest_gainers = %+v, want %+v", gainers, want)
	}
	if members := memberSet(t, want); len(members) != 6 {
		t.Fatalf("MarketMover members = %v, want the documented 6", members)
	}
	losers := assertFixtureParity[MarketMover](t, "biggest_losers.json")
	if len(losers) != 1 || losers[0].Symbol != "SPEC" || losers[0].Change != -0.002 ||
		losers[0].ChangesPercentage != -90.90909 {
		t.Fatalf("biggest_losers = %+v", losers)
	}
	actives := assertFixtureParity[MarketMover](t, "most_actives.json")
	if len(actives) != 1 || actives[0].Symbol != "LUCY" || actives[0].Price != 1.85 || actives[0].Exchange != "NASDAQ" {
		t.Fatalf("most_actives = %+v", actives)
	}

	var numericForms []MarketMover
	err := json.Unmarshal([]byte(`[{"symbol":"ZERO","price":0,"name":"Integer price and zero changes","change":0,`+
		`"changesPercentage":0.0,"exchange":"FUTURE / EXCHANGE"}]`), &numericForms)
	if err != nil || len(numericForms) != 1 || numericForms[0].Price != 0 || numericForms[0].Change != 0 ||
		numericForms[0].ChangesPercentage != 0 || numericForms[0].Exchange != "FUTURE / EXCHANGE" {
		t.Fatalf("numeric forms = %+v, %v", numericForms, err)
	}
}

// Every member of every market model is required and non-null, as the Rust
// assert_contract check enforces for each documented field, and the date
// member is a strict calendar date.
func TestMarketRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	var rows []map[string]jsontext.Value
	if err := json.Unmarshal(readFixture(t, "sector_performance_snapshot.json"), &rows); err != nil {
		t.Fatal(err)
	}
	mutate := func(member string, value jsontext.Value) []byte {
		row := make(map[string]jsontext.Value, len(rows[0]))
		for name, raw := range rows[0] {
			row[name] = raw
		}
		if value == nil {
			delete(row, member)
		} else {
			row[member] = value
		}
		encoded, err := json.Marshal([]map[string]jsontext.Value{row})
		if err != nil {
			t.Fatal(err)
		}
		return encoded
	}
	cases := []struct {
		name    string
		wire    []byte
		message string
	}{
		{"missing date", mutate("date", nil), `required member "date" of SectorPerformance is missing or null`},
		{"null sector", mutate("sector", jsontext.Value(`null`)),
			`required member "sector" of SectorPerformance is missing or null`},
		{"missing exchange", mutate("exchange", nil),
			`required member "exchange" of SectorPerformance is missing or null`},
		{"null average change", mutate("averageChange", jsontext.Value(`null`)),
			`required member "averageChange" of SectorPerformance is missing or null`},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var decoded []SectorPerformance
			err := json.Unmarshal(tc.wire, &decoded)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode || typed.Message != tc.message {
				t.Fatalf("error = %v (%T), want CategoryDecode %q", err, err, tc.message)
			}
		})
	}
	for _, invalid := range []string{"2024-2-01", "2024-02-1", "2024/02/01", "2024-02-30", "2024-02-01T00:00:00"} {
		var decoded []SectorPerformance
		if err := json.Unmarshal(mutate("date", jsontext.Value(`"`+invalid+`"`)), &decoded); err == nil {
			t.Fatalf("date %q decoded into SectorPerformance", invalid)
		}
	}
	var movers []MarketMover
	err := json.Unmarshal([]byte(`[{"symbol":"MOTS","price":0.0002,"name":"Motus GI Holdings, Inc.","change":0.0001,`+
		`"changesPercentage":null,"exchange":"OTC"}]`), &movers)
	var typed *Error
	if !errors.As(err, &typed) || typed.Message != `required member "changesPercentage" of MarketMover is missing or null` {
		t.Fatalf("MarketMover error = %v, want the null changesPercentage member", err)
	}
	var pes []IndustryPe
	if err := json.Unmarshal([]byte(`[{"date":"2024-02-01","industry":"Advertising Agencies","exchange":"NASDAQ"}]`),
		&pes); err == nil {
		t.Fatal("an industry row without pe decoded into IndustryPe")
	}
}
