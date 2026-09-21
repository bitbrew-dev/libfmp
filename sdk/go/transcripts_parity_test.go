package fmp

import (
	"encoding/json/jsontext"
	"encoding/json/v2"
	"errors"
	"strings"
	"testing"
)

// transcriptsExactContent is the complete content member of
// earnings_transcript.json, copied from crates/libfmp/tests/transcript_responses.rs.
const transcriptsExactContent = "Operator: Good day, everyone. Welcome to the Apple Incorporated Third Quarter Fiscal Year 2020 Earnings Conference Call. Today's call is being recorded. At this time, for opening remarks and introductions, I would like to turn things over to Mr. Tejas Gala, Senior Manager, Corporate Finance and Investor Relations. Please go ahead, sir.\nTejas Gala: Thank you. Good afternoon and thank you for joining us. Speaking first today is Apple's CEO, Tim Cook; and he'll be followed by CFO, Luca Maestri. Aft..."

// The three transcripts fixtures, one per response model. Every member is
// required and non-null in the Rust models, so no fixture carries an
// intentionally unknown member. The fourth fixture the Rust transcript tests
// read, directory_earnings_transcript_list.json, decodes into the
// directory-owned EarningsTranscriptAvailability and is covered by
// directory_parity_test.go.
func TestTranscriptsFixturesDecodeAndReencodeToTheSameMemberSet(t *testing.T) {
	t.Parallel()
	assertFixtureParity[LatestEarningsTranscript](t, "latest_earnings_transcripts.json")
	assertFixtureParity[EarningsTranscript](t, "earnings_transcript.json")
	assertFixtureParity[EarningsTranscriptDate](t, "earnings_transcript_dates.json")
}

// Exact values and member counts copied from
// crates/libfmp/tests/transcript_responses.rs (4, 5, and 3 members).
func TestDocumentedTranscriptFixturesDecodeExactly(t *testing.T) {
	t.Parallel()
	latest := assertFixtureParity[LatestEarningsTranscript](t, "latest_earnings_transcripts.json")
	wantLatest := LatestEarningsTranscript{Symbol: "VLO", Period: "Q2", FiscalYear: 2026, Date: mustParseDate(t, "2026-07-30")}
	if len(latest) != 1 || latest[0] != wantLatest {
		t.Fatalf("latest_earnings_transcripts = %+v, want %+v", latest, wantLatest)
	}
	if got := memberSet(t, latest[0]); len(got) != 4 {
		t.Fatalf("re-encoded latest members = %d, want 4", len(got))
	}

	transcripts := assertFixtureParity[EarningsTranscript](t, "earnings_transcript.json")
	wantTranscript := EarningsTranscript{
		Symbol: "AAPL", Period: "Q3", Year: 2020, Date: mustParseDate(t, "2020-07-30"),
		Content: transcriptsExactContent,
	}
	if len(transcripts) != 1 || transcripts[0] != wantTranscript {
		t.Fatalf("earnings_transcript = %+v, want %+v", transcripts, wantTranscript)
	}
	if !strings.Contains(transcripts[0].Content, "\nTejas Gala:") || !strings.HasSuffix(transcripts[0].Content, "Aft...") {
		t.Fatalf("content lost its documented line break or ending: %q", transcripts[0].Content)
	}
	if got := memberSet(t, transcripts[0]); len(got) != 5 {
		t.Fatalf("re-encoded transcript members = %d, want 5", len(got))
	}

	dates := assertFixtureParity[EarningsTranscriptDate](t, "earnings_transcript_dates.json")
	wantDate := EarningsTranscriptDate{Quarter: 2, FiscalYear: 2026, Date: mustParseDate(t, "2026-04-30")}
	if len(dates) != 1 || dates[0] != wantDate {
		t.Fatalf("earnings_transcript_dates = %+v, want %+v", dates, wantDate)
	}
	if got := memberSet(t, dates[0]); len(got) != 3 {
		t.Fatalf("re-encoded date members = %d, want 3", len(got))
	}
}

// Mirrors exact_dates_fixture_uses_numeric_response_quarter_and_year: the
// numeric quarter and year members reject string and decimal spellings.
func TestTranscriptNumericMembersRejectStringAndDecimalSpellings(t *testing.T) {
	t.Parallel()
	rewrite := func(fixture, member, value string) []byte {
		t.Helper()
		var rows []map[string]jsontext.Value
		if err := json.Unmarshal(readFixture(t, fixture), &rows); err != nil {
			t.Fatal(err)
		}
		rows[0][member] = jsontext.Value(value)
		encoded, err := json.Marshal(rows)
		if err != nil {
			t.Fatal(err)
		}
		return encoded
	}
	for _, invalid := range []string{`"2"`, "2.0"} {
		var dates []EarningsTranscriptDate
		if err := json.Unmarshal(rewrite("earnings_transcript_dates.json", "quarter", invalid), &dates); err == nil {
			t.Fatalf("quarter %s decoded into the uint8 member", invalid)
		}
	}
	for _, invalid := range []string{`"2026"`, "2026.0"} {
		var latest []LatestEarningsTranscript
		if err := json.Unmarshal(rewrite("latest_earnings_transcripts.json", "fiscalYear", invalid), &latest); err == nil {
			t.Fatalf("fiscalYear %s decoded into the uint32 member", invalid)
		}
		var transcripts []EarningsTranscript
		if err := json.Unmarshal(rewrite("earnings_transcript.json", "year", invalid), &transcripts); err == nil {
			t.Fatalf("year %s decoded into the uint32 member", invalid)
		}
	}
}

// Mirrors transcript_content_is_unbounded_and_preserved_exactly at a smaller
// size: escaped line breaks, quotes, backslashes, tabs, and non-ASCII text
// survive the decode byte for byte, and the year keeps the full u32 domain.
func TestTranscriptContentIsPreservedExactly(t *testing.T) {
	t.Parallel()
	chunk := "Operator: café 中文 📈.\nQuoted: \"guidance\"; path=C:\\reports\\Q4; tab=\tend.\n"
	content := strings.Repeat(chunk, 400)
	row := map[string]any{"symbol": "BIG", "period": "Q4", "year": uint32(4294967295), "date": "2026-07-30", "content": content}
	wire, err := json.Marshal([]map[string]any{row})
	if err != nil {
		t.Fatal(err)
	}
	for _, escape := range []string{`\n`, `\"`, `\\`} {
		if !strings.Contains(string(wire), escape) {
			t.Fatalf("wire text lacks the %s escape", escape)
		}
	}
	var rows []EarningsTranscript
	if err := json.Unmarshal(wire, &rows); err != nil {
		t.Fatal(err)
	}
	if len(rows) != 1 || rows[0].Year != 4294967295 || rows[0].Content != content {
		t.Fatalf("content or year changed in transit: year=%d len=%d want len=%d", rows[0].Year, len(rows[0].Content), len(content))
	}
}

// Mirrors all_transcript_rows_are_bare_arrays_preserving_empty_shapes.
func TestTranscriptRowsAreBareArraysPreservingEmptyShapes(t *testing.T) {
	t.Parallel()
	var latest []LatestEarningsTranscript
	var transcripts []EarningsTranscript
	var dates []EarningsTranscriptDate
	if err := json.Unmarshal([]byte("[]"), &latest); err != nil || len(latest) != 0 {
		t.Fatalf("latest [] = %+v, %v", latest, err)
	}
	if err := json.Unmarshal([]byte("[]"), &transcripts); err != nil || len(transcripts) != 0 {
		t.Fatalf("transcripts [] = %+v, %v", transcripts, err)
	}
	if err := json.Unmarshal([]byte("[]"), &dates); err != nil || len(dates) != 0 {
		t.Fatalf("dates [] = %+v, %v", dates, err)
	}
	if err := json.Unmarshal([]byte(`{"transcripts": []}`), &transcripts); err == nil {
		t.Fatal("a wrapping object decoded into the bare transcript array")
	}
}

// The missing-required-member path of the generated decoders, once for this
// domain: serde rejects a missing and a null member alike, and a date-only
// member rejects a timestamp.
func TestTranscriptRequiredMembersAreEnforcedLikeSerde(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name   string
		wire   string
		member string
	}{
		{"missing member", `[{"fiscalYear":2026,"date":"2026-04-30"}]`, "quarter"},
		{"null member", `[{"quarter":null,"fiscalYear":2026,"date":"2026-04-30"}]`, "quarter"},
		{"empty object", `[{}]`, "quarter"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			var rows []EarningsTranscriptDate
			err := json.Unmarshal([]byte(tc.wire), &rows)
			var typed *Error
			if !errors.As(err, &typed) || typed.Category != CategoryDecode {
				t.Fatalf("error = %v (%T), want a CategoryDecode *Error", err, err)
			}
			if !strings.Contains(typed.Message, `"`+tc.member+`"`) ||
				!strings.Contains(typed.Message, "EarningsTranscriptDate") {
				t.Fatalf("message = %q, want it to name member %q of EarningsTranscriptDate", typed.Message, tc.member)
			}
		})
	}
	var latest []LatestEarningsTranscript
	rewritten := strings.Replace(string(readFixture(t, "latest_earnings_transcripts.json")),
		`"date": "2026-07-30"`, `"date": "2026-07-30 00:00:00"`, 1)
	if err := json.Unmarshal([]byte(rewritten), &latest); err == nil {
		t.Fatal("a timestamp decoded into the Date member date")
	}
}
