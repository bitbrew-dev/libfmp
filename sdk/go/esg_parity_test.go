package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"slices"
	"testing"
)

func TestEsgFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[EsgDisclosure](t, "esg_disclosures.json")
	assertFixtureParity[EsgRating](t, "esg_ratings.json")
	assertFixtureParity[EsgBenchmark](t, "esg_benchmark.json")
}

// Exact values and wire-key hazards copied from
// crates/libfmp/tests/esg_responses.rs: the provider spells the score and
// rating keys with an upper-case ESG acronym, and the model must keep those
// spellings rather than the camelCase ones.
func TestDocumentedEsgDisclosureDecodesExactValuesAndKeepsAcronymKey(t *testing.T) {
	t.Parallel()
	disclosures := assertFixtureParity[EsgDisclosure](t, "esg_disclosures.json")
	if len(disclosures) != 1 {
		t.Fatalf("rows = %d, want 1", len(disclosures))
	}
	disclosure := disclosures[0]
	if disclosure.Date.String() != "2026-03-28" || disclosure.AcceptedDate.String() != "2026-04-30" ||
		disclosure.Symbol != "AAPL" || disclosure.Cik != "0000320193" || disclosure.CompanyName != "Apple Inc." ||
		disclosure.FormType != "8-K" || disclosure.EnvironmentalScore != 66.29 || disclosure.SocialScore != 45.21 ||
		disclosure.GovernanceScore != 58.87 || disclosure.EsgScore != 56.79 ||
		disclosure.URL != "https://www.sec.gov/Archives/edgar/data/320193/000032019326000011/0000320193-26-000011-index.htm" {
		t.Fatalf("esg_disclosures = %+v", disclosure)
	}
	members := memberSet(t, disclosure)
	if len(members) != 11 || !slices.Contains(members, "ESGScore") || slices.Contains(members, "esgScore") {
		t.Fatalf("disclosure members = %v, want the documented 11 including the exact ESGScore key", members)
	}
}

// Exact values copied from crates/libfmp/tests/esg_responses.rs.
func TestDocumentedEsgRatingAndBenchmarkDecodeExactValues(t *testing.T) {
	t.Parallel()
	ratings := assertFixtureParity[EsgRating](t, "esg_ratings.json")
	if len(ratings) != 1 {
		t.Fatalf("rows = %d, want 1", len(ratings))
	}
	rating := ratings[0]
	if rating.Symbol != "AAPL" || rating.Cik != "0000320193" || rating.CompanyName != "Apple Inc." ||
		rating.Industry != "CONSUMER ELECTRONICS" || rating.FiscalYear != 2025 || rating.EsgRiskRating != "B" ||
		rating.IndustryRank != "17 out of 20" {
		t.Fatalf("esg_ratings = %+v", rating)
	}
	members := memberSet(t, rating)
	if len(members) != 7 || !slices.Contains(members, "ESGRiskRating") || slices.Contains(members, "esgRiskRating") {
		t.Fatalf("rating members = %v, want the documented 7 including the exact ESGRiskRating key", members)
	}

	benchmarks := assertFixtureParity[EsgBenchmark](t, "esg_benchmark.json")
	want := EsgBenchmark{FiscalYear: 2023, Sector: "APPAREL RETAIL", EnvironmentalScore: 61.36, SocialScore: 67.44,
		GovernanceScore: 68.1, EsgScore: 65.63}
	if len(benchmarks) != 1 || benchmarks[0] != want {
		t.Fatalf("esg_benchmark = %+v, want %+v", benchmarks, want)
	}
	if members := memberSet(t, want); len(members) != 6 || !slices.Contains(members, "ESGScore") {
		t.Fatalf("benchmark members = %v, want the documented 6 including the exact ESGScore key", members)
	}
}

// Every member of every ESG model is required and non-null, as the Rust
// decoder enforces for each of the documented fields.
func TestEsgRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	var rows []map[string]jsontext.Value
	if err := json.Unmarshal(readFixture(t, "esg_disclosures.json"), &rows); err != nil {
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
		{"missing acronym score", mutate("ESGScore", nil),
			`required member "ESGScore" of EsgDisclosure is missing or null`},
		{"null acronym score", mutate("ESGScore", jsontext.Value(`null`)),
			`required member "ESGScore" of EsgDisclosure is missing or null`},
		{"missing date", mutate("acceptedDate", nil),
			`required member "acceptedDate" of EsgDisclosure is missing or null`},
		{"null text", mutate("url", jsontext.Value(`null`)),
			`required member "url" of EsgDisclosure is missing or null`},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var decoded []EsgDisclosure
			err := json.Unmarshal(tc.wire, &decoded)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode || typed.Message != tc.message {
				t.Fatalf("error = %v (%T), want CategoryDecode %q", err, err, tc.message)
			}
		})
	}
	var ratings []EsgRating
	err := json.Unmarshal([]byte(`[{"symbol":"AAPL","cik":"0000320193","companyName":"Apple Inc.",`+
		`"industry":"CONSUMER ELECTRONICS","fiscalYear":null,"ESGRiskRating":"B","industryRank":"17 out of 20"}]`), &ratings)
	var typed *Error
	if !errors.As(err, &typed) || typed.Message != `required member "fiscalYear" of EsgRating is missing or null` {
		t.Fatalf("EsgRating error = %v, want the null fiscalYear member", err)
	}
	var benchmarks []EsgBenchmark
	if err := json.Unmarshal([]byte(`[{"fiscalYear":2023,"sector":"APPAREL RETAIL"}]`), &benchmarks); err == nil {
		t.Fatal("a benchmark without scores decoded into EsgBenchmark")
	}
}
