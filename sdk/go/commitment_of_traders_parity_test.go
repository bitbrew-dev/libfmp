package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"slices"
	"strings"
	"testing"
)

func TestCommitmentOfTradersFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[CotReport](t, "cot_report.json")
	assertFixtureParity[CotAnalysis](t, "cot_analysis.json")
	assertFixtureParity[CotReportListing](t, "cot_report_list.json")
}

// Exact values and wire-key hazards copied from
// crates/libfmp/tests/cot_responses.rs: the provider misspells four keys and
// the model must keep those spellings rather than the corrected ones.
func TestDocumentedCotReportDecodesExactValuesAndKeepsWireHazards(t *testing.T) {
	t.Parallel()
	reports := assertFixtureParity[CotReport](t, "cot_report.json")
	if len(reports) != 1 {
		t.Fatalf("rows = %d, want 1", len(reports))
	}
	report := reports[0]
	if report.Symbol != "VX" || report.Date.String() != "2024-02-27 00:00:00" || report.Sector != "INDICES" ||
		report.Name != "CBOE VIX (VX)" || report.ChangeInNoncommSpreadAll != 9_257 ||
		report.TradersNoncommSpreadOld != 101 || report.ContractUnits != "($1000 X INDEX)" {
		t.Fatalf("cot_report = %+v", report)
	}
	members := memberSet(t, report)
	if len(members) != 128 {
		t.Fatalf("re-encoded members = %d, want the documented 128", len(members))
	}
	for _, key := range []string{"changeInNoncommSpeadAll", "tradersNoncommSpeadOl", "pctOfOpenInterestOl",
		"concGrossLe4TdrLongOl"} {
		if !slices.Contains(members, key) {
			t.Fatalf("missing exact wire key %q in %v", key, members)
		}
	}
	for _, key := range []string{"changeInNoncommSpreadAll", "tradersNoncommSpreadOld", "pctOfOpenInterestOld",
		"concGrossLe4TdrLongOld"} {
		if slices.Contains(members, key) {
			t.Fatalf("corrected but wrong wire key %q was emitted", key)
		}
	}
}

// The Number members keep the provider's spelling: 100 stays 100 and 20.6
// stays 20.6, which a float64 round trip would not guarantee.
func TestCotReportPreservesIntegerAndDecimalNumberSpellings(t *testing.T) {
	t.Parallel()
	report := assertFixtureParity[CotReport](t, "cot_report.json")[0]
	if string(report.PctOfOpenInterestAll) != "100" || string(report.PctOfOiNoncommLongAll) != "20.6" {
		t.Fatalf("raw numbers = %s and %s", report.PctOfOpenInterestAll, report.PctOfOiNoncommLongAll)
	}
	encoded, err := json.Marshal(report)
	if err != nil {
		t.Fatal(err)
	}
	text := string(encoded)
	for _, want := range []string{`"pctOfOpenInterestAll":100,`, `"pctOfOiNoncommLongAll":20.6,`,
		`"pctOfOiNoncommSpreadAll":27,`, `"concGrossLe8TdrShortAll":29,`} {
		if !strings.Contains(text, want) {
			t.Fatalf("re-encoded report lacks %s", want)
		}
	}
	for _, reject := range []string{`"pctOfOpenInterestAll":100.0`, `"pctOfOiNoncommSpreadAll":27.0`} {
		if strings.Contains(text, reject) {
			t.Fatalf("re-encoded report rewrote a number spelling: %s", reject)
		}
	}
}

// Exact values copied from crates/libfmp/tests/cot_responses.rs.
func TestDocumentedCotAnalysisAndListingDecodeExactValues(t *testing.T) {
	t.Parallel()
	analyses := assertFixtureParity[CotAnalysis](t, "cot_analysis.json")
	if len(analyses) != 1 {
		t.Fatalf("rows = %d, want 1", len(analyses))
	}
	analysis := analyses[0]
	if analysis.Exchange != "PALLADIUM - NEW YORK MERCANTILE EXCHANGE" || analysis.NetPosition != -12_315 ||
		!analysis.ReversalTrend || analysis.Symbol != "PA" || analysis.Sector != "METALS" ||
		analysis.ChangeInNetPosition != 1.11 || analysis.MarketSentiment != "Increasing Bullish" {
		t.Fatalf("cot_analysis = %+v", analysis)
	}
	if members := memberSet(t, analysis); len(members) != 16 || !slices.Contains(members, "netPostion") {
		t.Fatalf("analysis members = %v, want 16 including the netPostion wire key", members)
	}
	listings := assertFixtureParity[CotReportListing](t, "cot_report_list.json")
	if want := (CotReportListing{Symbol: "NG", Name: "Natural Gas (NG)"}); len(listings) != 1 || listings[0] != want {
		t.Fatalf("cot_report_list = %+v", listings)
	}
}

// Every member of every COT model is required and non-null, and a Number
// member must carry a JSON number, as the Rust decoder enforces.
func TestCotRequiredMembersAndNumberKindsAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	var rows []map[string]jsontext.Value
	if err := json.Unmarshal(readFixture(t, "cot_report.json"), &rows); err != nil {
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
		{"missing number", mutate("pctOfOpenInterestAll", nil),
			`required member "pctOfOpenInterestAll" of CotReport is missing or null`},
		{"null number", mutate("pctOfOpenInterestAll", jsontext.Value(`null`)),
			`required member "pctOfOpenInterestAll" of CotReport is missing or null`},
		{"string number", mutate("pctOfOpenInterestAll", jsontext.Value(`"100"`)),
			`member "pctOfOpenInterestAll" of CotReport must be a JSON number`},
		{"missing integer", mutate("changeInNoncommSpeadAll", nil),
			`required member "changeInNoncommSpeadAll" of CotReport is missing or null`},
		{"missing text", mutate("contractUnits", nil),
			`required member "contractUnits" of CotReport is missing or null`},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var decoded []CotReport
			err := json.Unmarshal(tc.wire, &decoded)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode || typed.Message != tc.message {
				t.Fatalf("error = %v (%T), want CategoryDecode %q", err, err, tc.message)
			}
		})
	}
	var analyses []CotAnalysis
	err := json.Unmarshal([]byte(`[{"symbol":"PA","date":"2024-02-27 00:00:00"}]`), &analyses)
	var typed *Error
	if !errors.As(err, &typed) || !strings.Contains(typed.Message, `"name"`) {
		t.Fatalf("CotAnalysis error = %v, want the first missing member name", err)
	}
	var listings []CotReportListing
	if err := json.Unmarshal([]byte(`[{"symbol":"NG","name":null}]`), &listings); err == nil {
		t.Fatal("a null name decoded into CotReportListing")
	}
}
